use crate::errors::{AppError, AppResult};
use crate::pipeline;
use crate::runtime::RuntimeStore;
use crate::settings::{AppSettings, TrayStatus};
use crate::tray;
use serde::Serialize;
use std::str::FromStr;
use tauri::{plugin::TauriPlugin, AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[cfg(test)]
const GLOBAL_SHORTCUT_PLUGIN_NAME: &str = "global-shortcut";
pub const GLOBAL_SHORTCUT_TRIGGERED_EVENT: &str = "question-scan:global-shortcut-triggered";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GlobalShortcutTriggeredPayload {
    shortcut: String,
    trigger_count: u64,
}

/// Builds the official Tauri global shortcut plugin without registering shortcuts yet.
pub fn global_shortcut_plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_global_shortcut::Builder::new().build()
}

/// Registers or unregisters the configured shortcut and keeps runtime state in sync.
pub fn sync_global_shortcut(
    app: &AppHandle,
    settings: &AppSettings,
    runtime: &RuntimeStore,
) -> AppResult<()> {
    let shortcut = settings.global_shortcut.trim();

    if !settings.global_shortcut_enabled {
        unregister_global_shortcuts(app);
        runtime.set_global_shortcut_active(None);
        runtime.set_global_shortcut_error(None);
        runtime.set_tray_status(TrayStatus::Idle);
        let _ = tray::sync_tray(app);
        return Ok(());
    }

    let parsed_shortcut = match parse_shortcut(shortcut) {
        Ok(shortcut) => shortcut,
        Err(error) => {
            runtime.set_global_shortcut_error(Some(error.to_string()));
            runtime.set_tray_status(TrayStatus::Failed);
            let _ = tray::sync_tray(app);
            return Err(error);
        }
    };

    if app.global_shortcut().is_registered(parsed_shortcut) {
        if runtime.is_global_shortcut_active(shortcut) {
            runtime.set_global_shortcut_active(Some(shortcut.to_string()));
            runtime.set_tray_status(TrayStatus::Idle);
            tracing::info!(shortcut = %shortcut, "Global shortcut already registered by this app");
            let _ = tray::sync_tray(app);
            return Ok(());
        }

        let error = AppError::GlobalShortcutAlreadyRegisteredByThisApp {
            shortcut: shortcut.to_string(),
        };
        runtime.set_global_shortcut_error(Some(error.to_string()));
        runtime.set_tray_status(TrayStatus::Failed);
        let _ = tray::sync_tray(app);
        return Err(error);
    }

    unregister_global_shortcuts(app);

    let shortcut_label = shortcut.to_string();
    if let Err(error) =
        app.global_shortcut()
            .on_shortcut(parsed_shortcut, move |app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    handle_shortcut_pressed(app, shortcut_label.clone());
                }
            })
    {
        let error = registration_error(shortcut, error.to_string());
        runtime.set_global_shortcut_error(Some(error.to_string()));
        runtime.set_tray_status(TrayStatus::Failed);
        let _ = tray::sync_tray(app);
        return Err(error);
    }

    runtime.set_global_shortcut_active(Some(shortcut.to_string()));
    runtime.set_tray_status(TrayStatus::Idle);
    tracing::info!(shortcut = %shortcut, "Global shortcut registered");
    let _ = tray::sync_tray(app);
    Ok(())
}

/// Releases all shortcuts owned by this process. Safe to call repeatedly during shutdown.
pub fn unregister_global_shortcuts(app: &AppHandle) {
    if let Err(error) = app.global_shortcut().unregister_all() {
        tracing::warn!(%error, "Could not unregister global shortcuts");
    }
}

fn parse_shortcut(shortcut: &str) -> AppResult<Shortcut> {
    if shortcut.is_empty() {
        return Err(shortcut_error(
            shortcut,
            "A shortcut must include one key and can include modifiers.".to_string(),
        ));
    }

    Shortcut::from_str(shortcut).map_err(|error| shortcut_error(shortcut, error.to_string()))
}

