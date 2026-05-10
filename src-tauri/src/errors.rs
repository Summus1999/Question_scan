use serde::Serialize;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error, Serialize)]
pub enum AppError {
    #[error("The app config directory is unavailable.")]
    ConfigDirUnavailable,
    #[error("The main window is unavailable.")]
    MainWindowUnavailable,
    #[error("The tray icon could not be created.")]
    TrayIconUnavailable,
    #[error("The settings file could not be saved.")]
    SettingsSaveFailed,
}
