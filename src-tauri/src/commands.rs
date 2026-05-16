/**
 * Question Scan Tauri 命令处理器模块
 *
 * 职责：暴露给前端的所有 Tauri 命令（#[tauri::command]）。
 * 这是前后端交互的唯一入口，所有前端通过 api.ts 调用的命令都在这里定义。
 */
use crate::errors::{AppError, AppResult};
use crate::history::{HistoryEntry, HistoryStore};
use crate::provider::{build_openai_multimodal_request, OpenAiImageInput};
use crate::runtime::RuntimeStore;
use crate::session::SessionContext;
use crate::language::LanguageId;
use crate::settings::{AppSettings, AppSnapshot, SettingsStore};
use crate::{prompts, shortcuts, streaming, tray};
use tauri::{AppHandle, Manager, State};

/**
 * 加载应用完整状态快照。
 * 前端启动时调用此命令获取初始状态，包含设置、运行时状态和窗口信息。
 */
#[tauri::command]
pub fn load_app_state(
    app: AppHandle,
    store: State<'_, SettingsStore>,
    runtime: State<'_, RuntimeStore>,
) -> AppResult<AppSnapshot> {
    snapshot(&app, &store, &runtime)
}

/**
 * 保存用户设置。
 * 校验设置值 → 同步快捷键 → 持久化到文件 → 返回新快照供前端同步。
 */
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

/**
 * 恢复默认设置。
 * 重置后端持久化数据和快捷键状态，返回与保存命令相同的快照结构。
 */
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

/**
 * 发起 AI 多模态请求（异步命令）。
 * 立即返回，不阻塞前端。请求结果通过 `question-scan:ai-stream-event` 事件流推送。
 * 同时保存请求数据到 SessionContext，供语言切换重新生成时使用。
 */
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

/**
 * 切换语言重新生成答案（异步命令）。
 * 复用 SessionContext 中保存的截图和识别数据，构造新语言的 prompt 重新请求。
 * 结果同样通过 `question-scan:ai-stream-event` 事件流推送。
 */
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

/**
 * 从解题 prompt 中提取识别出的标题和文本。
 * 用于将识别结果保存到 SessionContext，供语言切换时复用。
 * 这是一个尽力而为的解析，依赖 prompt 中的固定前缀。
 */
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

/**
 * 将 settings 模块的 LanguageId 转换为 language 模块的 LanguageId。
 * 两个枚举定义了相同的变体，此函数做简单的映射转换。
 * TODO: 后续可考虑统一为一个模块中的定义，消除重复。
 */
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

/** 清空系统临时目录中的遗留截图文件。 */
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

/** 获取已保存的历史记录列表。 */
#[tauri::command]
pub fn list_history(store: State<'_, HistoryStore>) -> AppResult<Vec<HistoryEntry>> {
    Ok(store.list())
}

/** 根据 ID 删除单条历史记录。 */
#[tauri::command]
pub fn delete_history_entry(store: State<'_, HistoryStore>, id: String) -> AppResult<bool> {
    store.delete(&id)
}

/** 清空全部历史记录。 */
#[tauri::command]
pub fn clear_history(store: State<'_, HistoryStore>) -> AppResult<()> {
    store.clear()
}

/**
 * 构建完整的应用状态快照。
 * 几乎所有命令处理器的最后一步都是调用此函数，将最新状态返回给前端。
 */
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
