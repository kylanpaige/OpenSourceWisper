#!/usr/bin/env python3
"""SessionStart hook: brief Claude Code on the OS state at the top of a session.

Prints the current directives, schedule and freshest reports so every session
starts with the same working memory the HUD and voice loop see.
"""

import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO))

try:
    from wisp.vault import Vault
except ImportError:
    sys.exit(0)  # never block a session on OS plumbing

vault = Vault(REPO / "vault")
if not vault.root.exists():
    sys.exit(0)

lines = ["[Wisp OS] Session briefing from vault/:"]
directives = vault.directives()
if directives:
    lines.append("Directives: " + "; ".join(f"({i}) {d}" for i, d in enumerate(directives, 1)))
schedule = vault.schedule()
if schedule:
    lines.append("Schedule: " + ", ".join(f"{s['time']} {s['item']}" for s in schedule))
reports = vault.reports(limit=3)
if reports:
    lines.append("Recent reports: " + "; ".join(f"{r.title} — {r.summary[:60]}" for r in reports))
stats = vault.stats()
lines.append(f"Vault: {stats['notes']} notes, {stats['inbox']} inbox items pending. "
             "Skills live in .claude/skills/; run them via `python3 -m wisp run <skill>`.")

print("\n".join(lines))
