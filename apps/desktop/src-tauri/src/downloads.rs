//! Whisper model downloads from Hugging Face, with progress events.
//!
//! Downloads stream to `<file>.part` and are renamed only on success, so a
//! killed download never leaves a corrupt model that whisper.cpp would choke on.

use crate::paths;
use crate::state::AppState;
use futures_util::StreamExt;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use wisper_core::models;

#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgress {
    pub file: String,
    pub downloaded: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadDone {
    pub file: String,
    pub error: String,
}

pub async fn download_model(app: AppHandle, file: String) -> Result<(), String> {
    let info = models::find(&file).ok_or_else(|| format!("unknown model: {file}"))?;

    {
        let state = app.state::<AppState>();
        let mut downloading = state.downloading.lock().unwrap();
        if !downloading.insert(file.clone()) {
            return Err("already downloading".into());
        }
    }

    let result = do_download(&app, &info).await;

    {
        let state = app.state::<AppState>();
        state.downloading.lock().unwrap().remove(&file);
    }
    let _ = app.emit(
        "download-done",
        DownloadDone {
            file: file.clone(),
            error: result.as_ref().err().cloned().unwrap_or_default(),
        },
    );
    result
}

async fn do_download(app: &AppHandle, info: &models::ModelInfo) -> Result<(), String> {
    let dest = paths::models_dir(app).join(&info.file);
    if dest.exists() {
        return Ok(());
    }
    let part = dest.with_extension("part");

    let client = reqwest::Client::new();
    let resp = client
        .get(&info.url)
        .send()
        .await
        .map_err(|e| format!("download failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("download failed: HTTP {}", resp.status()));
    }
    let total = resp.content_length().unwrap_or(info.size_mb * 1024 * 1024);

    let mut out = tokio::fs::File::create(&part)
        .await
        .map_err(|e| e.to_string())?;
    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut last_emit = std::time::Instant::now();

    use tokio::io::AsyncWriteExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("download interrupted: {e}"))?;
        out.write_all(&chunk).await.map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;
        if last_emit.elapsed().as_millis() > 150 {
            last_emit = std::time::Instant::now();
            let _ = app.emit(
                "download-progress",
                DownloadProgress {
                    file: info.file.clone(),
                    downloaded,
                    total,
                },
            );
        }
    }
    out.flush().await.map_err(|e| e.to_string())?;
    drop(out);

    tokio::fs::rename(&part, &dest)
        .await
        .map_err(|e| format!("could not finalize download: {e}"))?;
    Ok(())
}

pub fn delete_model(app: &AppHandle, file: &str) -> Result<(), String> {
    // Refuse to delete the active model out from under the engine.
    let state = app.state::<AppState>();
    {
        let s = state.settings.lock().unwrap();
        if s.model == file {
            return Err("cannot delete the currently selected model".into());
        }
    }
    let path = paths::models_dir(app).join(file);
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
