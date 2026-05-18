/**
 * Question Scan 用户资料导入模块（阶段 6）。
 *
 * 职责：解析用户主动导入的 Markdown、JSON 和 CSV 题解、笔记、代码模板，
 * 并用本地 JSON 文件记录来源、导入时间、更新时间和删除状态。
 */
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

const RAG_IMPORT_SCHEMA_VERSION: u32 = 1;

/** 用户导入资料的文件格式。 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RagImportFormat {
    Markdown,
    Json,
    Csv,
}

/** 用户导入资料的业务类型。 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RagImportKind {
    Solution,
    Note,
    Template,
}

/** 前端传入的导入请求。 */
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RagImportRequest {
    pub source_name: String,
    pub source_uri: Option<String>,
    pub format: RagImportFormat,
    pub kind: RagImportKind,
    pub content: String,
}

/** 已保存的用户导入资料。 */
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RagImportedDocument {
    pub id: String,
    pub source_name: String,
    pub source_uri: Option<String>,
    pub source_format: RagImportFormat,
    pub kind: RagImportKind,
    pub title: String,
    pub text: String,
    pub language: Option<String>,
    pub platform: Option<String>,
    pub tags: Vec<String>,
    pub algorithm_tags: Vec<String>,
    pub imported_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RagImportFile {
    schema_version: u32,
    entries: Vec<RagImportedDocument>,
}

impl Default for RagImportFile {
    fn default() -> Self {
        Self {
            schema_version: RAG_IMPORT_SCHEMA_VERSION,
            entries: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RagImportDraft {
    kind: RagImportKind,
    title: String,
    text: String,
    language: Option<String>,
    platform: Option<String>,
    tags: Vec<String>,
    algorithm_tags: Vec<String>,
}

#[derive(Debug, Error)]
pub enum RagImportError {
    #[error("Import source name must not be empty.")]
    EmptySourceName,
    #[error("Import content must not be empty.")]
    EmptyContent,
    #[error("Imported document at position {index} is invalid. {reason}")]
    InvalidDocument { index: usize, reason: String },
    #[error("JSON import could not be parsed. {source}")]
    JsonParseFailed { source: serde_json::Error },
    #[error("CSV import could not be parsed. {reason}")]
    CsvParseFailed { reason: String },
    #[error("RAG import file could not be serialized. {source}")]
    SerializeFailed { source: serde_json::Error },
    #[error("RAG import file could not be written. {source}")]
    IoFailed { source: std::io::Error },
}

#[derive(Debug)]
pub struct RagImportStore {
    path: PathBuf,
    entries: RwLock<Vec<RagImportedDocument>>,
}

impl RagImportStore {
    pub fn load(path: PathBuf) -> Self {
        let entries = match fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str::<RagImportFile>(&contents) {
                Ok(file) if file.schema_version == RAG_IMPORT_SCHEMA_VERSION => file.entries,
                Ok(file) => {
                    tracing::warn!(
                        path = %path.display(),
                        version = file.schema_version,
                        "Unsupported RAG import file schema version"
                    );
                    Vec::new()
                }
                Err(error) => {
                    tracing::warn!(path = %path.display(), %error, "Could not parse RAG import file");
                    Vec::new()
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "Could not read RAG import file");
                Vec::new()
            }
        };

        Self {
            path,
            entries: RwLock::new(entries),
        }
    }

    pub fn list(&self) -> Vec<RagImportedDocument> {
        self.entries
            .read()
            .expect("rag imports lock poisoned")
            .iter()
            .filter(|entry| entry.deleted_at.is_none())
            .cloned()
            .collect()
    }

    pub fn import_documents(
        &self,
        request: RagImportRequest,
    ) -> Result<Vec<RagImportedDocument>, RagImportError> {
        let timestamp = current_timestamp();
        self.import_documents_at(request, timestamp)
    }

    fn import_documents_at(
        &self,
        request: RagImportRequest,
        timestamp: String,
    ) -> Result<Vec<RagImportedDocument>, RagImportError> {
        validate_request(&request)?;
        let drafts = parse_import_request(&request)?;

        let mut saved = Vec::new();
        {
            let mut entries = self.entries.write().expect("rag imports lock poisoned");
            for draft in drafts {
                let key = import_identity_key(&request, &draft);
                let id = stable_import_id(&key);

                if let Some(existing) = entries.iter_mut().find(|entry| entry.id == id) {
                    existing.source_name = request.source_name.trim().to_string();
                    existing.source_uri = trimmed_optional(request.source_uri.as_deref());
                    existing.source_format = request.format;
                    existing.kind = draft.kind;
                    existing.title = draft.title;
                    existing.text = draft.text;
                    existing.language = draft.language;
                    existing.platform = draft.platform;
                    existing.tags = draft.tags;
                    existing.algorithm_tags = draft.algorithm_tags;
                    existing.updated_at = timestamp.clone();
                    existing.deleted_at = None;
                    saved.push(existing.clone());
                    continue;
                }

                let entry = RagImportedDocument {
                    id,
                    source_name: request.source_name.trim().to_string(),
                    source_uri: trimmed_optional(request.source_uri.as_deref()),
                    source_format: request.format,
                    kind: draft.kind,
                    title: draft.title,
                    text: draft.text,
                    language: draft.language,
                    platform: draft.platform,
                    tags: draft.tags,
                    algorithm_tags: draft.algorithm_tags,
                    imported_at: timestamp.clone(),
                    updated_at: timestamp.clone(),
                    deleted_at: None,
                };
                entries.push(entry.clone());
                saved.push(entry);
            }
        }

        self.write_to_disk()?;
        Ok(saved)
    }

    pub fn delete(&self, id: &str) -> Result<bool, RagImportError> {
        let timestamp = current_timestamp();
        self.delete_at(id, timestamp)
    }

    fn delete_at(&self, id: &str, timestamp: String) -> Result<bool, RagImportError> {
        let mut deleted = false;
        {
            let mut entries = self.entries.write().expect("rag imports lock poisoned");
            if let Some(entry) = entries
                .iter_mut()
                .find(|entry| entry.id == id && entry.deleted_at.is_none())
            {
                entry.deleted_at = Some(timestamp.clone());
                entry.updated_at = timestamp;
                deleted = true;
            }
        }

        if deleted {
            self.write_to_disk()?;
        }

        Ok(deleted)
    }

    fn all_entries(&self) -> Vec<RagImportedDocument> {
        self.entries
            .read()
            .expect("rag imports lock poisoned")
            .clone()
    }

    fn write_to_disk(&self) -> Result<(), RagImportError> {
        let file = RagImportFile {
            schema_version: RAG_IMPORT_SCHEMA_VERSION,
            entries: self.all_entries(),
        };

        let parent = self.path.parent().ok_or_else(|| RagImportError::IoFailed {
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "missing parent directory",
            ),
        })?;
        fs::create_dir_all(parent).map_err(|source| RagImportError::IoFailed { source })?;

        let temp_path = self.path.with_extension("tmp");
        let json = serde_json::to_string_pretty(&file)
            .map_err(|source| RagImportError::SerializeFailed { source })?;

        fs::write(&temp_path, json).map_err(|source| RagImportError::IoFailed { source })?;
        fs::rename(&temp_path, &self.path).map_err(|source| RagImportError::IoFailed { source })?;

        Ok(())
    }
}

fn validate_request(request: &RagImportRequest) -> Result<(), RagImportError> {
    if request.source_name.trim().is_empty() {
        return Err(RagImportError::EmptySourceName);
    }

    if request.content.trim().is_empty() {
        return Err(RagImportError::EmptyContent);
    }

    Ok(())
}

fn parse_import_request(request: &RagImportRequest) -> Result<Vec<RagImportDraft>, RagImportError> {
    match request.format {
        RagImportFormat::Markdown => parse_markdown_import(request),
        RagImportFormat::Json => parse_json_import(request),
        RagImportFormat::Csv => parse_csv_import(request),
    }
}

fn parse_markdown_import(
    request: &RagImportRequest,
) -> Result<Vec<RagImportDraft>, RagImportError> {
    let (metadata, body) = split_markdown_front_matter(&request.content);
    let title = metadata
        .get("title")
        .and_then(|value| non_empty_string(value))
        .or_else(|| first_markdown_heading(body))
        .unwrap_or_else(|| request.source_name.trim().to_string());
    let text = body.trim().to_string();

    if text.is_empty() {
        return Err(RagImportError::InvalidDocument {
            index: 0,
            reason: "Markdown body must not be empty".to_string(),
        });
    }

    Ok(vec![RagImportDraft {
        kind: metadata
            .get("kind")
            .and_then(|value| parse_kind(value))
            .unwrap_or(request.kind),
        title,
        text,
        language: metadata
            .get("language")
            .and_then(|value| parse_language(value)),
        platform: metadata
            .get("platform")
            .and_then(|value| parse_platform(value)),
        tags: metadata
            .get("tags")
            .map(|value| split_list(value))
            .unwrap_or_default(),
        algorithm_tags: metadata
            .get("algorithmtags")
            .or_else(|| metadata.get("algorithm_tags"))
            .map(|value| split_list(value))
            .unwrap_or_default(),
    }])
}

fn parse_json_import(request: &RagImportRequest) -> Result<Vec<RagImportDraft>, RagImportError> {
    let value: Value = serde_json::from_str(&request.content)
        .map_err(|source| RagImportError::JsonParseFailed { source })?;

    let records = match value {
        Value::Array(items) => items,
        Value::Object(_) => vec![value],
        _ => {
            return Err(RagImportError::InvalidDocument {
                index: 0,
                reason: "JSON import must be an object or an array of objects".to_string(),
            })
        }
    };

    records
        .iter()
        .enumerate()
        .map(|(index, record)| draft_from_json_record(index, record, request))
        .collect()
}

fn draft_from_json_record(
    index: usize,
    record: &Value,
    request: &RagImportRequest,
) -> Result<RagImportDraft, RagImportError> {
    let object = record
        .as_object()
        .ok_or_else(|| RagImportError::InvalidDocument {
            index,
            reason: "JSON array entries must be objects".to_string(),
        })?;

    let title = first_json_string(object, &["title", "name"])
        .unwrap_or_else(|| request.source_name.trim().to_string());
    let text = first_json_string(
        object,
        &[
            "text",
            "content",
            "solution",
            "note",
            "templateText",
            "body",
        ],
    )
    .ok_or_else(|| RagImportError::InvalidDocument {
        index,
        reason:
            "JSON import entry must include text, content, solution, note, templateText, or body"
                .to_string(),
    })?;

    if text.trim().is_empty() {
        return Err(RagImportError::InvalidDocument {
            index,
            reason: "document text must not be empty".to_string(),
        });
    }

    Ok(RagImportDraft {
        kind: first_json_string(object, &["kind", "type"])
            .and_then(|value| parse_kind(&value))
            .unwrap_or(request.kind),
        title,
        text,
        language: first_json_string(object, &["language"]).and_then(|value| parse_language(&value)),
        platform: first_json_string(object, &["platform"]).and_then(|value| parse_platform(&value)),
        tags: json_list(object.get("tags")),
        algorithm_tags: object
            .get("algorithmTags")
            .or_else(|| object.get("algorithm_tags"))
            .map(|value| json_list(Some(value)))
            .unwrap_or_default(),
    })
}

fn parse_csv_import(request: &RagImportRequest) -> Result<Vec<RagImportDraft>, RagImportError> {
    let rows = parse_csv_rows(&request.content)?;
    if rows.len() < 2 {
        return Err(RagImportError::CsvParseFailed {
            reason: "CSV import must include a header row and at least one data row".to_string(),
        });
    }

    let headers: Vec<String> = rows[0].iter().map(|header| normalize_key(header)).collect();
    let mut drafts = Vec::new();

    for (row_index, row) in rows.iter().skip(1).enumerate() {
        if row.iter().all(|field| field.trim().is_empty()) {
            continue;
        }

        let record = headers
            .iter()
            .cloned()
            .zip(row.iter().cloned())
            .collect::<HashMap<_, _>>();
        drafts.push(draft_from_csv_record(row_index, &record, request)?);
    }

    if drafts.is_empty() {
        return Err(RagImportError::CsvParseFailed {
            reason: "CSV import did not contain any non-empty data rows".to_string(),
        });
    }

    Ok(drafts)
}

fn draft_from_csv_record(
    index: usize,
    record: &HashMap<String, String>,
    request: &RagImportRequest,
) -> Result<RagImportDraft, RagImportError> {
    let title = first_csv_field(record, &["title", "name"])
        .unwrap_or_else(|| request.source_name.trim().to_string());
    let text = first_csv_field(
        record,
        &[
            "text",
            "content",
            "solution",
            "note",
            "templatetext",
            "body",
        ],
    )
    .ok_or_else(|| RagImportError::InvalidDocument {
        index,
        reason: "CSV row must include text, content, solution, note, templateText, or body"
            .to_string(),
    })?;

    if text.trim().is_empty() {
        return Err(RagImportError::InvalidDocument {
            index,
            reason: "document text must not be empty".to_string(),
        });
    }

    Ok(RagImportDraft {
        kind: first_csv_field(record, &["kind", "type"])
            .and_then(|value| parse_kind(&value))
            .unwrap_or(request.kind),
        title,
        text,
        language: first_csv_field(record, &["language"]).and_then(|value| parse_language(&value)),
        platform: first_csv_field(record, &["platform"]).and_then(|value| parse_platform(&value)),
        tags: first_csv_field(record, &["tags"])
            .map(|value| split_list(&value))
            .unwrap_or_default(),
        algorithm_tags: first_csv_field(record, &["algorithmtags", "algorithm_tags"])
            .map(|value| split_list(&value))
            .unwrap_or_default(),
    })
}

fn parse_csv_rows(input: &str) -> Result<Vec<Vec<String>>, RagImportError> {
    let mut rows = Vec::new();
    for (line_index, line) in input.lines().enumerate() {
        let mut row = Vec::new();
        let mut field = String::new();
        let mut chars = line.chars().peekable();
        let mut in_quotes = false;

        while let Some(ch) = chars.next() {
            match ch {
                '"' if in_quotes && chars.peek() == Some(&'"') => {
                    field.push('"');
                    chars.next();
                }
                '"' => {
                    in_quotes = !in_quotes;
                }
                ',' if !in_quotes => {
                    row.push(field.trim().to_string());
                    field.clear();
                }
                _ => field.push(ch),
            }
        }

        if in_quotes {
            return Err(RagImportError::CsvParseFailed {
                reason: format!("unterminated quoted field on line {}", line_index + 1),
            });
        }

        row.push(field.trim().to_string());
        rows.push(row);
    }

    Ok(rows)
}

fn split_markdown_front_matter(input: &str) -> (HashMap<String, String>, &str) {
    let trimmed = input.trim_start();
    if !trimmed.starts_with("---") {
        return (HashMap::new(), input);
    }

    let mut lines = trimmed.lines();
    let first = lines.next();
    if first != Some("---") {
        return (HashMap::new(), input);
    }

    let mut metadata_lines = Vec::new();
    let mut body_start = 0usize;
    let mut consumed = first.map(|line| line.len() + 1).unwrap_or_default();

    for line in lines {
        if line.trim() == "---" {
            body_start = consumed + line.len() + 1;
            break;
        }
        metadata_lines.push(line);
        consumed += line.len() + 1;
    }

    if body_start == 0 {
        return (HashMap::new(), input);
    }

    let metadata = metadata_lines
        .iter()
        .filter_map(|line| line.split_once(':'))
        .map(|(key, value)| {
            (
                normalize_key(key),
                value.trim().trim_matches('"').to_string(),
            )
        })
        .collect::<HashMap<_, _>>();
    let body = trimmed.get(body_start..).unwrap_or("").trim_start();

    (metadata, body)
}

fn first_markdown_heading(input: &str) -> Option<String> {
    input.lines().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix("# ")
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
    })
}

