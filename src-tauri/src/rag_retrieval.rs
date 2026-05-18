/**
 * Question Scan 本地 RAG 检索模块（阶段 6）。
 *
 * 职责：为历史分块、用户导入资料和轻量题目索引生成本地 embedding，
 * 并按题目标题、题干、样例、约束、标签和目标语言执行 top K 向量检索。
 * 范围：使用确定性的 local-hash embedding provider，不访问网络，不做 prompt 注入或结果面板展示。
 */
use crate::problem_index::{ProblemDifficulty, ProblemIndexEntry, ProblemPlatform, ProblemType};
use crate::rag_history::{RagHistoryChunk, RagHistoryChunkKind, RagHistoryDocument};
use crate::rag_imports::{RagImportKind, RagImportedDocument};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

const RAG_EMBEDDING_SCHEMA_VERSION: u32 = 1;
const LOCAL_EMBEDDING_PROVIDER: &str = "local-hash";
const LOCAL_EMBEDDING_MODEL: &str = "local-hash-v1";
const LOCAL_EMBEDDING_DIMENSION: usize = 128;
const DEFAULT_TOP_K: u32 = 5;
const MAX_TOP_K: u32 = 20;
const DEFAULT_MIN_SCORE: f32 = 0.12;
const MAX_SNIPPET_CHARS: usize = 240;
const IMPORT_CHUNK_MAX_CHARS: usize = 1600;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RagSearchSourceType {
    LeetcodeIndex,
    UserImport,
    History,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RagSearchChunkKind {
    ProblemStatement,
    Solution,
    Note,
    Template,
    HistorySummary,
    Metadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RagEmbeddingRecord {
    pub id: String,
    pub chunk_id: String,
    pub provider: String,
    pub model: String,
    pub dimension: u32,
    pub text_hash: String,
    pub vector: Vec<f32>,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RagEmbeddingFile {
    schema_version: u32,
    entries: Vec<RagEmbeddingRecord>,
}

impl Default for RagEmbeddingFile {
    fn default() -> Self {
        Self {
            schema_version: RAG_EMBEDDING_SCHEMA_VERSION,
            entries: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RagSearchRequest {
    pub recognized_title: Option<String>,
    pub recognized_text: Option<String>,
    #[serde(default)]
    pub examples: Vec<String>,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub algorithm_tags: Vec<String>,
    pub target_language: Option<String>,
    pub platform: Option<String>,
    pub top_k: Option<u32>,
    pub min_score: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RagSearchResult {
    pub chunk_id: String,
    pub document_id: String,
    pub source_type: RagSearchSourceType,
    pub source_id: Option<String>,
    pub title: String,
    pub snippet: String,
    pub score: f32,
    pub kind: RagSearchChunkKind,
    pub problem_type: Option<String>,
    pub language: Option<String>,
    pub platform: Option<String>,
    pub tags: Vec<String>,
    pub algorithm_tags: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RagSearchResponse {
    pub items: Vec<RagSearchResult>,
    pub skipped_reason: Option<String>,
}

#[derive(Debug, Error)]
pub enum RagRetrievalError {
    #[error("RAG embedding file could not be serialized. {source}")]
    SerializeFailed { source: serde_json::Error },
    #[error("RAG embedding file could not be written. {source}")]
    IoFailed { source: std::io::Error },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RagEmbeddingRebuildStats {
    pub cleared_count: u32,
    pub rebuilt_count: u32,
}

#[derive(Debug)]
pub struct RagEmbeddingStore {
    path: PathBuf,
    file: RwLock<RagEmbeddingFile>,
}

impl RagEmbeddingStore {
    pub fn load(path: PathBuf) -> Self {
        let file = match fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str::<RagEmbeddingFile>(&contents) {
                Ok(file) if file.schema_version == RAG_EMBEDDING_SCHEMA_VERSION => file,
                Ok(file) => {
                    tracing::warn!(
                        path = %path.display(),
                        version = file.schema_version,
                        "Unsupported RAG embedding file schema version"
                    );
                    RagEmbeddingFile::default()
                }
                Err(error) => {
                    tracing::warn!(path = %path.display(), %error, "Could not parse RAG embedding file");
                    RagEmbeddingFile::default()
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                RagEmbeddingFile::default()
            }
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "Could not read RAG embedding file");
                RagEmbeddingFile::default()
            }
        };

        Self {
            path,
            file: RwLock::new(file),
        }
    }

    pub fn list_embeddings(&self) -> Vec<RagEmbeddingRecord> {
        self.file
            .read()
            .expect("rag embedding lock poisoned")
            .entries
            .clone()
    }

    pub fn clear(&self) -> Result<u32, RagRetrievalError> {
        let cleared_count = {
            let mut file = self.file.write().expect("rag embedding lock poisoned");
            let cleared_count = file.entries.len() as u32;
            *file = RagEmbeddingFile::default();
            cleared_count
        };

        self.write_to_disk()?;
        Ok(cleared_count)
    }

    pub fn delete_import_embeddings(&self, import_id: &str) -> Result<u32, RagRetrievalError> {
        let prefix = format!("ragimpchunk_{import_id}_");
        let deleted_count = {
            let mut file = self.file.write().expect("rag embedding lock poisoned");
            let before = file.entries.len();
            file.entries
                .retain(|entry| !entry.chunk_id.starts_with(&prefix));
            before.saturating_sub(file.entries.len()) as u32
        };

        if deleted_count > 0 {
            self.write_to_disk()?;
        }

        Ok(deleted_count)
    }

    pub fn rebuild(
        &self,
        imports: &[RagImportedDocument],
        history_documents: &[RagHistoryDocument],
        history_chunks: &[RagHistoryChunk],
        problem_entries: &[ProblemIndexEntry],
    ) -> Result<RagEmbeddingRebuildStats, RagRetrievalError> {
        let candidates =
            collect_candidates(imports, history_documents, history_chunks, problem_entries);
        let cleared_count = {
            let mut file = self.file.write().expect("rag embedding lock poisoned");
            let cleared_count = file.entries.len() as u32;
            *file = RagEmbeddingFile::default();
            cleared_count
        };

        if candidates.is_empty() {
            self.write_to_disk()?;
            return Ok(RagEmbeddingRebuildStats {
                cleared_count,
                rebuilt_count: 0,
            });
        }

        let timestamp = current_timestamp();
        let mut embedding_file_changed = false;
        for candidate in &candidates {
            self.ensure_embedding(candidate, &timestamp, &mut embedding_file_changed);
        }

        if embedding_file_changed {
            self.write_to_disk()?;
        }

        Ok(RagEmbeddingRebuildStats {
            cleared_count,
            rebuilt_count: candidates.len() as u32,
        })
    }

    pub fn search(
        &self,
        request: RagSearchRequest,
        imports: &[RagImportedDocument],
        history_documents: &[RagHistoryDocument],
        history_chunks: &[RagHistoryChunk],
        problem_entries: &[ProblemIndexEntry],
    ) -> Result<RagSearchResponse, RagRetrievalError> {
        let query_text = build_query_text(&request);
        if query_text.trim().is_empty() {
            return Ok(RagSearchResponse {
                items: Vec::new(),
                skipped_reason: Some("emptyQuery".to_string()),
            });
        }

        let candidates =
            collect_candidates(imports, history_documents, history_chunks, problem_entries);
        if candidates.is_empty() {
            return Ok(RagSearchResponse {
                items: Vec::new(),
                skipped_reason: Some("emptyIndex".to_string()),
            });
        }

        let timestamp = current_timestamp();
        let query_vector = embed_text(&query_text);
        if vector_norm(&query_vector) == 0.0 {
            return Ok(RagSearchResponse {
                items: Vec::new(),
                skipped_reason: Some("emptyEmbedding".to_string()),
            });
        }

        let top_k = request.top_k.unwrap_or(DEFAULT_TOP_K).clamp(1, MAX_TOP_K);
        let min_score = request
            .min_score
            .unwrap_or(DEFAULT_MIN_SCORE)
            .clamp(0.0, 1.0);
        let mut embedding_file_changed = false;
        let mut scored = Vec::new();

        for candidate in candidates {
            let candidate_embedding =
                self.ensure_embedding(&candidate, &timestamp, &mut embedding_file_changed);
            let vector_score = cosine_similarity(&query_vector, &candidate_embedding);
            let (boost, reason_parts) = lexical_boosts(&request, &candidate);
            let score = (vector_score + boost).clamp(0.0, 1.0);
            if score < min_score {
                continue;
            }

            scored.push(RagSearchResult {
                chunk_id: candidate.chunk_id,
                document_id: candidate.document_id,
                source_type: candidate.source_type,
                source_id: candidate.source_id,
                title: candidate.title,
                snippet: snippet(&candidate.text),
                score,
                kind: candidate.kind,
                problem_type: candidate.problem_type,
                language: candidate.language,
                platform: candidate.platform,
                tags: candidate.tags,
                algorithm_tags: candidate.algorithm_tags,
                reason: build_reason(vector_score, boost, reason_parts),
            });
        }

        if embedding_file_changed {
            self.write_to_disk()?;
        }

        scored.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.title.cmp(&right.title))
                .then_with(|| left.chunk_id.cmp(&right.chunk_id))
        });
        scored.truncate(top_k as usize);

        let skipped_reason = if scored.is_empty() {
            Some("lowConfidence".to_string())
        } else {
            None
        };

        Ok(RagSearchResponse {
            items: scored,
            skipped_reason,
        })
    }

    fn ensure_embedding(
        &self,
        candidate: &SearchCandidate,
        timestamp: &str,
        changed: &mut bool,
    ) -> Vec<f32> {
        let provider = LOCAL_EMBEDDING_PROVIDER.to_string();
        let model = LOCAL_EMBEDDING_MODEL.to_string();
        let text_hash = stable_hash(&candidate.embedding_text());
        let mut file = self.file.write().expect("rag embedding lock poisoned");

        if let Some(existing) = file.entries.iter_mut().find(|entry| {
            entry.chunk_id == candidate.chunk_id
                && entry.provider == provider
                && entry.model == model
        }) {
            if existing.text_hash == text_hash {
                existing.last_used_at = Some(timestamp.to_string());
                *changed = true;
                return existing.vector.clone();
            }

            let vector = embed_text(&candidate.embedding_text());
            *existing = RagEmbeddingRecord {
                id: stable_embedding_id(&candidate.chunk_id, &provider, &model),
                chunk_id: candidate.chunk_id.clone(),
                provider,
                model,
                dimension: LOCAL_EMBEDDING_DIMENSION as u32,
                text_hash,
                vector: vector.clone(),
                created_at: timestamp.to_string(),
                last_used_at: Some(timestamp.to_string()),
            };
            *changed = true;
            return vector;
        }

        let vector = embed_text(&candidate.embedding_text());
        file.entries.push(RagEmbeddingRecord {
            id: stable_embedding_id(&candidate.chunk_id, &provider, &model),
            chunk_id: candidate.chunk_id.clone(),
            provider,
            model,
            dimension: LOCAL_EMBEDDING_DIMENSION as u32,
            text_hash,
            vector: vector.clone(),
            created_at: timestamp.to_string(),
            last_used_at: Some(timestamp.to_string()),
        });
        *changed = true;

        vector
    }

    fn write_to_disk(&self) -> Result<(), RagRetrievalError> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| RagRetrievalError::IoFailed {
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "missing parent directory",
                ),
            })?;
        fs::create_dir_all(parent).map_err(|source| RagRetrievalError::IoFailed { source })?;

        let temp_path = self.path.with_extension("tmp");
        let file = self.file.read().expect("rag embedding lock poisoned");
        let json = serde_json::to_string_pretty(&*file)
            .map_err(|source| RagRetrievalError::SerializeFailed { source })?;
        fs::write(&temp_path, json).map_err(|source| RagRetrievalError::IoFailed { source })?;
        fs::rename(&temp_path, &self.path)
            .map_err(|source| RagRetrievalError::IoFailed { source })?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct SearchCandidate {
    chunk_id: String,
    document_id: String,
    source_type: RagSearchSourceType,
    source_id: Option<String>,
    title: String,
    text: String,
    kind: RagSearchChunkKind,
    problem_type: Option<String>,
    language: Option<String>,
    platform: Option<String>,
    tags: Vec<String>,
    algorithm_tags: Vec<String>,
}

impl SearchCandidate {
    fn embedding_text(&self) -> String {
        let mut parts = vec![
            format!("Title: {}", self.title),
            format!("Content: {}", self.text),
        ];

        if let Some(language) = &self.language {
            parts.push(format!("Language: {language}"));
        }
        if let Some(platform) = &self.platform {
            parts.push(format!("Platform: {platform}"));
        }
        if !self.tags.is_empty() {
            parts.push(format!("Tags: {}", expanded_list_text(&self.tags)));
        }
        if !self.algorithm_tags.is_empty() {
            parts.push(format!(
                "Algorithm tags: {}",
                expanded_list_text(&self.algorithm_tags)
            ));
        }

        parts.join("\n")
    }
}

fn collect_candidates(
    imports: &[RagImportedDocument],
    history_documents: &[RagHistoryDocument],
    history_chunks: &[RagHistoryChunk],
    problem_entries: &[ProblemIndexEntry],
) -> Vec<SearchCandidate> {
    let mut candidates = Vec::new();
    candidates.extend(import_candidates(imports));
    candidates.extend(history_candidates(history_documents, history_chunks));
    candidates.extend(problem_index_candidates(problem_entries));
    candidates
}

fn import_candidates(imports: &[RagImportedDocument]) -> Vec<SearchCandidate> {
    let mut candidates = Vec::new();
    for document in imports {
        if document.deleted_at.is_some() {
            continue;
        }

        for (index, text) in split_text_chunks(&document.text).into_iter().enumerate() {
            candidates.push(SearchCandidate {
                chunk_id: format!("ragimpchunk_{}_{}", document.id, index),
                document_id: document.id.clone(),
                source_type: RagSearchSourceType::UserImport,
                source_id: Some(document.id.clone()),
                title: document.title.clone(),
                text,
                kind: import_kind_to_chunk_kind(document.kind),
                problem_type: None,
                language: document.language.clone(),
                platform: document.platform.clone(),
                tags: clean_list(&document.tags),
                algorithm_tags: clean_list(&document.algorithm_tags),
            });
        }
    }

    candidates
}

fn history_candidates(
    documents: &[RagHistoryDocument],
    chunks: &[RagHistoryChunk],
) -> Vec<SearchCandidate> {
    let document_by_id = documents
        .iter()
        .filter(|document| document.deleted_at.is_none())
        .map(|document| (document.id.as_str(), document))
        .collect::<HashMap<_, _>>();
    let mut candidates = Vec::new();

    for chunk in chunks {
        if chunk.deleted_at.is_some() {
            continue;
        }

        let Some(document) = document_by_id.get(chunk.document_id.as_str()) else {
            continue;
        };

        candidates.push(SearchCandidate {
            chunk_id: chunk.id.clone(),
            document_id: document.id.clone(),
            source_type: RagSearchSourceType::History,
            source_id: Some(document.source_id.clone()),
            title: chunk
                .title
                .clone()
                .unwrap_or_else(|| document.title.clone()),
            text: chunk.text.clone(),
            kind: history_kind_to_chunk_kind(chunk.kind),
            problem_type: None,
            language: chunk.language.map(language_to_string),
            platform: Some(platform_to_string(document.platform)),
            tags: clean_list(&chunk.tags),
            algorithm_tags: clean_list(&chunk.algorithm_tags),
        });
    }

    candidates
}

fn problem_index_candidates(problem_entries: &[ProblemIndexEntry]) -> Vec<SearchCandidate> {
    problem_entries
        .iter()
        .map(|entry| SearchCandidate {
            chunk_id: format!("problemidxchunk_{}", entry.id),
            document_id: entry.id.clone(),
            source_type: RagSearchSourceType::LeetcodeIndex,
            source_id: Some(entry.slug.clone()),
            title: entry.title.clone(),
            text: problem_index_metadata_text(entry),
            kind: RagSearchChunkKind::Metadata,
            problem_type: Some(problem_type_to_string(entry.problem_type).to_string()),
            language: None,
            platform: Some(problem_platform_to_string(entry.platform)),
            tags: clean_list(&entry.tags),
            algorithm_tags: clean_list(&entry.algorithm_tags),
        })
        .collect()
}

fn problem_index_metadata_text(entry: &ProblemIndexEntry) -> String {
    format!(
        "LeetCode metadata only.\nProblem number: {}\nTitle: {}\nSlug: {}\nDifficulty: {}\nTags: {}\nAlgorithm tags: {}\nProblem type: {}\nSource: {}",
        entry.problem_number,
        entry.title,
        entry.slug,
        difficulty_to_string(entry.difficulty),
        expanded_list_text(&entry.tags),
        expanded_list_text(&entry.algorithm_tags),
        problem_type_to_string(entry.problem_type),
        entry.source
    )
}

fn build_query_text(request: &RagSearchRequest) -> String {
    let mut parts = Vec::new();

    if let Some(title) = normalized_optional(request.recognized_title.as_deref()) {
        parts.push(format!("Title: {title}"));
    }
    if let Some(text) = normalized_optional(request.recognized_text.as_deref()) {
        parts.push(format!("Problem: {text}"));
    }
    if !request.examples.is_empty() {
        parts.push(format!(
            "Examples: {}",
            clean_list(&request.examples).join("\n")
        ));
    }
    if !request.constraints.is_empty() {
        parts.push(format!(
            "Constraints: {}",
            clean_list(&request.constraints).join("\n")
        ));
    }
    if !request.tags.is_empty() {
        parts.push(format!("Tags: {}", expanded_list_text(&request.tags)));
    }
    if !request.algorithm_tags.is_empty() {
        parts.push(format!(
            "Algorithm tags: {}",
            expanded_list_text(&request.algorithm_tags)
        ));
    }
    if let Some(language) = normalized_optional(request.target_language.as_deref()) {
        parts.push(format!("Target language: {language}"));
    }
    if let Some(platform) = normalized_optional(request.platform.as_deref()) {
        parts.push(format!("Platform: {platform}"));
    }

    parts.join("\n")
}

fn lexical_boosts(request: &RagSearchRequest, candidate: &SearchCandidate) -> (f32, Vec<String>) {
    let mut boost = 0.0;
    let mut reasons = Vec::new();

    if let Some(title) = normalized_optional(request.recognized_title.as_deref()) {
        let overlap = token_overlap_score(&title, &candidate.title);
        if overlap > 0.0 {
            boost += 0.18 * overlap;
            reasons.push("title".to_string());
        }
    }

    let tag_overlap = list_overlap_score(&request.tags, &candidate.tags);
    if tag_overlap > 0.0 {
        boost += 0.12 * tag_overlap;
        reasons.push("tags".to_string());
    }

    let algorithm_overlap = list_overlap_score(&request.algorithm_tags, &candidate.algorithm_tags);
    if algorithm_overlap > 0.0 {
        boost += 0.16 * algorithm_overlap;
        reasons.push("algorithmTags".to_string());
    }

    if let (Some(query_language), Some(candidate_language)) = (
        normalized_optional(request.target_language.as_deref()),
        candidate.language.as_deref(),
    ) {
        if normalize_key(&query_language) == normalize_key(candidate_language) {
            boost += 0.04;
            reasons.push("language".to_string());
        }
    }

    (boost.min(0.35), reasons)
}

fn embed_text(text: &str) -> Vec<f32> {
    let mut vector = vec![0.0f32; LOCAL_EMBEDDING_DIMENSION];
    for token in tokenize(text) {
        let hash = stable_hash_u64(&token);
        let index = (hash as usize) % LOCAL_EMBEDDING_DIMENSION;
        let sign = if hash & 1 == 0 { 1.0 } else { -1.0 };
        let weight = if token.len() > 2 { 1.0 } else { 0.65 };
        vector[index] += sign * weight;
    }

    normalize_vector(vector)
}

fn cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    let dot = left
        .iter()
        .zip(right.iter())
        .map(|(l, r)| l * r)
        .sum::<f32>();
    dot.clamp(0.0, 1.0)
}

fn normalize_vector(mut vector: Vec<f32>) -> Vec<f32> {
    let norm = vector_norm(&vector);
    if norm == 0.0 {
        return vector;
    }

    for value in vector.iter_mut() {
        *value /= norm;
    }
    vector
}

fn vector_norm(vector: &[f32]) -> f32 {
    vector.iter().map(|value| value * value).sum::<f32>().sqrt()
}

fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            current.push(ch.to_ascii_lowercase());
            continue;
        }

        if !current.is_empty() {
            tokens.push(current.clone());
            current.clear();
        }

        if !ch.is_whitespace() && !ch.is_ascii_punctuation() {
            tokens.push(ch.to_string());
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn token_overlap_score(left: &str, right: &str) -> f32 {
    let left_tokens = tokenize(left).into_iter().collect::<HashSet<_>>();
    let right_tokens = tokenize(right).into_iter().collect::<HashSet<_>>();
    if left_tokens.is_empty() || right_tokens.is_empty() {
        return 0.0;
    }

    let intersection = left_tokens.intersection(&right_tokens).count() as f32;
    intersection / left_tokens.len().max(right_tokens.len()) as f32
}

fn list_overlap_score(left: &[String], right: &[String]) -> f32 {
    let left_keys = clean_list(left)
        .into_iter()
        .map(|value| normalize_key(&value))
        .collect::<HashSet<_>>();
    let right_keys = clean_list(right)
        .into_iter()
        .map(|value| normalize_key(&value))
        .collect::<HashSet<_>>();
    if left_keys.is_empty() || right_keys.is_empty() {
        return 0.0;
    }

    let intersection = left_keys.intersection(&right_keys).count() as f32;
    intersection / left_keys.len().max(right_keys.len()) as f32
}

fn split_text_chunks(text: &str) -> Vec<String> {
    let text = text.trim();
    if text.is_empty() {
        return Vec::new();
    }

    let chars = text.chars().collect::<Vec<_>>();
    chars
        .chunks(IMPORT_CHUNK_MAX_CHARS)
        .map(|chunk| chunk.iter().collect::<String>().trim().to_string())
        .filter(|chunk| !chunk.is_empty())
        .collect()
}

fn snippet(text: &str) -> String {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    truncate_chars(&normalized, MAX_SNIPPET_CHARS)
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let mut truncated = value.chars().take(max_chars).collect::<String>();
    truncated.push_str("...");
    truncated
}

fn build_reason(vector_score: f32, boost: f32, mut reason_parts: Vec<String>) -> String {
    let mut parts = vec![format!("vector={vector_score:.3}")];
    if boost > 0.0 {
        parts.push(format!("boost={boost:.3}"));
    }
    parts.append(&mut reason_parts);
    parts.join(";")
}

fn import_kind_to_chunk_kind(kind: RagImportKind) -> RagSearchChunkKind {
    match kind {
        RagImportKind::Solution => RagSearchChunkKind::Solution,
        RagImportKind::Note => RagSearchChunkKind::Note,
        RagImportKind::Template => RagSearchChunkKind::Template,
    }
}

fn history_kind_to_chunk_kind(kind: RagHistoryChunkKind) -> RagSearchChunkKind {
    match kind {
        RagHistoryChunkKind::ProblemStatement => RagSearchChunkKind::ProblemStatement,
        RagHistoryChunkKind::HistorySummary => RagSearchChunkKind::HistorySummary,
        RagHistoryChunkKind::Note => RagSearchChunkKind::Note,
        RagHistoryChunkKind::Metadata => RagSearchChunkKind::Metadata,
    }
}

fn language_to_string(language: crate::language::LanguageId) -> String {
    match language {
        crate::language::LanguageId::Cpp17 => "cpp17",
        crate::language::LanguageId::Cpp20 => "cpp20",
        crate::language::LanguageId::Python => "python",
        crate::language::LanguageId::Java => "java",
        crate::language::LanguageId::JavaScript => "javascript",
        crate::language::LanguageId::TypeScript => "typescript",
        crate::language::LanguageId::Go => "go",
        crate::language::LanguageId::Rust => "rust",
    }
    .to_string()
}

fn platform_to_string(platform: crate::language::PlatformFormat) -> String {
    match platform {
        crate::language::PlatformFormat::Acm => "acm",
        crate::language::PlatformFormat::LeetCode => "leetcode",
        crate::language::PlatformFormat::Generic => "generic",
    }
    .to_string()
}

fn problem_platform_to_string(platform: ProblemPlatform) -> String {
    match platform {
        ProblemPlatform::Leetcode => "leetcode",
    }
    .to_string()
}

fn difficulty_to_string(difficulty: ProblemDifficulty) -> &'static str {
    match difficulty {
        ProblemDifficulty::Easy => "easy",
        ProblemDifficulty::Medium => "medium",
        ProblemDifficulty::Hard => "hard",
        ProblemDifficulty::Unknown => "unknown",
    }
}

fn problem_type_to_string(problem_type: ProblemType) -> &'static str {
    match problem_type {
        ProblemType::Array => "array",
        ProblemType::HashTable => "hash-table",
        ProblemType::LinkedList => "linked-list",
        ProblemType::String => "string",
        ProblemType::SlidingWindow => "sliding-window",
        ProblemType::TwoPointers => "two-pointers",
        ProblemType::DynamicProgramming => "dynamic-programming",
        ProblemType::BinarySearch => "binary-search",
        ProblemType::Tree => "tree",
        ProblemType::Graph => "graph",
        ProblemType::Stack => "stack",
        ProblemType::Backtracking => "backtracking",
        ProblemType::Design => "design",
        ProblemType::Math => "math",
    }
}

