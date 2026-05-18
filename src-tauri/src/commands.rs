/**
 * Question Scan Tauri 命令处理器模块
 *
 * 职责：暴露给前端的所有 Tauri 命令（#[tauri::command]）。
 * 这是前后端交互的唯一入口，所有前端通过 api.ts 调用的命令都在这里定义。
 */
use crate::errors::{AppError, AppResult};
use crate::history::{HistoryEntry, HistoryStore};
use crate::language::LanguageId;
use crate::problem_index::{self, ProblemIndexEntry};
use crate::provider::{build_openai_multimodal_request, OpenAiImageInput};
use crate::rag_history::RagHistoryStore;
use crate::rag_imports::{RagImportKind, RagImportRequest, RagImportStore, RagImportedDocument};
use crate::rag_prompt_context::{RagPromptContextRequest, RagPromptContextResponse};
use crate::rag_retrieval::{
    RagEmbeddingStore, RagSearchChunkKind, RagSearchRequest, RagSearchResponse, RagSearchResult,
    RagSearchSourceType,
};
use crate::rag_templates::{RagTemplateSelectionRequest, RagTemplateSelectionResponse};
use crate::runtime::RuntimeStore;
use crate::session::SessionContext;
use crate::settings::{AppSettings, AppSnapshot, SettingsStore};
use crate::{prompts, shortcuts, streaming, tray};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State};

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
    history_store: State<'_, HistoryStore>,
    runtime: State<'_, RuntimeStore>,
    settings: AppSettings,
) -> AppResult<AppSnapshot> {
    let settings = settings.sanitized();
    shortcuts::sync_global_shortcut(&app, &settings, &runtime)?;
    let saved = store.update(settings)?;
    history_store.set_enabled(saved.save_history);
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
    history_store: State<'_, HistoryStore>,
    runtime: State<'_, RuntimeStore>,
) -> AppResult<AppSnapshot> {
    let settings = AppSettings::default();
    shortcuts::sync_global_shortcut(&app, &settings, &runtime)?;
    store.reset()?;
    history_store.set_enabled(settings.save_history);
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
        Vec::new(),
        None,
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

    let image_bytes = session.image_bytes().expect("checked by can_regenerate");
    let image_mime_type = session
        .image_mime_type()
        .expect("checked by can_regenerate");
    let platform = session
        .platform_format()
        .unwrap_or(settings.platform_format);
    let title = session.recognized_title();
    let text = session.recognized_text();

    let rag_context_items = session.rag_context_items();
    let rag_prompt_section = session.rag_prompt_section();
    if !rag_context_items.is_empty() {
        let _ = app.emit(
            "question-scan:ai-stream-event",
            &streaming::AiStreamEvent::RagContext {
                items: rag_context_items.clone(),
                used_item_count: rag_context_items
                    .iter()
                    .filter(|item| item.used_in_prompt)
                    .count() as u32,
                token_estimate: rag_context_items
                    .iter()
                    .filter(|item| item.used_in_prompt)
                    .map(|item| item.token_estimate)
                    .sum(),
                skipped_reason: None,
            },
        );
    }
    let instruction = prompts::build_solution_prompt_with_rag(
        language,
        platform,
        title.as_deref(),
        text.as_deref(),
        rag_prompt_section.as_deref(),
    );

    // Update the session with the new language for future regenerations.
    session.save_request_data(
        image_bytes.clone(),
        image_mime_type.clone(),
        title.as_deref(),
        text.as_deref(),
        platform,
        language,
        rag_context_items,
        rag_prompt_section,
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
        .map(|line| {
            line.trim_start_matches("The user identified the problem title as: ")
                .to_string()
        });

    let text = instruction
        .lines()
        .find(|line| {
            line.starts_with("The user also extracted the following text from the image: ")
        })
        .map(|line| {
            line.trim_start_matches("The user also extracted the following text from the image: ")
                .to_string()
        });

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

pub(crate) fn language_to_string(language: LanguageId) -> String {
    match language {
        LanguageId::Cpp17 => "cpp17",
        LanguageId::Cpp20 => "cpp20",
        LanguageId::Python => "python",
        LanguageId::Java => "java",
        LanguageId::JavaScript => "javascript",
        LanguageId::TypeScript => "typescript",
        LanguageId::Go => "go",
        LanguageId::Rust => "rust",
    }
    .to_string()
}

pub(crate) fn platform_to_string(platform: crate::language::PlatformFormat) -> String {
    match platform {
        crate::language::PlatformFormat::Acm => "acm",
        crate::language::PlatformFormat::LeetCode => "leetcode",
        crate::language::PlatformFormat::Generic => "generic",
    }
    .to_string()
}

/** 前端提交的历史保存请求。 */
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveHistoryEntryRequest {
    pub recognized_title: Option<String>,
    pub recognized_text: Option<String>,
    pub language: String,
    pub platform: String,
    pub model: String,
    pub result: String,
    pub user_note: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub algorithm_tags: Vec<String>,
}

/** RAG 隐私维护命令的统计结果。 */
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RagIndexMaintenanceResult {
    pub cleared_embedding_count: u32,
    pub rebuilt_embedding_count: u32,
    pub cleared_history_document_count: u32,
    pub rebuilt_history_document_count: u32,
    pub cleared_history_chunk_count: u32,
    pub skipped_reason: Option<String>,
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

/** 保存一条历史，并同步建立本地 RAG 可检索记录。 */
#[tauri::command]
pub fn save_history_entry(
    history_store: State<'_, HistoryStore>,
    rag_history_store: State<'_, RagHistoryStore>,
    settings_store: State<'_, SettingsStore>,
    session: State<'_, SessionContext>,
    request: SaveHistoryEntryRequest,
) -> AppResult<Option<HistoryEntry>> {
    if !history_store.is_enabled() || request.result.trim().is_empty() {
        return Ok(None);
    }

    let settings = settings_store.current();
    let timestamp = current_timestamp();
    let recognized_title = normalized_optional(request.recognized_title.as_deref()).or_else(|| {
        session
            .recognized_title()
            .and_then(|value| normalized_optional(Some(&value)))
    });
    let recognized_text = normalized_optional(request.recognized_text.as_deref()).or_else(|| {
        session
            .recognized_text()
            .and_then(|value| normalized_optional(Some(&value)))
    });
    let language = parse_history_language(&request.language)
        .or_else(|| session.last_language())
        .unwrap_or_else(|| map_settings_language_to_language_module(settings.default_language));
    let platform = parse_history_platform(&request.platform)
        .or_else(|| session.platform_format())
        .unwrap_or(settings.platform_format);
    let model = normalized_optional(Some(&request.model)).unwrap_or(settings.provider_model);
    let result = request.result.trim().to_string();

    let entry = HistoryEntry {
        id: stable_history_id(&timestamp, recognized_title.as_deref(), &result),
        timestamp,
        recognized_title,
        recognized_text,
        language,
        platform,
        model,
        result,
        user_note: normalized_optional(request.user_note.as_deref()),
        tags: clean_list(&request.tags),
        algorithm_tags: clean_list(&request.algorithm_tags),
    };

    history_store.add(entry.clone())?;
    if settings.local_rag_enabled && settings.rag_history_indexing_enabled {
        rag_history_store
            .index_history_entry(&entry)
            .map_err(|error| AppError::RagHistoryIndexFailed {
                reason: error.to_string(),
            })?;
    }

    Ok(Some(entry))
}

/** 根据 ID 删除单条历史记录。 */
#[tauri::command]
pub fn delete_history_entry(
    store: State<'_, HistoryStore>,
    rag_history_store: State<'_, RagHistoryStore>,
    id: String,
) -> AppResult<bool> {
    let deleted = store.delete(&id)?;
    if deleted {
        rag_history_store
            .delete_by_history_entry(&id)
            .map_err(|error| AppError::RagHistoryIndexFailed {
                reason: error.to_string(),
            })?;
    }

    Ok(deleted)
}

/** 清空全部历史记录。 */
#[tauri::command]
pub fn clear_history(
    store: State<'_, HistoryStore>,
    rag_history_store: State<'_, RagHistoryStore>,
) -> AppResult<()> {
    store.clear()?;
    rag_history_store
        .delete_all_history_entries()
        .map_err(|error| AppError::RagHistoryIndexFailed {
            reason: error.to_string(),
        })
}

/** 获取内置 LeetCode 轻量题目索引。 */
#[tauri::command]
pub fn list_leetcode_problem_index() -> AppResult<Vec<ProblemIndexEntry>> {
    problem_index::load_leetcode_lightweight_index().map_err(|error| {
        AppError::ProblemIndexLoadFailed {
            reason: error.to_string(),
        }
    })
}

/** 导入用户自己的 RAG 资料，支持 Markdown、JSON 和 CSV 内容。 */
#[tauri::command]
pub fn import_rag_documents(
    store: State<'_, RagImportStore>,
    request: RagImportRequest,
) -> AppResult<Vec<RagImportedDocument>> {
    store
        .import_documents(request)
        .map_err(|error| AppError::RagImportFailed {
            reason: error.to_string(),
        })
}

/** 获取未删除的用户导入资料列表。 */
#[tauri::command]
pub fn list_rag_imports(store: State<'_, RagImportStore>) -> AppResult<Vec<RagImportedDocument>> {
    Ok(store.list())
}

/** 软删除单条用户导入资料。 */
#[tauri::command]
pub fn delete_rag_import(
    store: State<'_, RagImportStore>,
    embedding_store: State<'_, RagEmbeddingStore>,
    id: String,
) -> AppResult<bool> {
    let deleted = store
        .delete(&id)
        .map_err(|error| AppError::RagImportFailed {
            reason: error.to_string(),
        })?;

    if deleted {
        embedding_store
            .delete_import_embeddings(&id)
            .map_err(|error| AppError::RagRetrievalFailed {
                reason: error.to_string(),
            })?;
    }

    Ok(deleted)
}

/** 清除本地 RAG 索引，不删除用户导入原文、普通历史或内置轻量题目元数据。 */
#[tauri::command]
pub fn clear_rag_index(
    embedding_store: State<'_, RagEmbeddingStore>,
    rag_history_store: State<'_, RagHistoryStore>,
) -> AppResult<RagIndexMaintenanceResult> {
    let cleared_embedding_count =
        embedding_store
            .clear()
            .map_err(|error| AppError::RagRetrievalFailed {
                reason: error.to_string(),
            })?;
    let history_stats =
        rag_history_store
            .clear_index()
            .map_err(|error| AppError::RagHistoryIndexFailed {
                reason: error.to_string(),
            })?;

    Ok(RagIndexMaintenanceResult {
        cleared_embedding_count,
        rebuilt_embedding_count: 0,
        cleared_history_document_count: history_stats.document_count,
        rebuilt_history_document_count: 0,
        cleared_history_chunk_count: history_stats.chunk_count,
        skipped_reason: None,
    })
}

/** 基于当前允许的 RAG 来源重建本地 embedding 缓存和历史 RAG 索引。 */
#[tauri::command]
pub fn rebuild_rag_index(
    embedding_store: State<'_, RagEmbeddingStore>,
    import_store: State<'_, RagImportStore>,
    history_store: State<'_, HistoryStore>,
    rag_history_store: State<'_, RagHistoryStore>,
    settings_store: State<'_, SettingsStore>,
) -> AppResult<RagIndexMaintenanceResult> {
    let settings = settings_store.current();
    if !settings.local_rag_enabled {
        return Ok(RagIndexMaintenanceResult {
            cleared_embedding_count: 0,
            rebuilt_embedding_count: 0,
            cleared_history_document_count: 0,
            rebuilt_history_document_count: 0,
            cleared_history_chunk_count: 0,
            skipped_reason: Some("localRagDisabled".to_string()),
        });
    }

    let history_stats =
        rag_history_store
            .clear_index()
            .map_err(|error| AppError::RagHistoryIndexFailed {
                reason: error.to_string(),
            })?;

    let mut rebuilt_history_document_count = 0u32;
    if settings.rag_history_indexing_enabled {
        for entry in history_store.list() {
            rag_history_store
                .index_history_entry(&entry)
                .map_err(|error| AppError::RagHistoryIndexFailed {
                    reason: error.to_string(),
                })?;
            rebuilt_history_document_count += 1;
        }
    }

    let imports = filter_rag_imports_for_settings(import_store.list(), &settings);
    let history_documents = if settings.rag_history_indexing_enabled {
        rag_history_store.list_documents()
    } else {
        Vec::new()
    };
    let history_chunks = if settings.rag_history_indexing_enabled {
        rag_history_store.list_chunks()
    } else {
        Vec::new()
    };
    let problem_entries = if settings.rag_similar_problems_enabled {
        match problem_index::load_leetcode_lightweight_index() {
            Ok(entries) => entries,
            Err(error) => {
                tracing::warn!(%error, "Could not load lightweight problem index for RAG rebuild");
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };
    let embedding_stats = embedding_store
        .rebuild(
            &imports,
            &history_documents,
            &history_chunks,
            &problem_entries,
        )
        .map_err(|error| AppError::RagRetrievalFailed {
            reason: error.to_string(),
        })?;
    let skipped_reason = (embedding_stats.rebuilt_count == 0).then(|| "emptyIndex".to_string());

    Ok(RagIndexMaintenanceResult {
        cleared_embedding_count: embedding_stats.cleared_count,
        rebuilt_embedding_count: embedding_stats.rebuilt_count,
        cleared_history_document_count: history_stats.document_count,
        rebuilt_history_document_count,
        cleared_history_chunk_count: history_stats.chunk_count,
        skipped_reason,
    })
}

pub(crate) fn search_rag_context_with_settings(
    embedding_store: &RagEmbeddingStore,
    import_store: &RagImportStore,
    history_store: &RagHistoryStore,
    settings: &AppSettings,
    mut request: RagSearchRequest,
) -> AppResult<RagSearchResponse> {
    if !settings.local_rag_enabled {
        return Ok(RagSearchResponse {
            items: Vec::new(),
            skipped_reason: Some("localRagDisabled".to_string()),
        });
    }

    if request.top_k.is_none() {
        request.top_k = Some(settings.rag_max_recall_items);
    }

    let imports = filter_rag_imports_for_settings(import_store.list(), settings);
    let history_documents = if settings.rag_history_indexing_enabled {
        history_store.list_documents()
    } else {
        Vec::new()
    };
    let history_chunks = if settings.rag_history_indexing_enabled {
        history_store.list_chunks()
    } else {
        Vec::new()
    };
    let problem_entries = if settings.rag_similar_problems_enabled {
        match problem_index::load_leetcode_lightweight_index() {
            Ok(entries) => entries,
            Err(error) => {
                tracing::warn!(%error, "Could not load lightweight problem index for RAG search");
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    embedding_store
        .search(
            request,
            &imports,
            &history_documents,
            &history_chunks,
            &problem_entries,
        )
        .map_err(|error| AppError::RagRetrievalFailed {
            reason: error.to_string(),
        })
}

/** 执行本地 RAG 向量检索，返回 top K 相似上下文候选。 */
#[tauri::command]
pub fn search_rag_context(
    embedding_store: State<'_, RagEmbeddingStore>,
    import_store: State<'_, RagImportStore>,
    history_store: State<'_, RagHistoryStore>,
    settings_store: State<'_, SettingsStore>,
    request: RagSearchRequest,
) -> AppResult<RagSearchResponse> {
    let settings = settings_store.current();
    search_rag_context_with_settings(
        &embedding_store,
        &import_store,
        &history_store,
        &settings,
        request,
    )
}

pub(crate) fn select_rag_template_context_with_settings(
    import_store: &RagImportStore,
    settings: &AppSettings,
    mut request: RagTemplateSelectionRequest,
) -> AppResult<RagTemplateSelectionResponse> {
    let default_language = settings_language_to_string(settings.default_language);
    let default_platform = settings_platform_to_string(settings.platform_format);
    if !settings.local_rag_enabled {
        return Ok(RagTemplateSelectionResponse {
            target_language: request
                .target_language
                .clone()
                .unwrap_or_else(|| default_language.clone()),
            platform: request.platform.clone().or(Some(default_platform)),
            solution_modes: Vec::new(),
            templates: Vec::new(),
            skipped_reason: Some("localRagDisabled".to_string()),
        });
    }

    request
        .similar_items
        .retain(|item| rag_search_result_allowed_for_settings(item, settings));
    let imports = filter_rag_imports_for_settings(import_store.list(), settings);

    Ok(crate::rag_templates::select_rag_template_context(
        request,
        &imports,
        &default_language,
        Some(&default_platform),
    ))
}

/** 根据相似题、算法标签和用户默认语言选择解题模式与代码模板。 */
#[tauri::command]
pub fn select_rag_template_context(
    import_store: State<'_, RagImportStore>,
    settings_store: State<'_, SettingsStore>,
    request: RagTemplateSelectionRequest,
) -> AppResult<RagTemplateSelectionResponse> {
    let settings = settings_store.current();
    select_rag_template_context_with_settings(&import_store, &settings, request)
}

pub(crate) fn build_rag_prompt_context_with_settings(
    settings: &AppSettings,
    mut request: RagPromptContextRequest,
) -> AppResult<RagPromptContextResponse> {
    if request.max_items.is_none() {
        request.max_items = Some(settings.rag_max_recall_items);
    }

    if !settings.local_rag_enabled {
        request.search_results.clear();
        request.template_context = None;
        let mut response = crate::rag_prompt_context::build_rag_prompt_context(request);
        response.skipped_reason = Some("localRagDisabled".to_string());
        return Ok(response);
    }

    request
        .search_results
        .retain(|item| rag_search_result_allowed_for_settings(item, settings));
    if let Some(template_context) = &mut request.template_context {
        if !settings.rag_code_templates_retrieval_enabled {
            template_context.templates.clear();
        }
        if !settings.rag_similar_problems_enabled {
            template_context.solution_modes.clear();
        }
    }

    Ok(crate::rag_prompt_context::build_rag_prompt_context(request))
}

/** 构建压缩后的 RAG prompt 上下文，并返回已注入上下文的解题 prompt。 */
#[tauri::command]
pub fn build_rag_prompt_context(
    settings_store: State<'_, SettingsStore>,
    request: RagPromptContextRequest,
) -> AppResult<RagPromptContextResponse> {
    let settings = settings_store.current();
    build_rag_prompt_context_with_settings(&settings, request)
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

fn parse_history_language(value: &str) -> Option<LanguageId> {
    match normalize_history_key(value).as_str() {
        "cpp17" | "c17" => Some(LanguageId::Cpp17),
        "cpp20" | "c20" | "cpp" => Some(LanguageId::Cpp20),
        "python" | "python3" | "py" => Some(LanguageId::Python),
        "java" => Some(LanguageId::Java),
        "javascript" | "js" => Some(LanguageId::JavaScript),
        "typescript" | "ts" => Some(LanguageId::TypeScript),
        "go" | "golang" => Some(LanguageId::Go),
        "rust" | "rs" => Some(LanguageId::Rust),
        _ => None,
    }
}

fn parse_history_platform(value: &str) -> Option<crate::language::PlatformFormat> {
    match normalize_history_key(value).as_str() {
        "acm" | "stdinstdout" => Some(crate::language::PlatformFormat::Acm),
        "leetcode" | "leetcodefunction" | "leet" => Some(crate::language::PlatformFormat::LeetCode),
        "generic" | "function" => Some(crate::language::PlatformFormat::Generic),
        _ => None,
    }
}

fn settings_language_to_string(value: crate::settings::LanguageId) -> String {
    match value {
        crate::settings::LanguageId::Cpp17 => "cpp17",
        crate::settings::LanguageId::Cpp20 => "cpp20",
        crate::settings::LanguageId::Python => "python",
        crate::settings::LanguageId::Java => "java",
        crate::settings::LanguageId::JavaScript => "javascript",
        crate::settings::LanguageId::TypeScript => "typescript",
        crate::settings::LanguageId::Go => "go",
        crate::settings::LanguageId::Rust => "rust",
    }
    .to_string()
}

fn settings_platform_to_string(value: crate::language::PlatformFormat) -> String {
    match value {
        crate::language::PlatformFormat::Acm => "acm",
        crate::language::PlatformFormat::LeetCode => "leetcode",
        crate::language::PlatformFormat::Generic => "generic",
    }
    .to_string()
}

fn clean_list(items: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut cleaned = Vec::new();

    for item in items {
        let value = item.trim();
        if value.is_empty() {
            continue;
        }

        let key = value.to_lowercase();
        if seen.insert(key) {
            cleaned.push(value.to_string());
        }
    }

    cleaned
}

fn normalized_optional(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn stable_history_id(timestamp: &str, title: Option<&str>, result: &str) -> String {
    format!(
        "hist_{}",
        stable_hash(&format!(
            "{}|{}|{}",
            timestamp,
            title.unwrap_or_default(),
            stable_hash(result)
        ))
    )
}

fn stable_hash(value: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn normalize_history_key(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn filter_rag_imports_for_settings(
    imports: Vec<RagImportedDocument>,
    settings: &AppSettings,
) -> Vec<RagImportedDocument> {
    imports
        .into_iter()
        .filter(|document| rag_import_kind_allowed_for_settings(document.kind, settings))
        .collect()
}

fn rag_import_kind_allowed_for_settings(kind: RagImportKind, settings: &AppSettings) -> bool {
    match kind {
        RagImportKind::Solution | RagImportKind::Note => settings.rag_user_notes_retrieval_enabled,
        RagImportKind::Template => settings.rag_code_templates_retrieval_enabled,
    }
}

fn rag_search_result_allowed_for_settings(item: &RagSearchResult, settings: &AppSettings) -> bool {
    match item.source_type {
        RagSearchSourceType::LeetcodeIndex => settings.rag_similar_problems_enabled,
        RagSearchSourceType::History => settings.rag_history_indexing_enabled,
        RagSearchSourceType::UserImport => match item.kind {
            RagSearchChunkKind::Template => settings.rag_code_templates_retrieval_enabled,
            RagSearchChunkKind::Solution | RagSearchChunkKind::Note => {
                settings.rag_user_notes_retrieval_enabled
            }
            RagSearchChunkKind::ProblemStatement
            | RagSearchChunkKind::HistorySummary
            | RagSearchChunkKind::Metadata => settings.rag_user_notes_retrieval_enabled,
        },
    }
}

fn current_timestamp() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch");
    format!("{}.{:09}Z", duration.as_secs(), duration.subsec_nanos())
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

    #[test]
    fn rag_import_filters_follow_retrieval_settings() {
        let mut settings = AppSettings::default();
        settings.local_rag_enabled = true;
        settings.rag_user_notes_retrieval_enabled = false;
        settings.rag_code_templates_retrieval_enabled = true;
        let imports = vec![
            sample_rag_import("note-1", RagImportKind::Note),
            sample_rag_import("solution-1", RagImportKind::Solution),
            sample_rag_import("template-1", RagImportKind::Template),
        ];

        let filtered = filter_rag_imports_for_settings(imports, &settings);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "template-1");
    }

    #[test]
    fn rag_search_result_filters_follow_source_settings() {
        let mut settings = AppSettings::default();
        settings.local_rag_enabled = true;
        settings.rag_history_indexing_enabled = false;
        settings.rag_user_notes_retrieval_enabled = false;
        settings.rag_code_templates_retrieval_enabled = false;
        settings.rag_similar_problems_enabled = true;

        assert!(rag_search_result_allowed_for_settings(
            &sample_rag_search_result(
                RagSearchSourceType::LeetcodeIndex,
                RagSearchChunkKind::Metadata
            ),
            &settings
        ));
        assert!(!rag_search_result_allowed_for_settings(
            &sample_rag_search_result(
                RagSearchSourceType::History,
                RagSearchChunkKind::HistorySummary
            ),
            &settings
        ));
        assert!(!rag_search_result_allowed_for_settings(
            &sample_rag_search_result(RagSearchSourceType::UserImport, RagSearchChunkKind::Note),
            &settings
        ));
        assert!(!rag_search_result_allowed_for_settings(
            &sample_rag_search_result(
                RagSearchSourceType::UserImport,
                RagSearchChunkKind::Template
            ),
            &settings
        ));
    }

    #[test]
    fn rag_source_filters_disable_every_source_independently() {
        let mut settings = AppSettings::default();
        settings.local_rag_enabled = true;
        settings.rag_history_indexing_enabled = false;
        settings.rag_user_notes_retrieval_enabled = false;
        settings.rag_code_templates_retrieval_enabled = false;
        settings.rag_similar_problems_enabled = false;

        let sources = [
            sample_rag_search_result(
                RagSearchSourceType::LeetcodeIndex,
                RagSearchChunkKind::Metadata,
            ),
            sample_rag_search_result(
                RagSearchSourceType::History,
                RagSearchChunkKind::HistorySummary,
            ),
            sample_rag_search_result(RagSearchSourceType::UserImport, RagSearchChunkKind::Note),
            sample_rag_search_result(
                RagSearchSourceType::UserImport,
                RagSearchChunkKind::Template,
            ),
        ];

        assert!(sources
            .iter()
            .all(|source| !rag_search_result_allowed_for_settings(source, &settings)));
    }

    #[test]
    fn list_leetcode_problem_index_returns_lightweight_entries() {
        let entries = list_leetcode_problem_index().expect("index should load");

        assert!(entries.iter().any(|entry| entry.slug == "two-sum"));
        assert!(entries.iter().all(|entry| !entry.title.is_empty()));
    }

    #[test]
    fn language_and_platform_strings_match_frontend_contract() {
        assert_eq!(language_to_string(LanguageId::Cpp20), "cpp20");
        assert_eq!(language_to_string(LanguageId::Python), "python");
        assert_eq!(
            platform_to_string(crate::language::PlatformFormat::LeetCode),
            "leetcode"
        );
        assert_eq!(
            platform_to_string(crate::language::PlatformFormat::Acm),
            "acm"
        );
    }

    fn sample_rag_import(id: &str, kind: RagImportKind) -> RagImportedDocument {
        RagImportedDocument {
            id: id.to_string(),
            source_name: "unit-test".to_string(),
            source_uri: None,
            source_format: crate::rag_imports::RagImportFormat::Markdown,
            kind,
            title: id.to_string(),
            text: "content".to_string(),
            language: None,
            platform: None,
            tags: Vec::new(),
            algorithm_tags: Vec::new(),
            imported_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            deleted_at: None,
        }
    }

    fn sample_rag_search_result(
        source_type: RagSearchSourceType,
        kind: RagSearchChunkKind,
    ) -> RagSearchResult {
        RagSearchResult {
            chunk_id: "chunk-1".to_string(),
            document_id: "doc-1".to_string(),
            source_type,
            source_id: None,
            title: "Two Sum".to_string(),
            snippet: "hash map".to_string(),
            score: 0.8,
            kind,
            problem_type: None,
            language: None,
            platform: None,
            tags: Vec::new(),
            algorithm_tags: Vec::new(),
            reason: "unit test".to_string(),
        }
    }
}
