use serde::{Deserialize, Serialize};

/// How recording is triggered from the global hotkey.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecordingMode {
    /// Hold the hotkey to record, release to transcribe (Wispr Flow default).
    #[default]
    PushToTalk,
    /// Press once to start, press again to stop.
    Toggle,
}

/// How transcribed text is delivered into the focused app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InjectionMethod {
    /// Copy to clipboard, send Cmd/Ctrl+V, then restore the previous clipboard.
    /// Fastest and most reliable for long text.
    #[default]
    Paste,
    /// Simulate individual keystrokes. Slower but works in apps that block paste.
    Type,
    /// Only place the text on the clipboard; the user pastes manually.
    ClipboardOnly,
}

/// Which local LLM API the cleanup stage talks to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CleanupProvider {
    /// Ollama's native /api/chat endpoint.
    #[default]
    Ollama,
    /// Any OpenAI-compatible /v1/chat/completions endpoint (LM Studio, llama.cpp server, vLLM...).
    OpenAiCompat,
}

/// Writing tone the cleanup LLM should aim for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Tone {
    #[default]
    Auto,
    Casual,
    Professional,
    Technical,
}

/// Settings for the local-LLM cleanup ("polish") stage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CleanupSettings {
    pub enabled: bool,
    pub provider: CleanupProvider,
    /// Base URL of the local LLM server.
    pub base_url: String,
    /// Model name, e.g. "llama3.2:3b" or "qwen2.5:3b-instruct".
    pub model: String,
    /// Skip the LLM for transcripts shorter than this many characters
    /// (short utterances rarely need cleanup and the round-trip costs latency).
    pub min_chars: usize,
    /// Abort cleanup and fall back to the raw transcript after this many ms.
    pub timeout_ms: u64,
    /// Prompt sections that can be toggled individually.
    pub remove_fillers: bool,
    pub fix_punctuation: bool,
    pub fix_grammar: bool,
    pub format_lists: bool,
    /// Extra user-provided instructions appended to the prompt.
    pub custom_instructions: String,
}

impl Default for CleanupSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: CleanupProvider::Ollama,
            base_url: "http://localhost:11434".into(),
            model: "llama3.2:3b".into(),
            min_chars: 40,
            timeout_ms: 8000,
            remove_fillers: true,
            fix_punctuation: true,
            fix_grammar: true,
            format_lists: true,
            custom_instructions: String::new(),
        }
    }
}

/// A per-app profile: when the focused app matches, the tone / instructions override.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AppProfile {
    /// Human label, e.g. "Slack".
    pub name: String,
    /// Case-insensitive substring matched against the focused app name / window title.
    pub app_match: String,
    pub tone: Tone,
    /// Extra prompt instructions for this app (e.g. "keep it short and friendly").
    pub instructions: String,
    pub enabled: bool,
}

/// Non-LLM post-processing applied to every transcript.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FormatSettings {
    /// Strip "um", "uh", "er"... even when the LLM is off.
    pub remove_fillers: bool,
    /// Interpret spoken commands like "period", "comma", "new line".
    pub spoken_punctuation: bool,
    /// Capitalize sentence starts and tidy whitespace.
    pub smart_capitalization: bool,
}

impl Default for FormatSettings {
    fn default() -> Self {
        Self {
            remove_fillers: true,
            spoken_punctuation: false,
            smart_capitalization: true,
        }
    }
}

/// Audio capture / VAD settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioSettings {
    /// Preferred input device name; empty = system default.
    pub input_device: String,
    /// In toggle mode, auto-stop after this many ms of silence (0 = never).
    pub silence_autostop_ms: u64,
    /// RMS level (0.0-1.0) under which audio counts as silence.
    pub silence_threshold: f32,
    /// Hard cap on a single recording, in seconds.
    pub max_recording_secs: u64,
    /// Play a subtle sound when recording starts/stops.
    pub chimes: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            input_device: String::new(),
            silence_autostop_ms: 3000,
            silence_threshold: 0.012,
            max_recording_secs: 300,
            chimes: true,
        }
    }
}

