---
name: morning-report
description: Build the daily morning report from the vault, repo state and schedule — the "give me the rundown" skill.
triggers: rundown, morning report, morning brief, daily brief, what's happening today
runner: builtin
builtin: morning_report
hud-icon: ☀
hud-group: briefing
voice-reply: true
---

# Morning Report

Assemble the day's rundown and file it in the vault.

1. Gather repo state: current branch, last 5 commits, uncommitted change count.
2. Gather vault state: note/report/inbox counts, current `vault/directives/current.md`,
   today's `vault/schedule.md`.
3. If inbox items are pending, list them and point at the `inbox-brief` skill.
4. Write the report to `vault/reports/YYYY-MM-DD-morning-report.md` with frontmatter
   (`title`, `date`, `skill`, `engine`, `summary`) and wikilinks to related notes.
5. The `summary` must read like something an assistant would *say* out loud in one or
   two sentences — it is fed to TTS and the HUD toast.

When run by headless Claude instead of the builtin handler, you may additionally scan
recent `vault/reports/` for anything time-sensitive and fold the highlights in.
