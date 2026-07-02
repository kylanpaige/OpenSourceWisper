"""Shared fixtures: a temp vault and a temp skills directory."""

from __future__ import annotations

import tempfile
from pathlib import Path

from wisp.config import Config

SKILL_TEMPLATES = {
    "morning-report": """---
name: morning-report
description: Build the morning report from vault and repo state.
triggers: rundown, morning report
runner: builtin
builtin: morning_report
hud-icon: S
hud-group: briefing
voice-reply: true
---
Build the morning report.
""",
    "inbox-brief": """---
name: inbox-brief
description: Triage the vault inbox into a grouped brief.
triggers: inbox brief, triage my inbox
runner: builtin
builtin: inbox_brief
hud-group: briefing
---
Triage the inbox.
""",
    "research": """---
name: research
description: Research a question and file a cited report.
triggers: research
runner: claude
hud-group: knowledge
---
Research the user's question.
""",
}

INBOX_ITEM = """---
from: ACME Corp
subject: Sponsorship for OpenSourceWisper
kind: sponsor
---
We'd love to sponsor your next release.
"""


def make_workspace() -> tuple[tempfile.TemporaryDirectory, Config]:
    tmp = tempfile.TemporaryDirectory(prefix="wisp-test-")
    root = Path(tmp.name)
    skills = root / "skills"
    for name, text in SKILL_TEMPLATES.items():
        (skills / name).mkdir(parents=True)
        (skills / name / "SKILL.md").write_text(text)
    vault = root / "vault"
    (vault / "inbox").mkdir(parents=True)
    (vault / "inbox" / "acme.md").write_text(INBOX_ITEM)
    (vault / "directives").mkdir()
    (vault / "directives" / "current.md").write_text("# Directives\n\n1. Ship the HUD\n2. Write tests\n")
    (vault / "schedule.md").write_text("# Schedule\n\n- 09:00 Deep work\n- 14:00 Review PRs\n")

    config = Config(vault_dir=vault, skills_dir=skills, simulate=True,
                    router_engines=[], port=0)
    return tmp, config
