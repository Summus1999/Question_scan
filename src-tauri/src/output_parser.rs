/**
 * Question Scan AI 输出解析模块（阶段 6）。
 *
 * 职责：从 AI 返回的结构化文本中提取主代码块。
 * 策略1：查找 "完整代码" 章节标题，提取到下一章节之间的内容。
 * 策略2：回退到提取第一个 fenced code block（```lang ... ```）。
 */
/** 从 AI 解法响应中提取主代码块。 */
/// Extracts the primary code block from an AI solution response.
pub fn extract_main_code_block(text: &str) -> Option<String> {
    // Strategy 1: look for the structured section header.
    if let Some(block) = extract_after_section_header(text, "完整代码") {
        return Some(block);
    }

    // Strategy 2: fall back to the first fenced code block.
    extract_first_fenced_code_block(text)
}

/** 查找指定章节标题后的内容，直到下一个章节标题或文本结束。 */
fn extract_after_section_header(text: &str, header: &str) -> Option<String> {
    let header_pattern = format!("{header}:");
    let start = text.find(&header_pattern)?;
    let content_start = start + header_pattern.len();

    // Find the next section header (ends with ':') or end of text.
    let remainder = &text[content_start..];
    let end = remainder
        .lines()
        .skip(1) // skip the header line itself
        .enumerate()
        .find(|(_, line)| {
            let trimmed = line.trim();
            !trimmed.is_empty() && trimmed.ends_with(':') && trimmed.len() < 20
        })
        .map(|(idx, _)| idx)
        .unwrap_or_else(|| remainder.lines().count());

    let block: String = remainder
        .lines()
        .skip(1)
        .take(end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    if block.is_empty() {
        None
    } else {
        Some(block)
    }
}

/** 提取第一个 fenced code block（```lang ... ```）的内容。 */
fn extract_first_fenced_code_block(text: &str) -> Option<String> {
    let start = text.find("```")?;
    let after_open = &text[start + 3..];

    // Skip the language identifier line if present.
    let content_start = after_open.find('\n').map(|i| i + 1).unwrap_or(0);
    let content = &after_open[content_start..];

    let end = content.find("```")?;
    let block = content[..end].trim().to_string();

    if block.is_empty() {
        None
    } else {
        Some(block)
    }
}

#[cfg(test)]
mod tests {
    use super::extract_main_code_block;

    #[test]
    fn extracts_code_from_structured_section() {
        let text = "题目识别:\nTwo Sum\n\n完整代码:\n```cpp\nint main() {}\n```\n\n复杂度:\nO(n)";
        let code = extract_main_code_block(text).expect("should extract code");
        assert!(code.contains("int main()"));
    }

    #[test]
    fn extracts_code_without_fences_in_section() {
        let text = "完整代码:\nint main() { return 0; }\n\n复杂度:\nO(1)";
        let code = extract_main_code_block(text).expect("should extract code");
        assert!(code.contains("int main()"));
    }

    #[test]
    fn falls_back_to_first_fenced_block_when_no_section() {
        let text = "Here is the code:\n\n```python\ndef solve():\n    pass\n```\n\nDone.";
        let code = extract_main_code_block(text).expect("should extract code");
        assert!(code.contains("def solve()"));
    }

    #[test]
    fn returns_none_when_no_code_found() {
        let text = "Just some explanation without any code.";
        assert_eq!(extract_main_code_block(text), None);
    }

    #[test]
    fn handles_empty_fenced_block() {
        let text = "```cpp\n```";
        assert_eq!(extract_main_code_block(text), None);
    }

    #[test]
    fn extracts_multiline_code_block() {
        let text = r#"完整代码:
```rust
fn main() {
    println!("hello");
}
```
复杂度:"#;
        let code = extract_main_code_block(text).expect("should extract code");
        assert!(code.contains("fn main()"));
        assert!(code.contains("println!"));
    }
}
