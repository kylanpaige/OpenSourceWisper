---
name: daily-review
description: End-of-day review — what shipped, recent reports, and set tomorrow's top-3 directives.
triggers: daily review, end of day, wrap up the day, set directives
runner: builtin
builtin: daily_review
hud-icon: ✓
hud-group: briefing
voice-reply: true
---

# Daily Review

Close out the day and point tomorrow in the right direction.

1. Summarize today's commits (`git log --since=midnight`).
2. Link the most recent vault reports.
3. Derive **at most three** directives for tomorrow, most important first. Standing
   rules: a pending inbox beats everything except a broken build; a dirty working
   tree should be committed or cleaned; otherwise advance the OpenSourceWisper
   phase plan in RESEARCH.md.
4. Write the directives to `vault/directives/current.md` (numbered list) and the
   review to `vault/reports/`.
