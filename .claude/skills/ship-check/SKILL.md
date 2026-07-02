---
name: ship-check
description: Pre-commit gate — cargo fmt/check/test plus the wisp test suite, with a pass/fail table.
triggers: ship check, ready to ship, can I commit, run the checks, run tests
runner: builtin
builtin: ship_check
hud-icon: 🚦
hud-group: repo
---

# Ship Check

The gate before any commit. Run, in order, skipping steps whose tool is absent:

1. `cargo fmt --check`
2. `cargo check`
3. `cargo test`
4. `python3 -m unittest discover -s wisp/tests -t . -q`

Produce a pass/fail table; on failure include the tail of the failing step's output.
Summary must state clearly either "all green — clear to commit" or which steps failed.
Never auto-commit — this skill reports, the human (or a supervised agent) commits.
