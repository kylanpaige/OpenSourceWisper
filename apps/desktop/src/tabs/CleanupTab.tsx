import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, CleanupProvider, CleanupSettings } from "../types";

export default function CleanupTab({
  settings,
  onSave,
}: {
  settings: AppSettings;
  onSave: (s: AppSettings) => void;
}) {
  const c = settings.cleanup;
  const [testResult, setTestResult] = useState("");
  const [testing, setTesting] = useState(false);

  const set = (patch: Partial<CleanupSettings>) =>
    onSave({ ...settings, cleanup: { ...c, ...patch } });

  const runTest = async () => {
    setTesting(true);
    setTestResult("");
    try {
      const out = await invoke<string>("test_cleanup");
      setTestResult(`✓ Connected. Sample output: ${out}`);
    } catch (e) {
      setTestResult(`✗ ${String(e)}`);
    } finally {
      setTesting(false);
    }
  };

  return (
    <section>
      <h1>AI cleanup</h1>
      <p className="muted">
        A local LLM polishes each transcript — removing filler words, fixing punctuation and
        grammar, formatting lists — exactly like Wispr Flow's cloud "polish", but on your machine.
        If the LLM is slow or offline, the raw formatted transcript is inserted instead; dictation
        never blocks.
      </p>

      <div className="card">
        <label className="row">
          <input type="checkbox" checked={c.enabled} onChange={(e) => set({ enabled: e.target.checked })} />
          <strong>Enable AI cleanup</strong>
        </label>
        <p className="muted small">
          Requires <a href="https://ollama.com" target="_blank" rel="noreferrer">Ollama</a> (or any
          OpenAI-compatible server like LM Studio) running locally. Try{" "}
          <code>ollama pull llama3.2:3b</code>.
        </p>
      </div>

      <div className="card">
        <h3>Server</h3>
        <div className="grid2">
          <label>
            Provider
            <select
              value={c.provider}
              onChange={(e) => set({ provider: e.target.value as CleanupProvider })}
            >
              <option value="ollama">Ollama</option>
              <option value="open_ai_compat">OpenAI-compatible (LM Studio, llama.cpp…)</option>
            </select>
          </label>
          <label>
            Base URL
            <input value={c.base_url} onChange={(e) => set({ base_url: e.target.value })} />
          </label>
          <label>
            Model
            <input value={c.model} onChange={(e) => set({ model: e.target.value })} />
          </label>
          <label>
            Timeout (ms)
            <input
              type="number"
              value={c.timeout_ms}
              onChange={(e) => set({ timeout_ms: Number(e.target.value) || 8000 })}
            />
          </label>
        </div>
        <div className="controls">
          <button onClick={runTest} disabled={testing}>
            {testing ? "Testing…" : "Test connection"}
          </button>
        </div>
        {testResult && <p className={testResult.startsWith("✓") ? "small" : "error small"}>{testResult}</p>}
      </div>

      <div className="card">
        <h3>What the cleanup does</h3>
        {(
          [
            ["remove_fillers", "Remove filler words (um, uh, you know…)"],
            ["fix_punctuation", "Fix punctuation & capitalization"],
            ["fix_grammar", "Fix grammar slips (keeps your voice)"],
            ["format_lists", "Format spoken enumerations as lists"],
          ] as const
        ).map(([key, label]) => (
          <label key={key} className="row">
            <input
              type="checkbox"
              checked={c[key]}
              onChange={(e) => set({ [key]: e.target.checked } as Partial<CleanupSettings>)}
            />
            {label}
          </label>
        ))}
        <label>
          Custom instructions
          <textarea
            placeholder="e.g. Never use exclamation marks. Prefer short sentences."
            value={c.custom_instructions}
            onChange={(e) => set({ custom_instructions: e.target.value })}
          />
        </label>
        <label>
          Skip cleanup under{" "}
          <input
            type="number"
            className="inline"
            value={c.min_chars}
            onChange={(e) => set({ min_chars: Number(e.target.value) || 0 })}
          />{" "}
          characters
        </label>
      </div>
    </section>
  );
}
