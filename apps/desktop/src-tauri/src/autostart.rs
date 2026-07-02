//! Launch-at-login via tauri-plugin-autostart, kept in sync with settings.

use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

pub fn sync(app: &AppHandle, enabled: bool) {
    let manager = app.autolaunch();
    let result = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };
    if let Err(e) = result {
        log::warn!("could not update launch-at-login: {e}");
    }
}
