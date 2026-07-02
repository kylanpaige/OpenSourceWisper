---
name: inbox-brief
description: Triage everything in vault/inbox into a grouped brief (urgent, leads, sponsors, other).
triggers: inbox brief, inbox audit, triage my inbox, check my inbox
runner: builtin
builtin: inbox_brief
hud-icon: ✉
hud-group: briefing
voice-reply: true
---

# Inbox Brief

Triage `vault/inbox/*.md` into a single actionable brief.

1. Read every inbox item's frontmatter (`from`, `subject`, `kind`) and body.
2. Group by kind, ordered: **urgent → lead → sponsor → other**.
3. For each item give one line: subject, sender, and the first line of the body as a quote.
4. Write the brief to `vault/reports/` with a spoken-style `summary`
   ("Inbox brief done. N threads: …").
5. Do not delete or move inbox items — triage is read-only; clearing the inbox is a
   human decision.
