"""Voice adapters: local STT in, local TTS out — every stage optional.

STT chain:  faster-whisper (if installed) → stdin ("type instead of talk").
            The wisper-core engine from this repo slots in here once its audio
            pipeline lands (RESEARCH.md Phase 1 MVP) — same contract: text in, text out.
TTS chain:  kokoro (if installed) → macOS `say` → `espeak`/`espeak-ng` → print.

"Completely local, which allows it to be relatively fast and snappy compared to
routing this all through something like ElevenLabs."
"""

from __future__ import annotations

import shutil
import subprocess
import sys

from .config import Config


class SpeechToText:
    """Yields utterances from the best available input source."""

    def __init__(self, config: Config):
        self.config = config
        self.engine = self._pick_engine()

    def _pick_engine(self) -> str:
        for name in self.config.stt_engines:
            if name == "faster-whisper" and _importable("faster_whisper") and _importable("sounddevice"):
                return "faster-whisper"
            if name == "stdin":
                return "stdin"
        return "stdin"

    def listen(self):
        if self.engine == "faster-whisper":
            yield from self._listen_faster_whisper()
        else:
            yield from self._listen_stdin()

    def _listen_stdin(self):
        prompt = f"{self.config.assistant_name}> "
        while True:
            try:
                line = input(prompt)
            except (EOFError, KeyboardInterrupt):
                return
            line = line.strip()
            if line.lower() in ("exit", "quit"):
                return
            if line:
                yield line

    def _listen_faster_whisper(self):
        """Push-to-talk loop: Enter to record ~6s, Enter again to stop early."""
        import numpy as np  # noqa: F401 — faster-whisper deps guarantee numpy
        import sounddevice as sd
        from faster_whisper import WhisperModel

        model = WhisperModel("base.en", device="auto", compute_type="int8")
        samplerate = 16000
        print("[voice] faster-whisper ready — press Enter, speak, wait for transcript. Ctrl-C to exit.")
        while True:
            try:
                input("[voice] press Enter to record 6s… ")
            except (EOFError, KeyboardInterrupt):
                return
            audio = sd.rec(int(6 * samplerate), samplerate=samplerate, channels=1, dtype="float32")
            sd.wait()
            segments, _ = model.transcribe(audio.flatten(), language="en", vad_filter=True)
            text = " ".join(seg.text.strip() for seg in segments).strip()
            if text:
                print(f"[voice] heard: {text}")
                yield text


class TextToSpeech:
    def __init__(self, config: Config):
        self.config = config
        self.engine = self._pick_engine()

    def _pick_engine(self) -> str:
        for name in self.config.tts_engines:
            if name == "kokoro" and _importable("kokoro"):
                return "kokoro"
            if name == "say" and shutil.which("say"):
                return "say"
            if name == "espeak" and (shutil.which("espeak-ng") or shutil.which("espeak")):
                return "espeak"
            if name == "print":
                return "print"
        return "print"

    def speak(self, text: str) -> None:
        text = text.strip()
        if not text:
            return
        if self.engine == "kokoro":
            self._speak_kokoro(text)
        elif self.engine == "say":
            subprocess.run(["say", text], check=False)
        elif self.engine == "espeak":
            binary = shutil.which("espeak-ng") or "espeak"
            subprocess.run([binary, text], check=False)
        else:
            print(f"🔈 {text}", file=sys.stderr)

    def _speak_kokoro(self, text: str) -> None:
        try:
            import numpy as np
            import sounddevice as sd
            from kokoro import KPipeline

            pipeline = KPipeline(lang_code="a")
            chunks = [audio for _, _, audio in pipeline(text, voice="af_heart")]
            if chunks:
                sd.play(np.concatenate(chunks), samplerate=24000)
                sd.wait()
        except Exception:  # noqa: BLE001 — never let TTS take down the loop
            print(f"🔈 {text}", file=sys.stderr)


def _importable(module: str) -> bool:
    try:
        __import__(module)
        return True
    except Exception:  # noqa: BLE001 — broken optional deps count as absent
        return False