fn first_json_string(object: &serde_json::Map<String, Value>, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        object
            .get(*key)
            .and_then(value_to_string)
            .and_then(|value| non_empty_string(&value))
    })
}

fn json_list(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(value_to_string)
            .flat_map(|value| split_list(&value))
            .collect(),
        Some(value) => value_to_string(value)
            .map(|value| split_list(&value))
            .unwrap_or_default(),
        None => Vec::new(),
    }
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn first_csv_field(record: &HashMap<String, String>, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| record.get(*key).and_then(|value| non_empty_string(value)))
}

fn split_list(value: &str) -> Vec<String> {
    value
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(|ch| matches!(ch, ',' | ';' | '|'))
        .filter_map(|item| {
            let value = item.trim().trim_matches('"').trim_matches('\'').trim();
            if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            }
        })
        .collect()
}

fn non_empty_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn parse_kind(value: &str) -> Option<RagImportKind> {
    match normalize_key(value).as_str() {
        "solution" | "solutions" | "answer" => Some(RagImportKind::Solution),
        "note" | "notes" | "mistake" | "wronganswer" => Some(RagImportKind::Note),
        "template" | "codetemplate" | "snippet" => Some(RagImportKind::Template),
        _ => None,
    }
}

fn parse_language(value: &str) -> Option<String> {
    match normalize_key(value).as_str() {
        "cpp17" | "c17" => Some("cpp17".to_string()),
        "cpp20" | "c20" | "cpp" => Some("cpp20".to_string()),
        "python" | "python3" | "py" => Some("python".to_string()),
        "java" => Some("java".to_string()),
        "javascript" | "js" => Some("javascript".to_string()),
        "typescript" | "ts" => Some("typescript".to_string()),
        "go" | "golang" => Some("go".to_string()),
        "rust" | "rs" => Some("rust".to_string()),
        _ => None,
    }
}

