---
title: wisper-core status & Phase 2 (audio capture + VAD) requirements
date: 2026-07-02 14:59
skill: research
engine: claude
summary: wisper-core so far is pure logic — settings, dictionary, formatting, model catalog, prompts and an LLM cleanup client — with no audio, VAD, ASR, hotkey, or injection code anywhere yet. Building audio capture + VAD needs a new crate wired with cpal, Silero VAD (vad-rs) and rubato, though RESEARCH.md's own phase plan actually files that work under Phase 1, not Phase 2 as the directive states.
---

# wisper-core status & Phase 2 (audio capture + VAD) requirements

# wisper-core status & what Phase 2 (audio capture + VAD) needs

## Question
What has `wisper-core` implemented so far, and what does "Phase 2 (audio capture + VAD)" need, per `RESEARCH.md` and the `crates/` code?

## Answer

**Implemented so far:** `crates/wisper-core` is a single, platform-agnostic Rust crate (1,204 LOC across 7 modules) covering settings, dictionary/text-replacement, deterministic transcript formatting, a whisper.cpp model catalog, an LLM-cleanup prompt builder, a local-LLM (Ollama / OpenAI-compatible) HTTP cleanup client, and dictation history. There is **no audio capture, no VAD, no ASR execution, no hotkey, and no text injection code anywhere in the repo** — the workspace has exactly one member (`crates/wisper-core`); no `apps/desktop` crate exists yet.

**Important correction before scoping "Phase 2":** the session directive says "Start wisper-core Phase 2: audio capture + VAD (see RESEARCH.md §2a)," but `RESEARCH.md`'s own phase plan (§5) puts audio capture + VAD in **Phase 1 — MVP**, not Phase 2. `RESEARCH.md` §2(a) is a *building-block* subsection (not a phase), and §5's **Phase 2 is "The 'Flow' feel (AI cleanup)"** — i.e. the local-LLM polish stage. Confusingly, what's actually been built (`cleanup.rs`, `prompts.rs`) maps to RESEARCH.md's Phase 2 content, while the committed history labels that same work "Phase 1" (commit `96dd417`). So by RESEARCH.md's own numbering, audio capture + VAD is unstarted **Phase 1** work, not Phase 2 — worth resolving the label before continuing, though the technical requirements below hold regardless of the number assigned.

**What audio capture + VAD needs, concretely:**
1. A new crate (workspace currently has only `crates/wisper-core`, kept deliberately free of audio/OS deps per its own doc comment in `lib.rs`: "The desktop app (apps/desktop) wires these pieces to cpal/whisper-rs/enigo/Tauri"). Either stand up `apps/desktop` now or add an interim `crates/wisper-audio` crate, and add it to `[workspace] members` in the root `Cargo.toml`.
2. Dependencies per RESEARCH.md §2(a): `cpal` for cross-platform capture, `rubato` to resample to 16 kHz mono, and Silero VAD via the `vad-rs` crate for auto-stop-on-silence (Handy's reference stack). None of these appear in any `Cargo.toml` in the repo yet.
3. Wire to the recording-trigger semantics already defined in `settings.rs`: `RecordingMode::{PushToTalk, Toggle}` (default `PushToTalk`) — the audio module needs to start/stop capture on hotkey events (hotkey capture itself, e.g. `rdev`, is also unimplemented and is a separate Phase 1 piece).
4. VAD auto-stop threshold/behavior (RESEARCH.md cites local-whisper's "stop after 3s below −40 dB" as a reference default) and a buffer format that feeds directly into the (also unimplemented) whisper.cpp ASR stage.
5. Tests: the existing 7 modules are all unit-tested (pure logic, no hardware). Audio capture is hardware-dependent, so the new code will need a mockable/injectable capture abstraction to keep test coverage meaningful, per this repo's "ship-check" gate expectations.

Note: `cargo test`/`cargo check` were not re-run in this session (approval for the command wasn't granted), so current green/red build status is unverified here — `repo-pulse` last reported `cargo check: ✅ clean` on this branch earlier today.

## Evidence
- `crates/wisper-core/src/lib.rs:1-15` — module list and the "apps/desktop wires cpal/whisper-rs/enigo/Tauri" comment.
- `crates/wisper-core/src/{cleanup,dictionary,format,history,models,prompts,settings}.rs` — 1,204 LOC total, each with `#[cfg(test)]` coverage.
- `crates/wisper-core/Cargo.toml` — deps are `serde`, `serde_json`, `regex`, `thiserror`, `chrono`, `reqwest`; no `cpal`/`vad-rs`/`rubato`/`rdev`/`enigo`.
- `Cargo.toml` (workspace root) — `members = ["crates/wisper-core"]` only.
- `RESEARCH.md` §2(a) "Audio capture + Voice Activity Detection (VAD)" — recommends `cpal` + Silero VAD (`vad-rs`) + `rubato`.
- `RESEARCH.md` §5 "Phases" — Phase 0 spike, Phase 1 MVP (`cpal` capture, Silero VAD auto-stop, whisper.cpp ASR, `enigo` injection, hotkey, model picker), Phase 2 "AI cleanup" (Ollama LLM polish, two-pass trick), Phase 3 (dictionary, per-app tone, Linux/Wayland hardening), Phase 4 (distribution).
- `git log --oneline`: `96dd417 Phase 1: wisper-core crate — settings, dictionary, formatting, prompts, models, history, cleanup client`; `4887ee3 Add deep-research report...`.
- `vault/reports/2026-07-02-repo-pulse.md` — last known `cargo check: ✅ clean` on branch `claude/custom-claude-agentic-os-rwlqy4`.

## Open questions
- Should the phase numbering be reconciled (directive says "Phase 2" for audio/VAD, RESEARCH.md's own plan calls it Phase 1, and the last commit labeled unrelated cleanup/dictionary/settings work "Phase 1")? Recommend renaming/relabeling before more work lands so directives and RESEARCH.md agree.
- Where should audio/ASR/injection code live — a new `apps/desktop` (Tauri, as `lib.rs` implies) or an interim headless `crates/wisper-audio` + `crates/wisper-asr` split? Not decided anywhere in the repo yet.
- `cargo test`/`cargo fmt --check` were not re-run in this session (tool approval declined) — status per `repo-pulse` is stale as of its last run today.
