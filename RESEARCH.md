# OpenSourceWisper — Deep Research: Cloning Wispr Flow as a Fully Local App

> Goal: recreate the base functionality of **Wispr Flow** ("Whisperflow") — press a hotkey, talk, and clean, punctuated text is typed into whatever app you're focused on — but running **entirely on-device** with local models, no cloud.

Research method: fan-out web search across 5 angles → 22 sources fetched → 60 claims extracted → 25 adversarially verified (2-of-3 vote to kill a claim). Key claims below are marked **[verified]**; claims that failed verification are called out explicitly so we don't build on false assumptions.

---

## 1. What Wispr Flow actually does (and where it runs)

Wispr Flow is a two-stage pipeline wrapped in OS-level plumbing:

1. **Global hotkey → capture audio.** Runs in the background; a hotkey starts/stops recording. **[verified]** Marketed as ~4× faster than typing. **[verified]**
2. **Stage 1 — Transcription (ASR).** Speech → raw text.
3. **Stage 2 — AI cleanup (LLM "polish").** A **fine-tuned Llama model** removes filler words ("um"/"uh"), auto-capitalizes, adds punctuation, formats lists, and applies per-app tone. **[verified across vendor + multiple independent 2026 reviews]**
4. **Insert text** at the cursor in whatever app is focused (Gmail, Notion, Slack, VS Code…). **[verified]**
5. Extras: custom dictionary, "Command Mode" for natural-language edits, per-app tone adaptation. **[verified]**

### The critical finding for us: Wispr Flow is **cloud-only**

This is the single most important correction from the research, and it reverses a common assumption:

