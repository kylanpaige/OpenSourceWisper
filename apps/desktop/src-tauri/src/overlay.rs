//! Floating recording indicator: a small always-on-top pill that shows the
//! current pipeline phase near the bottom of the screen (Wispr Flow style).
//! The window is created lazily and shown/hidden as the phase changes.

use crate::state::{AppState, DictationPhase};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

const OVERLAY_LABEL: &str = "overlay";
const WIDTH: f64 = 220.0;
const HEIGHT: f64 = 56.0;

fn ensure_window(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    if let Some(w) = app.get_webview_window(OVERLAY_LABEL) {
        return Some(w);
    }
    let win = WebviewWindowBuilder::new(app, OVERLAY_LABEL, WebviewUrl::App("overlay.html".into()))
        .title("Recording")
        .inner_size(WIDTH, HEIGHT)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .shadow(false)
        .visible(false)
        .focused(false)
        .build()
        .ok()?;

    // Bottom-center of the primary monitor.
    if let Ok(Some(monitor)) = win.primary_monitor() {
        let size = monitor.size();
        let scale = monitor.scale_factor();
        let x = (size.width as f64 / scale - WIDTH) / 2.0;
        let y = size.height as f64 / scale - HEIGHT - 48.0;
        let _ = win.set_position(tauri::LogicalPosition::new(x, y));
    }
    Some(win)
}

/// Show/hide the overlay to match the pipeline phase.
pub fn sync(app: &AppHandle, phase: DictationPhase) {
    let enabled = {
        let state = app.state::<AppState>();
        let s = state.settings.lock().unwrap();
        s.show_overlay
    };
    if !enabled {
        if let Some(w) = app.get_webview_window(OVERLAY_LABEL) {
            let _ = w.hide();
        }
        return;
    }
    let Some(win) = ensure_window(app) else {
        return;
    };
    match phase {
        DictationPhase::Idle => {
            let _ = win.hide();
        }
        _ => {
            let _ = win.show();
        }
    }
}