fn shortcut_error(shortcut: &str, reason: String) -> AppError {
    AppError::GlobalShortcutRegistrationFailed {
        shortcut: if shortcut.is_empty() {
            "<empty>".to_string()
        } else {
            shortcut.to_string()
        },
        reason,
    }
}

fn registration_error(shortcut: &str, reason: String) -> AppError {
    if is_already_registered_reason(&reason) {
        AppError::GlobalShortcutOccupiedByAnotherApp {
            shortcut: shortcut.to_string(),
            reason,
        }
    } else {
        shortcut_error(shortcut, reason)
    }
}

fn is_already_registered_reason(reason: &str) -> bool {
    reason.starts_with("HotKey already registered")
}

fn handle_shortcut_pressed(app: &AppHandle, shortcut: String) {
    if let Some(runtime) = app.try_state::<RuntimeStore>() {
        // Prevent concurrent pipeline runs while already processing.
        let snapshot = runtime.snapshot();
        if snapshot.tray_status != TrayStatus::Idle
            && snapshot.tray_status != TrayStatus::Failed
            && snapshot.tray_status != TrayStatus::Complete
        {
            tracing::info!(
                shortcut = %shortcut,
                tray_status = ?snapshot.tray_status,
                "Ignoring shortcut press while already processing"
            );
            return;
        }

        let snapshot = runtime.record_global_shortcut_trigger();
        tracing::info!(
            shortcut = %shortcut,
            trigger_count = snapshot.global_shortcut_trigger_count,
            "Global shortcut triggered"
        );
        let _ = tray::sync_tray(app);

        if let Some(window) = app.get_webview_window("main") {
            let _ = window.show();
            let _ = window.set_focus();
        }

        let _ = app.emit(
            GLOBAL_SHORTCUT_TRIGGERED_EVENT,
            GlobalShortcutTriggeredPayload {
                shortcut,
                trigger_count: snapshot.global_shortcut_trigger_count,
            },
        );

        // Start the screenshot-to-AI pipeline in the background.
        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            pipeline::run_screenshot_to_ai_pipeline(app_clone).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::{plugin::Plugin, Wry};

    #[test]
    fn builds_official_global_shortcut_plugin() {
        let plugin = global_shortcut_plugin::<Wry>();

        assert_eq!(plugin.name(), GLOBAL_SHORTCUT_PLUGIN_NAME);
    }

    #[test]
    fn validates_supported_shortcut_strings() {
        assert!(parse_shortcut("Ctrl+Shift+Q").is_ok());
        assert!(parse_shortcut("Alt+Shift+S").is_ok());
    }

    #[test]
    fn rejects_empty_shortcut_strings() {
        let error = parse_shortcut("").expect_err("empty shortcut should fail");

        assert_eq!(error.to_string().contains("<empty>"), true);
    }

    #[test]
    fn maps_already_registered_errors_to_external_conflict() {
        let error = registration_error(
            "Ctrl+Shift+Q",
            "HotKey already registered: HotKey { mods: Modifiers(CONTROL | SHIFT), key: KeyQ }"
                .to_string(),
        );

        assert_eq!(
            error.to_string(),
            "The global shortcut `Ctrl+Shift+Q` is already in use by another app or the system. Choose another shortcut or disable the global shortcut. Details: HotKey already registered: HotKey { mods: Modifiers(CONTROL | SHIFT), key: KeyQ }"
        );
    }

    #[test]
    fn keeps_non_conflict_registration_errors_generic() {
        let error = registration_error("Ctrl+Shift+Q", "permission denied".to_string());

        assert_eq!(
            error.to_string(),
            "The global shortcut `Ctrl+Shift+Q` could not be registered. Choose another shortcut or disable the global shortcut. Details: permission denied"
        );
    }
}
