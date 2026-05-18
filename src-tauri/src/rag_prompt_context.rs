/**
 * Question Scan RAG prompt 上下文注入模块（阶段 6）。
 *
 * 职责：把本地检索命中和模板推荐压缩为可注入解题 prompt 的上下文 section。
 * 边界：本地上下文只作为参考，必须限制召回条数、最低分数和 token 预算，
 * 且不得覆盖当前截图识别内容或用户确认内容。
 */
use crate::language::{LanguageId, PlatformFormat};
use crate::rag_retrieval::{RagSearchChunkKind, RagSearchResult, RagSearchSourceType};
use crate::rag_templates::RagTemplateSelectionResponse;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const DEFAULT_MAX_ITEMS: u32 = 5;
const MAX_ITEMS: u32 = 10;
const DEFAULT_MAX_CONTEXT_TOKENS: u32 = 1200;
const MAX_CONTEXT_TOKENS: u32 = 3000;
const DEFAULT_MIN_SCORE: f32 = 0.25;
const MAX_SEARCH_SNIPPET_CHARS: usize = 320;
const MAX_TEMPLATE_SNIPPET_CHARS: usize = 520;
const MAX_MODE_SNIPPET_CHARS: usize = 220;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RagPromptContextRequest {
    pub recognized_title: Option<String>,
    pub recognized_text: Option<String>,
    pub target_language: Option<String>,
    pub platform: Option<String>,
    #[serde(default)]
    pub search_results: Vec<RagSearchResult>,
    pub template_context: Option<RagTemplateSelectionResponse>,
    pub max_items: Option<u32>,
    pub max_context_tokens: Option<u32>,
    pub min_score: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RagPromptContextSourceType {
    LeetcodeIndex,
    UserImport,
    History,
    CodeTemplate,
    SolutionMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RagPromptContextKind {
    ProblemStatement,
    Solution,
    Note,
    Template,
    HistorySummary,
    Metadata,
    SolutionMode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RagPromptContextItem {
    pub id: String,
    pub source_type: RagPromptContextSourceType,
    pub source_id: Option<String>,
    pub title: String,
    pub snippet: String,
    pub score: f32,
    pub kind: RagPromptContextKind,
    pub language: Option<String>,
    pub platform: Option<String>,
    pub tags: Vec<String>,
    pub algorithm_tags: Vec<String>,
    pub used_in_prompt: bool,
    pub skipped_reason: Option<String>,
    pub token_estimate: u32,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RagPromptContextResponse {
    pub solution_prompt: String,
    pub prompt_section: String,
    pub items: Vec<RagPromptContextItem>,
    pub used_item_count: u32,
    pub token_estimate: u32,
    pub skipped_reason: Option<String>,
}

#[derive(Debug, Clone)]
struct PromptCandidate {
    id: String,
    source_type: RagPromptContextSourceType,
    source_id: Option<String>,
    title: String,
    snippet: String,
    score: f32,
    kind: RagPromptContextKind,
    language: Option<String>,
    platform: Option<String>,
    tags: Vec<String>,
    algorithm_tags: Vec<String>,
    reason: String,
    order: usize,
}

pub fn build_rag_prompt_context(request: RagPromptContextRequest) -> RagPromptContextResponse {
    let language = parse_language(request.target_language.as_deref()).unwrap_or_default();
    let platform = parse_platform(request.platform.as_deref()).unwrap_or_default();
    let max_items = request
        .max_items
        .unwrap_or(DEFAULT_MAX_ITEMS)
        .clamp(1, MAX_ITEMS);
    let max_context_tokens = request
        .max_context_tokens
        .unwrap_or(DEFAULT_MAX_CONTEXT_TOKENS)
        .clamp(1, MAX_CONTEXT_TOKENS);
    let min_score = request
        .min_score
        .unwrap_or(DEFAULT_MIN_SCORE)
        .clamp(0.0, 1.0);

    let mut candidates = collect_candidates(&request);
    candidates.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                source_priority(left.source_type).cmp(&source_priority(right.source_type))
            })
            .then_with(|| left.order.cmp(&right.order))
            .then_with(|| left.id.cmp(&right.id))
    });

    let mut items = Vec::with_capacity(candidates.len());
    let mut used_count = 0u32;
    let mut total_tokens = 0u32;

    for candidate in candidates {
        let token_estimate = estimate_item_tokens(&candidate);
        let mut item = RagPromptContextItem {
            id: candidate.id,
            source_type: candidate.source_type,
            source_id: candidate.source_id,
            title: candidate.title,
            snippet: candidate.snippet,
            score: candidate.score,
            kind: candidate.kind,
            language: candidate.language,
            platform: candidate.platform,
            tags: candidate.tags,
            algorithm_tags: candidate.algorithm_tags,
            used_in_prompt: false,
            skipped_reason: None,
            token_estimate,
            reason: candidate.reason,
        };

        item.skipped_reason = skipped_reason_for_item(
            &item,
            min_score,
            used_count,
            max_items,
            total_tokens,
            max_context_tokens,
        );

        if item.skipped_reason.is_none() {
            item.used_in_prompt = true;
            used_count += 1;
            total_tokens += item.token_estimate;
        }

        items.push(item);
    }

    let prompt_section = if used_count > 0 {
        render_prompt_section(
            request.recognized_title.as_deref(),
            request.recognized_text.as_deref(),
            request.target_language.as_deref(),
            request.platform.as_deref(),
            &items,
            total_tokens,
            max_context_tokens,
        )
    } else {
        String::new()
    };
    let solution_prompt = crate::prompts::build_solution_prompt_with_rag(
        language,
        platform,
        request.recognized_title.as_deref(),
        request.recognized_text.as_deref(),
        (!prompt_section.is_empty()).then_some(prompt_section.as_str()),
    );
    let skipped_reason = skipped_reason_for_response(&items, used_count);

    RagPromptContextResponse {
        solution_prompt,
        prompt_section,
        items,
        used_item_count: used_count,
        token_estimate: total_tokens,
        skipped_reason,
    }
}

