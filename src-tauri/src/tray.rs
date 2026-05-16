/**
 * Question Scan 系统托盘模块
 *
 * 职责：创建托盘图标和菜单，处理托盘点击事件，同步托盘状态与运行时状态。
 * 托盘是应用后台运行的核心入口，用户通过托盘控制窗口显隐、快捷键和退出。
 */
use crate::commands::{set_window_visible, toggle_window};
use crate::errors::{AppError, AppResult};
use crate::runtime::RuntimeStore;
use crate::settings::{SettingsStore, TrayStatus};
use crate::shortcuts;
use serde::Serialize;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Wry,
};

/** 托盘图标二进制数据，打包时嵌入可执行文件。 */
const TRAY_ICON_BYTES: &[u8] = include_bytes!("../icons/tray-icon.png");
/** 托盘图标 ID，与 tauri.conf.json 中的 trayIcon.id 对应。 */
const TRAY_ID: &str = "main";
/** "打开设置"事件名。托盘菜单选择设置时发射给前端。 */
pub const OPEN_SETTINGS_EVENT: &str = "question-scan:open-settings";
/** 运行时状态变更事件名。托盘操作（如切换快捷键）后通知前端刷新。 */
pub const RUNTIME_STATE_CHANGED_EVENT: &str = "question-scan:runtime-state-changed";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeStateChangedPayload {
    reason: &'static str,
}

struct TrayController {
    status_item: MenuItem<Wry>,
    shortcut_item: MenuItem<Wry>,
}

/**
 * 构建托盘图标、菜单和事件处理器。
 * 应用启动时调用一次，创建完整的托盘交互体系。
 */
