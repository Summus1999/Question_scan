use crate::errors::{AppError, AppResult};
use crate::provider_key_store;
use crate::runtime::RuntimeSnapshot;
use crate::screenshot::ScreenshotCompressionConfig;
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
    #[serde(default = "default_provider_name")]
    pub provider_name: String,
    pub provider_base_url: String,
    #[serde(default)]
    pub provider_api_key: String,
    pub provider_model: String,
    #[serde(default = "default_request_timeout_seconds")]
    pub request_timeout_seconds: u32,
    #[serde(default = "default_streaming_enabled")]
    pub streaming_enabled: bool,
    pub default_language: LanguageId,
    pub output_speed: OutputSpeed,
    pub custom_characters_per_second: u32,
    pub global_shortcut: String,
    #[serde(default = "default_global_shortcut_enabled")]
    pub global_shortcut_enabled: bool,
    #[serde(default = "default_screenshot_max_long_edge")]
    pub screenshot_max_long_edge: u32,
    #[serde(default = "default_screenshot_jpeg_quality")]
    pub screenshot_jpeg_quality: u8,
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
            provider_name: default_provider_name(),
            provider_base_url: "https://api.openai.com/v1".to_string(),
            provider_api_key: String::new(),
            provider_model: "gpt-4o-mini".to_string(),
            request_timeout_seconds: default_request_timeout_seconds(),
            streaming_enabled: default_streaming_enabled(),
            default_language: LanguageId::Cpp20,
            output_speed: OutputSpeed::Normal,
            custom_characters_per_second: 24,
            global_shortcut: "Ctrl+Shift+Q".to_string(),
            global_shortcut_enabled: true,
            screenshot_max_long_edge: default_screenshot_max_long_edge(),
            screenshot_jpeg_quality: default_screenshot_jpeg_quality(),
            save_history: false,
            launch_to_tray: false,
            theme: ThemePreference::System,
            ui_locale: UiLocale::ZhCn,
        }
    }
}

/// Keeps older settings usable when provider naming is added after the first shell.
fn default_provider_name() -> String {
    "OpenAI-compatible".to_string()
}

/// Keeps image requests from hanging indefinitely while leaving room for vision latency.
fn default_request_timeout_seconds() -> u32 {
    60
}

/// Uses streaming by default so the MVP can show incremental answer output when available.
fn default_streaming_enabled() -> bool {
    true
}

/// Preserves old settings files that predate the explicit interface language.
fn default_ui_locale() -> UiLocale {
    UiLocale::ZhCn
}

/// Keeps legacy settings files active when they predate the shortcut toggle.
fn default_global_shortcut_enabled() -> bool {
    true
}

/// Keeps screenshot compression conservative enough for AI payloads without hurting readability.
fn default_screenshot_max_long_edge() -> u32 {
    1920
}

/// Keeps AI image uploads compact while preserving enough detail for problem statements.
fn default_screenshot_jpeg_quality() -> u8 {
    85
}

impl AppSettings {
    /// Trims user-editable string fields and repairs invalid custom speed, timeout, and screenshot values.
    pub fn sanitized(mut self) -> Self {
        self.provider_name = self.provider_name.trim().to_string();
        self.provider_base_url = self.provider_base_url.trim().to_string();
        self.provider_api_key = self.provider_api_key.trim().to_string();
        self.provider_model = self.provider_model.trim().to_string();
        self.global_shortcut = self.global_shortcut.trim().to_string();
        if self.request_timeout_seconds == 0 {
            self.request_timeout_seconds = default_request_timeout_seconds();
        }
        if self.custom_characters_per_second == 0 {
            self.custom_characters_per_second = 24;
        }
        if self.screenshot_max_long_edge == 0 {
            self.screenshot_max_long_edge = default_screenshot_max_long_edge();
        }
        if self.screenshot_jpeg_quality == 0 {
            self.screenshot_jpeg_quality = default_screenshot_jpeg_quality();
        } else {
            self.screenshot_jpeg_quality = self.screenshot_jpeg_quality.clamp(1, 100);
        }
        self
    }

    /// Removes secrets before the settings are serialized to disk or exposed to the frontend.
    pub fn redacted(mut self) -> Self {
        self.provider_api_key.clear();
        self
    }

    /// Exposes screenshot compression settings for the capture-to-AI pipeline.
    pub fn screenshot_compression_config(&self) -> ScreenshotCompressionConfig {
        ScreenshotCompressionConfig {
            max_long_edge: if self.screenshot_max_long_edge == 0 {
                default_screenshot_max_long_edge()
            } else {
                self.screenshot_max_long_edge
            },
            jpeg_quality: if self.screenshot_jpeg_quality == 0 {
                default_screenshot_jpeg_quality()
            } else {
                self.screenshot_jpeg_quality.clamp(1, 100)
            },
        }
    }
}

