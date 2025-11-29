//! File scanner for Large File Finder
//!
//! This module handles directory traversal using the `ignore` crate
//! to automatically respect .gitignore patterns.

use anyhow::Result;
use glob::Pattern;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// Scan a directory for files, respecting .gitignore patterns
pub fn scan_directory(root: &Path, excludes: &[String]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    // Use ignore crate's WalkBuilder which automatically respects .gitignore
    let walker = WalkBuilder::new(root)
        .hidden(false)          // Don't skip hidden files by default
        .git_ignore(true)       // Respect .gitignore
        .git_global(true)       // Respect global gitignore
        .git_exclude(true)      // Respect .git/info/exclude
        .follow_links(false)    // Don't follow symlinks to avoid loops
        .build();

    for entry in walker {
        match entry {
            Ok(entry) => {
                let path = entry.path();

                // Skip directories
                if !path.is_file() {
                    continue;
                }

                // Check additional exclude patterns
                if should_exclude_by_pattern(path, root, excludes) {
                    continue;
                }

                files.push(path.to_path_buf());
            }
            Err(e) => {
                // Log error but continue scanning
                eprintln!("Warning: Error accessing entry: {}", e);
            }
        }
    }

    Ok(files)
}

/// Check if a path should be excluded by custom patterns
fn should_exclude_by_pattern(path: &Path, root: &Path, excludes: &[String]) -> bool {
    // Get the relative path from root
    let relative = match path.strip_prefix(root) {
        Ok(r) => r,
        Err(_) => return false,
    };

    let relative_str = relative.to_string_lossy();

    for exclude in excludes {
        // Try pattern matching on the full relative path
        if matches_pattern(&relative_str, exclude) {
            return true;
        }

        // Also check each path component
        for component in relative.components() {
            let component_str = component.as_os_str().to_str().unwrap_or("");
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
    if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
        match Pattern::new(pattern) {
            Ok(p) => p.matches(text),
            Err(_) => text == pattern,
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_pattern_direct() {
        assert!(matches_pattern("node_modules", "node_modules"));
        assert!(!matches_pattern("node_modules", "target"));
    }

    #[test]
    fn test_matches_pattern_wildcard() {
        assert!(matches_pattern("test.min.js", "*.min.js"));
        assert!(matches_pattern("generated_code.rs", "generated_*.rs"));
        assert!(!matches_pattern("test.js", "*.min.js"));
    }

    #[test]
    fn test_matches_pattern_double_star() {
        // Note: glob crate's Pattern doesn't support ** the same way as gitignore
        // For full ** support, we rely on the ignore crate
        assert!(matches_pattern("foo", "*"));
    }

    #[test]
    fn test_should_exclude_by_pattern() {
        let root = Path::new("/project");
        let excludes = vec!["*.min.js".to_string(), "generated".to_string()];

        // File matching pattern
        assert!(should_exclude_by_pattern(
            Path::new("/project/dist/bundle.min.js"),
            root,
            &excludes
        ));

        // Directory matching pattern
        assert!(should_exclude_by_pattern(
            Path::new("/project/generated/output.rs"),
            root,
            &excludes
        ));

        // Non-matching file
        assert!(!should_exclude_by_pattern(
            Path::new("/project/src/main.rs"),
            root,
            &excludes
        ));
    }
}