fn collect_candidates(request: &RagPromptContextRequest) -> Vec<PromptCandidate> {
    let mut candidates = Vec::new();
    for (index, item) in request.search_results.iter().enumerate() {
        candidates.push(search_result_candidate(item, index));
    }

    if let Some(template_context) = &request.template_context {
        let base_order = candidates.len();

        for (index, mode) in template_context.solution_modes.iter().enumerate() {
            candidates.push(PromptCandidate {
                id: format!("mode_{}", mode.id),
                source_type: RagPromptContextSourceType::SolutionMode,
                source_id: Some(mode.id.clone()),
                title: mode.title.clone(),
                snippet: compress_text(&mode.description, MAX_MODE_SNIPPET_CHARS),
                score: mode.score.clamp(0.0, 1.0),
                kind: RagPromptContextKind::SolutionMode,
                language: None,
                platform: template_context.platform.clone(),
                tags: Vec::new(),
                algorithm_tags: clean_list(&mode.algorithm_tags),
                reason: mode.reason.clone(),
                order: base_order + index,
            });
        }

        let template_base_order = base_order + template_context.solution_modes.len();
        for (index, template) in template_context.templates.iter().enumerate() {
            let template_text = normalized_optional(template.template_text.as_str())
                .unwrap_or_else(|| template.snippet.clone());

            candidates.push(PromptCandidate {
                id: template.id.clone(),
                source_type: RagPromptContextSourceType::CodeTemplate,
                source_id: Some(template.source_document_id.clone()),
                title: template.title.clone(),
                snippet: compress_text(&template_text, MAX_TEMPLATE_SNIPPET_CHARS),
                score: template.score.clamp(0.0, 1.0),
                kind: RagPromptContextKind::Template,
                language: template.language.clone(),
                platform: template.platform.clone(),
                tags: Vec::new(),
                algorithm_tags: clean_list(&template.algorithm_tags),
                reason: template.reason.clone(),
                order: template_base_order + index,
            });
        }
    }

    candidates
}

fn search_result_candidate(item: &RagSearchResult, order: usize) -> PromptCandidate {
    let mut snippet = compress_text(&item.snippet, MAX_SEARCH_SNIPPET_CHARS);
    if item.source_type == RagSearchSourceType::LeetcodeIndex
        && !snippet.to_ascii_lowercase().contains("metadata")
    {
        snippet = format!("LeetCode lightweight metadata only. {snippet}");
    }

    PromptCandidate {
        id: item.chunk_id.clone(),
        source_type: search_source_type_to_prompt_source_type(item.source_type),
        source_id: item.source_id.clone(),
        title: item.title.clone(),
        snippet,
        score: item.score.clamp(0.0, 1.0),
        kind: search_chunk_kind_to_prompt_kind(item.kind),
        language: item.language.clone(),
        platform: item.platform.clone(),
        tags: clean_list(&item.tags),
        algorithm_tags: clean_list(&item.algorithm_tags),
        reason: item.reason.clone(),
        order,
    }
}

