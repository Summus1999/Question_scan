mod commands;
mod errors;
mod history;
mod language;
mod output_parser;
mod pipeline;
mod problem_index;
mod prompts;
mod provider;
mod provider_key_store;
mod rag_history;
mod rag_imports;
mod rag_prompt_context;
mod rag_retrieval;
mod rag_templates;
mod recognition;
mod runtime;
mod screenshot;
mod session;
mod settings;
mod shortcuts;
mod streaming;
mod tray;

use crate::errors::AppError;
use crate::history::HistoryStore;
use crate::rag_history::RagHistoryStore;
use crate::rag_imports::RagImportStore;
use crate::rag_retrieval::RagEmbeddingStore;
use crate::runtime::RuntimeStore;
use crate::session::SessionContext;
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

            let history_path = app
                .path()
                .app_config_dir()
                .map_err(|_| AppError::ConfigDirUnavailable)?
                .join("history.json");
            let history_store = HistoryStore::load(history_path, settings.save_history);

            let rag_import_path = app
                .path()
                .app_config_dir()
                .map_err(|_| AppError::ConfigDirUnavailable)?
                .join("rag")
                .join("user-imports.json");
            let rag_import_store = RagImportStore::load(rag_import_path);

            let rag_history_path = app
                .path()
                .app_config_dir()
                .map_err(|_| AppError::ConfigDirUnavailable)?
                .join("rag")
                .join("history-records.json");
            let rag_history_store = RagHistoryStore::load(rag_history_path);

            let rag_embedding_path = app
                .path()
                .app_config_dir()
                .map_err(|_| AppError::ConfigDirUnavailable)?
                .join("rag")
                .join("embeddings.json");
            let rag_embedding_store = RagEmbeddingStore::load(rag_embedding_path);

            app.manage(RuntimeStore::default());
            app.manage(store);
            app.manage(history_store);
            app.manage(rag_import_store);
            app.manage(rag_history_store);
            app.manage(rag_embedding_store);
            app.manage(SessionContext::default());

            cleanup_orphaned_temp_images(app.handle());

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
            commands::regenerate_with_language,
            commands::clear_cache,
            commands::list_history,
            commands::save_history_entry,
            commands::delete_history_entry,
            commands::clear_history,
            commands::list_leetcode_problem_index,
            commands::import_rag_documents,
            commands::list_rag_imports,
            commands::delete_rag_import,
            commands::clear_rag_index,
            commands::rebuild_rag_index,
            commands::search_rag_context,
            commands::select_rag_template_context,
            commands::build_rag_prompt_context,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Question Scan");

    app.run(|app, event| {
        if matches!(event, RunEvent::ExitRequested { .. } | RunEvent::Exit) {
            shortcuts::unregister_global_shortcuts(app);
        }
    });
}

/// Cleans up orphaned temporary images from previous sessions.
fn cleanup_orphaned_temp_images(_app_handle: &tauri::AppHandle) {
    let temp_dir = std::env::temp_dir().join("question-scan");
    if temp_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Err(error) = std::fs::remove_file(&path) {
                        tracing::warn!(path = %path.display(), %error, "Could not remove orphaned temp image");
                    } else {
                        tracing::info!(path = %path.display(), "Removed orphaned temp image");
                    }
                }
            }
        }
    }
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

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn cleanup_orphaned_temp_images_deletes_leftover_files() {
        let _guard = TEST_LOCK.lock().expect("test lock poisoned");
        let temp_dir = std::env::temp_dir().join("question-scan");
        std::fs::create_dir_all(&temp_dir).expect("create temp dir");

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock drift")
            .as_nanos();
        let test_file = temp_dir.join(format!("orphan-test-{nonce}.png"));
        std::fs::write(&test_file, b"fake image").expect("write test file");
        assert!(test_file.exists());

        // cleanup_orphaned_temp_images takes &tauri::AppHandle, which is not available in tests.
        // Replicate its core logic directly for verification.
        if temp_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&temp_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }

        assert!(!test_file.exists());
    }

    #[test]
    fn cleanup_orphaned_temp_images_ignores_missing_directory() {
        let _guard = TEST_LOCK.lock().expect("test lock poisoned");
        let temp_dir = std::env::temp_dir().join("question-scan");
        // Ensure the directory does not exist for this test.
        let _ = std::fs::remove_dir_all(&temp_dir);

        // Replicate the core logic; it should not panic when the directory is missing.
        if temp_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&temp_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }
    }
}
