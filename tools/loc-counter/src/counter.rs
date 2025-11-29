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
                // BlockCommentStart is only returned when the block doesn't end on the same line
                in_block_comment = true;
                block_end = end;
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
            // Block comment continues to the next line
            return LineType::BlockCommentStart(end);
        }
    }

    // Check for code with trailing line comment (simple heuristic)
    // Note: This is a simplified check that may have false positives with URLs etc.
    // For more accurate detection, a proper lexer would be needed.
    for prefix in &syntax.line_comments {
        if let Some(pos) = line.find(prefix) {
            // Only count as mixed if there's actual code before the comment
            let before_comment = &line[..pos];
            if !before_comment.trim().is_empty() {
                return LineType::Mixed;
            }
        }
    }

    LineType::Code
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
        let mut file = tempfile::Builder::new()
            .suffix(extension)
            .tempfile()
            .unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_classify_line_code() {
        let syntax = CommentSyntax {
            line_comments: vec!["//"],
            block_comments: vec![("/*", "*/")],
        };

        assert_eq!(classify_line("let x = 5;", &syntax), LineType::Code);
        assert_eq!(classify_line("fn main() {", &syntax), LineType::Code);
    }

    #[test]
    fn test_classify_line_comment() {
        let syntax = CommentSyntax {
            line_comments: vec!["//"],
            block_comments: vec![("/*", "*/")],
        };

        assert_eq!(classify_line("// this is a comment", &syntax), LineType::Comment);
        assert_eq!(classify_line("/* block comment */", &syntax), LineType::Comment);
    }

    #[test]
    fn test_classify_line_block_comment_start() {
        let syntax = CommentSyntax {
            line_comments: vec!["//"],
            block_comments: vec![("/*", "*/")],
        };

        assert_eq!(
            classify_line("/* start of block", &syntax),
            LineType::BlockCommentStart("*/")
        );
    }

    #[test]
    fn test_classify_line_mixed() {
        let syntax = CommentSyntax {
            line_comments: vec!["//"],
            block_comments: vec![("/*", "*/")],
        };

        assert_eq!(
            classify_line("let x = 5; // inline comment", &syntax),
            LineType::Mixed
        );
    }

    #[test]
    fn test_classify_line_python() {
        let syntax = CommentSyntax {
            line_comments: vec!["#"],
            block_comments: vec![("\"\"\"", "\"\"\"")],
        };

        assert_eq!(classify_line("# Python comment", &syntax), LineType::Comment);
        assert_eq!(classify_line("x = 5", &syntax), LineType::Code);
        assert_eq!(classify_line("x = 5  # inline", &syntax), LineType::Mixed);
    }

    #[test]
    fn test_classify_line_html() {
        let syntax = CommentSyntax {
            line_comments: vec![],
            block_comments: vec![("<!--", "-->")],
        };

        assert_eq!(classify_line("<!-- comment -->", &syntax), LineType::Comment);
        assert_eq!(classify_line("<div>content</div>", &syntax), LineType::Code);
    }

    #[test]
    fn test_classify_line_sql() {
        let syntax = CommentSyntax {
            line_comments: vec!["--"],
            block_comments: vec![("/*", "*/")],
        };

        assert_eq!(classify_line("-- SQL comment", &syntax), LineType::Comment);
        assert_eq!(classify_line("SELECT * FROM users;", &syntax), LineType::Code);
    }

    #[test]
    fn test_count_rust_file_basic() {
        let content = "fn main() {\n    // comment\n    println!(\"Hello\");\n\n}";
        let file = create_temp_file(content, ".rs");

        let stats = count_file(file.path()).unwrap();

        assert_eq!(stats.language, "Rust");
        assert_eq!(stats.code, 3); // fn main, println, }
        assert_eq!(stats.comments, 1); // comment line
        assert_eq!(stats.blanks, 1); // empty line
        assert_eq!(stats.lines, 5);
    }

    #[test]
    fn test_count_file_with_block_comments() {
        let content = "/* start\nmiddle\nend */\ncode line";
        let file = create_temp_file(content, ".rs");

        let stats = count_file(file.path()).unwrap();

        assert_eq!(stats.comments, 3); // 3 comment lines
        assert_eq!(stats.code, 1); // 1 code line
    }

    #[test]
    fn test_count_file_empty() {
        let content = "";
        let file = create_temp_file(content, ".rs");

        let stats = count_file(file.path()).unwrap();

        assert_eq!(stats.code, 0);
        assert_eq!(stats.comments, 0);
        assert_eq!(stats.blanks, 0);
        assert_eq!(stats.lines, 0);
    }

    #[test]
    fn test_count_file_only_blanks() {
        let content = "\n\n\n";
        let file = create_temp_file(content, ".rs");

        let stats = count_file(file.path()).unwrap();

        assert_eq!(stats.blanks, 3);
        assert_eq!(stats.code, 0);
        assert_eq!(stats.comments, 0);
    }

    #[test]
    fn test_count_file_only_comments() {
        let content = "// comment 1\n// comment 2\n// comment 3";
        let file = create_temp_file(content, ".rs");

        let stats = count_file(file.path()).unwrap();

        assert_eq!(stats.comments, 3);
        assert_eq!(stats.code, 0);
        assert_eq!(stats.blanks, 0);
    }

    #[test]
    fn test_count_python_file() {
        let content = "# Python script\ndef main():\n    print('hello')\n\nif __name__ == '__main__':\n    main()";
        let file = create_temp_file(content, ".py");

        let stats = count_file(file.path()).unwrap();

        assert_eq!(stats.language, "Python");
        assert_eq!(stats.comments, 1);
        assert_eq!(stats.blanks, 1);
    }

    #[test]
    fn test_count_javascript_file() {
        let content = "// JS file\nfunction hello() {\n    console.log('hello');\n}\n/* block */";
        let file = create_temp_file(content, ".js");

        let stats = count_file(file.path()).unwrap();

        assert_eq!(stats.language, "JavaScript");
    }

    #[test]
    fn test_is_comment_start() {
        let syntax = CommentSyntax {
            line_comments: vec!["//", "#"],
            block_comments: vec![("/*", "*/")],
        };

        assert!(is_comment_start("// comment", &syntax));
        assert!(is_comment_start("# comment", &syntax));
        assert!(is_comment_start("/* block", &syntax));
        assert!(!is_comment_start("code", &syntax));
    }

    #[test]
    fn test_multiline_block_comment() {
        let content = "code\n/*\nblock\ncomment\n*/\nmore code";
        let file = create_temp_file(content, ".rs");

        let stats = count_file(file.path()).unwrap();

        assert_eq!(stats.code, 2);
        assert_eq!(stats.comments, 4); // /*, block, comment, */
    }

    #[test]
    fn test_nested_like_block_comments() {
        // Note: This test verifies behavior, not necessarily correct nesting
        let content = "/* outer /* inner */ still comment */\ncode";
        let file = create_temp_file(content, ".rs");

        let stats = count_file(file.path()).unwrap();

        // The simple parser sees the first */ and closes the block
        // The "still comment */" part would be counted as code/mixed
        assert!(stats.code >= 1);
    }
}