- **Wispr Flow has NO offline mode.** Its own Help Center publishes a "Connection lost / network issues" article, and states **"Flow requires an internet connection for transcription."** Offline, the desktop app shows "No internet connection" and cannot transcribe. **[a claim that Wispr has an offline mode was REFUTED 0–3 — verifiers found Wispr's own docs plus 5+ independent 2026 reviews all saying "no offline mode and no plans to add one."]**
- **Both stages run in the cloud.** Audio is captured locally, sent over TLS to cloud servers (reportedly AWS us-east-1 via Baseten/OpenAI/Anthropic/Cerebras), transcribed, LLM-polished, and returned. **[verified — "Transcription always happens in the cloud to provide the best speed and accuracy."]**
- **"Privacy Mode" only changes retention, not location.** Audio is still sent to the cloud; it's just transcribed in real time and discarded rather than stored. Cloud Sync toggles whether transcripts are retained. **[verified]**

**Implication:** A fully-local OpenSourceWisper isn't just a clone — it's **strictly more private than the original** (no audio ever leaves the machine). The tradeoff we accept is doing ASR + LLM cleanup on our own hardware within a tight latency budget. The whole product is reproducible from open components; nothing about Wispr's design requires the cloud except their choice to use large server-side models.

---

## 2. Core building blocks for a local implementation

The pipeline maps cleanly to five local subsystems:

```
[hotkey] → [audio capture + VAD] → [local ASR] → [local LLM cleanup] → [text injection]
```

### (a) Audio capture + Voice Activity Detection (VAD)
- **Capture:** `cpal` (Rust, cross-platform) or `sounddevice`/`ffmpeg` (Python). Handy uses `cpal` + `rubato` for resampling to 16 kHz mono. **[verified]**
- **VAD:** **Silero VAD** is the de-facto choice (Handy uses it via `vad-rs`). **[verified]** Used to auto-stop on silence (e.g. local-whisper stops after 3 s below −40 dB **[verified]**). VAD both trims silence for accuracy and marks utterance end for latency.

### (b) Local ASR models — the core choice
| Model / engine | Strength | Latency (typical) | Notes |
|---|---|---|---|
| **whisper.cpp** (GGML/GGUF) | Runs everywhere: CPU / Metal / CUDA / Vulkan | ~10× real-time on Apple Silicon w/ Metal | Best cross-platform default; has `--stream` + VAD. **[verified]** |
| **faster-whisper** (CTranslate2) | Fastest on NVIDIA CUDA; batched | 0.5–2 s; ~3× real-time CPU-only on Mac | Python-friendly; wins on NVIDIA. **[verified]** |
| **NVIDIA Parakeet** (TDT / FastConformer) | Very high throughput, **strong CPU perf** | **~5× real-time CPU-only on a mid-range i5** | CPU-optimized V3 with auto language detect; used by Handy/VoiceInk/OpenWhispr. **[verified]** |
| **Moonshine v2** (Tiny/Small/Medium, 34M/123M/245M) | Lowest latency, constant time-to-first-token | **50 / 148 / 258 ms on Apple M3** | Matches Whisper Large v3 accuracy at ~6× smaller; built for streaming. **[verified]** |
| Vosk | Fully offline, lightweight | fast | Older accuracy; fallback/edge option. |

Accuracy anchors (WER): Moonshine Tiny 12.0%, Small 7.84%, Medium 6.65% **[verified]**; Whisper-large-v3-turbo is most robust on noisy/accented audio **[verified]**. Note NeMo/Parakeet WER numbers run ~2–3% inflated vs normalized scores because they emit punctuation. **[verified]**

**Recommendation:** default to **whisper.cpp** for portability; offer **Parakeet V3** for great CPU-only speed and **Moonshine v2** where sub-300 ms latency matters.

### (c) Streaming vs batch (how to get low latency)
- **Batch** (record whole utterance → transcribe once) is simplest and what most clones do (WhisperWriter is batch **[verified]**). Fine for push-to-talk of a sentence or two.
- **Streaming** hides latency by transcribing while you talk. Streaming encoders (Moonshine's "ergodic streaming encoder" / NVIDIA Cache-Aware FastConformer) encode each frame once and reuse cached state, so time-to-first-token stays constant regardless of clip length. **[verified]**
- **Two-pass trick (borrow this):** local-whisper transcribes a fast partial pass with `tiny` for instant feedback, then a final higher-accuracy pass with the selected model before inserting. **[verified]** Great UX compromise without full streaming complexity.

### (d) Local LLM cleanup (the "polish" stage)
- Run a small instruct model via **Ollama** or **llama.cpp**. Working example: local-whisper pairs whisper.cpp with an **Ollama** pass that fixes punctuation, removes fillers, formats lists — fully on-device, and only runs on text >50 chars to save latency. **[verified]** Tambourine Voice does the same with a customizable prompt split into core rules / filler removal / punctuation sections. **[verified — borrowable prompt template]**
- **Prompt shape** (from fluidvox teardown): *"Clean this transcribed speech. Remove filler words. Add punctuation. Fix grammar. Apply casual/professional/technical tone for the active app,"* returning polished text in ~200–500 ms. **[verified as the described design]**
- **Latency budget:** on GPU, cleanup of a short transcript is cheap — ~42 tok/s (RTX 3060, 8B), ~52 (4070), ~104 (4090); **CPU-only drops to ~8–12 tok/s**, which is the main risk for the cleanup stage on laptops without a GPU. **[verified]** Mitigations: use a 1–3B model (Qwen/Gemma/Llama-3.2-3B), keep outputs short, skip the LLM for very short utterances, or make cleanup optional.

### (e) Global hotkey + text injection (the hardest cross-platform piece)
- **Hotkey capture:** `rdev` (Rust, used by Handy) or `pynput`/`keyboard` (Python). Supports hold-to-talk (push-to-talk) and press-to-toggle.
- **Text injection:** **`enigo`** (Rust) is the canonical cross-platform input-simulation lib — full support on Windows + macOS, X11 stable, **Wayland/libei still experimental**. Injects into the focused app. **[verified]**
- **macOS/Windows:** simulated keystrokes or paste generally work (macOS needs Accessibility permission). **[verified — OS security may block injection without the grant]**
- **Linux is the pain point:** Wayland's security model blocks global input simulation, so you need a backend chain: **`xdotool` (X11)** and **`wtype` / `ydotool` / `dotool` (Wayland)**, with an IBus/FCITX path on some compositors. **[verified]** Robust real-world pattern (hyprvoice/Voxtype): try `wtype → dotool → ydotool → clipboard-paste`, and **restore the user's prior clipboard** after a paste-fallback. **[verified]** Preserve keyboard layout on Wayland (don't run `setxkbmap`). **[verified]**
- **Paste vs keystroke:** pasting (set clipboard → Ctrl/Cmd+V → restore clipboard) is faster and more reliable for long text than simulating each keystroke; several clones use paste. **[verified — local-whisper, Handy]**

---

## 3. Existing open-source clones (what to borrow)

| Project | Stack | Local ASR | LLM cleanup | Platforms | Why it matters |
|---|---|---|---|---|---|
| **Handy** ⭐ | Tauri (Rust + React) | whisper.cpp (GGML) + **Parakeet V3** | — | mac/Win/Linux | **Closest architectural match.** cpal + Silero VAD + rubato + rdev + text injection. Best reference for the whole Rust pipeline. **[verified]** |
| **VoiceInk** | Native Swift (macOS 14.4+) | whisper.cpp + Parakeet (FluidAudio) | yes (local or cloud) | macOS | Best for **Power Mode (per-app context)** + **personal dictionary**. GPLv3. Note: also supports **optional cloud** providers — *not* purely local. **[claim of "100% local" was REFUTED — it has opt-in cloud backends]** **[verified: features]** |
| **WhisperWriter** | Python | faster-whisper (or OpenAI API) | — | cross-platform | Simplest codebase to copy the core loop. Batch mode; default recording mode is **continuous/toggle, not push-to-talk** (PTT is a non-default `hold_to_record` mode). **[the "default push-to-talk" claim was REFUTED]** Injects via `pynput`. **[verified]** |
| **OpenWhispr** | Electron (React 19, TS) | Whisper + Parakeet (whisper.cpp, sherpa-onnx) | yes (local or BYOK cloud) | mac/Win/Linux | Good **hybrid local/cloud** reference + cross-platform paste injection. **[verified]** |
| **local-whisper** | Hammerspoon + ffmpeg | whisper.cpp | **Ollama (local)** | macOS | Best reference for **two-pass transcription + on-device LLM cleanup**. **[verified]** |
| **Tambourine Voice** | Tauri + Python/FastAPI + Pipecat | Faster-Whisper / MLX Whisper | **Ollama (local)** | desktop | Borrowable **customizable cleanup-prompt template** + dictionary. **[verified]** |
| **Vocalinux** | — | whisper.cpp / Whisper / **Vosk** | — | Linux | Best reference for **Wayland/X11 injection** + Vulkan GPU accel (AMD/Intel/NVIDIA). **[verified]** |
| VoiceTypr | Tauri + Rust | Whisper (offline) | yes | mac/Win | Second data point on the Rust/Tauri approach + packaging. |

**Takeaway:** we don't need to invent anything. **Handy is the reference architecture**; VoiceInk shows the advanced features (context/dictionary); local-whisper/Tambourine show the local-LLM cleanup; Vocalinux/hyprvoice solve Linux injection.

---

## 4. Hardware & realistic expectations

- **Apple Silicon (M1–M4):** the sweet spot. whisper.cpp + Metal hits ~10× real-time for tiny/base/small; large-v3 ~2–3× real-time on M3/M4. Moonshine hits 50–258 ms latency on M3. **[verified]** Excellent local dictation experience.
- **NVIDIA GPU:** faster-whisper on CUDA is fastest for ASR; LLM cleanup is cheap (42–104 tok/s). Best for a full clone with instant LLM polish. **[verified]**
- **CPU-only laptop:** viable but tighter. **Parakeet V3 runs ~5× real-time CPU-only on a mid-range i5** for ASR **[verified]** — the ASR half is fine. The risk is the **LLM cleanup at ~8–12 tok/s** **[verified]**; mitigate with a 1–3B model, short outputs, or making cleanup optional/skipped for short utterances.

**Rule of thumb:** ASR is comfortably real-time on almost any modern machine; the LLM cleanup stage is what to size to the user's hardware (or make optional on CPU-only).

---

## 5. Recommended architecture & phased build plan

### Recommended stack
- **Shell:** **Tauri (Rust + web frontend)** — mirrors Handy, small binaries, native OS access. (Python is fine for a faster prototype; Electron/OpenWhispr if you prefer JS.)
- **Audio:** `cpal` + **Silero VAD** (`vad-rs`) + `rubato` (16 kHz mono).
- **ASR:** **whisper.cpp** default (Metal/CUDA/Vulkan/CPU); **Parakeet V3** option for CPU-only speed; **Moonshine v2** option for lowest latency.
- **Cleanup LLM:** **Ollama** running a 1–3B instruct model (Qwen2.5-3B / Llama-3.2-3B / Gemma) with a sectioned cleanup prompt; optional and hardware-gated.
- **Hotkey:** `rdev`. **Injection:** `enigo` on mac/Win; on Linux a `wtype→ydotool→clipboard-paste` fallback chain with clipboard restore.

### Phases

**Phase 0 — Spike (days).** Python + `pynput` + faster-whisper (or whisper.cpp): hold hotkey → record → batch-transcribe → paste at cursor. Proves the end-to-end loop on your primary OS. (This is essentially WhisperWriter.)

**Phase 1 — MVP.** Port to Tauin/Rust (Handy-style): `cpal` capture, Silero VAD auto-stop, whisper.cpp ASR, `enigo` injection, configurable global hotkey with **push-to-talk (hold) + toggle** modes, system tray, model picker. No LLM yet — raw (but punctuated, via Whisper) text. Ship cross-platform mac/Win first.

**Phase 2 — The "Flow" feel (AI cleanup).** Add the Stage-2 **local LLM polish** via Ollama: filler removal, punctuation/grammar, list formatting, with a customizable prompt (borrow Tambourine's sectioned template). Add the **two-pass** trick (tiny partial → final) for perceived latency. Gate LLM on hardware / make optional.

**Phase 3 — Full clone parity.** Custom **dictionary**/text-replacements (VoiceInk-style), **per-app context/tone** ("Power Mode": detect focused app → pick prompt profile), Linux/Wayland injection hardening (backend chain + clipboard restore), Moonshine/Parakeet streaming for real-time partials, "Command Mode" natural-language edits.

**Phase 4 — Polish & distribution.** Auto-update, signed builds, onboarding for OS permissions (macOS Accessibility/Mic, Wayland tooling), model download manager, benchmarks per hardware tier.

### What makes ours different (and better on privacy)
Because Wispr itself is cloud-only, a fully-local OpenSourceWisper is a genuine improvement on its core privacy weakness — **audio and transcripts never leave the device** — at the cost of sizing models to local hardware. Every subsystem has a proven open-source reference implementation, so this is an integration project, not a research one.

---

## Sources
Primary (vendor / project repos): wisprflow.ai/features, wisprflow.ai/privacy, docs.wisprflow.ai; github.com/cjpais/handy, github.com/Beingpax/VoiceInk, github.com/savbell/whisper-writer, github.com/OpenWhispr/openwhispr, github.com/luisalima/local-whisper, github.com/kstonekuan/tambourine-voice, github.com/enigo-rs/enigo, vocalinux.com/wayland, github.com/LeonardoTrapani/hyprvoice, voxtype.io.
Benchmarks / technical: arxiv.org/html/2602.12241v1 (Moonshine v2), e2enetworks.com (Parakeet vs Whisper on L4), modelslab.com (Moonshine vs Whisper), promptquorum.com (whisper.cpp vs faster-whisper), voicci.com (Apple Silicon Whisper), singhajit.com (local LLM tok/s).
Independent Wispr reviews corroborating cloud-only + filler removal: willowvoice.com, getvoibe.com, spokenly.app, chrismenardtraining.com, letterly.app, kintal.co.

*Adversarial verification killed 3 claims: (1) "Wispr Flow has an offline mode" — false, it's cloud-only; (2) "VoiceInk is 100% local" — false, it has opt-in cloud backends; (3) "WhisperWriter defaults to push-to-talk" — false, it defaults to continuous/toggle mode.*
