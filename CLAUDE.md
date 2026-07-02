# OpenSourceWisper

A fully-local clone of Wispr Flow: press a hotkey, talk, and clean punctuated
text is typed into the focused app — ASR and LLM cleanup both run on-device,
no cloud.

## Roadmap

`RESEARCH.md` is the design document. **§5 "Recommended architecture & phased
build plan"** is the roadmap; work proceeds phase by phase:

- Phase 1 (done): `wisper-core` — platform-agnostic core logic
- Phase 2: local LLM cleanup wiring via Ollama + two-pass latency trick
- Phase 3: per-app profiles, streaming ASR, Command Mode, Wayland hardening
- Phase 4: polish & distribution

Read the relevant RESEARCH.md sections before starting a phase — key claims
in it are adversarially verified and design choices (e.g. whisper.cpp default,
fail-open cleanup) trace back to it.

## Workspace layout

Cargo workspace (Rust 2021, GPL-3.0):

- `crates/wisper-core` — pure logic + plain HTTP, unit-testable without OS
  windowing/mic/GPU. Modules: `settings`, `dictionary`, `format`, `prompts`,
  `cleanup`, `models`, `history`.
- `apps/desktop` (planned) — Tauri-style shell wiring core to
  cpal/whisper-rs/enigo.

## Commands

```bash
cargo build --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
```

CI (`.github/workflows/ci.yml`) enforces fmt, strict clippy, and tests on
every push/PR. A SessionStart hook (`.claude/hooks/session-start.sh`)
pre-builds the workspace in remote sessions.

## Conventions

- **Fail-open cleanup:** the LLM polish stage must never block dictation —
  any error returns `CleanupOutcome::Skipped(reason)` and the caller falls
  back to the deterministic formatter (`format.rs`).
- **Settings are forward-compatible:** every `AppSettings` field has a serde
  default so old config files keep parsing after upgrades.
- **Sectioned prompts:** the cleanup system prompt is built section-by-section
  (Tambourine-style) in `prompts.rs`; toggles in `CleanupSettings` add or
  remove sections rather than editing one blob.
- **Sanitize LLM output:** small local models misbehave (preambles, fences,
  wrapping quotes, essays); anything reaching the injector goes through
  `sanitize_llm_output`, which falls back to the raw transcript when in doubt.
- Keep `wisper-core` free of OS-specific dependencies; platform glue lives in
  the (future) app crate.
