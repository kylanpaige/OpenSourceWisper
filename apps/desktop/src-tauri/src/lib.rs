mod asr;
mod audio;
mod autostart;
mod commands;
mod context;
mod downloads;
mod hotkeys;
mod inject;
mod overlay;
mod paths;
mod pipeline;
mod state;

use state::AppState;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};
use wisper_core::dictionary::DictionaryEntry;
use wisper_core::AppSettings;

fn load_json<T: serde::de::DeserializeOwned + Default>(path: &std::path::Path) -> T {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open OpenSourceWisper", true, None::<&str>)?;
    let toggle = MenuItem::with_id(app, "toggle", "Start/Stop Dictation", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &toggle, &quit])?;

    TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("OpenSourceWisper — local dictation")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
            "toggle" => pipeline::toggle_recording(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    hotkeys::handle_shortcut(app, shortcut, event.state());
                })
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let handle = app.handle().clone();

            let settings: AppSettings = load_json(&paths::settings_path(&handle));
            let dictionary: Vec<DictionaryEntry> = load_json(&paths::dictionary_path(&handle));
            app.manage(AppState::new(settings, dictionary));

            build_tray(&handle)?;
            if let Err(e) = hotkeys::register_from_settings(&handle) {
                log::error!("hotkey registration failed: {e}");
            }

            // Closing the main window hides to tray; the app keeps listening.
            if let Some(win) = app.get_webview_window("main") {
                let w = win.clone();
                win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = w.hide();
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::set_settings,
            commands::get_dictionary,
            commands::set_dictionary,
            commands::list_models,
            commands::download_model,
            commands::delete_model,
            commands::list_input_devices,
            commands::start_dictation,
            commands::stop_dictation,
            commands::cancel_dictation,
            commands::get_phase,
            commands::get_history,
            commands::clear_history,
            commands::test_cleanup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OpenSourceWisper");
}