fn skipped_reason_for_item(
    item: &RagPromptContextItem,
    min_score: f32,
    used_count: u32,
    max_items: u32,
    total_tokens: u32,
    max_context_tokens: u32,
) -> Option<String> {
    if normalized_optional(item.title.as_str()).is_none()
        && normalized_optional(item.snippet.as_str()).is_none()
    {
        return Some("emptyContext".to_string());
    }

    if item.score < min_score {
        return Some("lowConfidence".to_string());
    }

    if used_count >= max_items {
        return Some("maxItems".to_string());
    }

    if item.token_estimate > max_context_tokens
        || total_tokens.saturating_add(item.token_estimate) > max_context_tokens
    {
        return Some("tokenBudgetExceeded".to_string());
    }

    None
}

fn skipped_reason_for_response(items: &[RagPromptContextItem], used_count: u32) -> Option<String> {
    if used_count > 0 {
        return None;
    }

    if items.is_empty() {
        return Some("emptyContext".to_string());
    }

    let reasons = items
        .iter()
        .filter_map(|item| item.skipped_reason.as_deref())
        .collect::<HashSet<_>>();

    if reasons.len() == 1 {
        return reasons.into_iter().next().map(ToString::to_string);
    }

    Some("noUsableContext".to_string())
}

fn render_prompt_section(
    recognized_title: Option<&str>,
    recognized_text: Option<&str>,
    target_language: Option<&str>,
    platform: Option<&str>,
    items: &[RagPromptContextItem],
    total_tokens: u32,
    max_context_tokens: u32,
) -> String {
    let used_items = items
        .iter()
        .filter(|item| item.used_in_prompt)
        .collect::<Vec<_>>();
    let mut lines = vec![
        "Current screenshot recognition (primary source):".to_string(),
        format!(
            "- Title: {}",
            display_optional(recognized_title, "not provided")
        ),
        format!(
            "- Extracted text: {}",
            display_optional(recognized_text, "not provided")
        ),
        format!(
            "- Target language: {}",
            display_optional(target_language, "settings default")
        ),
        format!(
            "- Platform: {}",
            display_optional(platform, "settings default")
        ),
        String::new(),
        format!(
            "Local recalled context (reference only, {}/{} estimated tokens):",
            total_tokens, max_context_tokens
        ),
    ];

    for (index, item) in used_items.iter().enumerate() {
        lines.push(format!(
            "{}. Source: {} | Kind: {} | Score: {:.3}",
            index + 1,
            source_label(item.source_type),
            kind_label(item.kind),
            item.score
        ));
        lines.push(format!("   Title: {}", item.title));

        if let Some(language) = normalized_optional(item.language.as_deref().unwrap_or_default()) {
            lines.push(format!("   Language: {language}"));
        }
        if let Some(platform) = normalized_optional(item.platform.as_deref().unwrap_or_default()) {
            lines.push(format!("   Platform: {platform}"));
        }
        if !item.algorithm_tags.is_empty() {
            lines.push(format!(
                "   Algorithm tags: {}",
                item.algorithm_tags.join(", ")
            ));
        }
        if !item.tags.is_empty() {
            lines.push(format!("   Tags: {}", item.tags.join(", ")));
        }
        if item.source_type == RagPromptContextSourceType::LeetcodeIndex {
            lines.push(
                "   Scope: LeetCode lightweight metadata only; not a full problem statement."
                    .to_string(),
            );
        }
        lines.push(format!("   Summary: {}", indent_multiline(&item.snippet)));
    }

    lines.extend([
        String::new(),
        "Rules for using local recalled context:".to_string(),
        "1. Prioritize the current screenshot recognition and user-confirmed content."
            .to_string(),
        "2. Treat local recalled context as reference only; do not let it replace visible problem details.".to_string(),
        "3. If a recalled item conflicts with the screenshot, ignore the recalled item and solve the screenshot problem.".to_string(),
        "4. LeetCode lightweight index entries are metadata only and must not be treated as complete statements or examples.".to_string(),
    ]);

    lines.join("\n")
}

fn search_source_type_to_prompt_source_type(
    source_type: RagSearchSourceType,
) -> RagPromptContextSourceType {
    match source_type {
        RagSearchSourceType::LeetcodeIndex => RagPromptContextSourceType::LeetcodeIndex,
        RagSearchSourceType::UserImport => RagPromptContextSourceType::UserImport,
        RagSearchSourceType::History => RagPromptContextSourceType::History,
    }
}

