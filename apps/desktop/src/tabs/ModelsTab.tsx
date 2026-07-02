import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AppSettings, DownloadDone, DownloadProgress, ModelStatus } from "../types";

export default function ModelsTab({
  settings,
  onSave,
}: {
  settings: AppSettings;
  onSave: (s: AppSettings) => void;
}) {
  const [models, setModels] = useState<ModelStatus[]>([]);
  const [progress, setProgress] = useState<Record<string, DownloadProgress>>({});
  const [error, setError] = useState("");

  const reload = useCallback(() => {
    invoke<ModelStatus[]>("list_models").then(setModels).catch((e) => setError(String(e)));
  }, []);

  useEffect(() => {
    reload();
    const unsubs: Array<() => void> = [];
    listen<DownloadProgress>("download-progress", (e) => {
      setProgress((p) => ({ ...p, [e.payload.file]: e.payload }));
    }).then((u) => unsubs.push(u));
    listen<DownloadDone>("download-done", (e) => {
      setProgress((p) => {
        const next = { ...p };
        delete next[e.payload.file];
        return next;
      });
      if (e.payload.error) setError(e.payload.error);
      reload();
    }).then((u) => unsubs.push(u));
    return () => unsubs.forEach((u) => u());
  }, [reload]);

  const download = (file: string) => {
    setError("");
    invoke("download_model", { file }).catch((e) => setError(String(e)));
    setModels((ms) => ms.map((m) => (m.file === file ? { ...m, downloading: true } : m)));
  };

  const remove = (file: string) => {
    invoke("delete_model", { file }).then(reload).catch((e) => setError(String(e)));
  };

  const activate = (file: string) => {
    onSave({ ...settings, model: file });
    setModels((ms) => ms.map((m) => ({ ...m, active: m.file === file })));
  };

  return (
    <section>
      <h1>Speech models</h1>
      <p className="muted">
        Models run fully on this machine via whisper.cpp. Bigger models are more accurate but
        slower; start with Base and move up if your hardware keeps up.
      </p>
      {error && <div className="banner error">{error}</div>}
      <div className="model-list">
        {models.map((m) => {
          const p = progress[m.file];
          const pct = p && p.total > 0 ? Math.round((p.downloaded / p.total) * 100) : null;
          return (
            <div key={m.file} className={m.active ? "card model active" : "card model"}>
              <div className="model-head">
                <strong>{m.label}</strong>
                {m.recommended && <span className="chip">recommended</span>}
                {m.english_only && <span className="chip subtle">EN only</span>}
                <span className="muted small">{m.size_mb} MB</span>
              </div>
              <p className="muted small">{m.description}</p>
              {pct !== null && (
                <div className="mic-meter">
                  <div className="mic-meter-fill" style={{ width: `${pct}%` }} />
                </div>
              )}
              <div className="controls">
                {!m.downloaded && !m.downloading && pct === null && (
                  <button onClick={() => download(m.file)}>Download</button>
                )}
                {(m.downloading || pct !== null) && <button disabled>Downloading… {pct ?? 0}%</button>}
                {m.downloaded && !m.active && (
                  <>
                    <button className="primary" onClick={() => activate(m.file)}>
                      Use this model
                    </button>
                    <button onClick={() => remove(m.file)}>Delete</button>
                  </>
                )}
                {m.downloaded && m.active && <span className="chip">active</span>}
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}
