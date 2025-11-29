//! File scanner for LOC Counter
//!
//! This module handles directory traversal and file filtering.

use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::language::is_supported_extension;

/// Scan a directory for source files
pub fn scan_directory(root: &Path, excludes: &[String]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !should_exclude(e.path(), root, excludes))
    {
        let entry = entry?;
        let path = entry.path();

        // Skip directories
        if !path.is_file() {
            continue;
        }

        // Check if the file extension is supported
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                if is_supported_extension(ext_str) {
                    files.push(path.to_path_buf());
                }
            }
        }
    }

    Ok(files)
}

/// Check if a path should be excluded
fn should_exclude(path: &Path, root: &Path, excludes: &[String]) -> bool {
    // Get the relative path from root
    let relative = match path.strip_prefix(root) {
        Ok(r) => r,
        Err(_) => return false,
    };

    // Check each path component
    for component in relative.components() {
        let component_str = component.as_os_str().to_str().unwrap_or("");
        
        for exclude in excludes {
            // Simple pattern matching
            if matches_pattern(component_str, exclude) {
                return true;
            }
        }
    }

    false
}

/// Simple pattern matching (supports * and ? wildcards)
fn matches_pattern(text: &str, pattern: &str) -> bool {
    // Direct match
    if text == pattern {
        return true;
    }

    // Hidden files/directories (starting with .)
    if pattern.starts_with('.') && text == pattern {
        return true;
    }

    // Simple glob matching
    if pattern.contains('*') || pattern.contains('?') {
        return glob_match(text, pattern);
    }

    false
}

/// Simple glob pattern matching
fn glob_match(text: &str, pattern: &str) -> bool {
    let mut text_chars = text.chars().peekable();
    let mut pattern_chars = pattern.chars().peekable();

    while let Some(p) = pattern_chars.next() {
        match p {
            '*' => {
                // Check if this is the last character
                if pattern_chars.peek().is_none() {
                    return true;
                }

                // Try all positions
                while text_chars.peek().is_some() {
                    let remaining_text: String = text_chars.clone().collect();
                    let remaining_pattern: String = pattern_chars.clone().collect();
                    if glob_match(&remaining_text, &remaining_pattern) {
                        return true;
                    }
                    text_chars.next();
                }
                return false;
            }
            '?' => {
                if text_chars.next().is_none() {
                    return false;
                }
            }
            c => {
                if text_chars.next() != Some(c) {
                    return false;
                }
            }
        }
    }

    text_chars.peek().is_none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::tempdir;

    #[test]
    fn test_should_exclude_node_modules() {
        let root = Path::new("/project");
        let path = Path::new("/project/node_modules/package/index.js");
        let excludes = vec!["node_modules".to_string()];

        assert!(should_exclude(path, root, &excludes));
    }

    #[test]
    fn test_should_exclude_target() {
        let root = Path::new("/project");
        let path = Path::new("/project/target/debug/main.rs");
        let excludes = vec!["target".to_string()];

        assert!(should_exclude(path, root, &excludes));
    }

    #[test]
    fn test_should_exclude_git() {
        let root = Path::new("/project");
        let path = Path::new("/project/.git/config");
        let excludes = vec![".git".to_string()];

        assert!(should_exclude(path, root, &excludes));
    }

    #[test]
    fn test_should_not_exclude_src() {
        let root = Path::new("/project");
        let path = Path::new("/project/src/main.rs");
        let excludes = vec!["node_modules".to_string(), "target".to_string()];

        assert!(!should_exclude(path, root, &excludes));
    }

    #[test]
    fn test_should_exclude_nested() {
        let root = Path::new("/project");
        let path = Path::new("/project/packages/app/node_modules/lib/index.js");
        let excludes = vec!["node_modules".to_string()];

        assert!(should_exclude(path, root, &excludes));
    }

    #[test]
    fn test_matches_pattern_exact() {
        assert!(matches_pattern("node_modules", "node_modules"));
        assert!(matches_pattern(".git", ".git"));
        assert!(!matches_pattern("src", "node_modules"));
    }

    #[test]
    fn test_matches_pattern_wildcard_star() {
        assert!(matches_pattern("test.log", "*.log"));
        assert!(matches_pattern("build", "build*"));
        assert!(matches_pattern("build123", "build*"));
        assert!(!matches_pattern("test.txt", "*.log"));
    }

    #[test]
    fn test_matches_pattern_wildcard_question() {
        assert!(matches_pattern("a1", "a?"));
        assert!(matches_pattern("ab", "a?"));
        assert!(!matches_pattern("abc", "a?"));
    }

    #[test]
    fn test_glob_match_star() {
        assert!(glob_match("test.log", "*.log"));
        assert!(glob_match("anything.log", "*.log"));
        assert!(glob_match(".log", "*.log"));
        assert!(!glob_match("test.txt", "*.log"));
    }

    #[test]
    fn test_glob_match_question() {
        assert!(glob_match("ab", "a?"));
        assert!(glob_match("ac", "a?"));
        assert!(!glob_match("a", "a?"));
        assert!(!glob_match("abc", "a?"));
    }

    #[test]
    fn test_glob_match_combined() {
        assert!(glob_match("test123.log", "test*.log"));
        assert!(glob_match("test.log", "test*.log"));
        assert!(!glob_match("test123.txt", "test*.log"));
    }

    #[test]
    fn test_scan_directory() -> Result<()> {
        let dir = tempdir()?;
        let root = dir.path();

        // Create test files
        fs::create_dir_all(root.join("src"))?;
        fs::create_dir_all(root.join("node_modules/pkg"))?;
        File::create(root.join("src/main.rs"))?;
        File::create(root.join("src/lib.rs"))?;
        File::create(root.join("node_modules/pkg/index.js"))?;
        File::create(root.join("README.md"))?;

        let excludes = vec!["node_modules".to_string()];
        let files = scan_directory(root, &excludes)?;

        // Should find main.rs, lib.rs, README.md but not files in node_modules
        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|f| f.ends_with("main.rs")));
        assert!(files.iter().any(|f| f.ends_with("lib.rs")));
        assert!(files.iter().any(|f| f.ends_with("README.md")));
        assert!(!files.iter().any(|f| f.to_str().unwrap().contains("node_modules")));

        Ok(())
    }

    #[test]
    fn test_scan_directory_with_multiple_excludes() -> Result<()> {
        let dir = tempdir()?;
        let root = dir.path();

        // Create test files
        fs::create_dir_all(root.join("src"))?;
        fs::create_dir_all(root.join("target/debug"))?;
        fs::create_dir_all(root.join(".git/objects"))?;
        File::create(root.join("src/main.rs"))?;
        File::create(root.join("target/debug/main.rs"))?;

        let excludes = vec!["target".to_string(), ".git".to_string()];
        let files = scan_directory(root, &excludes)?;

        // Should only find src/main.rs
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("main.rs"));
        assert!(files[0].to_str().unwrap().contains("src"));

        Ok(())
    }

    #[test]
    fn test_scan_directory_unsupported_files() -> Result<()> {
        let dir = tempdir()?;
        let root = dir.path();

        // Create test files
        File::create(root.join("test.rs"))?;
        File::create(root.join("test.exe"))?;
        File::create(root.join("test.bin"))?;
        File::create(root.join("test.png"))?;

        let files = scan_directory(root, &[])?;

        // Should only find test.rs
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("test.rs"));

        Ok(())
    }
}
