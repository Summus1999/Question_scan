/**
 * Question Scan RAG 题型和代码模板选择模块（阶段 6）。
 *
 * 职责：根据相似题召回结果、算法标签和目标语言选择可用解题模式与用户导入的代码模板。
 * 范围：只做本地排序和选择，不注入 prompt，不访问网络，不自动提交或填充第三方页面。
 */
use crate::rag_imports::{RagImportKind, RagImportedDocument};
use crate::rag_retrieval::RagSearchResult;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

const DEFAULT_MAX_SOLUTION_MODES: u32 = 3;
const MAX_SOLUTION_MODES: u32 = 8;
const DEFAULT_MAX_TEMPLATES: u32 = 3;
const MAX_TEMPLATES: u32 = 10;
const MAX_TEMPLATE_SNIPPET_CHARS: usize = 240;
const MIN_MODE_SCORE: f32 = 0.12;
const MIN_TEMPLATE_SCORE: f32 = 0.20;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RagTemplateSelectionRequest {
    pub target_language: Option<String>,
    pub platform: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub algorithm_tags: Vec<String>,
    #[serde(default)]
    pub similar_items: Vec<RagSearchResult>,
    pub max_solution_modes: Option<u32>,
    pub max_templates: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RagSolutionMode {
    pub id: String,
    pub title: String,
    pub description: String,
    pub problem_type: Option<String>,
    pub algorithm_tags: Vec<String>,
    pub score: f32,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RagCodeTemplate {
    pub id: String,
    pub source_document_id: String,
    pub title: String,
    pub language: Option<String>,
    pub platform: Option<String>,
    pub algorithm_tags: Vec<String>,
    pub matched_mode_ids: Vec<String>,
    pub snippet: String,
    pub template_text: String,
    pub score: f32,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RagTemplateSelectionResponse {
    pub target_language: String,
    pub platform: Option<String>,
    pub solution_modes: Vec<RagSolutionMode>,
    pub templates: Vec<RagCodeTemplate>,
    pub skipped_reason: Option<String>,
}

#[derive(Debug, Clone)]
struct ModeDefinition {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    problem_type: Option<&'static str>,
    algorithm_tags: &'static [&'static str],
}

#[derive(Debug, Default)]
struct SignalProfile {
    tags: HashMap<String, f32>,
    algorithm_tags: HashMap<String, f32>,
    problem_types: HashMap<String, f32>,
}

impl SignalProfile {
    fn has_signal(&self) -> bool {
        !self.tags.is_empty() || !self.algorithm_tags.is_empty() || !self.problem_types.is_empty()
    }
}

pub fn select_rag_template_context(
    request: RagTemplateSelectionRequest,
    imports: &[RagImportedDocument],
    fallback_language: &str,
    fallback_platform: Option<&str>,
) -> RagTemplateSelectionResponse {
    let target_language = normalized_optional(request.target_language.as_deref())
        .or_else(|| normalized_optional(Some(fallback_language)))
        .unwrap_or_else(|| "cpp20".to_string());
    let platform = normalized_optional(request.platform.as_deref())
        .or_else(|| fallback_platform.and_then(|p| normalized_optional(Some(p))));
    let signals = build_signal_profile(&request);
    let max_modes = request
        .max_solution_modes
        .unwrap_or(DEFAULT_MAX_SOLUTION_MODES)
        .clamp(1, MAX_SOLUTION_MODES);
    let max_templates = request
        .max_templates
        .unwrap_or(DEFAULT_MAX_TEMPLATES)
        .clamp(1, MAX_TEMPLATES);

    if !signals.has_signal() {
        return RagTemplateSelectionResponse {
            target_language,
            platform,
            solution_modes: Vec::new(),
            templates: Vec::new(),
            skipped_reason: Some("noRecommendationSignals".to_string()),
        };
    }

    let solution_modes = select_solution_modes(&signals, max_modes);
    let templates = select_code_templates(
        imports,
        &signals,
        &solution_modes,
        &target_language,
        platform.as_deref(),
        max_templates,
    );
    let skipped_reason = if solution_modes.is_empty() && templates.is_empty() {
        Some("noTemplateMatches".to_string())
    } else {
        None
    };

    RagTemplateSelectionResponse {
        target_language,
        platform,
        solution_modes,
        templates,
        skipped_reason,
    }
}

fn build_signal_profile(request: &RagTemplateSelectionRequest) -> SignalProfile {
    let mut profile = SignalProfile::default();
    add_weighted_list(&mut profile.tags, &request.tags, 0.65);
    add_weighted_list(&mut profile.algorithm_tags, &request.algorithm_tags, 1.0);

    for item in &request.similar_items {
        let score = item.score.clamp(0.0, 1.0).max(0.05);
        add_weighted_list(&mut profile.tags, &item.tags, score * 0.45);
        add_weighted_list(&mut profile.algorithm_tags, &item.algorithm_tags, score);

        if let Some(problem_type) = normalized_optional(item.problem_type.as_deref()) {
            add_weight(&mut profile.problem_types, &problem_type, score);
        }
    }

    profile
}

fn select_solution_modes(signals: &SignalProfile, max_modes: u32) -> Vec<RagSolutionMode> {
    let mut modes = mode_definitions()
        .iter()
        .filter_map(|definition| {
            let (score, reasons) = score_mode(definition, signals);
            if score < MIN_MODE_SCORE {
                return None;
            }

            Some(RagSolutionMode {
                id: definition.id.to_string(),
                title: definition.title.to_string(),
                description: definition.description.to_string(),
                problem_type: definition.problem_type.map(ToString::to_string),
                algorithm_tags: definition
                    .algorithm_tags
                    .iter()
                    .map(|tag| tag.to_string())
                    .collect(),
                score,
                reason: reasons.join(";"),
            })
        })
        .collect::<Vec<_>>();

    modes.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.id.cmp(&right.id))
    });
    modes.truncate(max_modes as usize);
    modes
}

