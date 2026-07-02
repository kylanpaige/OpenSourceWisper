"""Skill dispatch + the headless Claude Code engine.

"We have this headless version of Claude Code. It's like opening up Claude Code
but it's invisible." — the execution engine of the OS.

Engine selection per run:
    runner: claude  → `claude -p <prompt> --output-format json` when the CLI is
                      available and simulate mode is off; on any failure, fall
                      back to the skill's builtin handler; last resort: a stub
                      report so a HUD click never dead-ends.
    runner: builtin → the deterministic handler in wisp.builtins.

Every run is journaled to vault/log/runs.jsonl and its report written to the
vault, so the terminal (Claude Code), the HUD and Obsidian all see the same
memory.
"""

from __future__ import annotations

import datetime as _dt
import json
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path

from . import REPO_ROOT
from .builtins import HANDLERS, SkillContext, SkillResult
from .config import Config
from .skills import Skill, SkillRegistry
from .vault import Report, Vault


@dataclass
class RunOutcome:
    skill: str
    engine: str          # claude | builtin | stub
    summary: str
    report: Report | None
    ok: bool = True

    def to_json(self) -> dict:
        return {
            "skill": self.skill,
            "engine": self.engine,
            "summary": self.summary,
            "report": self.report.name if self.report else None,
            "ok": self.ok,
        }


class ClaudeEngine:
    """Wraps headless `claude -p` invocations."""

    def __init__(self, config: Config):
        self.config = config

    def available(self) -> bool:
        return not self.config.simulate and shutil.which(self.config.claude_bin) is not None

    def run(self, prompt: str, cwd: Path = REPO_ROOT) -> str | None:
        """Return the assistant's final text, or None on failure."""
        cmd = [
            self.config.claude_bin, "-p", prompt,
            "--output-format", "json",
            "--permission-mode", self.config.claude_permission_mode,
        ]
        try:
            proc = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True,
                                  timeout=self.config.claude_timeout_s)
        except (subprocess.TimeoutExpired, OSError):
            return None
        if proc.returncode != 0:
            return None
        try:
            data = json.loads(proc.stdout)
        except json.JSONDecodeError:
            return proc.stdout.strip() or None
        if isinstance(data, dict):
            if data.get("is_error"):
                return None
            return data.get("result") or None
        return None


class Dispatcher:
    def __init__(self, config: Config, registry: SkillRegistry, vault: Vault):
        self.config = config
        self.registry = registry
        self.vault = vault
        self.claude = ClaudeEngine(config)

    # -- engines -----------------------------------------------------------
    def _run_claude(self, skill: Skill, text: str) -> RunOutcome | None:
        prompt = (
            f"You are running the '{skill.name}' skill of Wisp OS headlessly.\n\n"
            f"{skill.body}\n\n"
            + (f"User request: {text}\n\n" if text else "")
            + "Write the report as instructed, then reply with ONLY a JSON object "
              '{"title": "...", "summary": "<= 2 spoken-style sentences", "body": "<markdown report>"} '
              "and nothing else."
        )
        raw = self.claude.run(prompt)
        if not raw:
            return None
        title, summary, body = _parse_report_json(raw, skill.name)
        report = self.vault.write_report(skill.name, title, body, summary, engine="claude")
        return RunOutcome(skill.name, "claude", summary, report)

    def _run_builtin(self, skill: Skill, text: str) -> RunOutcome | None:
        handler = HANDLERS.get(skill.builtin or skill.name.replace("-", "_"))
        if not handler:
            return None
        ctx = SkillContext(config=self.config, vault=self.vault, repo_root=REPO_ROOT, text=text)
        result: SkillResult = handler(ctx)
        if result.directives:
            self.vault.set_directives(result.directives)
        report = self.vault.write_report(skill.name, result.title, result.body,
                                         result.summary, engine="builtin", tags=result.tags)
        return RunOutcome(skill.name, "builtin", result.summary, report)

    def _run_stub(self, skill: Skill, text: str) -> RunOutcome:
        body = (
            f"Skill `{skill.name}` was invoked but no engine could run it "
            "(claude CLI unavailable/failed and no builtin handler).\n\n"
            f"Instructions that would have been executed:\n\n{skill.body}"
        )
        summary = f"{skill.name} queued as a stub — no execution engine available."
        report = self.vault.write_report(skill.name, f"{skill.name} (stub)", body, summary, engine="stub")
        return RunOutcome(skill.name, "stub", summary, report, ok=False)

    # -- public ------------------------------------------------------------
    def run_skill(self, name: str, text: str = "") -> RunOutcome:
        skill = self.registry.get(name)
        if not skill:
            return RunOutcome(name, "stub", f"Unknown skill: {name}", None, ok=False)

        outcome: RunOutcome | None = None
        if skill.runner == "claude" and self.claude.available():
            outcome = self._run_claude(skill, text)
        if outcome is None:
            outcome = self._run_builtin(skill, text)
        if outcome is None:
            outcome = self._run_stub(skill, text)
        self._journal(outcome, text)
        return outcome

    def chat(self, text: str) -> str:
        """Conversational fallback when routing says no skill applies."""
        if self.claude.available():
            raw = self.claude.run(
                "You are Wisp, the voice of the OpenSourceWisper agentic OS. "
                "Answer briefly (1-3 spoken-style sentences), using the vault/ and repo "
                f"context if helpful. User said: {text}"
            )
            if raw:
                return raw.strip()
        skills = ", ".join(s.name for s in self.registry.all())
        return (
            "I couldn't match that to a skill and no language model is configured. "
            f"Try one of: {skills}."
        )

    def _journal(self, outcome: RunOutcome, text: str) -> None:
        log_dir = self.vault.root / "log"
        log_dir.mkdir(parents=True, exist_ok=True)
        entry = {"ts": _dt.datetime.now().isoformat(timespec="seconds"), "text": text, **outcome.to_json()}
        with (log_dir / "runs.jsonl").open("a") as fh:
            fh.write(json.dumps(entry) + "\n")


def _parse_report_json(raw: str, skill_name: str) -> tuple[str, str, str]:
    """Extract {title, summary, body} from a headless-Claude reply, robustly."""
    start = raw.find("{")
    if start >= 0:
        depth, in_str, esc = 0, False, False
        for i in range(start, len(raw)):
            c = raw[i]
            if esc:
                esc = False
                continue
            if c == "\\":
                esc = True
                continue
            if c == '"':
                in_str = not in_str
            if in_str:
                continue
            if c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    try:
                        data = json.loads(raw[start:i + 1])
                        title = data.get("title") or f"{skill_name} report"
                        summary = data.get("summary") or title
                        body = data.get("body") or raw
                        return title, summary, body
                    except json.JSONDecodeError:
                        break
    date = _dt.date.today().isoformat()
    return f"{skill_name} {date}", raw.strip().splitlines()[0][:160], raw
