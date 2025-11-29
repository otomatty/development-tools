//! Line counter for LOC Counter
//!
//! This module handles counting code, comment, and blank lines.

use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::language::{detect_language, get_comment_syntax, CommentSyntax};
use crate::types::FileStats;

/// Count lines in a file
pub fn count_file(path: &Path) -> Result<FileStats> {
    let language = detect_language(path).unwrap_or("Unknown");
    let mut stats = FileStats::new(path.to_path_buf(), language.to_string());
    let syntax = get_comment_syntax(language);

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut in_block_comment = false;
    let mut block_end = "";

    let mut code_lines = 0;
    let mut comment_lines = 0;
    let mut blank_lines = 0;

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();

        // Empty line
        if trimmed.is_empty() {
            blank_lines += 1;
            continue;
        }

        // Check for block comment state
        if in_block_comment {
            comment_lines += 1;
            if trimmed.contains(block_end) {
                in_block_comment = false;
                block_end = "";
            }
            continue;
        }

        // Classify the line
        let line_type = classify_line(trimmed, &syntax);

        match line_type {
            LineType::Code => code_lines += 1,
            LineType::Comment => comment_lines += 1,
            LineType::BlockCommentStart(end) => {
                comment_lines += 1;
                // Check if the block comment ends on the same line
                if !trimmed.contains(end) || trimmed.find(end) <= trimmed.find(
                    syntax.block_comments.iter()
                        .find(|(_s, e)| *e == end)
                        .map(|(s, _)| *s)
                        .unwrap_or("")
                ) {
                    // Check if it doesn't end on the same line (properly)
                    let start = syntax.block_comments.iter()
                        .find(|(_, e)| *e == end)
                        .map(|(s, _)| *s)
                        .unwrap_or("");
                    
                    if let (Some(start_pos), Some(end_pos)) = (trimmed.find(start), trimmed.rfind(end)) {
                        if end_pos <= start_pos + start.len() {
                            in_block_comment = true;
                            block_end = end;
                        }
                    } else {
                        in_block_comment = true;
                        block_end = end;
                    }
                }
            }
            LineType::Mixed => {
                // Line has both code and comment - count as code
                code_lines += 1;
            }
        }
    }

    stats.add_counts(code_lines, comment_lines, blank_lines);
    Ok(stats)
}

