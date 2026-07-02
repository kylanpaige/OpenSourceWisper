import { useState } from "react";
import type { AppProfile, AppSettings, Tone } from "../types";

const TONES: Tone[] = ["auto", "casual", "professional", "technical"];

export default function ProfilesTab({
  settings,
  onSave,
}: {
  settings: AppSettings;
  onSave: (s: AppSettings) => void;
}) {
  const [draft, setDraft] = useState<AppProfile>({
    name: "",
    app_match: "",
    tone: "auto",
    instructions: "",
    enabled: true,
  });

  const persist = (profiles: AppProfile[]) => onSave({ ...settings, profiles });

  const update = (i: number, patch: Partial<AppProfile>) =>
    persist(settings.profiles.map((p, j) => (j === i ? { ...p, ...patch } : p)));

  const add = () => {
    if (!draft.name.trim() || !draft.app_match.trim()) return;
    persist([...settings.profiles, { ...draft, name: draft.name.trim(), app_match: draft.app_match.trim() }]);
    setDraft({ name: "", app_match: "", tone: "auto", instructions: "", enabled: true });
  };

  return (
    <section>
      <h1>App profiles</h1>
      <p className="muted">
        When dictation lands in a matching app, the AI cleanup adapts its tone and follows the
        profile's instructions — casual in Slack, polished in email, precise in your editor.
        Matching is a case-insensitive substring of the focused app's name.
      </p>

      {settings.profiles.map((p, i) => (
        <div key={i} className="card">
          <div className="row spread">
            <label className="row">
              <input
                type="checkbox"
                checked={p.enabled}
                onChange={(e) => update(i, { enabled: e.target.checked })}
              />
              <strong>{p.name}</strong>
              <span className="chip subtle">matches "{p.app_match}"</span>
            </label>
            <button onClick={() => persist(settings.profiles.filter((_, j) => j !== i))}>
              Remove
            </button>
          </div>
          <div className="row">
            <label>
              Tone{" "}
              <select value={p.tone} onChange={(e) => update(i, { tone: e.target.value as Tone })}>
                {TONES.map((t) => (
                  <option key={t} value={t}>
                    {t}
                  </option>
                ))}
              </select>
            </label>
          </div>
          <textarea
            placeholder="Extra instructions for this app…"
            value={p.instructions}
            onChange={(e) => update(i, { instructions: e.target.value })}
          />
        </div>
      ))}

      <div className="card">
        <h3>New profile</h3>
        <div className="row">
          <input
            placeholder="Name (e.g. Discord)"
            value={draft.name}
            onChange={(e) => setDraft({ ...draft, name: e.target.value })}
          />
          <input
            placeholder="App name contains… (e.g. discord)"
            value={draft.app_match}
            onChange={(e) => setDraft({ ...draft, app_match: e.target.value })}
          />
          <select
            value={draft.tone}
            onChange={(e) => setDraft({ ...draft, tone: e.target.value as Tone })}
          >
            {TONES.map((t) => (
              <option key={t} value={t}>
                {t}
              </option>
            ))}
          </select>
          <button className="primary" onClick={add}>
            Add
          </button>
        </div>
      </div>
    </section>
  );
}
