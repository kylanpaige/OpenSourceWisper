---
name: transcribe
description: Survey/exercise the local speech-to-text stack (faster-whisper, whisper.cpp, wisper-core).
triggers: transcribe, dictate, speech to text
runner: builtin
builtin: transcribe
hud-icon: 🎙
hud-group: voice
---

# Transcribe

The bridge between Wisp OS and this repo's reason to exist.

1. Detect available STT engines: `faster-whisper` (Python), a `whisper.cpp` CLI, and
   the in-repo `wisper-core` engine (audio pipeline lands with the RESEARCH.md Phase 1 MVP).
2. Report what's installed and what the recommended local pipeline is.
3. Once the wisper Phase 1 MVP ships, this skill becomes: capture microphone audio →
   local ASR → cleaned text (via `wisper-core`'s formatting/dictionary pass) → either
   type at the cursor or hand the text to the Wisp router as an utterance.