fn clean_list(items: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut cleaned = Vec::new();

    for item in items {
        let value = item.trim();
        if value.is_empty() {
            continue;
        }

        let key = normalize_key(value);
        if seen.insert(key) {
            cleaned.push(value.to_string());
        }
    }

    cleaned
}

fn expanded_list_text(items: &[String]) -> String {
    clean_list(items)
        .into_iter()
        .flat_map(|item| {
            let expanded = item.replace(['-', '_'], " ");
            if expanded == item {
                vec![item]
            } else {
                vec![item, expanded]
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn normalized_optional(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn normalize_key(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn stable_embedding_id(chunk_id: &str, provider: &str, model: &str) -> String {
    format!(
        "ragemb_{}",
        stable_hash(&format!("{chunk_id}|{provider}|{model}"))
    )
}

fn stable_hash(value: &str) -> String {
    format!("{:016x}", stable_hash_u64(value))
}

fn stable_hash_u64(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
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
    use crate::language::{LanguageId, PlatformFormat};
    use crate::problem_index::{ProblemDifficulty, ProblemType};
    use crate::rag_history::{RagHistoryChunk, RagHistoryDocument, RagPrivacyScope, RagSourceType};
    use crate::rag_imports::{RagImportFormat, RagImportKind};

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
            .join("embeddings.json")
    }

    fn import_document(
        id: &str,
        title: &str,
        text: &str,
        tags: &[&str],
        algorithm_tags: &[&str],
    ) -> RagImportedDocument {
        RagImportedDocument {
            id: id.to_string(),
            source_name: "notes.md".to_string(),
            source_uri: None,
            source_format: RagImportFormat::Markdown,
            kind: RagImportKind::Solution,
            title: title.to_string(),
            text: text.to_string(),
            language: Some("cpp20".to_string()),
            platform: Some("leetcode".to_string()),
            tags: tags.iter().map(|value| value.to_string()).collect(),
            algorithm_tags: algorithm_tags
                .iter()
                .map(|value| value.to_string())
                .collect(),
            imported_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            deleted_at: None,
        }
    }

    fn problem_entry() -> ProblemIndexEntry {
        ProblemIndexEntry {
            id: "leetcode_200".to_string(),
            platform: ProblemPlatform::Leetcode,
            problem_number: "200".to_string(),
            slug: "number-of-islands".to_string(),
            title: "Number of Islands".to_string(),
            difficulty: ProblemDifficulty::Medium,
            tags: vec!["Array".to_string(), "Graph".to_string()],
            algorithm_tags: vec!["dfs".to_string(), "graph".to_string()],
            problem_type: ProblemType::Graph,
            source: "leetcode-lightweight-seed-v1".to_string(),
        }
    }

    fn two_sum_query() -> RagSearchRequest {
        RagSearchRequest {
            recognized_title: Some("Two Sum".to_string()),
            recognized_text: Some("Return indices of two numbers that add to target.".to_string()),
            examples: vec!["nums = [2,7,11,15], target = 9".to_string()],
            constraints: vec!["2 <= nums.length <= 10^4".to_string()],
            tags: vec!["array".to_string()],
            algorithm_tags: vec!["hash-table".to_string()],
            target_language: Some("cpp20".to_string()),
            platform: Some("leetcode".to_string()),
            top_k: Some(2),
            min_score: Some(0.0),
        }
    }

    #[test]
    fn local_embedding_cache_reuses_unchanged_text() {
        let store = RagEmbeddingStore::load(temp_path("embedding-cache"));
        let imports = vec![import_document(
            "ragimp_1",
            "Two Sum",
            "Use a hash table to find complements.",
            &["array"],
            &["hash-table"],
        )];

        let first = store
            .search(two_sum_query(), &imports, &[], &[], &[])
            .expect("first search");
        let first_embeddings = store.list_embeddings();
        let second = store
            .search(two_sum_query(), &imports, &[], &[], &[])
            .expect("second search");
        let second_embeddings = store.list_embeddings();

        assert_eq!(first.items.len(), 1);
        assert_eq!(second.items.len(), 1);
        assert_eq!(first_embeddings.len(), 1);
        assert_eq!(second_embeddings.len(), 1);
        assert_eq!(first_embeddings[0].id, second_embeddings[0].id);
        assert_eq!(
            first_embeddings[0].text_hash,
            second_embeddings[0].text_hash
        );
    }

    #[test]
    fn text_changes_replace_cached_embedding_for_same_chunk() {
        let store = RagEmbeddingStore::load(temp_path("embedding-replace"));
        let first_imports = vec![import_document(
            "ragimp_1",
            "Two Sum",
            "Use a hash table to find complements.",
            &["array"],
            &["hash-table"],
        )];
        let second_imports = vec![import_document(
            "ragimp_1",
            "Two Sum",
            "Use sorting and two pointers.",
            &["array"],
            &["two-pointers"],
        )];

        store
            .search(two_sum_query(), &first_imports, &[], &[], &[])
            .expect("first search");
        let first_hash = store.list_embeddings()[0].text_hash.clone();
        store
            .search(two_sum_query(), &second_imports, &[], &[], &[])
            .expect("second search");
        let embeddings = store.list_embeddings();

        assert_eq!(embeddings.len(), 1);
        assert_ne!(first_hash, embeddings[0].text_hash);
    }

    #[test]
    fn clear_removes_all_cached_embeddings() {
        let store = RagEmbeddingStore::load(temp_path("embedding-clear"));
        let imports = vec![import_document(
            "ragimp_1",
            "Two Sum",
            "Use a hash table to find complements.",
            &["array"],
            &["hash-table"],
        )];
        store
            .search(two_sum_query(), &imports, &[], &[], &[])
            .expect("seed embeddings");

        let cleared = store.clear().expect("clear embeddings");

        assert_eq!(cleared, 1);
        assert!(store.list_embeddings().is_empty());
    }

    #[test]
    fn rebuild_recreates_embeddings_for_current_candidates() {
        let store = RagEmbeddingStore::load(temp_path("embedding-rebuild"));
        let imports = vec![import_document(
            "ragimp_1",
            "Two Sum",
            "Use a hash table to find complements.",
            &["array"],
            &["hash-table"],
        )];
        store
            .search(two_sum_query(), &imports, &[], &[], &[])
            .expect("seed embeddings");

        let stats = store
            .rebuild(&imports, &[], &[], &[problem_entry()])
            .expect("rebuild embeddings");

        assert_eq!(stats.cleared_count, 1);
        assert_eq!(stats.rebuilt_count, 2);
        assert_eq!(store.list_embeddings().len(), 2);
    }

    #[test]
    fn delete_import_embeddings_removes_only_that_import() {
        let store = RagEmbeddingStore::load(temp_path("embedding-delete-import"));
        let imports = vec![
            import_document(
                "ragimp_1",
                "Two Sum",
                "Use a hash table to find complements.",
                &["array"],
                &["hash-table"],
            ),
            import_document(
                "ragimp_2",
                "Number of Islands",
                "Use DFS on a grid graph.",
                &["graph"],
                &["dfs"],
            ),
        ];
        store
            .rebuild(&imports, &[], &[], &[])
            .expect("seed embeddings");

        let deleted = store
            .delete_import_embeddings("ragimp_1")
            .expect("delete import embeddings");

        assert_eq!(deleted, 1);
        let remaining = store.list_embeddings();
        assert_eq!(remaining.len(), 1);
        assert!(remaining[0].chunk_id.starts_with("ragimpchunk_ragimp_2_"));
    }

    #[test]
    fn long_import_text_creates_and_deletes_all_chunk_embeddings() {
        let store = RagEmbeddingStore::load(temp_path("long-import-chunks"));
        let long_text = "Use a hash table to find complements. ".repeat(140);
        let imports = vec![import_document(
            "ragimp_long",
            "Long Two Sum Notes",
            &long_text,
            &["array"],
            &["hash-table"],
        )];

        let stats = store
            .rebuild(&imports, &[], &[], &[])
            .expect("rebuild long import embeddings");
        let embeddings = store.list_embeddings();

        assert!(stats.rebuilt_count > 1);
        assert!(embeddings.len() > 1);
        assert!(embeddings
            .iter()
            .all(|entry| entry.chunk_id.starts_with("ragimpchunk_ragimp_long_")));

        let deleted = store
            .delete_import_embeddings("ragimp_long")
            .expect("delete long import embeddings");

        assert_eq!(deleted, embeddings.len() as u32);
        assert!(store.list_embeddings().is_empty());
    }

    #[test]
    fn search_returns_stable_top_k_similarity() {
        let store = RagEmbeddingStore::load(temp_path("top-k"));
        let imports = vec![
            import_document(
                "ragimp_two_sum",
                "Two Sum",
                "Use a hash table to find the target complement.",
                &["array"],
                &["hash-table"],
            ),
            import_document(
                "ragimp_graph",
                "Number of Islands",
                "Use DFS on a grid graph.",
                &["graph"],
                &["dfs"],
            ),
        ];

        let response = store
            .search(two_sum_query(), &imports, &[], &[], &[])
            .expect("search should succeed");

        assert_eq!(response.items.len(), 2);
        assert_eq!(response.items[0].title, "Two Sum");
        assert!(response.items[0].score >= response.items[1].score);
        assert_eq!(response.skipped_reason, None);
    }

    #[test]
    fn search_tie_breaks_equal_scores_by_chunk_id() {
        let store = RagEmbeddingStore::load(temp_path("stable-tie-sort"));
        let imports = vec![
            import_document(
                "ragimp_b",
                "Stable Match",
                "Same searchable content for deterministic equal vector score.",
                &[],
                &[],
            ),
            import_document(
                "ragimp_a",
                "Stable Match",
                "Same searchable content for deterministic equal vector score.",
                &[],
                &[],
            ),
        ];
        let request = RagSearchRequest {
            recognized_title: Some("Same Searchable Content".to_string()),
            recognized_text: Some("deterministic equal vector score".to_string()),
            examples: Vec::new(),
            constraints: Vec::new(),
            tags: Vec::new(),
            algorithm_tags: Vec::new(),
            target_language: None,
            platform: None,
            top_k: Some(2),
            min_score: Some(0.0),
        };

        let response = store
            .search(request, &imports, &[], &[], &[])
            .expect("search should succeed");

        assert_eq!(response.items.len(), 2);
        assert_eq!(response.items[0].score, response.items[1].score);
        assert!(response.items[0].chunk_id < response.items[1].chunk_id);
    }

    #[test]
    fn search_can_return_leetcode_metadata_without_full_statement() {
        let store = RagEmbeddingStore::load(temp_path("leetcode-metadata"));
        let request = RagSearchRequest {
            recognized_title: Some("Island Count".to_string()),
            recognized_text: Some("Count connected land cells in a grid.".to_string()),
            examples: Vec::new(),
            constraints: Vec::new(),
            tags: vec!["graph".to_string()],
            algorithm_tags: vec!["dfs".to_string(), "graph".to_string()],
            target_language: Some("cpp20".to_string()),
            platform: Some("leetcode".to_string()),
            top_k: Some(1),
            min_score: Some(0.0),
        };

        let response = store
            .search(request, &[], &[], &[], &[problem_entry()])
            .expect("search should succeed");

        assert_eq!(response.items.len(), 1);
        assert_eq!(
            response.items[0].source_type,
            RagSearchSourceType::LeetcodeIndex
        );
        assert_eq!(response.items[0].title, "Number of Islands");
        assert!(response.items[0].snippet.contains("metadata only"));
        assert!(!response.items[0].snippet.contains("Given"));
    }

    #[test]
    fn search_returns_low_confidence_reason_when_everything_is_filtered() {
        let store = RagEmbeddingStore::load(temp_path("low-confidence"));
        let imports = vec![import_document(
            "ragimp_1",
            "Two Sum",
            "Use a hash table to find complements.",
            &["array"],
            &["hash-table"],
        )];
        let request = RagSearchRequest {
            recognized_title: Some("Circle Area".to_string()),
            recognized_text: Some("Compute the area of a circle from its radius.".to_string()),
            examples: vec!["radius = 3".to_string()],
            constraints: vec!["radius > 0".to_string()],
            tags: vec!["geometry".to_string()],
            algorithm_tags: vec!["math".to_string()],
            target_language: Some("cpp20".to_string()),
            platform: Some("leetcode".to_string()),
            top_k: Some(2),
            min_score: Some(1.0),
        };

        let response = store
            .search(request, &imports, &[], &[], &[])
            .expect("search should succeed");

        assert!(response.items.is_empty());
        assert_eq!(response.skipped_reason, Some("lowConfidence".to_string()));
    }

    #[test]
    fn empty_index_returns_skipped_reason() {
        let store = RagEmbeddingStore::load(temp_path("empty-index"));

        let response = store
            .search(two_sum_query(), &[], &[], &[], &[])
            .expect("search should succeed");

        assert!(response.items.is_empty());
        assert_eq!(response.skipped_reason, Some("emptyIndex".to_string()));
    }

    #[test]
    fn history_chunks_are_searchable_candidates() {
        let store = RagEmbeddingStore::load(temp_path("history-candidate"));
        let document = RagHistoryDocument {
            id: "raghistdoc_1".to_string(),
            source_type: RagSourceType::History,
            source_id: "hist_1".to_string(),
            title: "Two Sum".to_string(),
            text: "Two Sum history".to_string(),
            answer_summary: "Use a hash table.".to_string(),
            user_note: None,
            language: LanguageId::Cpp20,
            platform: PlatformFormat::LeetCode,
            model: "gpt-4o-mini".to_string(),
            tags: vec!["array".to_string()],
            algorithm_tags: vec!["hash-table".to_string()],
            privacy_scope: RagPrivacyScope::LocalOnly,
            checksum: "abc".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            deleted_at: None,
        };
        let chunk = RagHistoryChunk {
            id: "raghistchunk_1".to_string(),
            document_id: document.id.clone(),
            chunk_index: 0,
            kind: RagHistoryChunkKind::ProblemStatement,
            text: "Return two indices whose values add to target.".to_string(),
            token_estimate: 12,
            title: Some("Two Sum".to_string()),
            language: Some(LanguageId::Cpp20),
            tags: vec!["array".to_string()],
            algorithm_tags: vec!["hash-table".to_string()],
            created_at: "2026-01-01T00:00:00Z".to_string(),
            deleted_at: None,
        };

        let response = store
            .search(two_sum_query(), &[], &[document], &[chunk], &[])
            .expect("history search");

        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].source_type, RagSearchSourceType::History);
        assert_eq!(response.items[0].source_id, Some("hist_1".to_string()));
    }
}
