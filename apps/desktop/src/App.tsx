import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AppSettings, DictationResult, Phase } from "./types";
import HomeTab from "./tabs/HomeTab";
import ModelsTab from "./tabs/ModelsTab";
import DictionaryTab from "./tabs/DictionaryTab";
import ProfilesTab from "./tabs/ProfilesTab";
import CleanupTab from "./tabs/CleanupTab";
import HistoryTab from "./tabs/HistoryTab";
import SettingsTab from "./tabs/SettingsTab";

const TABS = ["Home", "Models", "Dictionary", "Profiles", "AI Cleanup", "History", "Settings"] as const;
type Tab = (typeof TABS)[number];

export default function App() {
  const [tab, setTab] = useState<Tab>("Home");
  const [phase, setPhase] = useState<Phase>("idle");
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [lastResult, setLastResult] = useState<DictationResult | null>(null);
  const [error, setError] = useState("");

  const reloadSettings = useCallback(() => {
    invoke<AppSettings>("get_settings").then(setSettings).catch(console.error);
  }, []);

  useEffect(() => {
    reloadSettings();
    invoke<Phase>("get_phase").then(setPhase).catch(() => {});
    const unsubs: Array<() => void> = [];
    listen<Phase>("dictation-phase", (e) => setPhase(e.payload)).then((u) => unsubs.push(u));
    listen<DictationResult>("dictation-result", (e) => {
      setLastResult(e.payload);
      setError("");
    }).then((u) => unsubs.push(u));
    listen<string>("dictation-error", (e) => setError(e.payload)).then((u) => unsubs.push(u));
    return () => unsubs.forEach((u) => u());
  }, [reloadSettings]);

  const saveSettings = useCallback(
    async (next: AppSettings) => {
      setSettings(next);
      try {
        await invoke("set_settings", { settings: next });
        setError("");
      } catch (e) {
        setError(String(e));
      }
    },
    [],
  );

  if (!settings) {
    return <main className="shell">Loading…</main>;
  }

  return (
    <div className="layout">
      <nav className="sidebar">
        <div className="brand">
          <span className="brand-dot" data-phase={phase} />
          OpenSourceWisper
        </div>
        {TABS.map((t) => (
          <button key={t} className={t === tab ? "nav active" : "nav"} onClick={() => setTab(t)}>
            {t}
          </button>
        ))}
        <div className="sidebar-footer">
          <span className={`phase-badge phase-${phase}`}>{phase}</span>
        </div>
      </nav>
      <main className="content">
        {error && (
          <div className="banner error" onClick={() => setError("")}>
            {error}
          </div>
        )}
        {tab === "Home" && (
          <HomeTab settings={settings} phase={phase} lastResult={lastResult} />
        )}
        {tab === "Models" && (
          <ModelsTab settings={settings} onSave={saveSettings} />
        )}
        {tab === "Dictionary" && <DictionaryTab />}
        {tab === "Profiles" && <ProfilesTab settings={settings} onSave={saveSettings} />}
        {tab === "AI Cleanup" && <CleanupTab settings={settings} onSave={saveSettings} />}
        {tab === "History" && <HistoryTab />}
        {tab === "Settings" && <SettingsTab settings={settings} onSave={saveSettings} />}
      </main>
    </div>
  );
}
