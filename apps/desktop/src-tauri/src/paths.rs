use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub fn data_dir(app: &AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .expect("no app data dir on this platform");
    std::fs::create_dir_all(&dir).ok();
    dir
}

pub fn settings_path(app: &AppHandle) -> PathBuf {
    data_dir(app).join("settings.json")
}

pub fn dictionary_path(app: &AppHandle) -> PathBuf {
    data_dir(app).join("dictionary.json")
}

pub fn history_path(app: &AppHandle) -> PathBuf {
    data_dir(app).join("history.jsonl")
}

pub fn models_dir(app: &AppHandle) -> PathBuf {
    let dir = data_dir(app).join("models");
    std::fs::create_dir_all(&dir).ok();
    dir
}
