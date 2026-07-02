//! Focused-app detection for per-app profiles (Wispr Flow's "context awareness").

/// Name of the app that owns the focused window, best-effort.
/// Returns an empty string when detection fails (e.g. missing permissions).
pub fn focused_app_name() -> String {
    match active_win_pos_rs::get_active_window() {
        Ok(win) => {
            if !win.app_name.is_empty() {
                win.app_name
            } else {
                win.title
            }
        }
        Err(_) => String::new(),
    }
}
