# OpenSourceWisper + Wisp OS

Two things live in this repo:

1. **OpenSourceWisper** (`crates/`) — a fully local Wispr Flow clone in Rust:
   hotkey → mic → local ASR → local LLM cleanup → text at the cursor.
   The phase plan and verified research live in `RESEARCH.md`.
2. **Wisp OS** (`wisp/`, `.claude/`, `vault/`) — the agentic OS layered on top of
   Claude Code, blueprinted from `docs/video-notes-PW0sgog3kXY.md`:
   voice/text → router → skills → headless Claude Code → vault memory → HUD/TTS.

## Wisp OS conventions (follow these)

- **Skills are the unit of work.** Every recurring workflow becomes a directory under
  `.claude/skills/<name>/SKILL.md` with frontmatter:
  `name`, `description`, `triggers` (comma-separated spoken phrases),
  `runner` (`builtin` | `claude`), optional `builtin` (handler in `wisp/builtins.py`),
  `hud-icon`, `hud-group`, `voice-reply`. The markdown body is both the skill
  instructions Claude Code sees and the prompt for headless runs.
- **The vault is the memory.** All durable output goes to `vault/` as markdown with
  frontmatter (`title`, `date`, `skill`, `engine`, `summary`). `summary` is spoken
  aloud by TTS — write it like speech, max two sentences. Link notes with
  `[[wikilinks]]`. Never edit `Home.md` outside the `wisp:recent-reports` markers.
- **Directives** (`vault/directives/current.md`) are the top-3 priorities; the
  `daily-review` skill owns rewriting them.
- **Run skills** with `python3 -m wisp run <skill>`, or `python3 -m wisp ask "<utterance>"`
  to exercise routing. `python3 -m wisp serve` starts the HUD (default port 8737).
- **Tests**: `python3 -m unittest discover -s wisp/tests -t . -q` must pass, plus
  `cargo fmt --check && cargo check && cargo test` for the Rust side. The `ship-check`
  skill runs the whole gate.
- **Local-first is the product.** Never make core dictation or OS flow depend on the
  network; cloud calls (headless Claude, Anthropic router) must degrade gracefully —
  the router falls back to keyword matching and `runner: claude` skills fall back to
  builtins/stubs.
- The `wisp/` Python package is stdlib-only by design. Optional extras
  (faster-whisper, kokoro, sounddevice) are detected at runtime — never imported at
  module top level.

## Adding a skill (the loop from the video)

1. Notice a manual workflow you repeat.
2. `mkdir .claude/skills/<name>` and write `SKILL.md` (copy an existing one).
3. Deterministic? Add a handler in `wisp/builtins.py` + `runner: builtin`.
   Needs judgment? `runner: claude` and write the body as the prompt.
4. Add trigger phrases you'd naturally say.
5. It instantly appears as a HUD button and a routable voice command — verify with
   `python3 -m wisp skills` and a unit test if the handler is builtin.

## Subagents

- `vault-librarian` — vault curation/queries (vault content only).
- `rust-builder` — crates/ implementation; runs fmt/check/test before reporting done.
- `researcher` — multi-source research filed as a vault report.
