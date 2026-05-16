use serde::{Deserialize, Serialize};

/// Supported programming languages for direct algorithm solution output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LanguageId {
    Cpp17,
    Cpp20,
    Python,
    Java,
    JavaScript,
    TypeScript,
    Go,
    Rust,
}

impl LanguageId {
    /// Returns the human-readable label for the language.
    pub fn label(self) -> &'static str {
        match self {
            Self::Cpp17 => "C++17",
            Self::Cpp20 => "C++20",
            Self::Python => "Python",
            Self::Java => "Java",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::Go => "Go",
            Self::Rust => "Rust",
        }
    }

    /// Returns the default file extension for the language.
    pub fn file_extension(self) -> &'static str {
        match self {
            Self::Cpp17 | Self::Cpp20 => "cpp",
            Self::Python => "py",
            Self::Java => "java",
            Self::JavaScript => "js",
            Self::TypeScript => "ts",
            Self::Go => "go",
            Self::Rust => "rs",
        }
    }

    /// Returns the standard version or runtime hint for prompt constraints.
    pub fn standard_hint(self) -> &'static str {
        match self {
            Self::Cpp17 => "C++17",
            Self::Cpp20 => "C++20",
            Self::Python => "Python 3.10+",
            Self::Java => "Java 17",
            Self::JavaScript => "ES2022",
            Self::TypeScript => "TypeScript 5.x",
            Self::Go => "Go 1.22+",
            Self::Rust => "Rust 2021 edition",
        }
    }
}

impl Default for LanguageId {
    fn default() -> Self {
        Self::Cpp20
    }
}

/// Platform format variants that control how the solution code is structured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlatformFormat {
    /// ACM-style with standard input and output.
    Acm,
    /// LeetCode function signature style.
    LeetCode,
    /// Generic function pattern for open environments.
    Generic,
}

impl PlatformFormat {
    /// Returns the human-readable label for the platform format.
    pub fn label(self) -> &'static str {
        match self {
            Self::Acm => "ACM (stdin/stdout)",
            Self::LeetCode => "LeetCode (function)",
            Self::Generic => "Generic (function)",
        }
    }
}

impl Default for PlatformFormat {
    fn default() -> Self {
        Self::Acm
    }
}

/// Metadata for a supported language used when building prompts and rendering results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageMetadata {
    pub id: LanguageId,
    pub label: String,
    pub file_extension: String,
    pub standard_hint: String,
}

impl From<LanguageId> for LanguageMetadata {
    fn from(id: LanguageId) -> Self {
        Self {
            id,
            label: id.label().to_string(),
            file_extension: id.file_extension().to_string(),
            standard_hint: id.standard_hint().to_string(),
        }
    }
}

/// The fixed output sections that the AI is expected to produce.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OutputSection {
    TitleRecognition,
    Strategy,
    Code,
    Complexity,
    EdgeCases,
    Notes,
}

impl OutputSection {
    /// Returns the section header used in prompts and result parsing.
    pub fn header(self) -> &'static str {
        match self {
            Self::TitleRecognition => "题目识别",
            Self::Strategy => "解题策略",
            Self::Code => "完整代码",
            Self::Complexity => "复杂度",
            Self::EdgeCases => "边界用例",
            Self::Notes => "注意事项",
        }
    }
}

/// All supported language IDs in display order.
pub fn all_language_ids() -> Vec<LanguageId> {
    vec![
        LanguageId::Cpp20,
        LanguageId::Cpp17,
        LanguageId::Python,
        LanguageId::Java,
        LanguageId::JavaScript,
        LanguageId::TypeScript,
        LanguageId::Go,
        LanguageId::Rust,
    ]
}

/// All supported platform formats in display order.
pub fn all_platform_formats() -> Vec<PlatformFormat> {
    vec![
        PlatformFormat::Acm,
        PlatformFormat::LeetCode,
        PlatformFormat::Generic,
    ]
}

#[cfg(test)]
mod tests {
    use super::{all_language_ids, all_platform_formats, LanguageId, PlatformFormat};

    #[test]
    fn cpp20_is_the_default_language() {
        assert_eq!(LanguageId::default(), LanguageId::Cpp20);
    }

    #[test]
    fn language_labels_are_stable() {
        assert_eq!(LanguageId::Cpp20.label(), "C++20");
        assert_eq!(LanguageId::Cpp17.label(), "C++17");
        assert_eq!(LanguageId::Python.label(), "Python");
        assert_eq!(LanguageId::Java.label(), "Java");
        assert_eq!(LanguageId::JavaScript.label(), "JavaScript");
        assert_eq!(LanguageId::TypeScript.label(), "TypeScript");
        assert_eq!(LanguageId::Go.label(), "Go");
        assert_eq!(LanguageId::Rust.label(), "Rust");
    }

    #[test]
    fn file_extensions_match_language() {
        assert_eq!(LanguageId::Cpp20.file_extension(), "cpp");
        assert_eq!(LanguageId::Python.file_extension(), "py");
        assert_eq!(LanguageId::Java.file_extension(), "java");
        assert_eq!(LanguageId::Rust.file_extension(), "rs");
    }

    #[test]
    fn standard_hints_are_present() {
        for id in all_language_ids() {
            assert!(
                !id.standard_hint().is_empty(),
                "{id:?} missing standard hint"
            );
        }
    }

    #[test]
    fn all_languages_list_has_expected_order() {
        let ids = all_language_ids();
        assert_eq!(ids[0], LanguageId::Cpp20);
        assert_eq!(ids.len(), 8);
    }

    #[test]
    fn acm_is_the_default_platform_format() {
        assert_eq!(PlatformFormat::default(), PlatformFormat::Acm);
    }

    #[test]
    fn platform_format_labels_are_stable() {
        assert_eq!(PlatformFormat::Acm.label(), "ACM (stdin/stdout)");
        assert_eq!(PlatformFormat::LeetCode.label(), "LeetCode (function)");
        assert_eq!(PlatformFormat::Generic.label(), "Generic (function)");
    }

    #[test]
    fn language_metadata_derives_from_id() {
        let meta: super::LanguageMetadata = LanguageId::Cpp20.into();
        assert_eq!(meta.id, LanguageId::Cpp20);
        assert_eq!(meta.label, "C++20");
        assert_eq!(meta.file_extension, "cpp");
        assert_eq!(meta.standard_hint, "C++20");
    }
}
