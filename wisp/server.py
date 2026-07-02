"""The HUD server — stdlib HTTP + SSE, no dependencies.

Serves the single-file HUD and a small JSON API:

    GET  /                      HUD
    GET  /api/state             vitals, directives, schedule, trail, jobs, skills
    GET  /api/skills            skill manifests (buttons)
    POST /api/skills/<n>/run    enqueue a skill run        {"text": "..."} optional
    POST /api/say               voice/text intake → route → skill job or chat reply
    GET  /api/reports/<name>    a vault report (markdown)
    GET  /api/jobs              recent jobs
    GET  /api/events            SSE stream of job updates
"""

from __future__ import annotations

import json
import queue
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

from . import REPO_ROOT, __version__
from .config import Config
from .jobs import JobQueue
from .router import CHAT, Router
from .runner import Dispatcher
from .skills import SkillRegistry
from .vault import Vault

HUD_PATH = Path(__file__).parent / "hud" / "index.html"


class WispApp:
    """Wires together every subsystem; one instance per server."""

    def __init__(self, config: Config):
        self.config = config
        self.vault = Vault(config.vault_dir)
        self.vault.ensure_layout()
        self.registry = SkillRegistry(config.skills_dir)
        self.router = Router(config, self.registry)
        self.dispatcher = Dispatcher(config, self.registry, self.vault)
        self.jobs = JobQueue(self.dispatcher)

    # -- intake ---------------------------------------------------------------
    def say(self, text: str) -> dict:
        decision = self.router.route(text)
        if decision.skill == CHAT:
            reply = self.dispatcher.chat(text)
            return {"decision": decision.to_json(), "reply": reply}
        job = self.jobs.enqueue(decision.skill, text)
        return {"decision": decision.to_json(), "job": job.to_json()}

    # -- state ------------------------------------------------------------------
    def _git(self, *args: str) -> str:
        try:
            proc = subprocess.run(["git", *args], cwd=REPO_ROOT, capture_output=True, text=True, timeout=10)
            return proc.stdout.strip() if proc.returncode == 0 else ""
        except (subprocess.TimeoutExpired, OSError):
            return ""

    def vitals(self) -> dict:
        stats = self.vault.stats()
        commits_today = self._git("rev-list", "--count", "--since=midnight", "HEAD") or "0"
        commits_week = self._git("rev-list", "--count", "--since=7.days", "HEAD") or "0"
        dirty = len([l for l in self._git("status", "--porcelain").splitlines() if l.strip()])
        done_today = sum(1 for j in self.jobs.jobs(limit=100) if j.status == "done")
        return {
            "branch": self._git("rev-parse", "--abbrev-ref", "HEAD") or "?",
            "commits_today": int(commits_today),
            "commits_week": int(commits_week),
            "uncommitted": dirty,
            "vault_notes": stats["notes"],
            "vault_reports": stats["reports"],
            "inbox": stats["inbox"],
            "skills": len(self.registry.all()),
            "runs_today": done_today,
        }

    def state(self) -> dict:
        return {
            "assistant": self.config.assistant_name,
            "version": __version__,
            "engine": "claude" if self.dispatcher.claude.available() else "builtin",
            "vitals": self.vitals(),
            "directives": self.vault.directives(),
            "schedule": self.vault.schedule(),
            "trail": self.vault.trail(),
            "jobs": [j.to_json() for j in self.jobs.jobs(limit=10)],
            "skills": [s.to_json() for s in self.registry.all()],
            "reports": [
                {"name": r.name, "title": r.title, "date": r.date, "skill": r.skill, "summary": r.summary}
                for r in self.vault.reports(limit=8)
            ],
        }


def make_handler(app: WispApp):
    class Handler(BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"

        def log_message(self, fmt, *args):  # keep the console quiet
            pass

        # -- helpers ----------------------------------------------------------
        def _json(self, payload, status: int = 200) -> None:
            body = json.dumps(payload).encode()
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def _read_body(self) -> dict:
            length = int(self.headers.get("Content-Length") or 0)
            if not length:
                return {}
            try:
                return json.loads(self.rfile.read(length))
            except json.JSONDecodeError:
                return {}

        # -- GET -----------------------------------------------------------------
        def do_GET(self):  # noqa: N802 — http.server API
            path = self.path.split("?", 1)[0]
            if path in ("/", "/index.html"):
                body = HUD_PATH.read_bytes()
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
            elif path == "/api/state":
                self._json(app.state())
            elif path == "/api/skills":
                self._json([s.to_json() for s in app.registry.all()])
            elif path == "/api/jobs":
                self._json([j.to_json() for j in app.jobs.jobs()])
            elif path.startswith("/api/reports/"):
                name = path.rsplit("/", 1)[1]
                found = app.vault.read_report(name)
                if found:
                    meta, body = found
                    self._json({"meta": meta, "body": body})
                else:
                    self._json({"error": "not found"}, status=404)
            elif path == "/api/events":
                self._sse()
            else:
                self._json({"error": "not found"}, status=404)

        # -- POST -----------------------------------------------------------------
        def do_POST(self):  # noqa: N802 — http.server API
            path = self.path.split("?", 1)[0]
            data = self._read_body()
            if path == "/api/say":
                text = (data.get("text") or "").strip()
                if not text:
                    self._json({"error": "text required"}, status=400)
                    return
                self._json(app.say(text))
            elif path.startswith("/api/skills/") and path.endswith("/run"):
                name = path[len("/api/skills/"):-len("/run")]
                if not app.registry.get(name):
                    self._json({"error": f"unknown skill {name}"}, status=404)
                    return
                job = app.jobs.enqueue(name, data.get("text", ""))
                self._json({"job": job.to_json()})
            else:
                self._json({"error": "not found"}, status=404)

        # -- SSE -----------------------------------------------------------------
        def _sse(self) -> None:
            self.send_response(200)
            self.send_header("Content-Type", "text/event-stream")
            self.send_header("Cache-Control", "no-cache")
            self.end_headers()
            q = app.jobs.subscribe()
            try:
                while True:
                    try:
                        msg = q.get(timeout=15)
                        self.wfile.write(f"data: {msg}\n\n".encode())
                    except queue.Empty:
                        self.wfile.write(b": keepalive\n\n")
                    self.wfile.flush()
            except (BrokenPipeError, ConnectionResetError):
                pass
            finally:
                app.jobs.unsubscribe(q)

    return Handler


def serve(config: Config, block: bool = True) -> ThreadingHTTPServer:
    app = WispApp(config)
    server = ThreadingHTTPServer((config.host, config.port), make_handler(app))
    server.wisp_app = app  # type: ignore[attr-defined] — handy for tests
    if block:
        print(f"Wisp OS HUD → {config.base_url}   (Ctrl-C to stop)")
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            server.shutdown()
    else:
        thread = threading.Thread(target=server.serve_forever, daemon=True, name="wisp-http")
        thread.start()
    return server
