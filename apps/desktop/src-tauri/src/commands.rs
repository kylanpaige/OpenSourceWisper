//! Tauri commands exposed to the React frontend.

use crate::state::{AppState, DictationPhase};
use crate::{downloads, hotkeys, paths, pipeline};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use wisper_core::cleanup::{clean_transcript, CleanupOutcome};
use wisper_core::dictionary::DictionaryEntry;
use wisper_core::{history, models, AppSettings};

fn save_json<T: Serialize>(path: &std::path::Path, value: &T) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

// ---------- settings ----------

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> AppSettings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_settings(
    app: AppHandle,
    state: State<AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    let hotkeys_changed = {
        let mut current = state.settings.lock().unwrap();
        let changed =
            current.hotkey != settings.hotkey || current.hotkey_toggle != settings.hotkey_toggle;
        *current = settings.clone();
        changed
    };
    save_json(&paths::settings_path(&app), &settings)?;
    if hotkeys_changed {
        hotkeys::register_from_settings(&app)?;
    }
    crate::autostart::sync(&app, settings.launch_at_login);
    Ok(())
}

// ---------- dictionary ----------

#[tauri::command]
pub fn get_dictionary(state: State<AppState>) -> Vec<DictionaryEntry> {
    state.dictionary.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_dictionary(
    app: AppHandle,
    state: State<AppState>,
    entries: Vec<DictionaryEntry>,
) -> Result<(), String> {
    *state.dictionary.lock().unwrap() = entries.clone();
    save_json(&paths::dictionary_path(&app), &entries)
}

// ---------- models ----------

#[derive(Debug, Clone, Serialize)]
pub struct ModelStatus {
    #[serde(flatten)]
    pub info: models::ModelInfo,
    pub downloaded: bool,
    pub downloading: bool,
    pub active: bool,
}

#[tauri::command]
pub fn list_models(app: AppHandle, state: State<AppState>) -> Vec<ModelStatus> {
    let dir = paths::models_dir(&app);
    let active = state.settings.lock().unwrap().model.clone();
    let downloading = state.downloading.lock().unwrap().clone();
    models::catalog()
        .into_iter()
        .map(|info| ModelStatus {
            downloaded: dir.join(&info.file).exists(),
            downloading: downloading.contains(&info.file),
            active: info.file == active,
            info,
        })
        .collect()
}

#[tauri::command]
pub async fn download_model(app: AppHandle, file: String) -> Result<(), String> {
    downloads::download_model(app, file).await
}

#[tauri::command]
pub fn delete_model(app: AppHandle, file: String) -> Result<(), String> {
    downloads::delete_model(&app, &file)
}

// ---------- audio ----------

#[tauri::command]
pub fn list_input_devices() -> Vec<String> {
    crate::audio::list_input_devices()
}

// ---------- dictation control ----------

#[tauri::command]
pub fn start_dictation(app: AppHandle) {
    pipeline::start_recording(&app);
}

#[tauri::command]
pub fn stop_dictation(app: AppHandle) {
    pipeline::stop_and_process(&app);
}

#[tauri::command]
pub fn cancel_dictation(app: AppHandle) {
    pipeline::cancel_recording(&app);
}

#[tauri::command]
pub fn get_phase(state: State<AppState>) -> DictationPhase {
    *state.phase.lock().unwrap()
}

// ---------- history ----------

#[tauri::command]
pub fn get_history(app: AppHandle, limit: Option<usize>) -> Vec<history::HistoryEntry> {
    history::load_recent(&paths::history_path(&app), limit.unwrap_or(100))
}

#[tauri::command]
pub fn clear_history(app: AppHandle) -> Result<(), String> {
    history::clear(&paths::history_path(&app)).map_err(|e| e.to_string())
}

// ---------- cleanup (LLM) ----------

/// Round-trip a canned transcript through the configured LLM so users can
/// verify their Ollama/LM Studio setup from the settings screen.
#[tauri::command]
pub async fn test_cleanup(app: AppHandle) -> Result<String, String> {
    let cleanup = {
        let state = app.state::<AppState>();
        let mut c = state.settings.lock().unwrap().cleanup.clone();
        c.enabled = true;
        c.min_chars = 1;
        c
    };
    let sample = "um so basically i think we should uh ship the new feature on monday";
    match clean_transcript(&cleanup, sample, None, &[]).await {
        CleanupOutcome::Cleaned(t) => Ok(t),
        CleanupOutcome::Skipped(reason) => Err(reason),
    }
}
