use crate::asr::WhisperEngine;
use crate::audio::Recorder;
use serde::Serialize;
use std::collections::HashSet;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use wisper_core::dictionary::DictionaryEntry;
use wisper_core::AppSettings;

/// What the pipeline is currently doing; mirrored to the UI and tray.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DictationPhase {
    #[default]
    Idle,
    Recording,
    Transcribing,
    Cleaning,
    Injecting,
}

/// Why the current recording exists: normal dictation, or a Command Mode
/// instruction that should rewrite `selection`.
#[derive(Debug, Clone, Default)]
pub enum RecordingPurpose {
    #[default]
    Dictate,
    Command {
        selection: String,
    },
}

pub struct AppState {
    pub settings: Mutex<AppSettings>,
    pub purpose: Mutex<RecordingPurpose>,
    pub dictionary: Mutex<Vec<DictionaryEntry>>,
    pub recorder: Mutex<Option<Recorder>>,
    /// Loaded whisper context; rebuilt when the selected model changes.
    pub engine: Mutex<Option<WhisperEngine>>,
    pub phase: Mutex<DictationPhase>,
    /// True while a recording-processing run is in flight (debounces hotkeys).
    pub busy: AtomicBool,
    /// Model files currently being downloaded (guards duplicate downloads).
    pub downloading: Mutex<HashSet<String>>,
}

impl AppState {
    pub fn new(settings: AppSettings, dictionary: Vec<DictionaryEntry>) -> Self {
        Self {
            settings: Mutex::new(settings),
            purpose: Mutex::new(RecordingPurpose::Dictate),
            dictionary: Mutex::new(dictionary),
            recorder: Mutex::new(None),
            engine: Mutex::new(None),
            phase: Mutex::new(DictationPhase::Idle),
            busy: AtomicBool::new(false),
            downloading: Mutex::new(HashSet::new()),
        }
    }
}
