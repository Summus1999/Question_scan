use crate::commands::{set_window_visible, toggle_window};
use crate::errors::{AppError, AppResult};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle,
};

const TRAY_ICON_BYTES: &[u8] = include_bytes!("../icons/tray-icon.png");

/// Creates the tray icon, tray menu, and window visibility handlers for background mode.
pub fn build_tray(app: &AppHandle) -> AppResult<()> {
    let toggle_item = MenuItem::with_id(app, "toggle-window", "Toggle window", true, None::<&str>)
        .map_err(|error| {
            tracing::error!(%error, "Could not create the tray toggle item");
            AppError::TrayIconUnavailable
        })?;
    let show_item = MenuItem::with_id(app, "show-window", "Show window", true, None::<&str>)
        .map_err(|error| {
            tracing::error!(%error, "Could not create the tray show item");
            AppError::TrayIconUnavailable
        })?;
    let hide_item = MenuItem::with_id(app, "hide-window", "Hide window", true, None::<&str>)
        .map_err(|error| {
            tracing::error!(%error, "Could not create the tray hide item");
            AppError::TrayIconUnavailable
        })?;
    let quit_item = PredefinedMenuItem::quit(app, Some("Quit")).map_err(|error| {
        tracing::error!(%error, "Could not create the tray quit item");
        AppError::TrayIconUnavailable
    })?;

    let separator_b = PredefinedMenuItem::separator(app).map_err(|error| {
        tracing::error!(%error, "Could not create the tray separator");
        AppError::TrayIconUnavailable
    })?;
    let separator_c = PredefinedMenuItem::separator(app).map_err(|error| {
        tracing::error!(%error, "Could not create the tray separator");
        AppError::TrayIconUnavailable
    })?;

    let menu = Menu::with_items(
        app,
        &[
            &toggle_item,
            &separator_b,
            &show_item,
            &hide_item,
            &separator_c,
            &quit_item,
        ],
    )
    .map_err(|error| {
        tracing::error!(%error, "Could not create the tray menu");
        AppError::TrayIconUnavailable
    })?;

    let icon = Image::from_bytes(TRAY_ICON_BYTES).map_err(|error| {
        tracing::error!(%error, "Could not decode the tray icon");
        AppError::TrayIconUnavailable
    })?;

    TrayIconBuilder::with_id("main")
        .menu(&menu)
        .icon(icon)
        .tooltip("Question Scan")
        .show_menu_on_left_click(false)
        // Left-click shares the same toggle path as the menu item for easier tray debugging.
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button,
                button_state,
                ..
            } = event
            {
                if button == MouseButton::Left && button_state == MouseButtonState::Down {
                    let _ = toggle_window(tray.app_handle());
                }
            }
        })
        // Keep every menu action mapped in one match so tray command issues are localized.
        .on_menu_event(|app, event| match event.id().as_ref() {
            "toggle-window" => {
                let _ = toggle_window(app);
            }
            "show-window" => {
                let _ = set_window_visible(app, true);
            }
            "hide-window" => {
                let _ = set_window_visible(app, false);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)
        .map_err(|error| {
            tracing::error!(%error, "Could not build the tray icon");
            AppError::TrayIconUnavailable
        })?;

    Ok(())
}