fn score_mode(definition: &ModeDefinition, signals: &SignalProfile) -> (f32, Vec<String>) {
    let mut score = 0.0;
    let mut reasons = Vec::new();

    if let Some(problem_type) = definition.problem_type {
        if let Some(weight) = signals.problem_types.get(&normalize_key(problem_type)) {
            score += 0.38 * weight.min(1.0);
            reasons.push(format!("problemType:{problem_type}"));
        }
    }

    let algorithm_score = matching_weight(definition.algorithm_tags, &signals.algorithm_tags);
    if algorithm_score > 0.0 {
        score += 0.54 * algorithm_score;
        reasons.push("algorithmTags".to_string());
    }

    let tag_score = matching_weight(definition.algorithm_tags, &signals.tags);
    if tag_score > 0.0 {
        score += 0.18 * tag_score;
        reasons.push("tags".to_string());
    }

    (score.clamp(0.0, 1.0), reasons)
}

fn select_code_templates(
    imports: &[RagImportedDocument],
    signals: &SignalProfile,
    modes: &[RagSolutionMode],
    target_language: &str,
    target_platform: Option<&str>,
    max_templates: u32,
) -> Vec<RagCodeTemplate> {
    let mut templates = imports
        .iter()
        .filter(|document| {
            document.kind == RagImportKind::Template && document.deleted_at.is_none()
        })
        .filter_map(|document| {
            let (score, reasons, matched_mode_ids) =
                score_template(document, signals, modes, target_language, target_platform);
            if score < MIN_TEMPLATE_SCORE {
                return None;
            }

            Some(RagCodeTemplate {
                id: format!("codetpl_{}", document.id),
                source_document_id: document.id.clone(),
                title: document.title.clone(),
                language: document.language.clone(),
                platform: document.platform.clone(),
                algorithm_tags: clean_list(&document.algorithm_tags),
                matched_mode_ids,
                snippet: snippet(&document.text),
                template_text: document.text.trim().to_string(),
                score,
                reason: reasons.join(";"),
            })
        })
        .collect::<Vec<_>>();

    templates.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.title.cmp(&right.title))
            .then_with(|| left.id.cmp(&right.id))
    });
    templates.truncate(max_templates as usize);
    templates
}

