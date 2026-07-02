// Mirrors wisper-core's serde types (snake_case fields).

export type RecordingMode = "push_to_talk" | "toggle";
export type InjectionMethod = "paste" | "type" | "clipboard_only";
export type CleanupProvider = "ollama" | "open_ai_compat";
export type Tone = "auto" | "casual" | "professional" | "technical";
export type Phase = "idle" | "recording" | "transcribing" | "cleaning" | "injecting";

export interface CleanupSettings {
  enabled: boolean;
  provider: CleanupProvider;
  base_url: string;
  model: string;
  min_chars: number;
  timeout_ms: number;
  remove_fillers: boolean;
  fix_punctuation: boolean;
  fix_grammar: boolean;
  format_lists: boolean;
  custom_instructions: string;
}

export interface AppProfile {
  name: string;
  app_match: string;
  tone: Tone;
  instructions: string;
  enabled: boolean;
}

export interface FormatSettings {
  remove_fillers: boolean;
  spoken_punctuation: boolean;
  smart_capitalization: boolean;
}

export interface AudioSettings {
  input_device: string;
  silence_autostop_ms: number;
  silence_threshold: number;
  max_recording_secs: number;
  chimes: boolean;
}

export interface AppSettings {
  hotkey: string;
  hotkey_toggle: string;
  hotkey_command: string;
  recording_mode: RecordingMode;
  injection: InjectionMethod;
  model: string;
  language: string;
  threads: number;
  audio: AudioSettings;
  format: FormatSettings;
  cleanup: CleanupSettings;
  profiles: AppProfile[];
  show_overlay: boolean;
  save_history: boolean;
  max_history_entries: number;
  launch_at_login: boolean;
}

export interface DictionaryEntry {
  spoken: string;
  written: string;
  enabled: boolean;
}

export interface ModelStatus {
  file: string;
  label: string;
  description: string;
  size_mb: number;
  english_only: boolean;
  recommended: boolean;
  url: string;
  downloaded: boolean;
  downloading: boolean;
  active: boolean;
}

export interface HistoryEntry {
  timestamp: string;
  raw: string;
  final_text: string;
  app: string;
  duration_ms: number;
  cleaned: boolean;
}

export interface DictationResult {
  raw: string;
  final_text: string;
  app: string;
  duration_ms: number;
  cleaned: boolean;
  skip_reason: string;
}

export interface DownloadProgress {
  file: string;
  downloaded: number;
  total: number;
}

export interface DownloadDone {
  file: string;
  error: string;
}
