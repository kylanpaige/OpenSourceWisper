import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, InjectionMethod, RecordingMode } from "../types";

/** Turn a KeyboardEvent into a Tauri accelerator string like "Ctrl+Alt+Space". */
function eventToAccelerator(e: React.KeyboardEvent): string | null {
  const mods: string[] = [];
  if (e.ctrlKey) mods.push("Ctrl");
  if (e.altKey) mods.push("Alt");
  if (e.shiftKey) mods.push("Shift");
  if (e.metaKey) mods.push(navigator.platform.includes("Mac") ? "Cmd" : "Super");

  const key = e.key;
  if (["Control", "Alt", "Shift", "Meta"].includes(key)) return null; // modifier only so far
  let name = key;
  if (key === " ") name = "Space";
  else if (key.length === 1) name = key.toUpperCase();
  else if (key.startsWith("Arrow")) name = key.slice(5);
  return [...mods, name].join("+");
}

function HotkeyInput({
  value,
  onChange,
  placeholder,
}: {
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
}) {
  const [capturing, setCapturing] = useState(false);
  return (
    <input
      readOnly
      className={capturing ? "hotkey capturing" : "hotkey"}
      value={capturing ? "Press keys…" : value}
      placeholder={placeholder ?? "Click, then press a shortcut"}
      onFocus={() => setCapturing(true)}
      onBlur={() => setCapturing(false)}
      onKeyDown={(e) => {
        e.preventDefault();
        if (e.key === "Escape") {
          (e.target as HTMLInputElement).blur();
          return;
        }
        if (e.key === "Backspace" || e.key === "Delete") {
          onChange("");
          (e.target as HTMLInputElement).blur();
          return;
        }
        const acc = eventToAccelerator(e);
        if (acc) {
          onChange(acc);
          (e.target as HTMLInputElement).blur();
        }
      }}
    />
  );
}

export default function SettingsTab({
  settings,
  onSave,
}: {
  settings: AppSettings;
  onSave: (s: AppSettings) => void;
}) {
  const [devices, setDevices] = useState<string[]>([]);
  useEffect(() => {
    invoke<string[]>("list_input_devices").then(setDevices).catch(console.error);
  }, []);

  const s = settings;

  return (
    <section>
      <h1>Settings</h1>

      <div className="card">
        <h3>Hotkeys</h3>
        <div className="grid2">
          <label>
            Dictation hotkey
            <HotkeyInput value={s.hotkey} onChange={(hotkey) => onSave({ ...s, hotkey })} />
          </label>
          <label>
            Mode
            <select
              value={s.recording_mode}
              onChange={(e) => onSave({ ...s, recording_mode: e.target.value as RecordingMode })}
            >
              <option value="push_to_talk">Push-to-talk (hold to record)</option>
              <option value="toggle">Toggle (press to start/stop)</option>
            </select>
          </label>
          <label>
            Hands-free toggle hotkey (optional)
            <HotkeyInput
              value={s.hotkey_toggle}
              onChange={(hotkey_toggle) => onSave({ ...s, hotkey_toggle })}
              placeholder="None — click to set"
            />
          </label>
          <label>
            Command Mode hotkey (optional)
            <HotkeyInput
              value={s.hotkey_command}
              onChange={(hotkey_command) => onSave({ ...s, hotkey_command })}
              placeholder="None — click to set"
            />
          </label>
        </div>
        <p className="muted small">
          Backspace clears a hotkey. In toggle mode, recording also stops automatically after{" "}
          {Math.round(s.audio.silence_autostop_ms / 1000)}s of silence. Command Mode: select text
          anywhere, hold the hotkey, and speak an instruction like "make this more formal" — the
          selection is rewritten in place (needs AI cleanup enabled).
        </p>
      </div>

      <div className="card">
        <h3>Text insertion</h3>
        <label>
          Method
          <select
            value={s.injection}
            onChange={(e) => onSave({ ...s, injection: e.target.value as InjectionMethod })}
          >
            <option value="paste">Paste (fast, restores your clipboard)</option>
            <option value="type">Type keystrokes (for apps that block paste)</option>
            <option value="clipboard_only">Clipboard only (paste manually)</option>
          </select>
        </label>
      </div>

      <div className="card">
        <h3>Audio</h3>
        <div className="grid2">
          <label>
            Microphone
            <select
              value={s.audio.input_device}
              onChange={(e) => onSave({ ...s, audio: { ...s.audio, input_device: e.target.value } })}
            >
              <option value="">System default</option>
              {devices.map((d) => (
                <option key={d} value={d}>
                  {d}
                </option>
              ))}
            </select>
          </label>
          <label>
            Language
            <select value={s.language} onChange={(e) => onSave({ ...s, language: e.target.value })}>
              {["auto", "en", "es", "fr", "de", "it", "pt", "nl", "ja", "ko", "zh", "hi", "ru"].map(
                (l) => (
                  <option key={l} value={l}>
                    {l}
                  </option>
                ),
              )}
            </select>
          </label>
          <label>
            Silence auto-stop (toggle mode, ms)
            <input
              type="number"
              value={s.audio.silence_autostop_ms}
              onChange={(e) =>
                onSave({
                  ...s,
                  audio: { ...s.audio, silence_autostop_ms: Number(e.target.value) || 0 },
                })
              }
            />
          </label>
          <label>
            Max recording (seconds)
            <input
              type="number"
              value={s.audio.max_recording_secs}
              onChange={(e) =>
                onSave({
                  ...s,
                  audio: { ...s.audio, max_recording_secs: Number(e.target.value) || 0 },
                })
              }
            />
          </label>
        </div>
      </div>

      <div className="card">
        <h3>Formatting (always on, no LLM needed)</h3>
        <label className="row">
          <input
            type="checkbox"
            checked={s.format.remove_fillers}
            onChange={(e) =>
              onSave({ ...s, format: { ...s.format, remove_fillers: e.target.checked } })
            }
          />
          Remove filler words (um, uh…)
        </label>
        <label className="row">
          <input
            type="checkbox"
            checked={s.format.spoken_punctuation}
            onChange={(e) =>
              onSave({ ...s, format: { ...s.format, spoken_punctuation: e.target.checked } })
            }
          />
          Spoken punctuation commands ("period", "new line"…)
        </label>
        <label className="row">
          <input
            type="checkbox"
            checked={s.format.smart_capitalization}
            onChange={(e) =>
              onSave({ ...s, format: { ...s.format, smart_capitalization: e.target.checked } })
            }
          />
          Smart capitalization
        </label>
      </div>

      <div className="card">
        <h3>General</h3>
        <label className="row">
          <input
            type="checkbox"
            checked={s.audio.chimes}
            onChange={(e) => onSave({ ...s, audio: { ...s.audio, chimes: e.target.checked } })}
          />
          Play a chime when recording starts/stops
        </label>
        <label className="row">
          <input
            type="checkbox"
            checked={s.show_overlay}
            onChange={(e) => onSave({ ...s, show_overlay: e.target.checked })}
          />
          Show floating recording indicator
        </label>
        <label className="row">
          <input
            type="checkbox"
            checked={s.save_history}
            onChange={(e) => onSave({ ...s, save_history: e.target.checked })}
          />
          Save dictation history (local only)
        </label>
        <label className="row">
          <input
            type="checkbox"
            checked={s.launch_at_login}
            onChange={(e) => onSave({ ...s, launch_at_login: e.target.checked })}
          />
          Launch at login
        </label>
      </div>
    </section>
  );
}
