use crate::errors::{AppError, AppResult};
use crate::history::{HistoryEntry, HistoryStore};
use crate::provider::{build_openai_multimodal_request, OpenAiImageInput};
use crate::runtime::RuntimeStore;
use crate::session::SessionContext;
use crate::language::LanguageId;
use crate::settings::{AppSettings, AppSnapshot, SettingsStore};
use crate::{prompts, shortcuts, streaming, tray};
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
    session: State<'_, SessionContext>,
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

    // Save session data for potential language switching / regeneration.
    // Extract recognized title and text from the instruction for later reuse.
    let (recognized_title, recognized_text) = extract_recognition_from_instruction(&instruction);
    let language = map_settings_language_to_language_module(settings.default_language);
    session.save_request_data(
        image_bytes.clone(),
        image_mime_type.clone(),
        recognized_title.as_deref(),
        recognized_text.as_deref(),
        settings.platform_format,
        language,
    );

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

/// Regenerates the AI solution using the same cropped image and recognized text
/// but with a different programming language.
/// Streaming or non-streaming results are delivered via `question-scan:ai-stream-event`.
#[tauri::command]
pub async fn regenerate_with_language(
    app: AppHandle,
    store: State<'_, SettingsStore>,
    session: State<'_, SessionContext>,
    language: LanguageId,
) -> AppResult<()> {
    if !session.can_regenerate() {
        return Err(AppError::AiRequestFailed {
            code: "noSessionData".to_string(),
            message: "No previous screenshot data available. Please capture a screenshot first."
                .to_string(),
        });
    }

    let settings = store.current();
    let config = crate::provider::validate_provider_request_config(&settings).map_err(|error| {
        AppError::ProviderConfigurationInvalid {
            reason: error.to_string(),
        }
    })?;

    let image_bytes = session
        .image_bytes()
        .expect("checked by can_regenerate");
    let image_mime_type = session
        .image_mime_type()
        .expect("checked by can_regenerate");
    let platform = session.platform_format().unwrap_or(settings.platform_format);
    let title = session.recognized_title();
    let text = session.recognized_text();

    let instruction = prompts::build_solution_prompt(
        language,
        platform,
        title.as_deref(),
        text.as_deref(),
    );

    // Update the session with the new language for future regenerations.
    session.save_request_data(
        image_bytes.clone(),
        image_mime_type.clone(),
        title.as_deref(),
        text.as_deref(),
        platform,
        language,
    );

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
            tracing::error!(%error, "AI request failed during language regeneration");
        }
    });

    Ok(())
}

/// Extracts recognized title and text from a solution prompt by looking for
/// the standard prompt sections. This is a best-effort parse for session storage.
fn extract_recognition_from_instruction(instruction: &str) -> (Option<String>, Option<String>) {
    let title = instruction
        .lines()
        .find(|line| line.starts_with("The user identified the problem title as: "))
        .map(|line| line.trim_start_matches("The user identified the problem title as: ").to_string());

    let text = instruction
        .lines()
        .find(|line| line.starts_with("The user also extracted the following text from the image: "))
        .map(|line| line.trim_start_matches("The user also extracted the following text from the image: ").to_string());

    (title, text)
}

/// Maps the settings module's LanguageId to the language module's LanguageId.
/// Both enums have identical variants, so this is a straightforward conversion.
pub(crate) fn map_settings_language_to_language_module(
    id: crate::settings::LanguageId,
) -> crate::language::LanguageId {
    match id {
        crate::settings::LanguageId::Cpp17 => crate::language::LanguageId::Cpp17,
        crate::settings::LanguageId::Cpp20 => crate::language::LanguageId::Cpp20,
        crate::settings::LanguageId::Python => crate::language::LanguageId::Python,
        crate::settings::LanguageId::Java => crate::language::LanguageId::Java,
        crate::settings::LanguageId::JavaScript => crate::language::LanguageId::JavaScript,
        crate::settings::LanguageId::TypeScript => crate::language::LanguageId::TypeScript,
        crate::settings::LanguageId::Go => crate::language::LanguageId::Go,
        crate::settings::LanguageId::Rust => crate::language::LanguageId::Rust,
    }
}

