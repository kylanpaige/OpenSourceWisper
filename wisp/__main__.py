"""Wisp OS command line.

    python3 -m wisp serve                 # start the HUD server
    python3 -m wisp ask "the rundown"     # one-shot: route + run + print summary
    python3 -m wisp run morning-report    # run a skill directly
    python3 -m wisp voice                 # interactive voice/text loop with TTS
    python3 -m wisp skills                # list registered skills
    python3 -m wisp status                # engine + vault status
"""

from __future__ import annotations

import argparse
import json
import sys

from . import __version__
from .config import load_config
from .router import CHAT
from .server import WispApp, serve
from .voice import SpeechToText, TextToSpeech


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="wisp", description="Wisp OS — Claude Code agentic OS")
    parser.add_argument("--simulate", action="store_true", help="never invoke the claude CLI")
    sub = parser.add_subparsers(dest="cmd")

    p_serve = sub.add_parser("serve", help="start the HUD server")
    p_serve.add_argument("--port", type=int, default=None)

    p_ask = sub.add_parser("ask", help="route one utterance and run it")
    p_ask.add_argument("text", nargs="+")

    p_run = sub.add_parser("run", help="run a skill by name")
    p_run.add_argument("skill")
    p_run.add_argument("text", nargs="*", default=[])

    sub.add_parser("voice", help="interactive voice/text loop")
    sub.add_parser("skills", help="list skills")
    sub.add_parser("status", help="show engine and vault status")

    args = parser.parse_args(argv)
    config = load_config()
    if args.simulate:
        config.simulate = True

    if args.cmd == "serve":
        if args.port:
            config.port = args.port
        serve(config)
        return 0

    app = WispApp(config)

    if args.cmd == "ask":
        result = app.say(" ".join(args.text))
        decision = result["decision"]
        print(f"[route] {decision['skill']} via {decision['engine']} (confidence {decision['confidence']})")
        if "reply" in result:
            print(result["reply"])
        else:
            job = _wait_for_job(app, result["job"]["id"])
            print(f"[{job.status}] {job.summary}")
            if job.report:
                print(f"[report] vault/reports/{job.report}.md")
        return 0

    if args.cmd == "run":
        outcome = app.dispatcher.run_skill(args.skill, " ".join(args.text))
        print(f"[{outcome.engine}] {outcome.summary}")
        if outcome.report:
            print(f"[report] {outcome.report.path.relative_to(outcome.report.path.parents[2])}")
        return 0 if outcome.ok else 1

    if args.cmd == "voice":
        stt, tts = SpeechToText(config), TextToSpeech(config)
        print(f"Wisp OS voice loop — STT: {stt.engine}, TTS: {tts.engine}. Say 'exit' to leave.")
        for utterance in stt.listen():
            result = app.say(utterance)
            if "reply" in result:
                tts.speak(result["reply"])
            else:
                job = _wait_for_job(app, result["job"]["id"])
                tts.speak(job.summary or f"{job.skill} finished with status {job.status}.")
        return 0

    if args.cmd == "skills":
        for skill in app.registry.all():
            triggers = f"  (say: {', '.join(skill.triggers)})" if skill.triggers else ""
            print(f"{skill.hud_icon} {skill.name:<16} [{skill.runner}] {skill.description}{triggers}")
        return 0

    if args.cmd == "status":
        print(json.dumps({
            "version": __version__,
            "engine": "claude" if app.dispatcher.claude.available() else "builtin/simulate",
            "vitals": app.vitals(),
            "hud": config.base_url,
        }, indent=2))
        return 0

    parser.print_help()
    return 1


def _wait_for_job(app: WispApp, job_id: int):
    import time

    while True:
        job = app.jobs.get(job_id)
        if job and job.status in ("done", "error"):
            return job
        time.sleep(0.2)


if __name__ == "__main__":
    sys.exit(main())
