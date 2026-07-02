//! Global hotkey wiring via tauri-plugin-global-shortcut.
//!
//! Push-to-talk uses the plugin's Pressed/Released states: press starts
//! recording, release stops and processes. Toggle mode (and the dedicated
//! toggle hotkey) flips on press only.

use crate::pipeline;
use crate::state::AppState;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use wisper_core::settings::RecordingMode;

/// (Re-)register hotkeys from current settings. Call at startup and whenever
/// settings change. Returns a human-readable error for the UI if a shortcut
/// can't be parsed or registered (e.g. taken by another app).
pub fn register_from_settings(app: &AppHandle) -> Result<(), String> {
    let gs = app.global_shortcut();
    gs.unregister_all().map_err(|e| e.to_string())?;

    let (main_hotkey, toggle_hotkey) = {
        let state = app.state::<AppState>();
        let s = state.settings.lock().unwrap();
        (s.hotkey.clone(), s.hotkey_toggle.clone())
    };

    if !main_hotkey.trim().is_empty() {
        let shortcut: Shortcut = main_hotkey
            .parse()
            .map_err(|e| format!("invalid hotkey \"{main_hotkey}\": {e}"))?;
        gs.register(shortcut)
            .map_err(|e| format!("could not register \"{main_hotkey}\": {e}"))?;
    }
    if !toggle_hotkey.trim().is_empty() && toggle_hotkey != main_hotkey {
        let shortcut: Shortcut = toggle_hotkey
            .parse()
            .map_err(|e| format!("invalid toggle hotkey \"{toggle_hotkey}\": {e}"))?;
        gs.register(shortcut)
            .map_err(|e| format!("could not register \"{toggle_hotkey}\": {e}"))?;
    }
    Ok(())
}

/// Plugin event handler installed once at startup.
pub fn handle_shortcut(app: &AppHandle, shortcut: &Shortcut, state: ShortcutState) {
    let (main_hotkey, toggle_hotkey, mode) = {
        let st = app.state::<AppState>();
        let s = st.settings.lock().unwrap();
        (s.hotkey.clone(), s.hotkey_toggle.clone(), s.recording_mode)
    };

    let is_main = main_hotkey
        .parse::<Shortcut>()
        .map(|s| s == *shortcut)
        .unwrap_or(false);
    let is_toggle_key = !toggle_hotkey.is_empty()
        && toggle_hotkey
            .parse::<Shortcut>()
            .map(|s| s == *shortcut)
            .unwrap_or(false);

    if is_main {
        match (mode, state) {
            (RecordingMode::PushToTalk, ShortcutState::Pressed) => pipeline::start_recording(app),
            (RecordingMode::PushToTalk, ShortcutState::Released) => pipeline::stop_and_process(app),
            (RecordingMode::Toggle, ShortcutState::Pressed) => pipeline::toggle_recording(app),
            (RecordingMode::Toggle, ShortcutState::Released) => {}
        }
    } else if is_toggle_key && state == ShortcutState::Pressed {
        pipeline::toggle_recording(app);
    }
}
