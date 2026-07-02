import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AppSettings, DictationResult, DownloadProgress, ModelStatus, Phase } from "../types";

function SetupCard({ onReady }: { onReady: () => void }) {
  const [recommended, setRecommended] = useState<ModelStatus | null>(null);
  const [pct, setPct] = useState<number | null>(null);
  const [error, setError] = useState("");

  useEffect(() => {
    invoke<ModelStatus[]>("list_models").then((ms) => {
      setRecommended(
        ms.find((m) => m.file === "ggml-base.en.bin") ?? ms.find((m) => m.recommended) ?? null,
      );
    });
    const unsubs: Array<() => void> = [];
    listen<DownloadProgress>("download-progress", (e) => {
      if (e.payload.total > 0) setPct(Math.round((e.payload.downloaded / e.payload.total) * 100));
    }).then((u) => unsubs.push(u));
    listen<{ file: string; error: string }>("download-done", (e) => {
      setPct(null);
      if (e.payload.error) setError(e.payload.error);
      else onReady();
    }).then((u) => unsubs.push(u));
    return () => unsubs.forEach((u) => u());
  }, [onReady]);

  return (
    <div className="card">
      <h3>👋 One-time setup</h3>
      <ol className="muted">
        <li>
          Download a speech model — everything runs on this machine.
          {recommended && (
            <div className="controls">
              {pct === null ? (
                <button
                  className="primary"
                  onClick={() => {
                    setError("");
                    setPct(0);
                    invoke("download_model", { file: recommended.file }).catch((e) => {
                      setError(String(e));
                      setPct(null);
                    });
                  }}
                >
                  Download {recommended.label} ({recommended.size_mb} MB)
                </button>
              ) : (
                <button disabled>Downloading… {pct}%</button>
              )}
            </div>
          )}
        </li>
        <li>Allow microphone access when prompted.</li>
        <li>
          On macOS: grant Accessibility permission (System Settings → Privacy &amp; Security →
          Accessibility) so text can be inserted where you type.
        </li>
      </ol>
      {error && <p className="error small">{error}</p>}
    </div>
  );
}

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
  const [hasModel, setHasModel] = useState(true);

  const checkModels = useCallback(() => {
    invoke<ModelStatus[]>("list_models")
      .then((ms) => setHasModel(ms.some((m) => m.downloaded)))
      .catch(() => {});
  }, []);

  useEffect(() => {
    checkModels();
    let unsub: (() => void) | undefined;
    listen<number>("audio-level", (e) => setLevel(e.payload)).then((u) => (unsub = u));
    return () => unsub?.();
  }, [checkModels]);

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

      {!hasModel && <SetupCard onReady={checkModels} />}

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
