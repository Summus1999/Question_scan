mod commands;
mod errors;
mod language;
mod output_parser;
mod prompts;
mod provider;
mod provider_key_store;
mod recognition;
mod runtime;
mod screenshot;
mod settings;
mod shortcuts;
mod streaming;
mod tray;

use crate::errors::AppError;
use crate::runtime::RuntimeStore;
use crate::settings::SettingsStore;
use tauri::{Manager, RunEvent, WindowEvent};
use tracing_subscriber::EnvFilter;

// Boots the Tauri runtime, wires app state, tray integration, and frontend commands.
fn main() {
    init_logging();

    let app = tauri::Builder::default()
        .plugin(shortcuts::global_shortcut_plugin())
        .setup(|app| {
            let settings_path = app
                .path()
                .app_config_dir()
                .map_err(|_| AppError::ConfigDirUnavailable)?
                .join("settings.json");
            let store = SettingsStore::load(settings_path);
            let settings = store.current();
            let launch_to_tray = settings.launch_to_tray;

            app.manage(RuntimeStore::default());
            app.manage(store);
            tray::build_tray(app.handle())?;

            let runtime = app.state::<RuntimeStore>();
            if let Err(error) = shortcuts::sync_global_shortcut(app.handle(), &settings, &runtime) {
                runtime.set_global_shortcut_error(Some(error.to_string()));
                runtime.set_tray_status(crate::settings::TrayStatus::Failed);
                tray::sync_tray(app.handle())?;
                tracing::warn!(%error, "Global shortcut registration failed during startup");
            }

            let window = app
                .get_webview_window("main")
                .ok_or(AppError::MainWindowUnavailable)?;

            if launch_to_tray {
                let _ = window.hide();
            } else {
                let _ = window.show();
                let _ = window.set_focus();
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::load_app_state,
            commands::save_settings,
            commands::reset_settings,
            commands::show_main_window,
            commands::hide_main_window,
            commands::toggle_main_window,
            commands::set_tray_status,
            commands::send_ai_request,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Question Scan");

    app.run(|app, event| {
        if matches!(event, RunEvent::ExitRequested { .. } | RunEvent::Exit) {
            shortcuts::unregister_global_shortcuts(app);
        }
    });
}

// Initializes logging once; duplicate init attempts in dev/test should stay harmless.
fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("question_scan=info,tauri=info,wry=warn"));

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .without_time()
        .compact()
        .try_init();
}
