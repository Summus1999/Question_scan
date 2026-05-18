/**
 * Question Scan 截图→AI 流水线模块（阶段 5 核心 orchestrator）。
 *
 * 职责：协调从截图捕获到 AI 响应的完整端到端流程。
 * 流程：截图 → 压缩 → 构建提示词 → 验证配置 → 构造请求 → 保存会话 → 发送请求 → 清理。
 * 错误通过 `question-scan:ai-stream-event` 事件发射给前端，无需轮询。
 */
use crate::commands::map_settings_language_to_language_module;
use crate::provider::{build_openai_multimodal_request, OpenAiImageInput, ProviderRequestConfig};
use crate::rag_history::RagHistoryStore;
use crate::rag_imports::RagImportStore;
use crate::rag_prompt_context::{RagPromptContextRequest, RagPromptContextResponse};
use crate::rag_retrieval::{RagEmbeddingStore, RagSearchRequest};
use crate::rag_templates::RagTemplateSelectionRequest;
use crate::recognition::{
    build_question_region_recognition_request, crop_original_question_region,
    parse_question_recognition_result, RecognitionImageConfig,
};
use crate::runtime::RuntimeStore;
use crate::screenshot::{
    capture_screens, cleanup_captured_screens, compress_image_for_ai, ScreenSelection,
};
use crate::session::SessionContext;
use crate::settings::{AppSettings, SettingsStore, TrayStatus};
use crate::streaming::{send_openai_request, AiStreamEvent};
use crate::{prompts, tray};
use serde_json::Value;
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
            emit_pipeline_error(&app, "settingsUnavailable", "Settings store not available");
            set_tray_status_safe(&app, TrayStatus::Failed);
            return;
        }
    };

    // ------------------------------------------------------------------
    // 3. Validate provider configuration
    // ------------------------------------------------------------------
    let config = match crate::provider::validate_provider_request_config(&settings) {
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
    // 4. Recognize the question region before solving
    // ------------------------------------------------------------------
    set_tray_status_safe(&app, TrayStatus::Recognizing);
    let recognized_region = match recognize_question_region(&config, &captured_screen.image).await {
        Ok(Some(region)) => Some(region),
        Ok(None) => None,
        Err(error) => {
            tracing::warn!(%error, "Question recognition failed; continuing without RAG context");
            None
        }
    };

    let solution_image = recognized_region
        .as_ref()
        .map(|region| &region.image)
        .unwrap_or(&captured_screen.image);
    let recognized_title = recognized_region
        .as_ref()
        .and_then(|region| region.recognition.title.clone());
    let recognized_text = recognized_region
        .as_ref()
        .and_then(|region| region.recognition.question_text.clone());

    // ------------------------------------------------------------------
    // 5. Compress image for AI upload
    // ------------------------------------------------------------------
    let compression_config = settings.screenshot_compression_config();
    let compressed = match compress_image_for_ai(solution_image, compression_config) {
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
    // 6. Move tray into Generating phase
    // ------------------------------------------------------------------
    set_tray_status_safe(&app, TrayStatus::Generating);

    // ------------------------------------------------------------------
    // 7. Build the solution prompt, with optional RAG context
    // ------------------------------------------------------------------
    let language = map_settings_language_to_language_module(settings.default_language);
    let platform = settings.platform_format;
    let rag_context = build_pipeline_rag_context(
        &app,
        &settings,
        recognized_title.clone(),
        recognized_text.clone(),
    );
    if let Some(rag_context) = &rag_context {
        emit_rag_context(&app, rag_context);
    }
    let instruction = rag_context
        .as_ref()
        .map(|context| context.solution_prompt.clone())
        .unwrap_or_else(|| {
            prompts::build_solution_prompt(
                language,
                platform,
                recognized_title.as_deref(),
                recognized_text.as_deref(),
            )
        });

    // ------------------------------------------------------------------
    // 8. Build the multimodal request
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
    // 9. Save session data so language switching can reuse the image
    // ------------------------------------------------------------------
    if let Some(session) = app.try_state::<SessionContext>() {
        session.save_request_data(
            compressed.bytes,
            "image/jpeg",
            recognized_title.as_deref(),
            recognized_text.as_deref(),
            platform,
            language,
            rag_context
                .as_ref()
                .map(|context| context.items.clone())
                .unwrap_or_default(),
            rag_context
                .as_ref()
                .map(|context| context.prompt_section.clone())
                .filter(|section| !section.trim().is_empty()),
        );
    }

    // ------------------------------------------------------------------
    // 10. Send the AI request (streaming or non-streaming)
    // ------------------------------------------------------------------
    if let Err(error) = send_openai_request(app.clone(), request).await {
        tracing::error!(%error, "AI request failed");
        // The streaming layer already emitted an Error event to the frontend.
        set_tray_status_safe(&app, TrayStatus::Failed);
        let _ = cleanup_captured_screens(&captured_screens);
        return;
    }

    // ------------------------------------------------------------------
    // 11. Success
    // ------------------------------------------------------------------
    tracing::info!("Screenshot-to-AI pipeline completed successfully");
    set_tray_status_safe(&app, TrayStatus::Complete);
    let _ = cleanup_captured_screens(&captured_screens);
}

async fn recognize_question_region(
    config: &ProviderRequestConfig,
    image: &crate::screenshot::image::RgbaImage,
) -> Result<Option<crate::recognition::CroppedQuestionRegion>, String> {
    let recognition_request =
        build_question_region_recognition_request(image, RecognitionImageConfig::default())?;
    let mut recognition_config = config.clone();
    recognition_config.streaming_enabled = false;
    let request = build_openai_multimodal_request(
        &recognition_config,
        recognition_request.prompt,
        OpenAiImageInput::new(
            recognition_request.image.mime_type,
            recognition_request.image.bytes.clone(),
        ),
    )
    .map_err(|error| error.to_string())?;
    let response_text = execute_non_streaming_recognition_request(request).await?;
    let recognition = parse_question_recognition_result(&response_text)?;
    let cropped = crop_original_question_region(image, recognition, &recognition_request.image)?;

    Ok(Some(cropped))
}

async fn execute_non_streaming_recognition_request(
    request: crate::provider::PreparedOpenAiMultimodalRequest,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let response = client
        .post(&request.endpoint)
        .header("Authorization", &request.authorization_header)
        .header("Content-Type", request.content_type)
        .timeout(request.request_timeout)
        .json(&request.body)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let status = response.status();
    let body_text = response.text().await.map_err(|error| error.to_string())?;
    if !status.is_success() {
        return Err(format!(
            "Recognition request failed with status {status}: {body_text}"
        ));
    }

    Ok(extract_openai_content(&body_text).unwrap_or(body_text))
}

fn extract_openai_content(body_text: &str) -> Option<String> {
    let body = serde_json::from_str::<Value>(body_text).ok()?;
    let choices = body.get("choices")?.as_array()?;
    let mut combined = String::new();
    for choice in choices {
        if let Some(content) = choice
            .get("message")
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
        {
            combined.push_str(content);
        }
    }

    (!combined.is_empty()).then_some(combined)
}

fn build_pipeline_rag_context(
    app: &AppHandle,
    settings: &AppSettings,
    recognized_title: Option<String>,
    recognized_text: Option<String>,
) -> Option<RagPromptContextResponse> {
    if !settings.local_rag_enabled {
        return None;
    }

    let has_recognition_text = recognized_title
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
        || recognized_text
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| !value.is_empty());
    if !has_recognition_text {
        tracing::info!("Skipping RAG because recognition did not return title or text");
        return None;
    }

    let embedding_store = app.try_state::<RagEmbeddingStore>()?;
    let import_store = app.try_state::<RagImportStore>()?;
    let history_store = app.try_state::<RagHistoryStore>()?;
    let search = match crate::commands::search_rag_context_with_settings(
        &embedding_store,
        &import_store,
        &history_store,
        settings,
        RagSearchRequest {
            recognized_title: recognized_title.clone(),
            recognized_text: recognized_text.clone(),
            examples: Vec::new(),
            constraints: Vec::new(),
            tags: Vec::new(),
            algorithm_tags: Vec::new(),
            target_language: Some(crate::commands::language_to_string(
                map_settings_language_to_language_module(settings.default_language),
            )),
            platform: Some(crate::commands::platform_to_string(
                settings.platform_format,
            )),
            top_k: Some(settings.rag_max_recall_items),
            min_score: None,
        },
    ) {
        Ok(search) => search,
        Err(error) => {
            tracing::warn!(%error, "RAG search failed; continuing without local context");
            return None;
        }
    };

    let template_context = crate::commands::select_rag_template_context_with_settings(
        &import_store,
        settings,
        RagTemplateSelectionRequest {
            target_language: Some(crate::commands::language_to_string(
                map_settings_language_to_language_module(settings.default_language),
            )),
            platform: Some(crate::commands::platform_to_string(
                settings.platform_format,
            )),
            tags: Vec::new(),
            algorithm_tags: Vec::new(),
            similar_items: search.items.clone(),
            max_solution_modes: None,
            max_templates: None,
        },
    )
    .ok();

    crate::commands::build_rag_prompt_context_with_settings(
        settings,
        RagPromptContextRequest {
            recognized_title,
            recognized_text,
            target_language: Some(crate::commands::language_to_string(
                map_settings_language_to_language_module(settings.default_language),
            )),
            platform: Some(crate::commands::platform_to_string(
                settings.platform_format,
            )),
            search_results: search.items,
            template_context,
            max_items: Some(settings.rag_max_recall_items),
            max_context_tokens: None,
            min_score: None,
        },
    )
    .ok()
}

