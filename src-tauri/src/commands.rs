use crate::errors::{AppError, AppResult};
use crate::provider::{build_openai_multimodal_request, OpenAiImageInput};
use crate::runtime::RuntimeStore;
use crate::settings::{AppSettings, AppSnapshot, SettingsStore};
use crate::{shortcuts, streaming, tray};
use tauri::{AppHandle, Manager, State};

/// Returns the full state payload consumed by the React shell during startup and reload.
#[tauri::command]
pub fn load_app_state(
    app: AppHandle,
    store: State<'_, SettingsStore>,
    runtime: State<'_, RuntimeStore>,
) -> AppResult<AppSnapshot> {
    snapshot(&app, &store, &runtime)
}

/// Persists user settings and returns a fresh snapshot so the frontend can re-sync.
#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    store: State<'_, SettingsStore>,
    runtime: State<'_, RuntimeStore>,
    settings: AppSettings,
) -> AppResult<AppSnapshot> {
    let settings = settings.sanitized();
    shortcuts::sync_global_shortcut(&app, &settings, &runtime)?;
    store.update(settings)?;
    tray::sync_tray(&app)?;
    snapshot(&app, &store, &runtime)
}

/// Restores default settings and returns the same snapshot contract as a save.
#[tauri::command]
pub fn reset_settings(
    app: AppHandle,
    store: State<'_, SettingsStore>,
    runtime: State<'_, RuntimeStore>,
) -> AppResult<AppSnapshot> {
    let settings = AppSettings::default();
    shortcuts::sync_global_shortcut(&app, &settings, &runtime)?;
    store.reset()?;
    tray::sync_tray(&app)?;
    snapshot(&app, &store, &runtime)
}

/// Shows the main window from a frontend command and reports current visibility.
#[tauri::command]
pub fn show_main_window(
    app: AppHandle,
    store: State<'_, SettingsStore>,
    runtime: State<'_, RuntimeStore>,
) -> AppResult<AppSnapshot> {
    set_window_visible(&app, true)?;
    snapshot(&app, &store, &runtime)
}

/// Hides the main window while keeping the tray process alive.
#[tauri::command]
pub fn hide_main_window(
    app: AppHandle,
    store: State<'_, SettingsStore>,
    runtime: State<'_, RuntimeStore>,
) -> AppResult<AppSnapshot> {
    set_window_visible(&app, false)?;
    snapshot(&app, &store, &runtime)
}

/// Flips the main window visibility using the same path as tray toggles.
#[tauri::command]
pub fn toggle_main_window(
    app: AppHandle,
    store: State<'_, SettingsStore>,
    runtime: State<'_, RuntimeStore>,
) -> AppResult<AppSnapshot> {
    let visible = !window_visible(&app)?;
    set_window_visible(&app, visible)?;
    snapshot(&app, &store, &runtime)
}

/// Updates the tray phase for future capture/recognition/generation workflows.
#[tauri::command]
pub fn set_tray_status(
    app: AppHandle,
    store: State<'_, SettingsStore>,
    runtime: State<'_, RuntimeStore>,
    status: crate::settings::TrayStatus,
) -> AppResult<AppSnapshot> {
    tray::set_tray_status(&app, status)?;
    snapshot(&app, &store, &runtime)
}

/// Applies an explicit visibility state and normalizes Tauri window errors.
pub(crate) fn set_window_visible(app: &AppHandle, visible: bool) -> AppResult<bool> {
    let window = main_window(app)?;

    if visible {
        window.show().map_err(|error| {
            tracing::error!(%error, "Could not show the main window");
            AppError::MainWindowUnavailable
        })?;
        let _ = window.set_focus();
    } else {
        window.hide().map_err(|error| {
            tracing::error!(%error, "Could not hide the main window");
            AppError::MainWindowUnavailable
        })?;
    }

    window_visible(app)
}

/// Computes the next visibility state for tray and menu toggle actions.
pub(crate) fn toggle_window(app: &AppHandle) -> AppResult<bool> {
    let next_visible = !window_visible(app)?;
    set_window_visible(app, next_visible)
}

/// Finds the primary webview window by its configured label.
pub(crate) fn main_window(app: &AppHandle) -> AppResult<tauri::WebviewWindow> {
    app.get_webview_window("main")
        .ok_or(AppError::MainWindowUnavailable)
}

/// Reads Tauri's current visibility flag and maps inspection failures into AppError.
pub(crate) fn window_visible(app: &AppHandle) -> AppResult<bool> {
    let window = main_window(app)?;
    window.is_visible().map_err(|error| {
        tracing::error!(%error, "Could not inspect the main window visibility");
        AppError::MainWindowUnavailable
    })
}

/// Starts an AI multimodal request in the background and returns immediately.
/// Streaming or non-streaming results are delivered via `question-scan:ai-stream-event`.
#[tauri::command]
pub async fn send_ai_request(
    app: AppHandle,
    store: State<'_, SettingsStore>,
    instruction: String,
    image_bytes: Vec<u8>,
    image_mime_type: String,
) -> AppResult<()> {
    let settings = store.current();
    let config = crate::provider::validate_provider_request_config(&settings).map_err(|error| {
        AppError::ProviderConfigurationInvalid {
            reason: error.to_string(),
        }
    })?;

    let request = build_openai_multimodal_request(
        &config,
        instruction,
        OpenAiImageInput::new(image_mime_type, image_bytes),
    )
    .map_err(|error| AppError::ProviderConfigurationInvalid {
        reason: error.to_string(),
    })?;

    // Spawn the request so the command returns immediately.
    tauri::async_runtime::spawn(async move {
        if let Err(error) = streaming::send_openai_request(app, request).await {
            tracing::error!(%error, "AI request failed");
        }
    });

    Ok(())
}

fn snapshot(
    app: &AppHandle,
    store: &SettingsStore,
    runtime: &RuntimeStore,
) -> AppResult<AppSnapshot> {
    let visible = window_visible(app)?;
    Ok(store.snapshot(
        visible,
        app.package_info().version.to_string(),
        runtime.snapshot(),
    ))
}
