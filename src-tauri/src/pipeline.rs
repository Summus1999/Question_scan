/**
 * Question Scan 截图→AI 流水线模块（阶段 5 核心 orchestrator）。
 *
 * 职责：协调从截图捕获到 AI 响应的完整端到端流程。
 * 流程：截图 → 压缩 → 构建提示词 → 验证配置 → 构造请求 → 保存会话 → 发送请求 → 清理。
 * 错误通过 `question-scan:ai-stream-event` 事件发射给前端，无需轮询。
 */
use crate::commands::map_settings_language_to_language_module;
use crate::provider::{
    build_openai_multimodal_request, OpenAiImageInput, validate_provider_request_config,
};
use crate::runtime::RuntimeStore;
use crate::screenshot::{
    capture_screens, cleanup_captured_screens, compress_image_for_ai, ScreenSelection,
};
use crate::session::SessionContext;
use crate::settings::{SettingsStore, TrayStatus};
use crate::streaming::{send_openai_request, AiStreamEvent};
use crate::{prompts, tray};
use tauri::{AppHandle, Emitter, Manager};

/** 在后台运行完整的截图→AI 流水线。
 *
 * 流程：
 * 1. 捕获当前显示器截图。
 * 2. 将截图压缩为 JPEG 载荷。
 * 3. 根据当前设置构建解法提示词。
 * 4. 验证提供商配置。
 * 5. 构建并发送 OpenAI-compatible 多模态请求。
 * 6. 保存会话数据，支持后续语言切换复用。
 * 7. 在每个阶段更新托盘状态。
 *
 * 错误通过 `question-scan:ai-stream-event` 事件发射给前端，React 壳无需轮询。
 */
/// Runs the full screenshot-to-AI pipeline in the background.
pub async fn run_screenshot_to_ai_pipeline(app: AppHandle) {
    tracing::info!("Starting screenshot-to-AI pipeline");

    // ------------------------------------------------------------------
    // 1. Capture screenshot
    // ------------------------------------------------------------------
    let captured_screens = match capture_screens(ScreenSelection::CurrentDisplay, None) {
        Ok(screens) => screens,
        Err(error) => {
            tracing::error!(%error, "Screenshot capture failed");
            emit_pipeline_error(
                &app,
                "screenshotFailed",
                &format!("Screenshot capture failed: {error}"),
            );
            set_tray_status_safe(&app, TrayStatus::Failed);
            return;
        }
    };

    if captured_screens.is_empty() {
        tracing::error!("No screens were captured");
        emit_pipeline_error(&app, "screenshotFailed", "No screens were captured");
        set_tray_status_safe(&app, TrayStatus::Failed);
        return;
    }

    let captured_screen = &captured_screens[0];

    // ------------------------------------------------------------------
    // 2. Load settings
    // ------------------------------------------------------------------
    let settings = match app.try_state::<SettingsStore>() {
        Some(store) => store.current(),
        None => {
            tracing::error!("Settings store not available");
            emit_pipeline_error(
                &app,
                "settingsUnavailable",
                "Settings store not available",
            );
            set_tray_status_safe(&app, TrayStatus::Failed);
            return;
        }
    };

    // ------------------------------------------------------------------
    // 3. Compress image for AI upload
    // ------------------------------------------------------------------
    let compression_config = settings.screenshot_compression_config();
    let compressed = match compress_image_for_ai(&captured_screen.image, compression_config) {
        Ok(img) => img,
        Err(error) => {
            tracing::error!(%error, "Image compression failed");
            emit_pipeline_error(
                &app,
                "compressionFailed",
                &format!("Image compression failed: {error}"),
            );
            set_tray_status_safe(&app, TrayStatus::Failed);
            let _ = cleanup_captured_screens(&captured_screens);
            return;
        }
    };

    // ------------------------------------------------------------------
    // 4. Move tray into Generating phase
    // ------------------------------------------------------------------
    set_tray_status_safe(&app, TrayStatus::Generating);

    // ------------------------------------------------------------------
    // 5. Build the solution prompt
    // ------------------------------------------------------------------
    let language = map_settings_language_to_language_module(settings.default_language);
    let platform = settings.platform_format;
    let instruction = prompts::build_solution_prompt(language, platform, None, None);

    // ------------------------------------------------------------------
    // 6. Validate provider configuration
    // ------------------------------------------------------------------
    let config = match validate_provider_request_config(&settings) {
        Ok(config) => config,
        Err(error) => {
            tracing::error!(%error, "Provider configuration invalid");
            emit_pipeline_error(&app, "providerConfigInvalid", &error.to_string());
            set_tray_status_safe(&app, TrayStatus::Failed);
            let _ = cleanup_captured_screens(&captured_screens);
            return;
        }
    };

    // ------------------------------------------------------------------
    // 7. Build the multimodal request
    // ------------------------------------------------------------------
    let image_bytes = compressed.bytes.clone();
    let request = match build_openai_multimodal_request(
        &config,
        instruction,
        OpenAiImageInput::new("image/jpeg", image_bytes),
    ) {
        Ok(req) => req,
        Err(error) => {
            tracing::error!(%error, "Failed to build AI request");
            emit_pipeline_error(&app, "requestBuildFailed", &error.to_string());
            set_tray_status_safe(&app, TrayStatus::Failed);
            let _ = cleanup_captured_screens(&captured_screens);
            return;
        }
    };

    // ------------------------------------------------------------------
    // 8. Save session data so language switching can reuse the image
    // ------------------------------------------------------------------
    if let Some(session) = app.try_state::<SessionContext>() {
        session.save_request_data(
            compressed.bytes,
            "image/jpeg",
            None::<&str>,
            None::<&str>,
            platform,
            language,
        );
    }

    // ------------------------------------------------------------------
    // 9. Send the AI request (streaming or non-streaming)
    // ------------------------------------------------------------------
    if let Err(error) = send_openai_request(app.clone(), request).await {
        tracing::error!(%error, "AI request failed");
        // The streaming layer already emitted an Error event to the frontend.
        set_tray_status_safe(&app, TrayStatus::Failed);
        let _ = cleanup_captured_screens(&captured_screens);
        return;
    }

    // ------------------------------------------------------------------
    // 10. Success
    // ------------------------------------------------------------------
    tracing::info!("Screenshot-to-AI pipeline completed successfully");
    set_tray_status_safe(&app, TrayStatus::Complete);
    let _ = cleanup_captured_screens(&captured_screens);
}

/** 安全地设置托盘状态：如果 RuntimeStore 不可用则静默跳过。 */
fn set_tray_status_safe(app: &AppHandle, status: TrayStatus) {
    if let Some(runtime) = app.try_state::<RuntimeStore>() {
        runtime.set_tray_status(status);
    }
    let _ = tray::sync_tray(app);
}

/** 发射流水线错误事件给前端，并记录警告日志。 */
fn emit_pipeline_error(app: &AppHandle, code: &str, message: &str) {
    let event = AiStreamEvent::Error {
        code: code.to_string(),
        message: message.to_string(),
    };
    if let Err(e) = app.emit("question-scan:ai-stream-event", &event) {
        tracing::warn!(%e, "Failed to emit pipeline error event");
    }
}