fn score_template(
    document: &RagImportedDocument,
    signals: &SignalProfile,
    modes: &[RagSolutionMode],
    target_language: &str,
    target_platform: Option<&str>,
) -> (f32, Vec<String>, Vec<String>) {
    let mut score = 0.0;
    let mut reasons = Vec::new();
    let template_language = document.language.as_deref().and_then(|language| {
        normalized_optional(Some(language)).map(|value| normalize_key(&value))
    });
    let target_language_key = normalize_key(target_language);

    match template_language.as_deref() {
        Some(language) if language == target_language_key => {
            score += 0.48;
            reasons.push("language".to_string());
        }
        None => {
            score += 0.24;
            reasons.push("genericLanguage".to_string());
        }
        Some(_) => return (0.0, Vec::new(), Vec::new()),
    }

    if let (Some(target_platform), Some(template_platform)) = (
        target_platform.and_then(|value| normalized_optional(Some(value))),
        document.platform.as_deref(),
    ) {
        if normalize_key(&target_platform) == normalize_key(template_platform) {
            score += 0.08;
            reasons.push("platform".to_string());
        }
    } else if document.platform.is_none() {
        score += 0.03;
        reasons.push("genericPlatform".to_string());
    }

    let algorithm_tags = clean_list(&document.algorithm_tags);
    let algorithm_score = matching_weight_from_strings(&algorithm_tags, &signals.algorithm_tags);
    if algorithm_score > 0.0 {
        score += 0.34 * algorithm_score;
        reasons.push("algorithmTags".to_string());
    }

    let tag_score = matching_weight_from_strings(&algorithm_tags, &signals.tags);
    if tag_score > 0.0 {
        score += 0.10 * tag_score;
        reasons.push("tags".to_string());
    }

    let matched_mode_ids = modes
        .iter()
        .filter(|mode| list_overlap_score(&algorithm_tags, &mode.algorithm_tags) > 0.0)
        .map(|mode| mode.id.clone())
        .collect::<Vec<_>>();

    if !matched_mode_ids.is_empty() {
        score += 0.08;
        reasons.push("solutionMode".to_string());
    }

    (score.clamp(0.0, 1.0), reasons, matched_mode_ids)
}

fn mode_definitions() -> &'static [ModeDefinition] {
    &[
        ModeDefinition {
            id: "hash-table-lookup",
            title: "Hash Table Lookup",
            description: "Use a map or set to turn repeated lookup into near O(1) checks.",
            problem_type: Some("hash-table"),
            algorithm_tags: &["hash-table", "map", "set"],
        },
        ModeDefinition {
            id: "two-pointers",
            title: "Two Pointers",
            description:
                "Move two indexes through a sorted array, linked list, or paired sequence.",
            problem_type: Some("two-pointers"),
            algorithm_tags: &["two-pointers", "sorting"],
        },
        ModeDefinition {
            id: "sliding-window",
            title: "Sliding Window",
            description: "Maintain a valid interval while expanding and shrinking its boundaries.",
            problem_type: Some("sliding-window"),
            algorithm_tags: &["sliding-window", "two-pointers"],
        },
        ModeDefinition {
            id: "dynamic-programming",
            title: "Dynamic Programming",
            description:
                "Define states and transitions, then compute overlapping subproblems once.",
            problem_type: Some("dynamic-programming"),
            algorithm_tags: &["dynamic-programming", "dp"],
        },
        ModeDefinition {
            id: "binary-search",
            title: "Binary Search",
            description: "Search an ordered answer space or sorted collection by monotonic checks.",
            problem_type: Some("binary-search"),
            algorithm_tags: &["binary-search"],
        },
        ModeDefinition {
            id: "graph-search",
            title: "Graph Search",
            description: "Use BFS, DFS, or union-find to traverse connectivity and reachability.",
            problem_type: Some("graph"),
            algorithm_tags: &["graph", "dfs", "bfs", "union-find"],
        },
        ModeDefinition {
            id: "tree-traversal",
            title: "Tree Traversal",
            description: "Use recursive or iterative traversal to aggregate subtree information.",
            problem_type: Some("tree"),
            algorithm_tags: &["tree", "dfs", "bfs", "binary-tree"],
        },
        ModeDefinition {
            id: "monotonic-stack",
            title: "Monotonic Stack",
            description: "Keep a monotonic stack to resolve nearest greater or smaller elements.",
            problem_type: Some("stack"),
            algorithm_tags: &["stack", "monotonic-stack"],
        },
        ModeDefinition {
            id: "backtracking",
            title: "Backtracking",
            description: "Explore choices depth-first and prune invalid partial states early.",
            problem_type: Some("backtracking"),
            algorithm_tags: &["backtracking", "dfs"],
        },
        ModeDefinition {
            id: "math",
            title: "Math",
            description: "Use numeric properties, combinatorics, modular arithmetic, or formulas.",
            problem_type: Some("math"),
            algorithm_tags: &["math", "number-theory", "combinatorics"],
        },
    ]
}

