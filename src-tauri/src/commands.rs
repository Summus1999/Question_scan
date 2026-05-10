use crate::errors::{AppError, AppResult};
use crate::settings::{AppSettings, AppSnapshot, SettingsStore};
use tauri::{AppHandle, Manager, State};

/// Returns the full state payload consumed by the React shell during startup and reload.
#[tauri::command]
pub fn load_app_state(app: AppHandle, store: State<'_, SettingsStore>) -> AppResult<AppSnapshot> {
    let visible = window_visible(&app)?;
    Ok(store.snapshot(visible, app.package_info().version.to_string()))
}

/// Persists user settings and returns a fresh snapshot so the frontend can re-sync.
#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    store: State<'_, SettingsStore>,
    settings: AppSettings,
) -> AppResult<AppSnapshot> {
    store.update(settings)?;
    let visible = window_visible(&app)?;
    Ok(store.snapshot(visible, app.package_info().version.to_string()))
}

/// Restores default settings and returns the same snapshot contract as a save.
#[tauri::command]
pub fn reset_settings(app: AppHandle, store: State<'_, SettingsStore>) -> AppResult<AppSnapshot> {
    store.reset()?;
    let visible = window_visible(&app)?;
    Ok(store.snapshot(visible, app.package_info().version.to_string()))
}

/// Shows the main window from a frontend command and reports current visibility.
#[tauri::command]
pub fn show_main_window(app: AppHandle, store: State<'_, SettingsStore>) -> AppResult<AppSnapshot> {
    set_window_visible(&app, true)?;
    let visible = window_visible(&app)?;
    Ok(store.snapshot(visible, app.package_info().version.to_string()))
}

/// Hides the main window while keeping the tray process alive.
#[tauri::command]
pub fn hide_main_window(app: AppHandle, store: State<'_, SettingsStore>) -> AppResult<AppSnapshot> {
    set_window_visible(&app, false)?;
    let visible = window_visible(&app)?;
    Ok(store.snapshot(visible, app.package_info().version.to_string()))
}

/// Flips the main window visibility using the same path as tray toggles.
#[tauri::command]
pub fn toggle_main_window(
    app: AppHandle,
    store: State<'_, SettingsStore>,
) -> AppResult<AppSnapshot> {
    let visible = !window_visible(&app)?;
    set_window_visible(&app, visible)?;
    let visible = window_visible(&app)?;
    Ok(store.snapshot(visible, app.package_info().version.to_string()))
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