fn parse_platform(value: &str) -> Option<String> {
    match normalize_key(value).as_str() {
        "acm" | "stdinstdout" => Some("acm".to_string()),
        "leetcode" | "leet" => Some("leetcode".to_string()),
        "generic" | "function" => Some("generic".to_string()),
        _ => None,
    }
}

fn normalize_key(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn trimmed_optional(value: Option<&str>) -> Option<String> {
    value.and_then(non_empty_string)
}

fn import_identity_key(request: &RagImportRequest, draft: &RagImportDraft) -> String {
    format!(
        "{}|{}|{:?}",
        request
            .source_uri
            .as_deref()
            .unwrap_or(request.source_name.as_str())
            .trim()
            .to_lowercase(),
        draft.title.trim().to_lowercase(),
        draft.kind
    )
}

fn stable_import_id(key: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in key.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("ragimp_{hash:016x}")
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
            .join("user-imports.json")
    }

    fn request(
        format: RagImportFormat,
        kind: RagImportKind,
        content: impl Into<String>,
    ) -> RagImportRequest {
        RagImportRequest {
            source_name: "my-notes.md".to_string(),
            source_uri: Some("C:/notes/my-notes.md".to_string()),
            format,
            kind,
            content: content.into(),
        }
    }

    #[test]
    fn parses_markdown_front_matter_and_heading() {
        let request = request(
            RagImportFormat::Markdown,
            RagImportKind::Solution,
            r#"---
title: Two Sum Notes
tags: array, hash-table
algorithmTags: hash-table
language: C++20
platform: leetcode
kind: note
---
# Ignored Heading

Use a map to find complements."#,
        );

        let drafts = parse_import_request(&request).expect("markdown should parse");

        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].title, "Two Sum Notes");
        assert_eq!(drafts[0].kind, RagImportKind::Note);
        assert_eq!(drafts[0].language, Some("cpp20".to_string()));
        assert_eq!(drafts[0].platform, Some("leetcode".to_string()));
        assert_eq!(drafts[0].tags, vec!["array", "hash-table"]);
        assert_eq!(drafts[0].algorithm_tags, vec!["hash-table"]);
        assert!(drafts[0].text.contains("Use a map"));
    }

    #[test]
    fn parses_json_array_with_solution_and_template_entries() {
        let request = request(
            RagImportFormat::Json,
            RagImportKind::Solution,
            r#"[
              {
                "title": "Maximum Subarray",
                "solution": "Kadane keeps the best suffix.",
                "tags": ["Array", "Dynamic Programming"],
                "algorithmTags": ["dynamic-programming"],
                "language": "cpp20",
                "platform": "leetcode"
              },
              {
                "title": "DFS Template",
                "templateText": "void dfs(int u) {}",
                "kind": "template",
                "language": "C++17",
                "algorithmTags": "graph, dfs"
              }
            ]"#,
        );

        let drafts = parse_import_request(&request).expect("json should parse");

        assert_eq!(drafts.len(), 2);
        assert_eq!(drafts[0].title, "Maximum Subarray");
        assert_eq!(drafts[0].kind, RagImportKind::Solution);
        assert_eq!(drafts[0].algorithm_tags, vec!["dynamic-programming"]);
        assert_eq!(drafts[1].kind, RagImportKind::Template);
        assert_eq!(drafts[1].language, Some("cpp17".to_string()));
        assert_eq!(drafts[1].algorithm_tags, vec!["graph", "dfs"]);
    }

    #[test]
    fn parses_csv_rows_with_quoted_commas() {
        let request = request(
            RagImportFormat::Csv,
            RagImportKind::Note,
            "title,text,tags,algorithmTags,language,platform\n\"3Sum\",\"Sort, then use two pointers\",\"array; two-pointers\",\"two-pointers\",python,leetcode",
        );

        let drafts = parse_import_request(&request).expect("csv should parse");

        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].title, "3Sum");
        assert_eq!(drafts[0].text, "Sort, then use two pointers");
        assert_eq!(drafts[0].tags, vec!["array", "two-pointers"]);
        assert_eq!(drafts[0].language, Some("python".to_string()));
        assert_eq!(drafts[0].platform, Some("leetcode".to_string()));
    }

    #[test]
    fn rejects_empty_source_name_and_content() {
        let empty_source = RagImportRequest {
            source_name: "  ".to_string(),
            source_uri: None,
            format: RagImportFormat::Markdown,
            kind: RagImportKind::Solution,
            content: "# Two Sum\n\nUse a hash map.".to_string(),
        };
        let empty_content = RagImportRequest {
            source_name: "notes.md".to_string(),
            source_uri: None,
            format: RagImportFormat::Markdown,
            kind: RagImportKind::Solution,
            content: "  ".to_string(),
        };

        assert!(matches!(
            validate_request(&empty_source),
            Err(RagImportError::EmptySourceName)
        ));
        assert!(matches!(
            validate_request(&empty_content),
            Err(RagImportError::EmptyContent)
        ));
    }

    #[test]
    fn rejects_json_and_csv_rows_without_import_text() {
        let json_request = request(
            RagImportFormat::Json,
            RagImportKind::Solution,
            r#"{"title":"Two Sum","tags":["array"]}"#,
        );
        let csv_request = request(
            RagImportFormat::Csv,
            RagImportKind::Solution,
            "title,tags\nTwo Sum,array",
        );

        let json_error =
            parse_import_request(&json_request).expect_err("json without text should fail");
        let csv_error =
            parse_import_request(&csv_request).expect_err("csv without text should fail");

        assert!(matches!(json_error, RagImportError::InvalidDocument { .. }));
        assert!(matches!(csv_error, RagImportError::InvalidDocument { .. }));
    }

    #[test]
    fn rejects_csv_with_unterminated_quoted_field() {
        let request = request(
            RagImportFormat::Csv,
            RagImportKind::Note,
            "title,text\n\"Two Sum\",\"unterminated note",
        );

        let error = parse_import_request(&request).expect_err("bad csv should fail");

        assert!(matches!(error, RagImportError::CsvParseFailed { .. }));
    }

    #[test]
    fn store_upserts_same_source_title_and_kind() {
        let path = temp_path("rag-import-upsert");
        let store = RagImportStore::load(path);
        let first = request(
            RagImportFormat::Markdown,
            RagImportKind::Solution,
            "# Two Sum\n\nFirst version.",
        );
        let second = request(
            RagImportFormat::Markdown,
            RagImportKind::Solution,
            "# Two Sum\n\nSecond version.",
        );

        let saved_first = store
            .import_documents_at(first, "2026-01-01T00:00:00Z".to_string())
            .expect("first import");
        let saved_second = store
            .import_documents_at(second, "2026-01-02T00:00:00Z".to_string())
            .expect("second import");

        assert_eq!(saved_first[0].id, saved_second[0].id);
        assert_eq!(store.list().len(), 1);
        assert!(store.list()[0].text.contains("Second version"));
        assert_eq!(store.list()[0].imported_at, "2026-01-01T00:00:00Z");
        assert_eq!(store.list()[0].updated_at, "2026-01-02T00:00:00Z");
    }

    #[test]
    fn delete_hides_imported_document_from_default_list() {
        let path = temp_path("rag-import-delete");
        let store = RagImportStore::load(path);
        let saved = store
            .import_documents_at(
                request(
                    RagImportFormat::Markdown,
                    RagImportKind::Solution,
                    "# Two Sum\n\nUse a hash map.",
                ),
                "2026-01-01T00:00:00Z".to_string(),
            )
            .expect("import");

        let deleted = store
            .delete_at(&saved[0].id, "2026-01-03T00:00:00Z".to_string())
            .expect("delete");

        assert!(deleted);
        assert!(store.list().is_empty());
        assert_eq!(
            store.all_entries()[0].deleted_at,
            Some("2026-01-03T00:00:00Z".to_string())
        );
    }

    #[test]
    fn persists_imports_across_reload() {
        let path = temp_path("rag-import-persist");
        let store = RagImportStore::load(path.clone());
        store
            .import_documents_at(
                request(
                    RagImportFormat::Markdown,
                    RagImportKind::Solution,
                    "# Two Sum\n\nUse a hash map.",
                ),
                "2026-01-01T00:00:00Z".to_string(),
            )
            .expect("import");

        let reloaded = RagImportStore::load(path);

        assert_eq!(reloaded.list().len(), 1);
        assert_eq!(reloaded.list()[0].title, "Two Sum");
    }
}
