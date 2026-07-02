import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { DictionaryEntry } from "../types";

export default function DictionaryTab() {
  const [entries, setEntries] = useState<DictionaryEntry[]>([]);
  const [spoken, setSpoken] = useState("");
  const [written, setWritten] = useState("");
  const [error, setError] = useState("");

  useEffect(() => {
    invoke<DictionaryEntry[]>("get_dictionary").then(setEntries).catch((e) => setError(String(e)));
  }, []);

  const persist = (next: DictionaryEntry[]) => {
    setEntries(next);
    invoke("set_dictionary", { entries: next }).catch((e) => setError(String(e)));
  };

  const add = () => {
    if (!spoken.trim() || !written.trim()) return;
    persist([...entries, { spoken: spoken.trim(), written: written.trim(), enabled: true }]);
    setSpoken("");
    setWritten("");
  };

  return (
    <section>
      <h1>Personal dictionary</h1>
      <p className="muted">
        Teach the transcriber your names, jargon, and snippets. "Spoken" is what the model tends to
        hear; "written" is what should appear. Longer phrases win, and terms are also hinted to the
        speech model and the AI cleanup.
      </p>
      {error && <div className="banner error">{error}</div>}

      <div className="card">
        <div className="row">
          <input
            placeholder='Spoken (e.g. "jira")'
            value={spoken}
            onChange={(e) => setSpoken(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && add()}
          />
          <span className="muted">→</span>
          <input
            placeholder='Written (e.g. "Jira")'
            value={written}
            onChange={(e) => setWritten(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && add()}
          />
          <button className="primary" onClick={add}>
            Add
          </button>
        </div>
      </div>

      {entries.length === 0 && <p className="muted">No entries yet.</p>}
      {entries.map((entry, i) => (
        <div key={i} className="card row spread">
          <label className="row">
            <input
              type="checkbox"
              checked={entry.enabled}
              onChange={(e) =>
                persist(entries.map((x, j) => (j === i ? { ...x, enabled: e.target.checked } : x)))
              }
            />
            <span className={entry.enabled ? "" : "muted"}>
              <code>{entry.spoken}</code> → <code>{entry.written}</code>
            </span>
          </label>
          <button onClick={() => persist(entries.filter((_, j) => j !== i))}>Remove</button>
        </div>
      ))}
    </section>
  );
}
