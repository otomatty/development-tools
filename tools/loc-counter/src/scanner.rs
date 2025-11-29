//! File scanner for LOC Counter
//!
//! This module handles directory traversal and file filtering.

use anyhow::Result;
use glob::Pattern;
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
            // Use glob pattern matching
            if matches_pattern(component_str, exclude) {
                return true;
            }
        }
    }

    false
}

/// Pattern matching using glob crate (supports * and ? wildcards)
fn matches_pattern(text: &str, pattern: &str) -> bool {
    // Direct match
    if text == pattern {
        return true;
    }

    // Use glob pattern for wildcard matching
    if pattern.contains('*') || pattern.contains('?') {
        match Pattern::new(pattern) {
            Ok(p) => p.matches(text),
            Err(_) => {
                // Fallback for invalid glob patterns
                text == pattern
            }
        }
    } else {
        false
    }
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
        let path = Path::new("/project/.git/objects/test");
        let excludes = vec![".git".to_string()];

        assert!(should_exclude(path, root, &excludes));
    }

    #[test]
    fn test_should_not_exclude_regular_file() {
        let root = Path::new("/project");
        let path = Path::new("/project/src/main.rs");
        let excludes = vec!["node_modules".to_string(), "target".to_string()];

        assert!(!should_exclude(path, root, &excludes));
    }

    #[test]
    fn test_should_exclude_glob_pattern() {
        let root = Path::new("/project");
        let path = Path::new("/project/test_file.txt");
        let excludes = vec!["test_*".to_string()];

        assert!(should_exclude(path, root, &excludes));
    }

    #[test]
    fn test_scan_directory() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        // Create a rust file
        File::create(src_dir.join("main.rs")).unwrap();

        // Create a non-source file
        File::create(dir.path().join("README.md")).unwrap();

        let files = scan_directory(dir.path(), &[]).unwrap();

        // Should find both .rs and .md files (Markdown is supported)
        assert!(files.len() >= 1);
        assert!(files.iter().any(|f| f.extension().map_or(false, |e| e == "rs")));
    }

    #[test]
    fn test_scan_directory_with_excludes() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("src");
        let node_modules = dir.path().join("node_modules");
        fs::create_dir_all(&src_dir).unwrap();
        fs::create_dir_all(&node_modules).unwrap();

        // Create source files
        File::create(src_dir.join("main.rs")).unwrap();
        File::create(node_modules.join("package.json")).unwrap();

        let excludes = vec!["node_modules".to_string()];
        let files = scan_directory(dir.path(), &excludes).unwrap();

        // Should only find main.rs, not the file in node_modules
        assert!(!files.iter().any(|f| f.to_string_lossy().contains("node_modules")));
    }

    #[test]
    fn test_matches_pattern_direct() {
        assert!(matches_pattern("node_modules", "node_modules"));
        assert!(!matches_pattern("src", "node_modules"));
    }

    #[test]
    fn test_matches_pattern_glob_asterisk() {
        assert!(matches_pattern("test_file", "test_*"));
        assert!(matches_pattern("test_", "test_*"));
        assert!(!matches_pattern("file_test", "test_*"));
    }

    #[test]
    fn test_matches_pattern_glob_question() {
        assert!(matches_pattern("test1", "test?"));
        assert!(matches_pattern("testa", "test?"));
        assert!(!matches_pattern("test12", "test?"));
    }

    #[test]
    fn test_matches_pattern_hidden_files() {
        assert!(matches_pattern(".git", ".git"));
        assert!(matches_pattern(".gitignore", ".gitignore"));
        assert!(!matches_pattern("git", ".git"));
    }

    #[test]
    fn test_matches_pattern_complex_glob() {
        assert!(matches_pattern("file.test.rs", "*.rs"));
        assert!(matches_pattern("a", "*"));
        assert!(matches_pattern("abc", "a*c"));
    }
}