fn search_chunk_kind_to_prompt_kind(kind: RagSearchChunkKind) -> RagPromptContextKind {
    match kind {
        RagSearchChunkKind::ProblemStatement => RagPromptContextKind::ProblemStatement,
        RagSearchChunkKind::Solution => RagPromptContextKind::Solution,
        RagSearchChunkKind::Note => RagPromptContextKind::Note,
        RagSearchChunkKind::Template => RagPromptContextKind::Template,
        RagSearchChunkKind::HistorySummary => RagPromptContextKind::HistorySummary,
        RagSearchChunkKind::Metadata => RagPromptContextKind::Metadata,
    }
}

fn source_priority(source_type: RagPromptContextSourceType) -> u8 {
    match source_type {
        RagPromptContextSourceType::UserImport => 0,
        RagPromptContextSourceType::History => 1,
        RagPromptContextSourceType::CodeTemplate => 2,
        RagPromptContextSourceType::SolutionMode => 3,
        RagPromptContextSourceType::LeetcodeIndex => 4,
    }
}

fn source_label(source_type: RagPromptContextSourceType) -> &'static str {
    match source_type {
        RagPromptContextSourceType::LeetcodeIndex => "LeetCode lightweight index",
        RagPromptContextSourceType::UserImport => "User import",
        RagPromptContextSourceType::History => "History",
        RagPromptContextSourceType::CodeTemplate => "Code template",
        RagPromptContextSourceType::SolutionMode => "Solution mode",
    }
}

fn kind_label(kind: RagPromptContextKind) -> &'static str {
    match kind {
        RagPromptContextKind::ProblemStatement => "problem statement",
        RagPromptContextKind::Solution => "solution",
        RagPromptContextKind::Note => "note",
        RagPromptContextKind::Template => "template",
        RagPromptContextKind::HistorySummary => "history summary",
        RagPromptContextKind::Metadata => "metadata",
        RagPromptContextKind::SolutionMode => "solution mode",
    }
}

fn estimate_item_tokens(candidate: &PromptCandidate) -> u32 {
    let text = format!(
        "{}\n{}\n{}\n{}\n{}",
        source_label(candidate.source_type),
        kind_label(candidate.kind),
        candidate.title,
        candidate.snippet,
        candidate.algorithm_tags.join(",")
    );
    estimate_tokens(&text)
}

fn estimate_tokens(text: &str) -> u32 {
    let mut ascii_chars = 0u32;
    let mut non_ascii_chars = 0u32;

    for ch in text.chars() {
        if ch.is_whitespace() {
            continue;
        }
        if ch.is_ascii() {
            ascii_chars += 1;
        } else {
            non_ascii_chars += 1;
        }
    }

    let ascii_tokens = ascii_chars.div_ceil(4);
    ascii_tokens.saturating_add(non_ascii_chars).max(1)
}

fn compress_text(value: &str, max_chars: usize) -> String {
    let normalized = value
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    truncate_chars(&normalized, max_chars)
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let mut truncated = value.chars().take(max_chars).collect::<String>();
    truncated.push_str("...");
    truncated
}

fn indent_multiline(value: &str) -> String {
    value.replace('\n', "\n   ")
}

fn display_optional(value: Option<&str>, fallback: &str) -> String {
    normalized_optional(value.unwrap_or_default()).unwrap_or_else(|| fallback.to_string())
}

