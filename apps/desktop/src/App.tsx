import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

type Phase = "idle" | "recording" | "transcribing" | "cleaning" | "injecting";

export default function App() {
  const [phase, setPhase] = useState<Phase>("idle");
  const [lastText, setLastText] = useState("");
  const [error, setError] = useState("");

  useEffect(() => {
    const unsubs: Array<() => void> = [];
    listen<Phase>("dictation-phase", (e) => setPhase(e.payload)).then((u) => unsubs.push(u));
    listen<{ final_text: string }>("dictation-result", (e) => {
      setLastText(e.payload.final_text);
      setError("");
    }).then((u) => unsubs.push(u));
    listen<string>("dictation-error", (e) => setError(e.payload)).then((u) => unsubs.push(u));
    invoke<Phase>("get_phase").then(setPhase).catch(() => {});
    return () => unsubs.forEach((u) => u());
  }, []);

  return (
    <main className="shell">
      <h1>OpenSourceWisper</h1>
      <p className="tagline">Hold your hotkey, talk, release — clean text lands in the focused app. Fully local.</p>
      <div className={`status status-${phase}`}>{phase}</div>
      <div className="controls">
        <button onClick={() => invoke("start_dictation")}>Start</button>
        <button onClick={() => invoke("stop_dictation")}>Stop &amp; insert</button>
        <button onClick={() => invoke("cancel_dictation")}>Cancel</button>
      </div>
      {lastText && (
        <section>
          <h2>Last dictation</h2>
          <p className="result">{lastText}</p>
        </section>
      )}
      {error && <p className="error">{error}</p>}
    </main>
  );
}
