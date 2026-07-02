import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AppSettings, DictationResult, Phase } from "../types";

export default function HomeTab({
  settings,
  phase,
  lastResult,
}: {
  settings: AppSettings;
  phase: Phase;
  lastResult: DictationResult | null;
}) {
  const [level, setLevel] = useState(0);

  useEffect(() => {
    let unsub: (() => void) | undefined;
    listen<number>("audio-level", (e) => setLevel(e.payload)).then((u) => (unsub = u));
    return () => unsub?.();
  }, []);

  const recording = phase === "recording";
  const busy = phase !== "idle";

  return (
    <section>
      <h1>Speak anywhere. Stay local.</h1>
      <p className="muted">
        {settings.recording_mode === "push_to_talk" ? (
          <>
            Hold <kbd>{settings.hotkey}</kbd>, talk, release — clean text lands in whatever app has
            focus.
          </>
        ) : (
          <>
            Press <kbd>{settings.hotkey}</kbd> to start and stop dictation.
          </>
        )}{" "}
        Nothing ever leaves this computer.
      </p>

      <div className="mic-card">
        <div className="mic-meter">
          <div
            className="mic-meter-fill"
            style={{ width: `${Math.min(100, level * 900)}%`, opacity: recording ? 1 : 0.25 }}
          />
        </div>
        <div className="controls">
          {!recording ? (
            <button className="primary" disabled={busy} onClick={() => invoke("start_dictation")}>
              Start dictation
            </button>
          ) : (
            <button className="primary" onClick={() => invoke("stop_dictation")}>
              Stop &amp; insert
            </button>
          )}
          <button disabled={!recording} onClick={() => invoke("cancel_dictation")}>
            Cancel
          </button>
        </div>
      </div>

      {lastResult && (
        <div className="card">
          <h3>Last dictation {lastResult.app && <span className="muted">→ {lastResult.app}</span>}</h3>
          <p className="result">{lastResult.final_text}</p>
          <p className="muted small">
            {(lastResult.duration_ms / 1000).toFixed(1)}s of audio ·{" "}
            {lastResult.cleaned ? "AI-polished" : `formatted (${lastResult.skip_reason || "LLM off"})`}
          </p>
          {lastResult.cleaned && lastResult.raw !== lastResult.final_text && (
            <details>
              <summary className="muted small">Raw transcript</summary>
              <p className="result muted">{lastResult.raw}</p>
            </details>
          )}
        </div>
      )}
    </section>
  );
}