/// Deletes all orphaned temporary images from the system temp directory.
#[tauri::command]
pub fn clear_cache() -> AppResult<()> {
    let temp_dir = std::env::temp_dir().join("question-scan");
    if temp_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Err(error) = std::fs::remove_file(&path) {
                        tracing::warn!(path = %path.display(), %error, "Could not remove temp file during clear_cache");
                    }
                }
            }
        }
    }
    Ok(())
}

/// Returns the list of saved history entries.
#[tauri::command]
pub fn list_history(store: State<'_, HistoryStore>) -> AppResult<Vec<HistoryEntry>> {
    Ok(store.list())
}

/// Deletes a single history entry by its id.
#[tauri::command]
pub fn delete_history_entry(store: State<'_, HistoryStore>, id: String) -> AppResult<bool> {
    store.delete(&id)
}

/// Clears all history entries.
#[tauri::command]
pub fn clear_history(store: State<'_, HistoryStore>) -> AppResult<()> {
    store.clear()
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn clear_cache_deletes_temp_files() {
        let _guard = TEST_LOCK.lock().expect("test lock poisoned");
        let temp_dir = std::env::temp_dir().join("question-scan");
        std::fs::create_dir_all(&temp_dir).expect("create temp dir");

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock drift")
            .as_nanos();
        let test_file = temp_dir.join(format!("cache-test-{nonce}.png"));
        std::fs::write(&test_file, b"fake image").expect("write test file");
        assert!(test_file.exists());

        clear_cache().expect("clear_cache should succeed");

        assert!(!test_file.exists());
    }

    #[test]
    fn clear_cache_succeeds_when_directory_missing() {
        let _guard = TEST_LOCK.lock().expect("test lock poisoned");
        let temp_dir = std::env::temp_dir().join("question-scan");
        let _ = std::fs::remove_dir_all(&temp_dir);

        assert!(clear_cache().is_ok());
    }

    #[test]
    fn extract_recognition_from_instruction_parses_title_and_text() {
        let instruction = r#"You are an algorithm problem solver.

The user identified the problem title as: Two Sum
The user also extracted the following text from the image: Find two numbers that add up to the target.

Your response MUST follow this exact structure."#;

        let (title, text) = extract_recognition_from_instruction(instruction);

        assert_eq!(title, Some("Two Sum".to_string()));
        assert_eq!(
            text,
            Some("Find two numbers that add up to the target.".to_string())
        );
    }

    #[test]
    fn extract_recognition_from_instruction_returns_none_when_missing() {
        let instruction = "You are an algorithm problem solver. No recognition data here.";

        let (title, text) = extract_recognition_from_instruction(instruction);

        assert_eq!(title, None);
        assert_eq!(text, None);
    }

    #[test]
    fn extract_recognition_from_instruction_parses_partial_data() {
        let instruction = r#"You are an algorithm problem solver.

The user identified the problem title as: Longest Substring

Your response MUST follow this exact structure."#;

        let (title, text) = extract_recognition_from_instruction(instruction);

        assert_eq!(title, Some("Longest Substring".to_string()));
        assert_eq!(text, None);
    }

    #[test]
    fn map_settings_language_to_language_module_converts_all_variants() {
        use crate::settings::LanguageId as SettingsLanguageId;

        assert_eq!(
            map_settings_language_to_language_module(SettingsLanguageId::Cpp17),
            crate::language::LanguageId::Cpp17
        );
        assert_eq!(
            map_settings_language_to_language_module(SettingsLanguageId::Cpp20),
            crate::language::LanguageId::Cpp20
        );
        assert_eq!(
            map_settings_language_to_language_module(SettingsLanguageId::Python),
            crate::language::LanguageId::Python
        );
        assert_eq!(
            map_settings_language_to_language_module(SettingsLanguageId::Java),
            crate::language::LanguageId::Java
        );
        assert_eq!(
            map_settings_language_to_language_module(SettingsLanguageId::JavaScript),
            crate::language::LanguageId::JavaScript
        );
        assert_eq!(
            map_settings_language_to_language_module(SettingsLanguageId::TypeScript),
            crate::language::LanguageId::TypeScript
        );
        assert_eq!(
            map_settings_language_to_language_module(SettingsLanguageId::Go),
            crate::language::LanguageId::Go
        );
        assert_eq!(
            map_settings_language_to_language_module(SettingsLanguageId::Rust),
            crate::language::LanguageId::Rust
        );
    }
}