/// Top-level persisted settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    /// Global shortcut in Tauri accelerator syntax, e.g. "Ctrl+Shift+Space" or "F9".
    pub hotkey: String,
    /// Optional second hotkey that always records in toggle mode ("hands-free").
    pub hotkey_toggle: String,
    /// Optional Command Mode hotkey: hold it, speak an instruction, and the
    /// currently selected text is rewritten by the local LLM.
    pub hotkey_command: String,
    pub recording_mode: RecordingMode,
    pub injection: InjectionMethod,
    /// Filename of the active whisper model, e.g. "ggml-base.en.bin".
    pub model: String,
    /// Spoken language hint ("auto" or ISO 639-1 like "en").
    pub language: String,
    /// Whisper decode threads (0 = auto).
    pub threads: usize,
    pub audio: AudioSettings,
    pub format: FormatSettings,
    pub cleanup: CleanupSettings,
    pub profiles: Vec<AppProfile>,
    /// Show the floating recording indicator overlay.
    pub show_overlay: bool,
    /// Keep dictation history on disk (local only).
    pub save_history: bool,
    pub max_history_entries: usize,
    pub launch_at_login: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hotkey: default_hotkey().into(),
            hotkey_toggle: String::new(),
            hotkey_command: String::new(),
            recording_mode: RecordingMode::PushToTalk,
            injection: InjectionMethod::Paste,
            model: "ggml-base.en.bin".into(),
            language: "auto".into(),
            threads: 0,
            audio: AudioSettings::default(),
            format: FormatSettings::default(),
            cleanup: CleanupSettings::default(),
            profiles: default_profiles(),
            show_overlay: true,
            save_history: true,
            max_history_entries: 500,
            launch_at_login: false,
        }
    }
}

fn default_hotkey() -> &'static str {
    // Fn/Globe isn't capturable cross-platform; Ctrl+Space-style chords collide
    // with IME switching, so mirror Wispr's alternative default.
    if cfg!(target_os = "macos") {
        "Alt+Space"
    } else {
        "Ctrl+Alt+Space"
    }
}

fn default_profiles() -> Vec<AppProfile> {
    vec![
        AppProfile {
            name: "Slack".into(),
            app_match: "slack".into(),
            tone: Tone::Casual,
            instructions: "Keep the message short and conversational.".into(),
            enabled: true,
        },
        AppProfile {
            name: "Email".into(),
            app_match: "mail".into(),
            tone: Tone::Professional,
            instructions: "Write complete, well-structured sentences suitable for email.".into(),
            enabled: true,
        },
        AppProfile {
            name: "Code editor".into(),
            app_match: "code".into(),
            tone: Tone::Technical,
            instructions: "Preserve technical terms, identifiers and casing exactly as spoken.".into(),
            enabled: true,
        },
    ]
}

impl AppSettings {
    /// Find the first enabled profile matching the focused app name (case-insensitive substring).
    pub fn profile_for_app(&self, app_name: &str) -> Option<&AppProfile> {
        let needle = app_name.to_lowercase();
        self.profiles
            .iter()
            .filter(|p| p.enabled && !p.app_match.is_empty())
            .find(|p| needle.contains(&p.app_match.to_lowercase()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_roundtrip_json() {
        let s = AppSettings::default();
        let json = serde_json::to_string_pretty(&s).unwrap();
        let back: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn unknown_fields_and_missing_fields_are_tolerated() {
        // Forward/backward compat: old configs must load after upgrades.
        let back: AppSettings =
            serde_json::from_str(r#"{"hotkey":"F9","future_field":123}"#).unwrap();
        assert_eq!(back.hotkey, "F9");
        assert_eq!(back.recording_mode, RecordingMode::PushToTalk);
    }

    #[test]
    fn profile_matching_is_case_insensitive_substring() {
        let s = AppSettings::default();
        let p = s.profile_for_app("Slack — #general").unwrap();
        assert_eq!(p.name, "Slack");
        assert!(s.profile_for_app("Firefox").is_none());
        let p = s.profile_for_app("Visual Studio Code").unwrap();
        assert_eq!(p.name, "Code editor");
    }
}
