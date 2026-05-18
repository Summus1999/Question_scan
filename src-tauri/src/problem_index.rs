/**
 * Question Scan 轻量题目索引模块（阶段 6）。
 *
 * 职责：提供只含元数据的 LeetCode 题目索引，供后续本地 RAG 检索使用。
 * 边界：不保存完整题面、输入输出样例、约束正文、题解正文或任何账号凭据。
 */
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

const LEETCODE_LIGHTWEIGHT_INDEX_JSON: &str =
    include_str!("../data/leetcode-lightweight-index.json");

/** 题目来源平台。 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProblemPlatform {
    Leetcode,
}

/** 题目难度。 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProblemDifficulty {
    Easy,
    Medium,
    Hard,
    Unknown,
}

/** 题型，用于后续按算法类别召回。 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProblemType {
    Array,
    HashTable,
    LinkedList,
    String,
    SlidingWindow,
    TwoPointers,
    DynamicProgramming,
    BinarySearch,
    Tree,
    Graph,
    Stack,
    Backtracking,
    Design,
    Math,
}

/** 单条轻量题目索引。 */
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProblemIndexEntry {
    pub id: String,
    pub platform: ProblemPlatform,
    pub problem_number: String,
    pub slug: String,
    pub title: String,
    pub difficulty: ProblemDifficulty,
    pub tags: Vec<String>,
    pub algorithm_tags: Vec<String>,
    pub problem_type: ProblemType,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProblemIndexFile {
    schema_version: u32,
    entries: Vec<ProblemIndexEntry>,
}

#[derive(Debug, Error)]
pub enum ProblemIndexError {
    #[error("Problem index JSON could not be parsed. {source}")]
    ParseFailed { source: serde_json::Error },
    #[error("Problem index schema version {version} is not supported.")]
    UnsupportedSchemaVersion { version: u32 },
    #[error("Problem index is empty.")]
    EmptyIndex,
    #[error("Problem index entry at position {index} is invalid. {reason}")]
    InvalidEntry { index: usize, reason: String },
    #[error("Problem index has a duplicate slug `{slug}`.")]
    DuplicateSlug { slug: String },
}

/** 加载内置 LeetCode 轻量题目索引。 */
pub fn load_leetcode_lightweight_index() -> Result<Vec<ProblemIndexEntry>, ProblemIndexError> {
    parse_problem_index(LEETCODE_LIGHTWEIGHT_INDEX_JSON)
}

/** 解析并校验题目索引。 */
pub fn parse_problem_index(input: &str) -> Result<Vec<ProblemIndexEntry>, ProblemIndexError> {
    let file: ProblemIndexFile =
        serde_json::from_str(input).map_err(|source| ProblemIndexError::ParseFailed { source })?;

    if file.schema_version != 1 {
        return Err(ProblemIndexError::UnsupportedSchemaVersion {
            version: file.schema_version,
        });
    }

    if file.entries.is_empty() {
        return Err(ProblemIndexError::EmptyIndex);
    }

    validate_entries(&file.entries)?;
    Ok(file.entries)
}

fn validate_entries(entries: &[ProblemIndexEntry]) -> Result<(), ProblemIndexError> {
    let mut slugs = HashSet::new();

    for (index, entry) in entries.iter().enumerate() {
        validate_entry(index, entry)?;

        let slug_key = format!("{:?}:{}", entry.platform, entry.slug);
        if !slugs.insert(slug_key) {
            return Err(ProblemIndexError::DuplicateSlug {
                slug: entry.slug.clone(),
            });
        }
    }

    Ok(())
}

fn validate_entry(index: usize, entry: &ProblemIndexEntry) -> Result<(), ProblemIndexError> {
    let required_fields = [
        ("id", entry.id.as_str()),
        ("problemNumber", entry.problem_number.as_str()),
        ("slug", entry.slug.as_str()),
        ("title", entry.title.as_str()),
        ("source", entry.source.as_str()),
    ];

    for (field, value) in required_fields {
        if value.trim().is_empty() {
            return Err(ProblemIndexError::InvalidEntry {
                index,
                reason: format!("{field} must not be empty"),
            });
        }
    }

    if entry.tags.is_empty() {
        return Err(ProblemIndexError::InvalidEntry {
            index,
            reason: "tags must not be empty".to_string(),
        });
    }

    if entry.algorithm_tags.is_empty() {
        return Err(ProblemIndexError::InvalidEntry {
            index,
            reason: "algorithmTags must not be empty".to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn loads_embedded_leetcode_lightweight_index() {
        let entries = load_leetcode_lightweight_index().expect("embedded index should parse");

        assert!(entries.len() >= 10);
        assert!(entries.iter().any(|entry| entry.slug == "two-sum"));
        assert!(entries.iter().any(|entry| {
            entry.problem_type == ProblemType::Graph && entry.slug == "number-of-islands"
        }));
    }

    #[test]
    fn embedded_index_exposes_only_lightweight_metadata_fields() {
        let entries = load_leetcode_lightweight_index().expect("embedded index should parse");
        let allowed_fields = [
            "id",
            "platform",
            "problemNumber",
            "slug",
            "title",
            "difficulty",
            "tags",
            "algorithmTags",
            "problemType",
            "source",
        ];

        for entry in entries {
            let value = serde_json::to_value(entry).expect("entry should serialize");
            let object = value.as_object().expect("entry should be an object");
            for key in object.keys() {
                assert!(
                    allowed_fields.contains(&key.as_str()),
                    "unexpected lightweight index field: {key}"
                );
            }
        }
    }

    #[test]
    fn rejects_unknown_full_problem_statement_fields() {
        let input = r#"{
          "schemaVersion": 1,
          "entries": [
            {
              "id": "leetcode_1",
              "platform": "leetcode",
              "problemNumber": "1",
              "slug": "two-sum",
              "title": "Two Sum",
              "difficulty": "easy",
              "tags": ["Array", "Hash Table"],
              "algorithmTags": ["array", "hash-table"],
              "problemType": "hash-table",
              "source": "leetcode-lightweight-seed-v1",
              "problemStatement": "This full statement must not be accepted."
            }
          ]
        }"#;

        let error = parse_problem_index(input).expect_err("unknown full text field should fail");

        assert!(matches!(error, ProblemIndexError::ParseFailed { .. }));
    }

    #[test]
    fn rejects_duplicate_platform_slug_pairs() {
        let input = r#"{
          "schemaVersion": 1,
          "entries": [
            {
              "id": "leetcode_1",
              "platform": "leetcode",
              "problemNumber": "1",
              "slug": "two-sum",
              "title": "Two Sum",
              "difficulty": "easy",
              "tags": ["Array", "Hash Table"],
              "algorithmTags": ["array", "hash-table"],
              "problemType": "hash-table",
              "source": "leetcode-lightweight-seed-v1"
            },
            {
              "id": "leetcode_1_copy",
              "platform": "leetcode",
              "problemNumber": "1",
              "slug": "two-sum",
              "title": "Two Sum",
              "difficulty": "easy",
              "tags": ["Array", "Hash Table"],
              "algorithmTags": ["array", "hash-table"],
              "problemType": "hash-table",
              "source": "leetcode-lightweight-seed-v1"
            }
          ]
        }"#;

        let error = parse_problem_index(input).expect_err("duplicate slug should fail");

        assert!(matches!(error, ProblemIndexError::DuplicateSlug { .. }));
    }

    #[test]
    fn rejects_unsupported_schema_version() {
        let input = r#"{
          "schemaVersion": 99,
          "entries": [
            {
              "id": "leetcode_1",
              "platform": "leetcode",
              "problemNumber": "1",
              "slug": "two-sum",
              "title": "Two Sum",
              "difficulty": "easy",
              "tags": ["Array", "Hash Table"],
              "algorithmTags": ["array", "hash-table"],
              "problemType": "hash-table",
              "source": "leetcode-lightweight-seed-v1"
            }
          ]
        }"#;

        let error = parse_problem_index(input).expect_err("unsupported schema should fail");

        assert!(matches!(
            error,
            ProblemIndexError::UnsupportedSchemaVersion { version: 99 }
        ));
    }

    #[test]
    fn rejects_empty_required_metadata() {
        let input = r#"{
          "schemaVersion": 1,
          "entries": [
            {
              "id": "leetcode_1",
              "platform": "leetcode",
              "problemNumber": "",
              "slug": "two-sum",
              "title": "Two Sum",
              "difficulty": "easy",
              "tags": ["Array", "Hash Table"],
              "algorithmTags": ["array", "hash-table"],
              "problemType": "hash-table",
              "source": "leetcode-lightweight-seed-v1"
            }
          ]
        }"#;

        let error = parse_problem_index(input).expect_err("empty problem number should fail");

        assert!(matches!(error, ProblemIndexError::InvalidEntry { .. }));
    }

    #[test]
    fn rejects_entries_without_tags_or_algorithm_tags() {
        let missing_tags = r#"{
          "schemaVersion": 1,
          "entries": [
            {
              "id": "leetcode_1",
              "platform": "leetcode",
              "problemNumber": "1",
              "slug": "two-sum",
              "title": "Two Sum",
              "difficulty": "easy",
              "tags": [],
              "algorithmTags": ["array", "hash-table"],
              "problemType": "hash-table",
              "source": "leetcode-lightweight-seed-v1"
            }
          ]
        }"#;
        let missing_algorithm_tags = r#"{
          "schemaVersion": 1,
          "entries": [
            {
              "id": "leetcode_1",
              "platform": "leetcode",
              "problemNumber": "1",
              "slug": "two-sum",
              "title": "Two Sum",
              "difficulty": "easy",
              "tags": ["Array", "Hash Table"],
              "algorithmTags": [],
              "problemType": "hash-table",
              "source": "leetcode-lightweight-seed-v1"
            }
          ]
        }"#;

        let tag_error = parse_problem_index(missing_tags).expect_err("missing tags should fail");
        let algorithm_error = parse_problem_index(missing_algorithm_tags)
            .expect_err("missing algorithm tags should fail");

        assert!(matches!(tag_error, ProblemIndexError::InvalidEntry { .. }));
        assert!(matches!(
            algorithm_error,
            ProblemIndexError::InvalidEntry { .. }
        ));
    }

    #[test]
    fn serializes_platform_and_problem_type_for_frontend_contract() {
        let entry = ProblemIndexEntry {
            id: "leetcode_200".to_string(),
            platform: ProblemPlatform::Leetcode,
            problem_number: "200".to_string(),
            slug: "number-of-islands".to_string(),
            title: "Number of Islands".to_string(),
            difficulty: ProblemDifficulty::Medium,
            tags: vec!["Depth-First Search".to_string(), "Graph".to_string()],
            algorithm_tags: vec!["dfs".to_string(), "graph".to_string()],
            problem_type: ProblemType::Graph,
            source: "leetcode-lightweight-seed-v1".to_string(),
        };

        let value: Value = serde_json::to_value(entry).expect("entry should serialize");

        assert_eq!(value["platform"], "leetcode");
        assert_eq!(value["problemNumber"], "200");
        assert_eq!(value["problemType"], "graph");
        assert_eq!(value["difficulty"], "medium");
    }
}