fn add_weighted_list(target: &mut HashMap<String, f32>, values: &[String], weight: f32) {
    for value in values {
        add_weight(target, value, weight);
    }
}

fn add_weight(target: &mut HashMap<String, f32>, value: &str, weight: f32) {
    let key = normalize_key(value);
    if key.is_empty() {
        return;
    }

    target
        .entry(key)
        .and_modify(|current| *current = (*current).max(weight))
        .or_insert(weight);
}

fn matching_weight(tags: &[&str], weights: &HashMap<String, f32>) -> f32 {
    if tags.is_empty() {
        return 0.0;
    }

    tags.iter()
        .filter_map(|tag| weights.get(&normalize_key(tag)).copied())
        .fold(0.0, f32::max)
        .clamp(0.0, 1.0)
}

fn matching_weight_from_strings(tags: &[String], weights: &HashMap<String, f32>) -> f32 {
    if tags.is_empty() {
        return 0.0;
    }

    tags.iter()
        .filter_map(|tag| weights.get(&normalize_key(tag)).copied())
        .fold(0.0, f32::max)
        .clamp(0.0, 1.0)
}

fn list_overlap_score(left: &[String], right: &[String]) -> f32 {
    let left_keys = left
        .iter()
        .map(|value| normalize_key(value))
        .collect::<HashSet<_>>();
    let right_keys = right
        .iter()
        .map(|value| normalize_key(value))
        .collect::<HashSet<_>>();
    if left_keys.is_empty() || right_keys.is_empty() {
        return 0.0;
    }

    let intersection = left_keys.intersection(&right_keys).count() as f32;
    intersection / left_keys.len().max(right_keys.len()) as f32
}

fn clean_list(items: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut cleaned = Vec::new();

    for item in items {
        let value = item.trim();
        if value.is_empty() {
            continue;
        }

        if seen.insert(normalize_key(value)) {
            cleaned.push(value.to_string());
        }
    }

    cleaned
}