fn append_startup_warning(current: Option<String>, next: impl Into<String>) -> Option<String> {
    let next = next.into();
    match current {
        Some(current) => Some(format!("{current} {next}")),
        None => Some(next),
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
        let (mut settings, mut startup_warning) = match fs::read_to_string(&path) {
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

        let legacy_api_key = settings.provider_api_key.trim().to_string();
        let mut should_redact_persisted_settings = false;

        if legacy_api_key.is_empty() {
            match provider_key_store::load_provider_api_key() {
                Ok(Some(provider_api_key)) => {
                    settings.provider_api_key = provider_api_key;
                }
                Ok(None) => {}
                Err(error) => {
                    tracing::warn!(path = %path.display(), %error, "Could not read the secure API key");
                    startup_warning = append_startup_warning(
                        startup_warning,
                        "The API key could not be loaded from secure storage. Please enter it again.",
                    );
                }
            }
        } else {
            match provider_key_store::save_provider_api_key(&legacy_api_key) {
                Ok(()) => {
                    should_redact_persisted_settings = true;
                }
                Err(error) => {
                    tracing::warn!(path = %path.display(), %error, "Could not migrate the API key to secure storage");
                    startup_warning = append_startup_warning(
                        startup_warning,
                        "The API key could not be moved to secure storage. It remains in the settings file until the next successful save.",
                    );
                }
            }
        }

        let store = Self {
            path,
            settings: RwLock::new(settings.sanitized()),
            startup_warning: Mutex::new(startup_warning),
        };

        if should_redact_persisted_settings {
            if let Err(error) = store.write_to_disk(&store.current()) {
                tracing::warn!(path = %store.path.display(), %error, "Could not rewrite the settings file without the API key");
                let mut warning = store.startup_warning.lock().expect("warning lock poisoned");
                *warning = append_startup_warning(
                    warning.clone(),
                    "The API key was saved securely, but the settings file could not be rewritten without it.",
                );
            }
        }

        store
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
        let current_provider_api_key = self.current().provider_api_key;
        let mut settings = settings.sanitized();

        if settings.provider_api_key.is_empty() {
            settings.provider_api_key = current_provider_api_key;
        } else {
            provider_key_store::save_provider_api_key(&settings.provider_api_key)?;
        }

        *self.settings.write().expect("settings lock poisoned") = settings.clone();
        self.write_to_disk(&settings)?;
        *self.startup_warning.lock().expect("warning lock poisoned") = None;
        Ok(settings)
    }

    /// Restores persisted settings to the same defaults used on a first launch.
    pub fn reset(&self) -> AppResult<AppSettings> {
        provider_key_store::clear_provider_api_key()?;
        let settings = AppSettings::default();
        *self.settings.write().expect("settings lock poisoned") = settings.clone();
        self.write_to_disk(&settings)?;
        *self.startup_warning.lock().expect("warning lock poisoned") = None;
        Ok(settings)
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
            settings: self.current().redacted(),
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
        let settings = settings.clone().redacted();
        let parent = self.path.parent().ok_or(AppError::SettingsSaveFailed)?;
        fs::create_dir_all(parent).map_err(|error| {
            tracing::error!(path = %parent.display(), %error, "Could not create the settings directory");
            AppError::SettingsSaveFailed
        })?;

        let file = fs::File::create(&self.path).map_err(|error| {
            tracing::error!(path = %self.path.display(), %error, "Could not open the settings file for writing");
            AppError::SettingsSaveFailed
        })?;

        serde_json::to_writer_pretty(file, &settings).map_err(|error| {
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
        assert_eq!(settings.provider_name, "OpenAI-compatible");
        assert_eq!(settings.provider_base_url, "https://api.openai.com/v1");
        assert_eq!(settings.provider_api_key, "");
        assert_eq!(settings.provider_model, "gpt-4o-mini");
        assert_eq!(settings.request_timeout_seconds, 60);
        assert!(settings.streaming_enabled);
        assert_eq!(settings.default_language, LanguageId::Cpp20);
        assert_eq!(settings.output_speed, OutputSpeed::Normal);
        assert_eq!(settings.custom_characters_per_second, 24);
        assert_eq!(settings.global_shortcut, "Ctrl+Shift+Q");
        assert!(settings.global_shortcut_enabled);
        assert_eq!(settings.screenshot_max_long_edge, 1920);
        assert_eq!(settings.screenshot_jpeg_quality, 85);
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
            provider_name: "Example AI".to_string(),
            provider_base_url: "https://example.com/v1".to_string(),
            provider_api_key: "sk-demo".to_string(),
            provider_model: "demo-model".to_string(),
            request_timeout_seconds: 90,
            streaming_enabled: false,
            default_language: LanguageId::Python,
            output_speed: OutputSpeed::Custom,
            custom_characters_per_second: 42,
            global_shortcut: "Alt+Shift+S".to_string(),
            global_shortcut_enabled: false,
            screenshot_max_long_edge: 1600,
            screenshot_jpeg_quality: 78,
            save_history: true,
            launch_to_tray: true,
            theme: ThemePreference::Dark,
            ui_locale: UiLocale::EnUs,
        };
        let expected = updated.clone().sanitized();

        assert_eq!(store.update(updated).expect("save"), expected.clone());
        assert_eq!(store.current().provider_api_key, "sk-demo");

        let file_contents = fs::read_to_string(&path).expect("read redacted settings file");
        let persisted: serde_json::Value =
            serde_json::from_str(&file_contents).expect("parse redacted settings file");
        assert_eq!(persisted["providerApiKey"], "");

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
        updated.screenshot_max_long_edge = 1280;
        updated.screenshot_jpeg_quality = 72;

        store
            .update(updated.clone())
            .expect("save disabled shortcut");

        let reloaded = SettingsStore::load(path);
        assert_eq!(reloaded.current().global_shortcut, "Alt+Shift+S");
        assert!(!reloaded.current().global_shortcut_enabled);
        assert_eq!(reloaded.current().screenshot_max_long_edge, 1280);
        assert_eq!(reloaded.current().screenshot_jpeg_quality, 72);
    }

    #[test]
    fn load_legacy_settings_moves_api_key_to_secure_storage() {
        let path = temp_path("legacy-secret");
        fs::create_dir_all(path.parent().expect("parent path")).expect("create temp dir");
        fs::write(
            &path,
            r#"{
              "providerName": "Legacy AI",
              "providerBaseUrl": "https://example.com/v1",
              "providerApiKey": "legacy-key",
              "providerModel": "legacy-model",
              "requestTimeoutSeconds": 75,
              "streamingEnabled": true,
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

        let store = SettingsStore::load(path.clone());

        assert_eq!(store.current().provider_api_key, "legacy-key");

        let file_contents = fs::read_to_string(&path).expect("read migrated settings file");
        let persisted: serde_json::Value =
            serde_json::from_str(&file_contents).expect("parse migrated settings file");
        assert_eq!(persisted["providerApiKey"], "");
        assert_eq!(
            SettingsStore::load(path).current().provider_api_key,
            "legacy-key"
        );
        assert_eq!(store.startup_warning(), None);
    }

    #[test]
    fn reset_clears_the_secure_api_key_store() {
        let path = temp_path("reset-secret");
        let store = SettingsStore::load(path.clone());
        let mut updated = AppSettings::default();
        updated.provider_api_key = "reset-key".to_string();
        store.update(updated).expect("save api key");

        assert_eq!(store.current().provider_api_key, "reset-key");

        let reset = store.reset().expect("reset settings");
        assert_eq!(reset.provider_api_key, "");
        assert_eq!(store.current().provider_api_key, "");

        let file_contents = fs::read_to_string(&path).expect("read reset settings file");
        let persisted: serde_json::Value =
            serde_json::from_str(&file_contents).expect("parse reset settings file");
        assert_eq!(persisted["providerApiKey"], "");
        assert_eq!(SettingsStore::load(path).current().provider_api_key, "");
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

        assert_eq!(store.current().provider_name, "OpenAI-compatible");
        assert_eq!(store.current().provider_model, "legacy-model");
        assert_eq!(store.current().provider_api_key, "");
        assert_eq!(store.current().request_timeout_seconds, 60);
        assert!(store.current().streaming_enabled);
        assert!(store.current().global_shortcut_enabled);
        assert_eq!(store.current().screenshot_max_long_edge, 1920);
        assert_eq!(store.current().screenshot_jpeg_quality, 85);
        assert_eq!(store.current().ui_locale, UiLocale::ZhCn);
        assert_eq!(store.startup_warning(), None);
    }

    #[test]
    fn sanitizes_invalid_screenshot_compression_settings() {
        let mut settings = AppSettings::default();
        settings.provider_name = " Example Provider ".to_string();
        settings.provider_base_url = " https://example.com/v1 ".to_string();
        settings.provider_api_key = " sk-demo ".to_string();
        settings.provider_model = " vision-model ".to_string();
        settings.request_timeout_seconds = 0;
        settings.screenshot_max_long_edge = 0;
        settings.screenshot_jpeg_quality = 0;

        let sanitized = settings.sanitized();

        assert_eq!(sanitized.provider_name, "Example Provider");
        assert_eq!(sanitized.provider_base_url, "https://example.com/v1");
        assert_eq!(sanitized.provider_api_key, "sk-demo");
        assert_eq!(sanitized.provider_model, "vision-model");
        assert_eq!(sanitized.request_timeout_seconds, 60);
        assert_eq!(sanitized.screenshot_max_long_edge, 1920);
        assert_eq!(sanitized.screenshot_jpeg_quality, 85);
    }

    #[test]
    fn screenshot_compression_config_uses_defaults_for_zero_values() {
        let mut settings = AppSettings::default();
        settings.screenshot_max_long_edge = 0;
        settings.screenshot_jpeg_quality = 0;

        let config = settings.screenshot_compression_config();

        assert_eq!(config.max_long_edge, 1920);
        assert_eq!(config.jpeg_quality, 85);
    }
}