fn normalized_optional(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
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

fn parse_language(value: Option<&str>) -> Option<LanguageId> {
    match normalize_key(value.unwrap_or_default()).as_str() {
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

fn parse_platform(value: Option<&str>) -> Option<PlatformFormat> {
    match normalize_key(value.unwrap_or_default()).as_str() {
        "acm" | "stdinstdout" => Some(PlatformFormat::Acm),
        "leetcode" | "leetcodefunction" | "leet" => Some(PlatformFormat::LeetCode),
        "generic" | "function" => Some(PlatformFormat::Generic),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag_retrieval::{RagSearchChunkKind, RagSearchSourceType};
    use crate::rag_templates::{RagCodeTemplate, RagSolutionMode};

    fn search_result(
        chunk_id: &str,
        title: &str,
        source_type: RagSearchSourceType,
        kind: RagSearchChunkKind,
        score: f32,
    ) -> RagSearchResult {
        RagSearchResult {
            chunk_id: chunk_id.to_string(),
            document_id: format!("doc_{chunk_id}"),
            source_type,
            source_id: Some(format!("source_{chunk_id}")),
            title: title.to_string(),
            snippet: "Use a hash table to record complements and return the matching pair."
                .to_string(),
            score,
            kind,
            problem_type: Some("hash-table".to_string()),
            language: Some("cpp20".to_string()),
            platform: Some("leetcode".to_string()),
            tags: vec!["array".to_string()],
            algorithm_tags: vec!["hash-table".to_string()],
            reason: "vector=0.700;algorithmTags".to_string(),
        }
    }

    fn template_context() -> RagTemplateSelectionResponse {
        RagTemplateSelectionResponse {
            target_language: "cpp20".to_string(),
            platform: Some("leetcode".to_string()),
            solution_modes: vec![RagSolutionMode {
                id: "hash-table-lookup".to_string(),
                title: "Hash Table Lookup".to_string(),
                description: "Use a map or set to turn repeated lookup into O(1) checks."
                    .to_string(),
                problem_type: Some("hash-table".to_string()),
                algorithm_tags: vec!["hash-table".to_string()],
                score: 0.84,
                reason: "algorithmTags".to_string(),
            }],
            templates: vec![RagCodeTemplate {
                id: "codetpl_ragimp_hash".to_string(),
                source_document_id: "ragimp_hash".to_string(),
                title: "C++ Hash Map Template".to_string(),
                language: Some("cpp20".to_string()),
                platform: Some("leetcode".to_string()),
                algorithm_tags: vec!["hash-table".to_string()],
                matched_mode_ids: vec!["hash-table-lookup".to_string()],
                snippet: "unordered_map<int, int> seen;".to_string(),
                template_text: "unordered_map<int, int> seen;\nfor (int i = 0; i < n; ++i) {}"
                    .to_string(),
                score: 0.91,
                reason: "language;algorithmTags;solutionMode".to_string(),
            }],
            skipped_reason: None,
        }
    }

    #[test]
    fn builds_prompt_section_that_preserves_screenshot_priority() {
        let response = build_rag_prompt_context(RagPromptContextRequest {
            recognized_title: Some("Two Sum".to_string()),
            recognized_text: Some("Given nums and target, return indices.".to_string()),
            target_language: Some("cpp20".to_string()),
            platform: Some("leetcode".to_string()),
            search_results: vec![search_result(
                "ragimpchunk_1_0",
                "Two Sum note",
                RagSearchSourceType::UserImport,
                RagSearchChunkKind::Solution,
                0.87,
            )],
            template_context: Some(template_context()),
            max_items: Some(3),
            max_context_tokens: Some(800),
            min_score: Some(0.25),
        });

        assert_eq!(response.skipped_reason, None);
        assert_eq!(response.used_item_count, 3);
        assert!(response
            .prompt_section
            .contains("Current screenshot recognition"));
        assert!(response.prompt_section.contains("Local recalled context"));
        assert!(response
            .prompt_section
            .contains("Prioritize the current screenshot recognition"));
        assert!(response.solution_prompt.contains("Two Sum"));
        assert!(response.solution_prompt.contains("Local recalled context"));
    }

    #[test]
    fn filters_low_score_and_limits_max_items() {
        let response = build_rag_prompt_context(RagPromptContextRequest {
            recognized_title: Some("Two Sum".to_string()),
            recognized_text: None,
            target_language: Some("cpp20".to_string()),
            platform: Some("leetcode".to_string()),
            search_results: vec![
                search_result(
                    "high_1",
                    "Two Sum high",
                    RagSearchSourceType::History,
                    RagSearchChunkKind::HistorySummary,
                    0.90,
                ),
                search_result(
                    "high_2",
                    "Two Sum second",
                    RagSearchSourceType::UserImport,
                    RagSearchChunkKind::Note,
                    0.80,
                ),
                search_result(
                    "low_1",
                    "Unrelated",
                    RagSearchSourceType::UserImport,
                    RagSearchChunkKind::Note,
                    0.10,
                ),
            ],
            template_context: None,
            max_items: Some(1),
            max_context_tokens: Some(800),
            min_score: Some(0.25),
        });

        assert_eq!(response.used_item_count, 1);
        assert_eq!(
            response
                .items
                .iter()
                .find(|item| item.id == "low_1")
                .and_then(|item| item.skipped_reason.as_deref()),
            Some("lowConfidence")
        );
        assert_eq!(
            response
                .items
                .iter()
                .find(|item| item.id == "high_2")
                .and_then(|item| item.skipped_reason.as_deref()),
            Some("maxItems")
        );
    }

    #[test]
    fn returns_original_prompt_when_context_is_unusable() {
        let response = build_rag_prompt_context(RagPromptContextRequest {
            recognized_title: Some("Circle Area".to_string()),
            recognized_text: Some("Compute area from radius.".to_string()),
            target_language: Some("python".to_string()),
            platform: Some("acm".to_string()),
            search_results: vec![search_result(
                "low_1",
                "Two Sum",
                RagSearchSourceType::UserImport,
                RagSearchChunkKind::Solution,
                0.10,
            )],
            template_context: None,
            max_items: Some(3),
            max_context_tokens: Some(800),
            min_score: Some(0.5),
        });

        assert_eq!(response.used_item_count, 0);
        assert_eq!(response.skipped_reason, Some("lowConfidence".to_string()));
        assert!(response.prompt_section.is_empty());
        assert!(response.solution_prompt.contains("Circle Area"));
        assert!(!response.solution_prompt.contains("Local recalled context"));
    }

    #[test]
    fn token_budget_can_skip_all_context_without_breaking_prompt() {
        let response = build_rag_prompt_context(RagPromptContextRequest {
            recognized_title: Some("Two Sum".to_string()),
            recognized_text: None,
            target_language: Some("cpp20".to_string()),
            platform: Some("leetcode".to_string()),
            search_results: vec![search_result(
                "expensive",
                "Two Sum",
                RagSearchSourceType::UserImport,
                RagSearchChunkKind::Solution,
                0.90,
            )],
            template_context: None,
            max_items: Some(3),
            max_context_tokens: Some(1),
            min_score: Some(0.25),
        });

        assert_eq!(response.used_item_count, 0);
        assert_eq!(
            response.skipped_reason,
            Some("tokenBudgetExceeded".to_string())
        );
        assert!(response.solution_prompt.contains("Two Sum"));
    }

    #[test]
    fn compresses_long_search_and_template_context_before_prompt_injection() {
        let mut long_search = search_result(
            "ragimpchunk_long_0",
            "Two Sum long note",
            RagSearchSourceType::UserImport,
            RagSearchChunkKind::Solution,
            0.95,
        );
        long_search.snippet = "Use a hash map and keep the complement index. ".repeat(40);
        let mut templates = template_context();
        templates.templates[0].template_text = "unordered_map<int, int> seen;\n".repeat(80);

        let response = build_rag_prompt_context(RagPromptContextRequest {
            recognized_title: Some("Two Sum".to_string()),
            recognized_text: Some("Given nums and target, return indices.".to_string()),
            target_language: Some("cpp20".to_string()),
            platform: Some("leetcode".to_string()),
            search_results: vec![long_search],
            template_context: Some(templates),
            max_items: Some(3),
            max_context_tokens: Some(1200),
            min_score: Some(0.25),
        });

        let long_note = response
            .items
            .iter()
            .find(|item| item.id == "ragimpchunk_long_0")
            .expect("long search item should exist");
        let template = response
            .items
            .iter()
            .find(|item| item.id == "codetpl_ragimp_hash")
            .expect("template item should exist");

        assert!(long_note.snippet.ends_with("..."));
        assert!(long_note.snippet.chars().count() <= MAX_SEARCH_SNIPPET_CHARS + 3);
        assert!(template.snippet.ends_with("..."));
        assert!(template.snippet.chars().count() <= MAX_TEMPLATE_SNIPPET_CHARS + 3);
        assert!(response.prompt_section.contains("Local recalled context"));
    }

    #[test]
    fn leetcode_metadata_gets_explicit_scope_warning() {
        let mut item = search_result(
            "problemidxchunk_leetcode_1",
            "Two Sum",
            RagSearchSourceType::LeetcodeIndex,
            RagSearchChunkKind::Metadata,
            0.88,
        );
        item.snippet = "Problem number: 1. Tags: Array, Hash Table.".to_string();

        let response = build_rag_prompt_context(RagPromptContextRequest {
            recognized_title: Some("Two Sum".to_string()),
            recognized_text: None,
            target_language: Some("cpp20".to_string()),
            platform: Some("leetcode".to_string()),
            search_results: vec![item],
            template_context: None,
            max_items: Some(3),
            max_context_tokens: Some(800),
            min_score: Some(0.25),
        });

        assert!(response
            .prompt_section
            .contains("LeetCode lightweight metadata only"));
        assert!(response
            .prompt_section
            .contains("must not be treated as complete statements"));
    }
}