/// Line classification result
#[derive(Debug, Clone, PartialEq)]
enum LineType {
    Code,
    Comment,
    BlockCommentStart(&'static str), // Contains the block comment end marker
    Mixed,
}

/// Classify a single line
fn classify_line(line: &str, syntax: &CommentSyntax) -> LineType {
    // Check for line comments first
    for prefix in &syntax.line_comments {
        if line.starts_with(prefix) {
            return LineType::Comment;
        }
    }

    // Check for block comment start
    for (start, end) in &syntax.block_comments {
        if line.starts_with(start) {
            // Check if block ends on same line
            let after_start = &line[start.len()..];
            if after_start.contains(end) {
                // Check if there's code after the comment ends
                if let Some(pos) = after_start.find(end) {
                    let after_end = &after_start[pos + end.len()..];
                    if !after_end.trim().is_empty() && !is_comment_start(after_end.trim(), syntax) {
                        return LineType::Mixed;
                    }
                }
                return LineType::Comment;
            }
            return LineType::BlockCommentStart(end);
        }
    }

    // Check if line contains inline comment
    let has_inline_comment = syntax.line_comments.iter().any(|prefix| line.contains(prefix))
        || syntax.block_comments.iter().any(|(start, _)| line.contains(start));

    if has_inline_comment {
        // Line has code with inline comment
        LineType::Mixed
    } else {
        LineType::Code
    }
}

/// Check if a string starts with any comment marker
fn is_comment_start(s: &str, syntax: &CommentSyntax) -> bool {
    syntax.line_comments.iter().any(|prefix| s.starts_with(prefix))
        || syntax.block_comments.iter().any(|(start, _)| s.starts_with(start))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_file(content: &str, extension: &str) -> NamedTempFile {
        let mut file = NamedTempFile::with_suffix(&format!(".{}", extension)).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_count_rust_file_basic() {
        let content = r#"// This is a comment
fn main() {
    println!("Hello");
}"#;
        let file = create_temp_file(content, "rs");
        let stats = count_file(file.path()).unwrap();

        assert_eq!(stats.language, "Rust");
        assert_eq!(stats.blanks, 0); // No blank lines
        assert_eq!(stats.comments, 1); // One comment line
        assert_eq!(stats.code, 3); // fn main, println, closing brace
    }

    #[test]
    fn test_count_rust_file_block_comment() {
        let content = r#"/*
 * Multi-line
 * comment
 */
fn main() {
    // inline comment
    let x = 1;
}
"#;
        let file = create_temp_file(content, "rs");
        let stats = count_file(file.path()).unwrap();

        assert_eq!(stats.language, "Rust");
        assert_eq!(stats.comments, 5); // 4 block comment lines + 1 inline
        assert_eq!(stats.code, 3); // fn main, let x, closing brace
    }

    #[test]
    fn test_count_python_file() {
        let content = r#"# Comment
def main():
    print("Hello")

# Another comment
x = 1
"#;
        let file = create_temp_file(content, "py");
        let stats = count_file(file.path()).unwrap();

        assert_eq!(stats.language, "Python");
        assert_eq!(stats.comments, 2);
        assert_eq!(stats.code, 3);
        assert_eq!(stats.blanks, 1);
    }

    #[test]
    fn test_count_javascript_file() {
        let content = r#"// Single line comment
function hello() {
    /* block */ console.log("hi");
}
"#;
        let file = create_temp_file(content, "js");
        let stats = count_file(file.path()).unwrap();

        assert_eq!(stats.language, "JavaScript");
        assert_eq!(stats.comments, 1);
        // The mixed line (code + inline block comment) counts as code
        assert_eq!(stats.code, 3);
    }

    #[test]
    fn test_count_html_file() {
        let content = r#"<!DOCTYPE html>
<html>
<!-- Comment -->
<body>
</body>
</html>
"#;
        let file = create_temp_file(content, "html");
        let stats = count_file(file.path()).unwrap();

        assert_eq!(stats.language, "HTML");
        assert_eq!(stats.comments, 1);
        assert_eq!(stats.code, 5);
    }

    #[test]
    fn test_count_empty_file() {
        let content = "";
        let file = create_temp_file(content, "rs");
        let stats = count_file(file.path()).unwrap();

        assert_eq!(stats.code, 0);
        assert_eq!(stats.comments, 0);
        assert_eq!(stats.blanks, 0);
        assert_eq!(stats.lines, 0);
    }

    #[test]
    fn test_count_only_blanks() {
        let content = "\n\n\n";
        let file = create_temp_file(content, "rs");
        let stats = count_file(file.path()).unwrap();

        assert_eq!(stats.code, 0);
        assert_eq!(stats.comments, 0);
        assert_eq!(stats.blanks, 3);
    }

    #[test]
    fn test_count_only_comments() {
        let content = r#"// Comment 1
// Comment 2
// Comment 3
"#;
        let file = create_temp_file(content, "rs");
        let stats = count_file(file.path()).unwrap();

        assert_eq!(stats.code, 0);
        assert_eq!(stats.comments, 3);
        assert_eq!(stats.blanks, 0);
    }

    #[test]
    fn test_count_yaml_file() {
        let content = r#"# Comment
name: test
version: 1.0.0
# Another comment
dependencies:
  - dep1
"#;
        let file = create_temp_file(content, "yaml");
        let stats = count_file(file.path()).unwrap();

        assert_eq!(stats.language, "YAML");
        assert_eq!(stats.comments, 2);
        assert_eq!(stats.code, 4);
    }

    #[test]
    fn test_classify_line_code() {
        let syntax = get_comment_syntax("Rust");
        assert_eq!(classify_line("fn main() {", &syntax), LineType::Code);
        assert_eq!(classify_line("let x = 1;", &syntax), LineType::Code);
    }

    #[test]
    fn test_classify_line_comment() {
        let syntax = get_comment_syntax("Rust");
        assert_eq!(classify_line("// comment", &syntax), LineType::Comment);
        assert_eq!(classify_line("/// doc comment", &syntax), LineType::Comment);
    }

    #[test]
    fn test_classify_line_block_comment_start() {
        let syntax = get_comment_syntax("Rust");
        match classify_line("/* start of block", &syntax) {
            LineType::BlockCommentStart(end) => assert_eq!(end, "*/"),
            _ => panic!("Expected BlockCommentStart"),
        }
    }

    #[test]
    fn test_classify_line_mixed() {
        let syntax = get_comment_syntax("Rust");
        assert_eq!(classify_line("let x = 1; // comment", &syntax), LineType::Mixed);
    }

    #[test]
    fn test_classify_line_python_comment() {
        let syntax = get_comment_syntax("Python");
        assert_eq!(classify_line("# comment", &syntax), LineType::Comment);
        assert_eq!(classify_line("x = 1  # inline", &syntax), LineType::Mixed);
    }

    #[test]
    fn test_single_line_block_comment() {
        let content = r#"/* single line block */
fn main() {}
"#;
        let file = create_temp_file(content, "rs");
        let stats = count_file(file.path()).unwrap();

        assert_eq!(stats.comments, 1);
        assert_eq!(stats.code, 1);
    }

    #[test]
    fn test_json_file_no_comments() {
        let content = r#"{
  "name": "test",
  "version": "1.0.0"
}
"#;
        let file = create_temp_file(content, "json");
        let stats = count_file(file.path()).unwrap();

        assert_eq!(stats.language, "JSON");
        assert_eq!(stats.comments, 0);
        assert_eq!(stats.code, 4);
    }

    #[test]
    fn test_markdown_file() {
        let content = r#"# Title

Some text.

## Section

More text.
"#;
        let file = create_temp_file(content, "md");
        let stats = count_file(file.path()).unwrap();

        assert_eq!(stats.language, "Markdown");
        // Markdown has no comment syntax, so all non-blank lines are code
        assert_eq!(stats.code, 4);
        assert_eq!(stats.blanks, 3);
    }
}
