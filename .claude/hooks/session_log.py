#!/usr/bin/env python3
"""Stop hook: append a session heartbeat to the vault document trail.

Keeps the HUD's "document trail" aware that a Claude Code session touched the
repo, mirroring how skill runs are journaled.
"""

import datetime
import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
log_dir = REPO / "vault" / "log"
try:
    log_dir.mkdir(parents=True, exist_ok=True)
    entry = {
        "ts": datetime.datetime.now().isoformat(timespec="seconds"),
        "note": "log/sessions",
        "action": "claude-code-session",
    }
    with (log_dir / "trail.jsonl").open("a") as fh:
        fh.write(json.dumps(entry) + "\n")
except OSError:
    pass
sys.exit(0)
