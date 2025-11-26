//! Comment detector for TODO-like patterns

use anyhow::Result;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::types::{TodoItem, TodoType};

/// Detect comment markers in a file
pub fn detect_todos(file_path: &Path, patterns: &[String]) -> Result<Vec<TodoItem>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut todos = Vec::new();

    // Build the regex pattern for matching comment markers
    // Matches patterns like: TODO: message, TODO(author): message, TODO - message
    let pattern_str = patterns.join("|");
    let regex = Regex::new(&format!(
        r"(?i)\b({})(?:\s*\([^)]*\))?\s*[:：\-]\s*(.*)",
        pattern_str
    ))?;

    let file_str = file_path.to_string_lossy().to_string();

    for (line_num, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue, // Skip lines that can't be read (e.g., binary content)
        };

        // Check if this line is a comment and contains a TODO pattern
        if let Some(todo) = extract_todo_from_line(&line, &regex, &file_str, line_num + 1) {
            todos.push(todo);
        }
    }

    Ok(todos)
}

/// Extract a TODO item from a line if it matches the pattern
fn extract_todo_from_line(
    line: &str,
    regex: &Regex,
    file_path: &str,
    line_num: usize,
) -> Option<TodoItem> {
    // Check if this line contains a comment
    if !is_likely_comment(line) {
        return None;
    }

    // Try to match the TODO pattern
    let captures = regex.captures(line)?;
    
    let type_str = captures.get(1)?.as_str().to_uppercase();
    let todo_type = TodoType::from_str(&type_str)?;
    
    let content = captures.get(2)
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_default();

    Some(TodoItem {
        todo_type,
        file: file_path.to_string(),
        line: line_num,
        content,
    })
}

/// Check if a line is likely a comment
fn is_likely_comment(line: &str) -> bool {
    let trimmed = line.trim();
    
    // Single-line comment markers
    if trimmed.starts_with("//")
        || trimmed.starts_with('#')
        || trimmed.starts_with("--")
        || trimmed.starts_with(';')
        || trimmed.starts_with('%')
        || trimmed.starts_with("*")
        || trimmed.starts_with("/*")
        || trimmed.starts_with("<!--")
        || trimmed.starts_with("'''")
        || trimmed.starts_with("\"\"\"")
        || trimmed.starts_with("REM ")
        || trimmed.starts_with("rem ")
    {
        return true;
    }

    // Multi-line comment content (line contains comment markers)
    if trimmed.contains("//")
        || trimmed.contains("/*")
        || trimmed.contains("*/")
        || trimmed.contains("<!--")
        || trimmed.contains("-->")
    {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_likely_comment() {
        assert!(is_likely_comment("// TODO: fix this"));
        assert!(is_likely_comment("# TODO: fix this"));
        assert!(is_likely_comment("/* TODO: fix this */"));
        assert!(is_likely_comment("<!-- TODO: fix this -->"));
        assert!(is_likely_comment("   * TODO: fix this"));
        assert!(!is_likely_comment("let todo = 5;"));
    }

    #[test]
    fn test_extract_todo_from_line() {
        let patterns = vec!["TODO".to_string(), "FIXME".to_string()];
        let pattern_str = patterns.join("|");
        let regex = Regex::new(&format!(r"(?i)\b({})(?:\s*\([^)]*\))?\s*[:：\-]\s*(.*)", pattern_str)).unwrap();

        // Test basic TODO with colon
        let todo = extract_todo_from_line(
            "// TODO: fix this bug",
            &regex,
            "test.rs",
            10,
        );
        assert!(todo.is_some());
        let todo = todo.unwrap();
        assert_eq!(todo.todo_type, TodoType::Todo);
        assert_eq!(todo.content, "fix this bug");
        assert_eq!(todo.line, 10);

        // Test TODO with author
        let todo = extract_todo_from_line(
            "// TODO(john): review this",
            &regex,
            "test.rs",
            20,
        );
        assert!(todo.is_some());
        let todo = todo.unwrap();
        assert_eq!(todo.todo_type, TodoType::Todo);
        assert_eq!(todo.content, "review this");

        // Should NOT match "TODO Collector" without delimiter
        let todo = extract_todo_from_line(
            "// TODO Collector is a tool",
            &regex,
            "test.rs",
            30,
        );
        assert!(todo.is_none());
    }
}

