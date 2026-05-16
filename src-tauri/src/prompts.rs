use crate::language::{LanguageId, PlatformFormat};

/// Builds the system instruction that guides the AI to produce structured algorithm solutions.
pub fn build_solution_prompt(
    language: LanguageId,
    platform: PlatformFormat,
    recognized_title: Option<&str>,
    recognized_text: Option<&str>,
) -> String {
    let lang_hint = language.standard_hint();
    let platform_hint = platform.label();

    let title_section = recognized_title
        .filter(|t| !t.is_empty())
        .map(|t| format!("The user identified the problem title as: {t}"))
        .unwrap_or_default();

    let text_section = recognized_text
        .filter(|t| !t.is_empty())
        .map(|t| format!("The user also extracted the following text from the image: {t}"))
        .unwrap_or_default();

    format!(
        r#"You are an algorithm problem solver. Analyze the image and produce a direct solution in the requested language and platform format.

Language: {lang_hint}
Platform format: {platform_hint}
{title_section}
{text_section}

Your response MUST follow this exact structure with these section headers:

题目识别:
- State the problem title and a concise summary of what the problem asks.

解题策略:
- Explain the core algorithm or data structure choice in 2-4 sentences.
- Do not write a tutorial; keep it brief and focused.

完整代码:
- Provide the complete, compilable solution.
- For {lang_hint}, respect the standard version and import conventions.
- For {platform_hint}, use the correct input/output pattern or function signature.

复杂度:
- State the time complexity and space complexity with brief justification.

边界用例:
- List 2-3 edge cases that could break an incorrect solution.

注意事项:
- Mention any language-specific pitfalls, overflow risks, or input parsing tricks.

Rules:
1. Do NOT include markdown code fences (```) inside the "完整代码" section header line; place the code directly under the header.
2. The code must be ready to run in {lang_hint} for {platform_hint}.
3. If the image is unclear, do your best to infer the problem from any visible text."#
    )
}

/// Language-specific constraints injected into the prompt to improve code quality.
pub fn build_language_constraints(language: LanguageId) -> &'static str {
    match language {
        LanguageId::Cpp17 => {
            "Use C++17. Include <bits/stdc++.h> or standard headers. Use ios::sync_with_stdio(false); cin.tie(nullptr); for fast IO."
        }
        LanguageId::Cpp20 => {
            "Use C++20. Include <bits/stdc++.h> or standard headers. Use ios::sync_with_stdio(false); cin.tie(nullptr); for fast IO. You may use concepts and ranges where helpful."
        }
        LanguageId::Python => {
            "Use Python 3.10+. Use sys.stdin.read() or input() for reading. Avoid external libraries beyond the standard library."
        }
        LanguageId::Java => {
            "Use Java 17. Use BufferedReader and PrintWriter for fast IO. Define a public class named Main."
        }
        LanguageId::JavaScript => {
            "Use ES2022. Use readline or process.stdin for input. Keep the solution in a single runnable script."
        }
        LanguageId::TypeScript => {
            "Use TypeScript 5.x. Define types where they help clarity. Use standard Node.js input methods."
        }
        LanguageId::Go => {
            "Use Go 1.22+. Use bufio.NewReader for input. Define package main with func main()."
        }
        LanguageId::Rust => {
            "Use Rust 2021 edition. Use std::io::stdin() with BufRead for input. Define fn main()."
        }
    }
}

/// Platform-specific signature hints injected into the prompt.
pub fn build_platform_signature_hint(platform: PlatformFormat) -> &'static str {
    match platform {
        PlatformFormat::Acm => {
            "The solution must read from standard input and write to standard output. Include a main function or equivalent entry point."
        }
        PlatformFormat::LeetCode => {
            "The solution must be a class or function that matches the LeetCode signature. Do NOT include a main function or I/O handling."
        }
        PlatformFormat::Generic => {
            "The solution should be a clean function or class that solves the problem. Include a small main or test harness only if it helps clarity."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{build_language_constraints, build_platform_signature_hint, build_solution_prompt};
    use crate::language::{LanguageId, PlatformFormat};

    #[test]
    fn prompt_includes_language_and_platform() {
        let prompt = build_solution_prompt(LanguageId::Cpp20, PlatformFormat::Acm, None, None);
        assert!(prompt.contains("C++20"));
        assert!(prompt.contains("ACM"));
    }

    #[test]
    fn prompt_includes_recognized_title_when_present() {
        let prompt = build_solution_prompt(
            LanguageId::Python,
            PlatformFormat::LeetCode,
            Some("Two Sum"),
            None,
        );
        assert!(prompt.contains("Two Sum"));
        assert!(prompt.contains("problem title"));
    }

    #[test]
    fn prompt_omits_empty_title_and_text() {
        let prompt = build_solution_prompt(
            LanguageId::Java,
            PlatformFormat::Generic,
            Some(""),
            Some(""),
        );
        assert!(!prompt.contains("identified the problem title"));
        assert!(!prompt.contains("extracted the following text"));
    }

    #[test]
    fn prompt_contains_all_required_sections() {
        let prompt = build_solution_prompt(LanguageId::Rust, PlatformFormat::Acm, None, None);
        assert!(prompt.contains("题目识别"));
        assert!(prompt.contains("解题策略"));
        assert!(prompt.contains("完整代码"));
        assert!(prompt.contains("复杂度"));
        assert!(prompt.contains("边界用例"));
        assert!(prompt.contains("注意事项"));
    }

    #[test]
    fn language_constraints_cover_all_languages() {
        for id in [
            LanguageId::Cpp17,
            LanguageId::Cpp20,
            LanguageId::Python,
            LanguageId::Java,
            LanguageId::JavaScript,
            LanguageId::TypeScript,
            LanguageId::Go,
            LanguageId::Rust,
        ] {
            let constraint = build_language_constraints(id);
            assert!(!constraint.is_empty(), "{id:?} missing constraint");
        }
    }

    #[test]
    fn platform_hints_cover_all_formats() {
        for platform in [
            PlatformFormat::Acm,
            PlatformFormat::LeetCode,
            PlatformFormat::Generic,
        ] {
            let hint = build_platform_signature_hint(platform);
            assert!(!hint.is_empty(), "{platform:?} missing hint");
        }
    }

    #[test]
    fn cpp17_constraint_mentions_fast_io() {
        let c = build_language_constraints(LanguageId::Cpp17);
        assert!(c.contains("ios::sync_with_stdio"));
    }

    #[test]
    fn leetcode_hint_forbids_main_function() {
        let h = build_platform_signature_hint(PlatformFormat::LeetCode);
        assert!(h.contains("Do NOT include a main function"));
    }
}