pub fn build_tray(app: &AppHandle) -> AppResult<()> {
    let status_item = MenuItem::with_id(app, "tray-status", "Status: Idle", false, None::<&str>)
        .map_err(|error| {
            tracing::error!(%error, "Could not create the tray status item");
            AppError::TrayIconUnavailable
        })?;
    let open_item = MenuItem::with_id(app, "open-window", "Open window", true, None::<&str>)
        .map_err(|error| {
            tracing::error!(%error, "Could not create the tray open item");
            AppError::TrayIconUnavailable
        })?;
    let settings_item =
        MenuItem::with_id(app, "open-settings", "Open settings", true, None::<&str>).map_err(
            |error| {
                tracing::error!(%error, "Could not create the tray settings item");
                AppError::TrayIconUnavailable
            },
        )?;
    let hide_item = MenuItem::with_id(app, "hide-window", "Hide window", true, None::<&str>)
        .map_err(|error| {
            tracing::error!(%error, "Could not create the tray hide item");
            AppError::TrayIconUnavailable
        })?;
    let shortcut_item = MenuItem::with_id(
        app,
        "toggle-shortcut",
        "Disable shortcut",
        true,
        None::<&str>,
    )
    .map_err(|error| {
        tracing::error!(%error, "Could not create the tray shortcut item");
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
            &status_item,
            &separator_b,
            &open_item,
            &settings_item,
            &hide_item,
            &shortcut_item,
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

    let tray = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .icon(icon)
        .tooltip("Question Scan - Idle")
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
            "open-window" => {
                let _ = set_window_visible(app, true);
            }
            "open-settings" => {
                let _ = open_settings(app);
            }
            "hide-window" => {
                let _ = set_window_visible(app, false);
            }
            "toggle-shortcut" => {
                let _ = toggle_shortcut_enabled(app);
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

    app.manage(TrayController {
        status_item,
        shortcut_item,
    });
    sync_tray(app)?;
    let _ = tray;

    Ok(())
}

/**
 * 同步托盘显示状态。
 * 根据当前运行时状态和设置更新 tooltip 和菜单项标签。
 * 截图/识别/生成等流程中应调用此函数反馈进度。
 */
pub fn sync_tray(app: &AppHandle) -> AppResult<()> {
    let runtime = app
        .try_state::<RuntimeStore>()
        .map(|state| state.snapshot())
        .unwrap_or_default();
    let settings = app
        .try_state::<SettingsStore>()
        .map(|state| state.current());

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_tooltip(Some(tray_tooltip(runtime.tray_status)))
            .map_err(|error| {
                tracing::error!(%error, "Could not update the tray tooltip");
                AppError::TrayIconUnavailable
            })?;
    }

    if let Some(controller) = app.try_state::<TrayController>() {
        controller
            .status_item
            .set_text(status_menu_label(runtime.tray_status))
            .map_err(|error| {
                tracing::error!(%error, "Could not update the tray status item");
                AppError::TrayIconUnavailable
            })?;

        if let Some(settings) = settings {
            controller
                .shortcut_item
                .set_text(if settings.global_shortcut_enabled {
                    "Disable shortcut"
                } else {
                    "Enable shortcut"
                })
                .map_err(|error| {
                    tracing::error!(%error, "Could not update the tray shortcut item");
                    AppError::TrayIconUnavailable
                })?;
        }
    }

    Ok(())
}

/**
 * 设置托盘状态。
 * 通过运行时状态机统一更新，保证前端和托盘显示一致。
 */
pub fn set_tray_status(app: &AppHandle, status: TrayStatus) -> AppResult<()> {
    if let Some(runtime) = app.try_state::<RuntimeStore>() {
        runtime.set_tray_status(status);
    }
    sync_tray(app)
}

fn open_settings(app: &AppHandle) -> AppResult<()> {
    set_window_visible(app, true)?;
    app.emit(
        OPEN_SETTINGS_EVENT,
        RuntimeStateChangedPayload {
            reason: "open-settings",
        },
    )
    .map_err(|error| {
        tracing::error!(%error, "Could not emit open settings event");
        AppError::MainWindowUnavailable
    })?;
    Ok(())
}

fn toggle_shortcut_enabled(app: &AppHandle) -> AppResult<()> {
    let Some(store) = app.try_state::<SettingsStore>() else {
        return Ok(());
    };
    let Some(runtime) = app.try_state::<RuntimeStore>() else {
        return Ok(());
    };

    let mut settings = store.current();
    settings.global_shortcut_enabled = !settings.global_shortcut_enabled;

    match shortcuts::sync_global_shortcut(app, &settings, &runtime) {
        Ok(()) => {
            store.update(settings)?;
            sync_tray(app)?;
            let _ = app.emit(
                RUNTIME_STATE_CHANGED_EVENT,
                RuntimeStateChangedPayload {
                    reason: "shortcut-toggle",
                },
            );
            Ok(())
        }
        Err(error) => {
            runtime.set_global_shortcut_error(Some(error.to_string()));
            runtime.set_tray_status(TrayStatus::Failed);
            sync_tray(app)?;
            let _ = app.emit(
                RUNTIME_STATE_CHANGED_EVENT,
                RuntimeStateChangedPayload {
                    reason: "shortcut-error",
                },
            );
            Err(error)
        }
    }
}

fn tray_tooltip(status: TrayStatus) -> String {
    format!("Question Scan - {}", tray_status_label(status))
}

fn status_menu_label(status: TrayStatus) -> String {
    format!("Status: {}", tray_status_label(status))
}

fn tray_status_label(status: TrayStatus) -> &'static str {
    match status {
        TrayStatus::Idle => "Idle",
        TrayStatus::Screenshot => "Screenshot",
        TrayStatus::Recognizing => "Recognizing",
        TrayStatus::Generating => "Generating",
        TrayStatus::Complete => "Complete",
        TrayStatus::Failed => "Failed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_status_labels_cover_all_runtime_statuses() {
        assert_eq!(tray_status_label(TrayStatus::Idle), "Idle");
        assert_eq!(tray_status_label(TrayStatus::Screenshot), "Screenshot");
        assert_eq!(tray_status_label(TrayStatus::Recognizing), "Recognizing");
        assert_eq!(tray_status_label(TrayStatus::Generating), "Generating");
        assert_eq!(tray_status_label(TrayStatus::Complete), "Complete");
        assert_eq!(tray_status_label(TrayStatus::Failed), "Failed");
    }

    #[test]
    fn tray_tooltip_includes_current_status() {
        assert_eq!(tray_tooltip(TrayStatus::Failed), "Question Scan - Failed");
    }
}
