# OpenSourceWisper

**Fully local voice dictation for macOS and Windows.** Hold a hotkey, talk, release — clean, punctuated, AI-polished text lands in whatever app you're typing in. A open-source clone of [Wispr Flow](https://wisprflow.ai), except nothing ever leaves your computer.

> Wispr Flow sends every word you dictate to cloud servers — its own docs say transcription "always happens in the cloud," and it simply stops working offline. OpenSourceWisper reproduces the whole experience with on-device models: [whisper.cpp](https://github.com/ggerganov/whisper.cpp) for speech-to-text and your local [Ollama](https://ollama.com)/LM Studio LLM for the polish. See [RESEARCH.md](RESEARCH.md) for the deep-dive that shaped this design.

## Features

- 🎙 **Push-to-talk or toggle dictation** with a global hotkey, in any app — email, Slack, your editor, a browser form
- 🧠 **Local speech-to-text** via whisper.cpp (Metal-accelerated on Apple Silicon; optional CUDA build), with a built-in model manager: download, switch, and delete models from the UI
- ✨ **AI cleanup** through any local LLM (Ollama or OpenAI-compatible): removes "um"/"uh", fixes punctuation and grammar, formats spoken lists — and *fails open*, so dictation never blocks on a slow model
- 🗣 **Command Mode**: select text anywhere, hold the command hotkey, and say "make this more formal" — the selection is rewritten in place by your local LLM
- 🎯 **Per-app profiles**: casual tone in Slack, professional in email, technical in your editor — matched against the focused app automatically
- 📖 **Personal dictionary**: spoken → written corrections and snippets, also hinted to both the speech model and the LLM
- ⌨️ **Smart text injection**: paste (with your clipboard restored), simulated keystrokes, or clipboard-only
- 🧾 **Local-only history** with raw-vs-polished audit view, filler-word removal and spoken punctuation ("period", "new line") even with the LLM off, silence auto-stop, floating recording indicator, start/stop chimes, system tray, launch-at-login
- 🔒 **Private by architecture**: no accounts, no telemetry, no network calls except model downloads you trigger and the localhost LLM you configure

## Install / build

Prereqs: [Rust](https://rustup.rs) **1.88+** (the repo pins 1.90 via `rust-toolchain.toml`, which rustup installs automatically — if you have an older Rust, run `rustup update stable` first), Node 20+, and on Windows the [WebView2 runtime](https://developer.microsoft.com/microsoft-edge/webview2/) (preinstalled on Windows 11).

```bash
git clone https://github.com/kylanpaige/OpenSourceWisper
cd OpenSourceWisper/apps/desktop
npm install
npm run tauri dev      # run in dev mode
npm run tauri build    # produce .dmg / .msi installers
```

Or drive it from the repo root (the app lives in `apps/desktop`):

```bash
cd OpenSourceWisper
npm run setup   # installs apps/desktop deps
npm run dev     # runs the app
npm run build   # builds installers
```

CI builds installers for macOS and Windows on every push (see Actions artifacts).

### First run

1. Open the app and click **Download Base (English)** on the Home tab (~142 MB, a good default), or pick another model under **Models**.
2. Allow **microphone** access when prompted.
3. **macOS only:** grant Accessibility permission (System Settings → Privacy & Security → Accessibility) so the app can insert text where you type.
4. Hold the hotkey (`Alt+Space` on macOS, `Ctrl+Alt+Space` on Windows by default), speak, release.

### Optional: AI cleanup

```bash
# install Ollama, then:
ollama pull llama3.2:3b
```

Enable it under **AI Cleanup**, hit **Test connection**, and dictations get the full Wispr-style polish. LM Studio, llama.cpp server, or any OpenAI-compatible endpoint works too. Without it you still get deterministic cleanup (filler removal, capitalization, spoken punctuation).

## Choosing a model

| Model | Size | Best for |
|---|---|---|
| Base (English) | 142 MB | Default — real-time on any modern machine |
| Small (English) | 466 MB | Better accuracy, still fast on Apple Silicon / recent CPUs |
| Large v3 Turbo | 1.6 GB | Best accuracy, robust to noise/accents — Apple Silicon or GPU |
| Large v3 Turbo Q5 | 574 MB | Near-turbo accuracy at a third of the memory |

NVIDIA users can build with `cargo build --features cuda` for GPU transcription.

## Architecture

```
hold hotkey ──▶ cpal mic capture (16 kHz mono) ──▶ whisper.cpp ASR
                                                        │
        personal dictionary + deterministic formatting ◀┘
                          │
        local LLM cleanup (Ollama / OpenAI-compat, fail-open)
                          │
        inject into focused app (paste w/ clipboard restore, or keystrokes)
                          │
        local history (JSONL)
```

- `crates/wisper-core` — platform-agnostic logic: settings, dictionary engine, formatting, prompt builder, LLM client, model catalog, history. Fully unit-tested.
- `apps/desktop` — Tauri v2 app: Rust backend (audio, whisper-rs, global shortcuts, enigo/arboard injection, tray, overlay) + React UI.

## Comparison with Wispr Flow

| | Wispr Flow | OpenSourceWisper |
|---|---|---|
| Transcription | Cloud only (no offline mode) | 100% on-device |
| AI cleanup | Cloud LLM | Your local LLM |
| Audio leaves device | Always | Never |
| Push-to-talk / toggle | ✓ | ✓ |
| Filler removal, punctuation, lists | ✓ | ✓ |
| Per-app tone | ✓ | ✓ |
| Custom dictionary | ✓ | ✓ |
| Command Mode (edit by voice) | ✓ | ✓ |
| Price | Subscription | Free, GPL-3.0 |

## License

GPL-3.0. Built on the shoulders of whisper.cpp, whisper-rs, Tauri, cpal, enigo, and the open-source dictation apps documented in [RESEARCH.md](RESEARCH.md).
