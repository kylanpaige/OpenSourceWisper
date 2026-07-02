---
name: research
description: Research a question (codebase or web) and file a cited report in the vault.
triggers: research, look into, dig into, find out about
runner: claude
hud-icon: 🔍
hud-group: knowledge
voice-reply: true
---

# Research

Answer the user's question properly, then file the answer as vault memory.

1. Restate the question in one line.
2. Investigate: prefer the codebase (`crates/`, `RESEARCH.md`, `docs/`, `vault/`) for
   project questions; use web search for external questions when available.
3. Structure the report: **Answer** (the conclusion first), **Evidence** (with paths
   or links), **Open questions**.
4. Keep claims honest — mark anything unverified as such rather than asserting it.
5. File to `vault/reports/` with a two-sentence spoken `summary`.

This skill runs on headless Claude Code when available; there is intentionally no
builtin fallback — deterministic code can't research.
