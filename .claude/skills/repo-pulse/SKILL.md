---
name: repo-pulse
description: Health check of the OpenSourceWisper repo — branch, cargo check, working tree, recent commits.
triggers: repo pulse, repo status, how's the repo, build status
runner: builtin
builtin: repo_pulse
hud-icon: ⚙
hud-group: repo
---

# Repo Pulse

Snapshot the health of the codebase.

1. Current branch and last 8 commits.
2. `cargo check` result for the Rust workspace (skip gracefully if cargo is absent).
3. Working-tree status (`git status --short`).
4. If the build fails, include the tail of the compiler output in the report.
5. File the report in `vault/reports/` with a one-line spoken summary.
