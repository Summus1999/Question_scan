use crate::errors::{AppError, AppResult};
use crate::runtime::RuntimeSnapshot;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LanguageId {
    Cpp17,
    Cpp20,
    Python,
    Java,
    JavaScript,
    TypeScript,
    Go,
    Rust,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OutputSpeed {
    Fast,
    Normal,
    Slow,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThemePreference {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UiLocale {
    ZhCn,
    EnUs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AppStatus {
    Loading,
    Ready,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TrayStatus {
    Idle,
    Screenshot,
    Recognizing,
    Generating,
    Complete,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScreenshotState {
    Idle,
    Capturing,
    Cropping,
    Selecting,
    Ready,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AiResultState {
    Idle,
    Loading,
    Streaming,
    Complete,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub provider_base_url: String,
    pub provider_model: String,
    pub default_language: LanguageId,
    pub output_speed: OutputSpeed,
    pub custom_characters_per_second: u32,
    pub global_shortcut: String,
    #[serde(default = "default_global_shortcut_enabled")]
    pub global_shortcut_enabled: bool,
    pub save_history: bool,
    pub launch_to_tray: bool,
    pub theme: ThemePreference,
    #[serde(default = "default_ui_locale")]
    pub ui_locale: UiLocale,
}

impl Default for AppSettings {
    /// Centralizes MVP defaults so frontend fallback state and backend persistence stay aligned.
    fn default() -> Self {
        Self {
            provider_base_url: "https://api.openai.com/v1".to_string(),
            provider_model: "gpt-4o-mini".to_string(),
            default_language: LanguageId::Cpp20,
            output_speed: OutputSpeed::Normal,
            custom_characters_per_second: 24,
            global_shortcut: "Ctrl+Shift+Q".to_string(),
            global_shortcut_enabled: true,
            save_history: false,
            launch_to_tray: false,
            theme: ThemePreference::System,
            ui_locale: UiLocale::ZhCn,
        }
    }
}

/// Preserves old settings files that predate the explicit interface language.
fn default_ui_locale() -> UiLocale {
    UiLocale::ZhCn
}

/// Keeps legacy settings files active when they predate the shortcut toggle.
fn default_global_shortcut_enabled() -> bool {
    true
}

impl AppSettings {
    /// Trims user-editable string fields and repairs invalid custom speed values.
    pub fn sanitized(mut self) -> Self {
        self.provider_base_url = self.provider_base_url.trim().to_string();
        self.provider_model = self.provider_model.trim().to_string();
        self.global_shortcut = self.global_shortcut.trim().to_string();
        if self.custom_characters_per_second == 0 {
            self.custom_characters_per_second = 24;
        }
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub status: AppStatus,
    pub tray_status: TrayStatus,
    pub screenshot_state: ScreenshotState,
    pub ai_result_state: AiResultState,
    pub settings: AppSettings,
    pub settings_path: String,
    pub startup_warning: Option<String>,
    pub window_visible: bool,
    pub version: String,
    pub global_shortcut_registered: bool,
    pub global_shortcut_error: Option<String>,
    pub global_shortcut_trigger_count: u64,
}

#[derive(Debug)]
pub struct SettingsStore {
    path: PathBuf,
    settings: RwLock<AppSettings>,
    startup_warning: Mutex<Option<String>>,
}

impl SettingsStore {
    /// Loads settings from disk while keeping startup recoverable when the file is missing or corrupt.
    pub fn load(path: PathBuf) -> Self {
        let (settings, startup_warning) = match fs::read_to_string(&path) {
            Ok(contents) => {
                match serde_json::from_str::<AppSettings>(&contents) {
                    Ok(settings) => (settings.sanitized(), None),
                    Err(error) => {
                        tracing::warn!(path = %path.display(), %error, "Could not parse the settings file");
                        (
            AppSettings::default(),
            Some("The settings file could not be parsed. Default values were loaded.".to_string()),
          )
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                (AppSettings::default(), None)
            }
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "Could not read the settings file");
                (
                    AppSettings::default(),
                    Some(
                        "The settings file could not be read. Default values were loaded."
                            .to_string(),
                    ),
                )
            }
        };

        Self {
            path,
            settings: RwLock::new(settings),
            startup_warning: Mutex::new(startup_warning),
        }
    }

    /// Returns a cloned settings copy so callers never hold the read lock.
    pub fn current(&self) -> AppSettings {
        self.settings
            .read()
            .expect("settings lock poisoned")
            .clone()
    }

    /// Returns the recoverable startup warning, if loading fell back to defaults.
    pub fn startup_warning(&self) -> Option<String> {
        self.startup_warning
            .lock()
            .expect("warning lock poisoned")
            .clone()
    }

    /// Sanitizes, writes, and publishes settings in one path to avoid frontend/backend drift.
    pub fn update(&self, settings: AppSettings) -> AppResult<AppSettings> {
        let settings = settings.sanitized();
        self.write_to_disk(&settings)?;
        *self.settings.write().expect("settings lock poisoned") = settings.clone();
        *self.startup_warning.lock().expect("warning lock poisoned") = None;
        Ok(settings)
    }

    /// Restores persisted settings to the same defaults used on a first launch.
    pub fn reset(&self) -> AppResult<AppSettings> {
        self.update(AppSettings::default())
    }

    /// Builds the complete frontend snapshot from persisted settings and transient runtime state.
    pub fn snapshot(
        &self,
        window_visible: bool,
        version: impl Into<String>,
        runtime: RuntimeSnapshot,
    ) -> AppSnapshot {
        let warning = self
            .startup_warning()
            .or_else(|| runtime.global_shortcut_error.clone());
        AppSnapshot {
            status: if warning.is_some() {
                AppStatus::Warning
            } else {
                AppStatus::Ready
            },
            tray_status: runtime.tray_status,
            screenshot_state: runtime.screenshot_state,
            ai_result_state: runtime.ai_result_state,
            settings: self.current(),
            settings_path: self.path.display().to_string(),
            startup_warning: warning,
            window_visible,
            version: version.into(),
            global_shortcut_registered: runtime.global_shortcut_registered,
            global_shortcut_error: runtime.global_shortcut_error,
            global_shortcut_trigger_count: runtime.global_shortcut_trigger_count,
        }
    }

    /// Writes the settings file and maps all filesystem failures to a user-safe error.
    fn write_to_disk(&self, settings: &AppSettings) -> AppResult<()> {
        let parent = self.path.parent().ok_or(AppError::SettingsSaveFailed)?;
        fs::create_dir_all(parent).map_err(|error| {
      tracing::error!(path = %parent.display(), %error, "Could not create the settings directory");
      AppError::SettingsSaveFailed
    })?;

        let file = fs::File::create(&self.path).map_err(|error| {
      tracing::error!(path = %self.path.display(), %error, "Could not open the settings file for writing");
      AppError::SettingsSaveFailed
    })?;

        serde_json::to_writer_pretty(file, settings).map_err(|error| {
      tracing::error!(path = %self.path.display(), %error, "Could not serialize the settings file");
      AppError::SettingsSaveFailed
    })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Generates isolated test paths under target/ so settings tests never share state.
    fn temp_path(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock drift")
            .as_nanos();
        std::env::current_dir()
            .expect("cwd")
            .join("target")
            .join("question-scan-tests")
            .join(format!("{name}-{nonce}"))
            .join("settings.json")
    }

    #[test]
    fn defaults_are_stable() {
        let settings = AppSettings::default();
        assert_eq!(settings.provider_base_url, "https://api.openai.com/v1");
        assert_eq!(settings.provider_model, "gpt-4o-mini");
        assert_eq!(settings.default_language, LanguageId::Cpp20);
        assert_eq!(settings.output_speed, OutputSpeed::Normal);
        assert_eq!(settings.custom_characters_per_second, 24);
        assert_eq!(settings.global_shortcut, "Ctrl+Shift+Q");
        assert!(settings.global_shortcut_enabled);
        assert!(!settings.save_history);
        assert!(!settings.launch_to_tray);
        assert_eq!(settings.theme, ThemePreference::System);
        assert_eq!(settings.ui_locale, UiLocale::ZhCn);
    }

    #[test]
    fn load_missing_file_uses_defaults_without_warning() {
        let path = temp_path("missing");
        let store = SettingsStore::load(path.clone());

        assert_eq!(store.current(), AppSettings::default());
        assert_eq!(store.startup_warning(), None);
        assert_eq!(store.path, path);
    }

    #[test]
    fn load_corrupted_file_keeps_the_app_alive() {
        let path = temp_path("corrupted");
        fs::create_dir_all(path.parent().expect("parent path")).expect("create temp dir");
        fs::write(&path, "{ not valid json").expect("write file");

        let store = SettingsStore::load(path);

        assert_eq!(store.current(), AppSettings::default());
        assert!(store.startup_warning().is_some());
    }

    #[test]
    fn save_and_reload_round_trips_settings() {
        let path = temp_path("roundtrip");
        let store = SettingsStore::load(path.clone());
        let updated = AppSettings {
            provider_base_url: "https://example.com/v1".to_string(),
            provider_model: "demo-model".to_string(),
            default_language: LanguageId::Python,
            output_speed: OutputSpeed::Custom,
            custom_characters_per_second: 42,
            global_shortcut: "Alt+Shift+S".to_string(),
            global_shortcut_enabled: false,
            save_history: true,
            launch_to_tray: true,
            theme: ThemePreference::Dark,
            ui_locale: UiLocale::EnUs,
        };
        let expected = updated.clone().sanitized();

        assert_eq!(store.update(updated).expect("save"), expected.clone());

        let reloaded = SettingsStore::load(path);
        assert_eq!(reloaded.current(), expected);
        assert_eq!(reloaded.startup_warning(), None);
    }

    #[test]
    fn disabled_global_shortcut_setting_round_trips() {
        let path = temp_path("shortcut-disabled");
        let store = SettingsStore::load(path.clone());
        let mut updated = AppSettings::default();
        updated.global_shortcut = "Alt+Shift+S".to_string();
        updated.global_shortcut_enabled = false;

        store
            .update(updated.clone())
            .expect("save disabled shortcut");

        let reloaded = SettingsStore::load(path);
        assert_eq!(reloaded.current().global_shortcut, "Alt+Shift+S");
        assert!(!reloaded.current().global_shortcut_enabled);
    }

    #[test]
    fn load_legacy_settings_defaults_interface_language() {
        let path = temp_path("legacy-locale");
        fs::create_dir_all(path.parent().expect("parent path")).expect("create temp dir");
        fs::write(
            &path,
            r#"{
              "providerBaseUrl": "https://example.com/v1",
              "providerModel": "legacy-model",
              "defaultLanguage": "cpp20",
              "outputSpeed": "normal",
              "customCharactersPerSecond": 24,
              "globalShortcut": "Ctrl+Shift+Q",
              "saveHistory": false,
              "launchToTray": false,
              "theme": "system"
            }"#,
        )
        .expect("write legacy file");

        let store = SettingsStore::load(path);

        assert_eq!(store.current().provider_model, "legacy-model");
        assert!(store.current().global_shortcut_enabled);
        assert_eq!(store.current().ui_locale, UiLocale::ZhCn);
        assert_eq!(store.startup_warning(), None);
    }
}
