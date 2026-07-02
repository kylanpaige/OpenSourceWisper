//! Deliver text into the focused app.
//!
//! Paste is the default: set the clipboard, synthesize Cmd/Ctrl+V, then
//! restore the user's previous clipboard (the pattern used by Handy and
//! hyprvoice). Type mode simulates real keystrokes for apps that block paste.

use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::time::Duration;
use wisper_core::settings::InjectionMethod;

pub fn inject_text(text: &str, method: InjectionMethod) -> Result<(), String> {
    if text.is_empty() {
        return Ok(());
    }
    match method {
        InjectionMethod::Paste => paste(text),
        InjectionMethod::Type => type_text(text),
        InjectionMethod::ClipboardOnly => set_clipboard(text),
    }
}

fn new_enigo() -> Result<Enigo, String> {
    Enigo::new(&Settings::default()).map_err(|e| {
        format!(
            "input simulation unavailable: {e}. On macOS, grant Accessibility \
permission in System Settings > Privacy & Security > Accessibility."
        )
    })
}

fn set_clipboard(text: &str) -> Result<(), String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    cb.set_text(text.to_string()).map_err(|e| e.to_string())
}

fn paste(text: &str) -> Result<(), String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    // Save whatever the user had (text only; images are left alone but will
    // be replaced — acceptable tradeoff, same as other dictation tools).
    let previous = cb.get_text().ok();

    cb.set_text(text.to_string()).map_err(|e| e.to_string())?;
    // Give the clipboard a beat to settle before synthesizing the shortcut.
    std::thread::sleep(Duration::from_millis(60));

    let mut enigo = new_enigo()?;
    let modifier = if cfg!(target_os = "macos") {
        Key::Meta
    } else {
        Key::Control
    };
    enigo
        .key(modifier, Direction::Press)
        .map_err(|e| e.to_string())?;
    enigo
        .key(Key::Unicode('v'), Direction::Click)
        .map_err(|e| e.to_string())?;
    enigo
        .key(modifier, Direction::Release)
        .map_err(|e| e.to_string())?;

    // Let the paste land before restoring the clipboard.
    std::thread::sleep(Duration::from_millis(250));
    if let Some(prev) = previous {
        let _ = cb.set_text(prev);
    }
    Ok(())
}

fn type_text(text: &str) -> Result<(), String> {
    let mut enigo = new_enigo()?;
    enigo.text(text).map_err(|e| e.to_string())
}

/// Copy the current selection (Cmd/Ctrl+C) and return it, restoring the
/// user's clipboard afterwards. Returns None when nothing is selected.
/// Used by Command Mode to grab the text the instruction should edit.
pub fn copy_selection() -> Result<Option<String>, String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    let previous = cb.get_text().ok();
    // Clear so we can tell "nothing selected" from "old clipboard content".
    let _ = cb.clear();

    let mut enigo = new_enigo()?;
    let modifier = if cfg!(target_os = "macos") {
        Key::Meta
    } else {
        Key::Control
    };
    enigo
        .key(modifier, Direction::Press)
        .map_err(|e| e.to_string())?;
    enigo
        .key(Key::Unicode('c'), Direction::Click)
        .map_err(|e| e.to_string())?;
    enigo
        .key(modifier, Direction::Release)
        .map_err(|e| e.to_string())?;
    std::thread::sleep(Duration::from_millis(150));

    let selection = cb.get_text().ok().filter(|s| !s.is_empty());
    if let Some(prev) = previous {
        let _ = cb.set_text(prev);
    }
    Ok(selection)
}
