# Reflection Notes — Claude Code Setup Audit

Date: 2026-07-02
Auditor: Claude Code (remote session, `opensourcewisper-b3`)

## Scope caveat — read this first

The audit was asked to mine "all my Code sessions in `.claude`". **This remote
container is ephemeral and holds exactly one transcript: the current audit
session itself** (`~/.claude/projects/-home-user-OpenSourceWisper/` contains
only `1ebec9ef-….jsonl`; `~/.claude/sessions/` only the live session's
metadata). There are also **zero pull requests** on `kylanpaige/OpenSourceWisper`
to mine.

What *is* recoverable:

| Session | Evidence source | What happened |
|---|---|---|
| S1 — `session_015wkm4s8hJqyHRnnhi1GLeK` | git commits `4887ee3` + `96dd417` (transcript not available) | Deep-research report (`RESEARCH.md`, authored as Opus 4.8) **and** Phase 1 `wisper-core` crate (authored as Fable 5, same session — model switched mid-session) |
| S2 — `session_01AMFCwTYnnT4TA1KitX1edK` (this one) | full transcript on disk | This audit |

So the evidence base is **n = 2 sessions, one of them only via git archaeology**.
Per your own bar ("only propose a skill for something that actually recurs"),
almost nothing clears the recurrence threshold yet. Verdicts below are ranked
by leverage, with confidence stated honestly. **To get the audit you actually
asked for, run this same prompt on the machine where your full
`~/.claude/projects/` history lives** (local install), or in a session with
that history mounted — that's Candidate 1 for a reason.

---

## Ranked candidates

### 1. Re-run this audit where your transcripts live — verdict: **fix (process)**
- **Evidence:** S2. Four shell probes + a GitHub API call all confirmed the
  corpus is absent here (`find ~/.claude/projects -name "*.jsonl"` → 1 file;
  `list_pull_requests` → `[]`).
- **Why top-ranked:** every other candidate is throttled by n=2. The
  cross-session clustering you wanted is only possible against your local
  `~/.claude/projects/`, which accumulates one JSONL per session per project.
- **Build cost:** zero — same prompt, different machine.

### 2. Add a `CLAUDE.md` to OpenSourceWisper — verdict: **fix (small config)**
- **Evidence:** S1 and S2 both had to derive project context from scratch.
  S1 produced a 15 KB `RESEARCH.md` whose §5 phased plan (Phase 0–4) is the
  de facto roadmap; S1's Phase 1 commit already implements part of it. Every
  future "Phase N" session will re-read/re-derive architecture, build
  commands, and conventions unless captured. Repo currently has **no
  CLAUDE.md, no README, no `.claude/` dir** (verified by find/glob).
- **Recurrence:** prospective but near-certain — Phases 2–4 are explicitly
  planned and each will be its own session(s).
- **Proposed content:** workspace layout (`crates/wisper-core`, more crates
  coming per the plan), `cargo test`/`clippy`/`fmt` commands, pointer to
  `RESEARCH.md §5` as the roadmap, GPL-3.0 note, the Tambourine-style prompt
  and fail-open cleanup-client conventions established in Phase 1.
- **Build cost:** minutes (`/init` does most of it).

### 3. Project SessionStart hook for cargo — verdict: **automation**
- **Evidence:** your environment ships a `session-start-hook` skill
  (`~/.claude/skills/session-start-hook/`) that exists precisely to prep
  remote sessions for a repo's toolchain — and it explicitly lists
  `Cargo.toml → cargo` as a target. It has **not been applied**: the repo has
  no `.claude/settings.json` and no hook. Each fresh remote container
  (like this one) starts with a cold cargo cache and no verified toolchain.
- **Recurrence:** every future remote session on this repo pays the cost.
- **Proposed:** `.claude/hooks/session-start.sh` that pre-warms
  `cargo fetch`/`cargo build` and verifies `cargo test` runs, registered in
  repo `.claude/settings.json`.
- **Build cost:** low — the installed skill is a guided workflow for exactly
  this.

### 4. CI: `cargo test` + `clippy` + `fmt` on push — verdict: **automation**
- **Evidence:** S1's Phase 1 commit message claims "24 passing unit tests",
  but there is no `.github/` at all — nothing re-verifies on push. Meanwhile
  the environment's Stop hook (`stop-hook-git-check.sh`) *forces* every
  session to commit and push before ending, so unverified pushes are the
  default failure mode as the codebase grows past one crate.
- **Recurrence:** structural — applies to every session that touches code.
- **Build cost:** low (one workflow file), though note it must be added from
  a session/environment with workflow-write permission.

### 5. A `/reflect` (setup-audit) skill — verdict: **nothing, for now**
- **Evidence:** S2 only. You typed a full methodology inline (subagent
  fan-out → cluster → skill/automation/fix/nothing verdicts → recurrence vs
  build cost → cited evidence → `reflection-notes.md`). That is exactly the
  shape of prompt a skill exists to capture.
- **Why "nothing":** n = 1. By your own recurrence bar, it doesn't qualify.
  **Flag to revisit:** if you run this audit a second time (see Candidate 1),
  that's recurrence — capture the methodology as a skill then, and this file
  becomes its input format.

### 6. MCP surface bloat — verdict: **nothing (observation only)**
- **Evidence:** S2's session had ~291 deferred MCP tools attached (Figma,
  Shopify, Higgsfield, Supabase, Gmail, Calendar, Drive, Notion, Vercel,
  GitHub) for a solo Rust systems project; only GitHub was touched. Harmless
  today (deferred tools cost little until loaded), and likely account-level
  config — but if you notice irrelevant-tool confusion in future sessions,
  a per-project connector trim is the fix.

### 7. Environment hooks — verdict: **nothing (no action available)**
- **Evidence:** the four hooks in `~/.claude/` (git-identity SessionStart,
  git-check Stop, Slack reply-gate, reply reminder) are platform-injected
  CCR infrastructure, not your config. One latent quirk worth knowing: the
  Stop hook blocks turn-end on any untracked file, which is why even a
  "diagnosis-only" session like this one must commit its notes file to
  finish. Not fixable from user land; just expected behavior.

---

## What was explicitly *not* found (friction that usually drives skill proposals)

In the one full transcript available: zero permission denials, zero tool
errors, zero user corrections or retries (`permissionMode: auto`). No
evidence of repeated manual steps across sessions — because there is only
one observable transcript. Absence of evidence here is absence of *data*,
not proof the setup is clean.

## Suggested order of operations

1. Re-run the audit locally against your real session history (Candidate 1).
2. Regardless of that outcome, do Candidates 2 and 3 before the next build
   session — both are minutes of work and pay off on the very next session.
3. Add CI (Candidate 4) once a second crate or the Tauri frontend lands.
4. Revisit the `/reflect` skill (Candidate 5) after audit #2.
