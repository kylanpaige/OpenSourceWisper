import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { HistoryEntry } from "../types";

export default function HistoryTab() {
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [copied, setCopied] = useState<number | null>(null);

  const reload = useCallback(() => {
    invoke<HistoryEntry[]>("get_history", { limit: 200 }).then(setEntries).catch(console.error);
  }, []);

  useEffect(() => {
    reload();
    let unsub: (() => void) | undefined;
    listen("dictation-result", reload).then((u) => (unsub = u));
    return () => unsub?.();
  }, [reload]);

  const copy = async (text: string, i: number) => {
    await navigator.clipboard.writeText(text);
    setCopied(i);
    setTimeout(() => setCopied(null), 1200);
  };

  return (
    <section>
      <div className="row spread">
        <h1>History</h1>
        <button
          onClick={() => invoke("clear_history").then(reload)}
          disabled={entries.length === 0}
        >
          Clear all
        </button>
      </div>
      <p className="muted">Stored only on this computer. Click any entry to copy it.</p>

      {entries.length === 0 && <p className="muted">Nothing here yet — go dictate something.</p>}
      {entries.map((e, i) => (
        <div key={i} className="card clickable" onClick={() => copy(e.final_text, i)}>
          <p className="result">{e.final_text}</p>
          <p className="muted small">
            {new Date(e.timestamp).toLocaleString()} · {(e.duration_ms / 1000).toFixed(1)}s
            {e.app && <> · {e.app}</>} · {e.cleaned ? "AI-polished" : "formatted"}
            {copied === i && <strong> · copied!</strong>}
          </p>
        </div>
      ))}
    </section>
  );
}