fn snippet(text: &str) -> String {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    truncate_chars(&normalized, MAX_TEMPLATE_SNIPPET_CHARS)
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let mut truncated = value.chars().take(max_chars).collect::<String>();
    truncated.push_str("...");
    truncated
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag_imports::{RagImportFormat, RagImportKind};
    use crate::rag_retrieval::{RagSearchChunkKind, RagSearchSourceType};

    fn template_document(
        id: &str,
        title: &str,
        language: Option<&str>,
        algorithm_tags: &[&str],
        text: &str,
    ) -> RagImportedDocument {
        RagImportedDocument {
            id: id.to_string(),
            source_name: "templates.md".to_string(),
            source_uri: None,
            source_format: RagImportFormat::Markdown,
            kind: RagImportKind::Template,
            title: title.to_string(),
            text: text.to_string(),
            language: language.map(ToString::to_string),
            platform: Some("leetcode".to_string()),
            tags: Vec::new(),
            algorithm_tags: algorithm_tags.iter().map(|tag| tag.to_string()).collect(),
            imported_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            deleted_at: None,
        }
    }

    fn graph_search_result() -> RagSearchResult {
        RagSearchResult {
            chunk_id: "problemidxchunk_leetcode_200".to_string(),
            document_id: "leetcode_200".to_string(),
            source_type: RagSearchSourceType::LeetcodeIndex,
            source_id: Some("number-of-islands".to_string()),
            title: "Number of Islands".to_string(),
            snippet: "LeetCode metadata only.".to_string(),
            score: 0.82,
            kind: RagSearchChunkKind::Metadata,
            problem_type: Some("graph".to_string()),
            language: None,
            platform: Some("leetcode".to_string()),
            tags: vec!["Graph".to_string()],
            algorithm_tags: vec!["dfs".to_string(), "graph".to_string()],
            reason: "vector=0.700;algorithmTags".to_string(),
        }
    }

    #[test]
    fn similar_problem_type_selects_solution_mode() {
        let request = RagTemplateSelectionRequest {
            target_language: Some("cpp20".to_string()),
            platform: Some("leetcode".to_string()),
            tags: Vec::new(),
            algorithm_tags: Vec::new(),
            similar_items: vec![graph_search_result()],
            max_solution_modes: Some(2),
            max_templates: Some(2),
        };

        let response = select_rag_template_context(request, &[], "python", Some("generic"));

        assert_eq!(response.target_language, "cpp20");
        assert_eq!(response.solution_modes[0].id, "graph-search");
        assert!(response.solution_modes[0]
            .reason
            .contains("problemType:graph"));
        assert_eq!(response.skipped_reason, None);
    }

    #[test]
    fn target_language_template_is_preferred_over_other_languages() {
        let imports = vec![
            template_document(
                "ragimp_cpp_graph",
                "C++ DFS Template",
                Some("cpp20"),
                &["graph", "dfs"],
                "class Solution { void dfs(int r, int c) {} };",
            ),
            template_document(
                "ragimp_py_graph",
                "Python DFS Template",
                Some("python"),
                &["graph", "dfs"],
                "def dfs(r, c): pass",
            ),
        ];
        let request = RagTemplateSelectionRequest {
            target_language: None,
            platform: Some("leetcode".to_string()),
            tags: Vec::new(),
            algorithm_tags: vec!["dfs".to_string(), "graph".to_string()],
            similar_items: vec![graph_search_result()],
            max_solution_modes: Some(3),
            max_templates: Some(3),
        };

        let response = select_rag_template_context(request, &imports, "cpp20", Some("leetcode"));

        assert_eq!(response.target_language, "cpp20");
        assert_eq!(response.templates.len(), 1);
        assert_eq!(response.templates[0].title, "C++ DFS Template");
        assert_eq!(response.templates[0].language, Some("cpp20".to_string()));
        assert!(response.templates[0]
            .matched_mode_ids
            .contains(&"graph-search".to_string()));
    }

    #[test]
    fn generic_language_template_is_used_as_fallback() {
        let imports = vec![template_document(
            "ragimp_generic_dp",
            "Generic DP Checklist",
            None,
            &["dynamic-programming"],
            "Define state, transition, base case, and iteration order.",
        )];
        let request = RagTemplateSelectionRequest {
            target_language: Some("rust".to_string()),
            platform: None,
            tags: Vec::new(),
            algorithm_tags: vec!["dynamic-programming".to_string()],
            similar_items: Vec::new(),
            max_solution_modes: Some(2),
            max_templates: Some(2),
        };

        let response = select_rag_template_context(request, &imports, "cpp20", Some("leetcode"));

        assert_eq!(response.templates.len(), 1);
        assert_eq!(response.templates[0].language, None);
        assert!(response.templates[0].reason.contains("genericLanguage"));
    }

    #[test]
    fn empty_signals_skip_template_selection() {
        let imports = vec![template_document(
            "ragimp_generic",
            "Generic Template",
            None,
            &[],
            "Read input, solve, print output.",
        )];
        let request = RagTemplateSelectionRequest {
            target_language: None,
            platform: None,
            tags: Vec::new(),
            algorithm_tags: Vec::new(),
            similar_items: Vec::new(),
            max_solution_modes: None,
            max_templates: None,
        };

        let response = select_rag_template_context(request, &imports, "cpp20", Some("acm"));

        assert!(response.solution_modes.is_empty());
        assert!(response.templates.is_empty());
        assert_eq!(
            response.skipped_reason,
            Some("noRecommendationSignals".to_string())
        );
    }
}