fn emit_rag_context(app: &AppHandle, context: &RagPromptContextResponse) {
    let event = AiStreamEvent::RagContext {
        items: context.items.clone(),
        used_item_count: context.used_item_count,
        token_estimate: context.token_estimate,
        skipped_reason: context.skipped_reason.clone(),
    };
    if let Err(error) = app.emit("question-scan:ai-stream-event", &event) {
        tracing::warn!(%error, "Failed to emit RAG context event");
    }
}

#[cfg(test)]
mod tests {
    use super::extract_openai_content;

    #[test]
    fn extracts_content_from_openai_recognition_response() {
        let body = r#"{
            "choices": [
                {
                    "message": {
                        "content": "{\"boundingBox\":{\"x\":0,\"y\":0,\"width\":10,\"height\":10},\"confidence\":0.9,\"title\":\"Two Sum\",\"questionText\":\"Return two indices.\",\"reason\":\"Visible problem text.\"}"
                    }
                }
            ]
        }"#;

        let content = extract_openai_content(body).expect("content should parse");

        assert!(content.contains("\"title\":\"Two Sum\""));
        assert!(content.contains("\"questionText\":\"Return two indices.\""));
    }

    #[test]
    fn recognition_content_parser_ignores_unknown_response_shapes() {
        assert_eq!(extract_openai_content(r#"{"error":"bad request"}"#), None);
        assert_eq!(extract_openai_content("not json"), None);
    }
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
