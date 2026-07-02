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

    let (main_hotkey, toggle_hotkey, command_hotkey) = {
        let state = app.state::<AppState>();
        let s = state.settings.lock().unwrap();
        (
            s.hotkey.clone(),
            s.hotkey_toggle.clone(),
            s.hotkey_command.clone(),
        )
    };

    let mut seen: Vec<String> = Vec::new();
    for (label, key) in [
        ("hotkey", &main_hotkey),
        ("toggle hotkey", &toggle_hotkey),
        ("command hotkey", &command_hotkey),
    ] {
        let key = key.trim();
        if key.is_empty() || seen.iter().any(|s| s == key) {
            continue;
        }
        let shortcut: Shortcut = key
            .parse()
            .map_err(|e| format!("invalid {label} \"{key}\": {e}"))?;
        gs.register(shortcut)
            .map_err(|e| format!("could not register {label} \"{key}\": {e}"))?;
        seen.push(key.to_string());
    }
    Ok(())
}

/// Plugin event handler installed once at startup.
pub fn handle_shortcut(app: &AppHandle, shortcut: &Shortcut, state: ShortcutState) {
    let (main_hotkey, toggle_hotkey, command_hotkey, mode) = {
        let st = app.state::<AppState>();
        let s = st.settings.lock().unwrap();
        (
            s.hotkey.clone(),
            s.hotkey_toggle.clone(),
            s.hotkey_command.clone(),
            s.recording_mode,
        )
    };

    let matches = |accel: &str| {
        !accel.is_empty()
            && accel
                .parse::<Shortcut>()
                .map(|s| s == *shortcut)
                .unwrap_or(false)
    };

    if matches(&main_hotkey) {
        match (mode, state) {
            (RecordingMode::PushToTalk, ShortcutState::Pressed) => pipeline::start_recording(app),
            (RecordingMode::PushToTalk, ShortcutState::Released) => pipeline::stop_and_process(app),
            (RecordingMode::Toggle, ShortcutState::Pressed) => pipeline::toggle_recording(app),
            (RecordingMode::Toggle, ShortcutState::Released) => {}
        }
    } else if matches(&command_hotkey) {
        // Command Mode is always push-to-talk: hold, speak the instruction, release.
        match state {
            ShortcutState::Pressed => pipeline::start_command(app),
            ShortcutState::Released => pipeline::stop_and_process(app),
        }
    } else if matches(&toggle_hotkey) && state == ShortcutState::Pressed {
        pipeline::toggle_recording(app);
    }
}
