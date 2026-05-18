use serde::{ser::SerializeStruct, Serialize, Serializer};
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("The app config directory is unavailable.")]
    ConfigDirUnavailable,
    #[error("The main window is unavailable.")]
    MainWindowUnavailable,
    #[error("The tray icon could not be created.")]
    TrayIconUnavailable,
    #[error("The settings file could not be saved.")]
    SettingsSaveFailed,
    #[allow(dead_code)]
    #[error("The API key could not be stored securely. {reason}")]
    ApiKeyStorageFailed { reason: String },
    #[error("The provider configuration is invalid. {reason}")]
    ProviderConfigurationInvalid { reason: String },
    #[error("The global shortcut `{shortcut}` is already registered by this Question Scan process. Restart the app if this message keeps appearing.")]
    GlobalShortcutAlreadyRegisteredByThisApp { shortcut: String },
    #[error("The global shortcut `{shortcut}` is already in use by another app or the system. Choose another shortcut or disable the global shortcut. Details: {reason}")]
    GlobalShortcutOccupiedByAnotherApp { shortcut: String, reason: String },
    #[error("The global shortcut `{shortcut}` could not be registered. Choose another shortcut or disable the global shortcut. Details: {reason}")]
    GlobalShortcutRegistrationFailed { shortcut: String, reason: String },
    #[error("The AI request failed. {message}")]
    AiRequestFailed { code: String, message: String },
    #[error("The lightweight problem index could not be loaded. {reason}")]
    ProblemIndexLoadFailed { reason: String },
    #[error("The RAG import operation failed. {reason}")]
    RagImportFailed { reason: String },
    #[error("The history RAG record could not be updated. {reason}")]
    RagHistoryIndexFailed { reason: String },
    #[error("The RAG retrieval operation failed. {reason}")]
    RagRetrievalFailed { reason: String },
}

impl AppError {
    fn code(&self) -> &'static str {
        match self {
            Self::ConfigDirUnavailable => "configDirUnavailable",
            Self::MainWindowUnavailable => "mainWindowUnavailable",
            Self::TrayIconUnavailable => "trayIconUnavailable",
            Self::SettingsSaveFailed => "settingsSaveFailed",
            Self::ApiKeyStorageFailed { .. } => "apiKeyStorageFailed",
            Self::ProviderConfigurationInvalid { .. } => "providerConfigurationInvalid",
            Self::GlobalShortcutAlreadyRegisteredByThisApp { .. } => {
                "globalShortcutAlreadyRegisteredByThisApp"
            }
            Self::GlobalShortcutOccupiedByAnotherApp { .. } => "globalShortcutOccupiedByAnotherApp",
            Self::GlobalShortcutRegistrationFailed { .. } => "globalShortcutRegistrationFailed",
            Self::AiRequestFailed { .. } => "aiRequestFailed",
            Self::ProblemIndexLoadFailed { .. } => "problemIndexLoadFailed",
            Self::RagImportFailed { .. } => "ragImportFailed",
            Self::RagHistoryIndexFailed { .. } => "ragHistoryIndexFailed",
            Self::RagRetrievalFailed { .. } => "ragRetrievalFailed",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}
