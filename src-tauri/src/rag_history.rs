/**
 * Question Scan 历史 RAG 入库模块（阶段 6）。
 *
 * 职责：把用户可控历史记录转换为本地可检索文档、分块和历史链接。
 * 范围：只建立 JSON-backed 检索记录，不生成 embedding，不执行相似召回。
 * 隐私：不保存完整截图、临时图片路径、API Key 或浏览器凭据。
 */
use crate::history::HistoryEntry;
use crate::language::{LanguageId, PlatformFormat};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

const RAG_HISTORY_SCHEMA_VERSION: u32 = 1;
const MAX_SUMMARY_CHARS: usize = 1200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RagSourceType {
    History,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RagPrivacyScope {
    LocalOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RagHistoryChunkKind {
    ProblemStatement,
    HistorySummary,
    Note,
    Metadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RagIndexStatus {
    Pending,
    Indexed,
    Failed,
    Disabled,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RagHistoryDocument {
    pub id: String,
    pub source_type: RagSourceType,
    pub source_id: String,
    pub title: String,
    pub text: String,
    pub answer_summary: String,
    pub user_note: Option<String>,
    pub language: LanguageId,
    pub platform: PlatformFormat,
    pub model: String,
    pub tags: Vec<String>,
    pub algorithm_tags: Vec<String>,
    pub privacy_scope: RagPrivacyScope,
    pub checksum: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RagHistoryChunk {
    pub id: String,
    pub document_id: String,
    pub chunk_index: u32,
    pub kind: RagHistoryChunkKind,
    pub text: String,
    pub token_estimate: u32,
    pub title: Option<String>,
    pub language: Option<LanguageId>,
    pub tags: Vec<String>,
    pub algorithm_tags: Vec<String>,
    pub created_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryRagLink {
    pub history_entry_id: String,
    pub document_id: String,
    pub indexed_at: String,
    pub index_status: RagIndexStatus,
    pub disabled_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RagHistoryFile {
    pub schema_version: u32,
    pub documents: Vec<RagHistoryDocument>,
    pub chunks: Vec<RagHistoryChunk>,
    pub history_links: Vec<HistoryRagLink>,
}

impl Default for RagHistoryFile {
    fn default() -> Self {
        Self {
            schema_version: RAG_HISTORY_SCHEMA_VERSION,
            documents: Vec::new(),
            chunks: Vec::new(),
            history_links: Vec::new(),
        }
    }
}

#[derive(Debug, Error)]
pub enum RagHistoryError {
    #[error("RAG history file could not be serialized. {source}")]
    SerializeFailed { source: serde_json::Error },
    #[error("RAG history file could not be written. {source}")]
    IoFailed { source: std::io::Error },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RagHistoryClearStats {
    pub document_count: u32,
    pub chunk_count: u32,
    pub history_link_count: u32,
}

#[derive(Debug)]
pub struct RagHistoryStore {
    path: PathBuf,
    file: RwLock<RagHistoryFile>,
}

impl RagHistoryStore {
    pub fn load(path: PathBuf) -> Self {
        let file = match fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str::<RagHistoryFile>(&contents) {
                Ok(file) if file.schema_version == RAG_HISTORY_SCHEMA_VERSION => file,
                Ok(file) => {
                    tracing::warn!(
                        path = %path.display(),
                        version = file.schema_version,
                        "Unsupported RAG history file schema version"
                    );
                    RagHistoryFile::default()
                }
                Err(error) => {
                    tracing::warn!(path = %path.display(), %error, "Could not parse RAG history file");
                    RagHistoryFile::default()
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => RagHistoryFile::default(),
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "Could not read RAG history file");
                RagHistoryFile::default()
            }
        };

        Self {
            path,
            file: RwLock::new(file),
        }
    }

    pub fn list_documents(&self) -> Vec<RagHistoryDocument> {
        self.file
            .read()
            .expect("rag history lock poisoned")
            .documents
            .iter()
            .filter(|document| document.deleted_at.is_none())
            .cloned()
            .collect()
    }

    pub fn list_chunks(&self) -> Vec<RagHistoryChunk> {
        self.file
            .read()
            .expect("rag history lock poisoned")
            .chunks
            .iter()
            .filter(|chunk| chunk.deleted_at.is_none())
            .cloned()
            .collect()
    }

    pub fn list_history_links(&self) -> Vec<HistoryRagLink> {
        self.file
            .read()
            .expect("rag history lock poisoned")
            .history_links
            .iter()
            .filter(|link| link.index_status != RagIndexStatus::Deleted)
            .cloned()
            .collect()
    }

    pub fn index_history_entry(
        &self,
        entry: &HistoryEntry,
    ) -> Result<RagHistoryDocument, RagHistoryError> {
        self.index_history_entry_at(entry, current_timestamp())
    }

    fn index_history_entry_at(
        &self,
        entry: &HistoryEntry,
        timestamp: String,
    ) -> Result<RagHistoryDocument, RagHistoryError> {
        let document = build_history_document(entry, &timestamp);
        let chunks = build_history_chunks(&document, entry, &timestamp);
        let link = HistoryRagLink {
            history_entry_id: entry.id.clone(),
            document_id: document.id.clone(),
            indexed_at: timestamp.clone(),
            index_status: RagIndexStatus::Indexed,
            disabled_reason: None,
        };

        {
            let mut file = self.file.write().expect("rag history lock poisoned");
            upsert_document(&mut file.documents, document.clone());
            file.chunks.retain(|chunk| chunk.document_id != document.id);
            file.chunks.extend(chunks);
            upsert_history_link(&mut file.history_links, link);
        }

        self.write_to_disk()?;
        Ok(document)
    }

    pub fn delete_by_history_entry(&self, history_entry_id: &str) -> Result<bool, RagHistoryError> {
        self.delete_by_history_entry_at(
            history_entry_id,
            current_timestamp(),
            "historyDeleted".to_string(),
        )
    }

    fn delete_by_history_entry_at(
        &self,
        history_entry_id: &str,
        timestamp: String,
        reason: String,
    ) -> Result<bool, RagHistoryError> {
        let mut changed = false;
        {
            let mut file = self.file.write().expect("rag history lock poisoned");
            let document_ids: Vec<String> = file
                .documents
                .iter()
                .filter(|document| {
                    document.source_type == RagSourceType::History
                        && document.source_id == history_entry_id
                        && document.deleted_at.is_none()
                })
                .map(|document| document.id.clone())
                .collect();

            for document in file.documents.iter_mut() {
                if document_ids.iter().any(|id| id == &document.id) {
                    document.deleted_at = Some(timestamp.clone());
                    document.updated_at = timestamp.clone();
                    changed = true;
                }
            }

            for chunk in file.chunks.iter_mut() {
                if document_ids.iter().any(|id| id == &chunk.document_id)
                    && chunk.deleted_at.is_none()
                {
                    chunk.deleted_at = Some(timestamp.clone());
                    changed = true;
                }
            }

            for link in file.history_links.iter_mut() {
                if link.history_entry_id == history_entry_id
                    && link.index_status != RagIndexStatus::Deleted
                {
                    link.index_status = RagIndexStatus::Deleted;
                    link.disabled_reason = Some(reason.clone());
                    changed = true;
                }
            }
        }

        if changed {
            self.write_to_disk()?;
        }
        Ok(changed)
    }

    pub fn delete_all_history_entries(&self) -> Result<(), RagHistoryError> {
        let timestamp = current_timestamp();
        {
            let mut file = self.file.write().expect("rag history lock poisoned");
            for document in file.documents.iter_mut() {
                if document.source_type == RagSourceType::History && document.deleted_at.is_none() {
                    document.deleted_at = Some(timestamp.clone());
                    document.updated_at = timestamp.clone();
                }
            }

            for chunk in file.chunks.iter_mut() {
                if chunk.deleted_at.is_none() {
                    chunk.deleted_at = Some(timestamp.clone());
                }
            }

            for link in file.history_links.iter_mut() {
                if link.index_status != RagIndexStatus::Deleted {
                    link.index_status = RagIndexStatus::Deleted;
                    link.disabled_reason = Some("historyCleared".to_string());
                }
            }
        }

        self.write_to_disk()
    }

    pub fn clear_index(&self) -> Result<RagHistoryClearStats, RagHistoryError> {
        let stats = {
            let mut file = self.file.write().expect("rag history lock poisoned");
            let stats = RagHistoryClearStats {
                document_count: file.documents.len() as u32,
                chunk_count: file.chunks.len() as u32,
                history_link_count: file.history_links.len() as u32,
            };
            *file = RagHistoryFile::default();
            stats
        };

        self.write_to_disk()?;
        Ok(stats)
    }

    fn all(&self) -> RagHistoryFile {
        self.file.read().expect("rag history lock poisoned").clone()
    }

    fn write_to_disk(&self) -> Result<(), RagHistoryError> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| RagHistoryError::IoFailed {
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "missing parent directory",
                ),
            })?;
        fs::create_dir_all(parent).map_err(|source| RagHistoryError::IoFailed { source })?;

        let temp_path = self.path.with_extension("tmp");
        let json = serde_json::to_string_pretty(&self.all())
            .map_err(|source| RagHistoryError::SerializeFailed { source })?;
        fs::write(&temp_path, json).map_err(|source| RagHistoryError::IoFailed { source })?;
        fs::rename(&temp_path, &self.path)
            .map_err(|source| RagHistoryError::IoFailed { source })?;

        Ok(())
    }
}

fn build_history_document(entry: &HistoryEntry, timestamp: &str) -> RagHistoryDocument {
    let title = entry
        .recognized_title
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Untitled history problem")
        .to_string();
    let answer_summary = summarize_ai_output(&entry.result);
    let tags = clean_list(&entry.tags);
    let algorithm_tags = clean_list(&entry.algorithm_tags);
    let text = compose_document_text(&title, entry, &answer_summary, &tags, &algorithm_tags);
    let checksum = stable_hash(&format!(
        "{}|{}|{}|{}|{:?}|{:?}|{}|{}",
        title,
        entry.recognized_text.as_deref().unwrap_or_default(),
        answer_summary,
        entry.user_note.as_deref().unwrap_or_default(),
        entry.language,
        entry.platform,
        tags.join(","),
        algorithm_tags.join(",")
    ));

    RagHistoryDocument {
        id: stable_id("raghistdoc", &entry.id),
        source_type: RagSourceType::History,
        source_id: entry.id.clone(),
        title,
        text,
        answer_summary,
        user_note: normalized_optional(entry.user_note.as_deref()),
        language: entry.language,
        platform: entry.platform,
        model: entry.model.trim().to_string(),
        tags,
        algorithm_tags,
        privacy_scope: RagPrivacyScope::LocalOnly,
        checksum,
        created_at: timestamp.to_string(),
        updated_at: timestamp.to_string(),
        deleted_at: None,
    }
}

fn compose_document_text(
    title: &str,
    entry: &HistoryEntry,
    answer_summary: &str,
    tags: &[String],
    algorithm_tags: &[String],
) -> String {
    let mut lines = vec![format!("Title: {title}")];

    if let Some(text) = normalized_optional(entry.recognized_text.as_deref()) {
        lines.push(format!("Problem:\n{text}"));
    }

    if !answer_summary.is_empty() {
        lines.push(format!("Answer summary:\n{answer_summary}"));
    }

    if let Some(note) = normalized_optional(entry.user_note.as_deref()) {
        lines.push(format!("User note:\n{note}"));
    }

    lines.push(format!("Language: {}", entry.language.label()));
    lines.push(format!("Platform: {}", entry.platform.label()));

    if !tags.is_empty() {
        lines.push(format!("Tags: {}", tags.join(", ")));
    }
    if !algorithm_tags.is_empty() {
        lines.push(format!("Algorithm tags: {}", algorithm_tags.join(", ")));
    }

    lines.join("\n\n")
}

fn build_history_chunks(
    document: &RagHistoryDocument,
    entry: &HistoryEntry,
    timestamp: &str,
) -> Vec<RagHistoryChunk> {
    let mut chunks = Vec::new();
    push_chunk(
        &mut chunks,
        document,
        RagHistoryChunkKind::Metadata,
        format!(
            "Title: {}\nLanguage: {}\nPlatform: {}\nModel: {}\nTags: {}\nAlgorithm tags: {}",
            document.title,
            document.language.label(),
            document.platform.label(),
            document.model,
            document.tags.join(", "),
            document.algorithm_tags.join(", ")
        ),
        Some("History metadata".to_string()),
        Some(document.language),
        timestamp,
    );

    if let Some(problem_text) = normalized_optional(entry.recognized_text.as_deref()) {
        push_chunk(
            &mut chunks,
            document,
            RagHistoryChunkKind::ProblemStatement,
            problem_text,
            Some(document.title.clone()),
            Some(document.language),
            timestamp,
        );
    }

    if !document.answer_summary.is_empty() {
        push_chunk(
            &mut chunks,
            document,
            RagHistoryChunkKind::HistorySummary,
            document.answer_summary.clone(),
            Some("AI output summary".to_string()),
            Some(document.language),
            timestamp,
        );
    }

    if let Some(note) = normalized_optional(entry.user_note.as_deref()) {
        push_chunk(
            &mut chunks,
            document,
            RagHistoryChunkKind::Note,
            note,
            Some("User note".to_string()),
            Some(document.language),
            timestamp,
        );
    }

    chunks
}

fn push_chunk(
    chunks: &mut Vec<RagHistoryChunk>,
    document: &RagHistoryDocument,
    kind: RagHistoryChunkKind,
    text: String,
    title: Option<String>,
    language: Option<LanguageId>,
    timestamp: &str,
) {
    let chunk_index = chunks.len() as u32;
    chunks.push(RagHistoryChunk {
        id: stable_id(
            "raghistchunk",
            &format!("{}|{:?}|{}", document.id, kind, chunk_index),
        ),
        document_id: document.id.clone(),
        chunk_index,
        kind,
        token_estimate: estimate_tokens(&text),
        text,
        title,
        language,
        tags: document.tags.clone(),
        algorithm_tags: document.algorithm_tags.clone(),
        created_at: timestamp.to_string(),
        deleted_at: None,
    });
}

fn upsert_document(documents: &mut Vec<RagHistoryDocument>, next: RagHistoryDocument) {
    if let Some(existing) = documents.iter_mut().find(|document| document.id == next.id) {
        let created_at = existing.created_at.clone();
        *existing = RagHistoryDocument { created_at, ..next };
        return;
    }

    documents.push(next);
}

fn upsert_history_link(links: &mut Vec<HistoryRagLink>, next: HistoryRagLink) {
    if let Some(existing) = links
        .iter_mut()
        .find(|link| link.history_entry_id == next.history_entry_id)
    {
        *existing = next;
        return;
    }

    links.push(next);
}

pub fn summarize_ai_output(result: &str) -> String {
    let mut parts = Vec::new();

    if let Some(strategy) = extract_section(result, "解题策略") {
        parts.push(strategy);
    }

    if let Some(complexity) = extract_section(result, "复杂度") {
        parts.push(format!("复杂度: {complexity}"));
    }

    let summary = if parts.is_empty() {
        fallback_summary(result)
    } else {
        parts.join("\n\n")
    };

    truncate_chars(summary.trim(), MAX_SUMMARY_CHARS)
}

fn extract_section(text: &str, header: &str) -> Option<String> {
    let header_pattern = format!("{header}:");
    let start = text.find(&header_pattern)?;
    let remainder = &text[start + header_pattern.len()..];
    let mut lines = Vec::new();

    for line in remainder.lines() {
        let trimmed = line.trim();
        if !lines.is_empty() && is_solution_section_header(trimmed) {
            break;
        }
        if lines.is_empty() && trimmed.is_empty() {
            continue;
        }
        if lines.is_empty() && is_solution_section_header(trimmed) {
            break;
        }
        lines.push(line);
    }

    let section = strip_fenced_code_blocks(&lines.join("\n"))
        .trim()
        .to_string();
    if section.is_empty() {
        None
    } else {
        Some(section)
    }
}

fn fallback_summary(result: &str) -> String {
    strip_fenced_code_blocks(result)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !is_solution_section_header(line))
        .take(12)
        .collect::<Vec<_>>()
        .join("\n")
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let mut truncated = value.chars().take(max_chars).collect::<String>();
    truncated.push_str("...");
    truncated
}

fn strip_fenced_code_blocks(text: &str) -> String {
    let mut output = String::new();
    let mut in_code_block = false;

    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        if !in_code_block {
            output.push_str(line);
            output.push('\n');
        }
    }

    output
}

fn is_solution_section_header(line: &str) -> bool {
    matches!(
        line,
        "题目识别:"
            | "解题策略:"
            | "代码语言:"
            | "完整代码:"
            | "复杂度:"
            | "边界用例:"
            | "注意事项:"
    )
}

fn estimate_tokens(text: &str) -> u32 {
    let chars = text.chars().count();
    let estimate = (chars + 3) / 4;
    estimate.max(1) as u32
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

fn stable_id(prefix: &str, value: &str) -> String {
    format!("{prefix}_{}", stable_hash(value))
}

fn stable_hash(value: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
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
    use std::time::{SystemTime, UNIX_EPOCH};

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
            .join("rag")
            .join("history-records.json")
    }

    fn sample_history_entry() -> HistoryEntry {
        HistoryEntry {
            id: "hist_1".to_string(),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            recognized_title: Some("Two Sum".to_string()),
            recognized_text: Some("Given nums and target, return two indices.".to_string()),
            language: LanguageId::Cpp20,
            platform: PlatformFormat::LeetCode,
            model: "gpt-4o-mini".to_string(),
            result: "题目识别:\nTwo Sum\n\n解题策略:\nUse a hash map to find complements.\n\n完整代码:\n```cpp\nclass Solution {};\n```\n\n复杂度:\nO(n) time and O(n) space.".to_string(),
            user_note: Some("Review duplicate values.".to_string()),
            tags: vec!["array".to_string(), "array".to_string()],
            algorithm_tags: vec!["hash-table".to_string()],
        }
    }

    #[test]
    fn summarizes_ai_output_without_copying_full_code() {
        let summary = summarize_ai_output(&sample_history_entry().result);

        assert!(summary.contains("Use a hash map"));
        assert!(summary.contains("O(n) time"));
        assert!(!summary.contains("class Solution"));
    }

    #[test]
    fn indexes_history_entry_as_document_chunks_and_link() {
        let store = RagHistoryStore::load(temp_path("rag-history-index"));

        let document = store
            .index_history_entry_at(&sample_history_entry(), "2026-01-02T00:00:00Z".to_string())
            .expect("index history");

        assert_eq!(store.list_documents(), vec![document.clone()]);
        assert_eq!(document.source_type, RagSourceType::History);
        assert_eq!(document.source_id, "hist_1");
        assert_eq!(document.title, "Two Sum");
        assert_eq!(document.tags, vec!["array"]);
        assert_eq!(document.algorithm_tags, vec!["hash-table"]);
        assert!(document.text.contains("Given nums and target"));
        assert!(document.text.contains("Review duplicate values"));
        assert!(!document.text.contains("class Solution"));

        let chunks = store.list_chunks();
        assert_eq!(chunks.len(), 4);
        assert!(chunks
            .iter()
            .any(|chunk| chunk.kind == RagHistoryChunkKind::ProblemStatement));
        assert!(chunks
            .iter()
            .any(|chunk| chunk.kind == RagHistoryChunkKind::HistorySummary));
        assert!(chunks
            .iter()
            .any(|chunk| chunk.kind == RagHistoryChunkKind::Note));

        let links = store.list_history_links();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].history_entry_id, "hist_1");
        assert_eq!(links[0].document_id, document.id);
        assert_eq!(links[0].index_status, RagIndexStatus::Indexed);
    }

    #[test]
    fn reindexing_same_history_entry_updates_without_duplicates() {
        let store = RagHistoryStore::load(temp_path("rag-history-upsert"));
        let mut entry = sample_history_entry();
        store
            .index_history_entry_at(&entry, "2026-01-02T00:00:00Z".to_string())
            .expect("first index");

        entry.user_note = Some("Updated note.".to_string());
        let updated = store
            .index_history_entry_at(&entry, "2026-01-03T00:00:00Z".to_string())
            .expect("second index");

        assert_eq!(store.list_documents().len(), 1);
        assert_eq!(store.list_history_links().len(), 1);
        assert_eq!(store.list_documents()[0].created_at, "2026-01-02T00:00:00Z");
        assert_eq!(store.list_documents()[0].updated_at, "2026-01-03T00:00:00Z");
        assert!(updated.text.contains("Updated note"));
    }

    #[test]
    fn deleting_history_entry_hides_document_chunks_and_link() {
        let store = RagHistoryStore::load(temp_path("rag-history-delete"));
        store
            .index_history_entry_at(&sample_history_entry(), "2026-01-02T00:00:00Z".to_string())
            .expect("index");

        let deleted = store
            .delete_by_history_entry_at(
                "hist_1",
                "2026-01-04T00:00:00Z".to_string(),
                "historyDeleted".to_string(),
            )
            .expect("delete");

        assert!(deleted);
        assert!(store.list_documents().is_empty());
        assert!(store.list_chunks().is_empty());
        assert!(store.list_history_links().is_empty());
    }

    #[test]
    fn clear_history_hides_all_history_rag_records() {
        let store = RagHistoryStore::load(temp_path("rag-history-clear"));
        store
            .index_history_entry_at(&sample_history_entry(), "2026-01-02T00:00:00Z".to_string())
            .expect("index");

        store.delete_all_history_entries().expect("clear");

        assert!(store.list_documents().is_empty());
        assert!(store.list_chunks().is_empty());
        assert!(store.list_history_links().is_empty());
    }

    #[test]
    fn clear_index_removes_history_rag_records_from_disk_state() {
        let store = RagHistoryStore::load(temp_path("rag-history-hard-clear"));
        store
            .index_history_entry_at(&sample_history_entry(), "2026-01-02T00:00:00Z".to_string())
            .expect("index");

        let stats = store.clear_index().expect("clear index");

        assert_eq!(stats.document_count, 1);
        assert_eq!(stats.chunk_count, 4);
        assert_eq!(stats.history_link_count, 1);
        assert!(store.list_documents().is_empty());
        assert!(store.list_chunks().is_empty());
        assert!(store.list_history_links().is_empty());
        assert!(store.all().documents.is_empty());
        assert!(store.all().chunks.is_empty());
        assert!(store.all().history_links.is_empty());
    }
}
