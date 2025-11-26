//! Directory scanner for TODO Collector

use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// File extensions to scan for comments
const SCANNABLE_EXTENSIONS: &[&str] = &[
    // Rust
    "rs",
    // JavaScript/TypeScript
    "js", "jsx", "ts", "tsx", "mjs", "cjs",
    // Web
    "html", "htm", "css", "scss", "sass", "less", "vue", "svelte",
    // Python
    "py", "pyw",
    // Ruby
    "rb", "rake",
    // Go
    "go",
    // C/C++
    "c", "h", "cpp", "hpp", "cc", "cxx",
    // Java/Kotlin
    "java", "kt", "kts",
    // Swift
    "swift",
    // Shell
    "sh", "bash", "zsh", "fish",
    // PHP
    "php",
    // Lua
    "lua",
    // SQL
    "sql",
    // Config/Data
    "json", "yaml", "yml", "toml", "xml",
    // Markdown
    "md", "mdx",
    // Other
    "r", "R", "scala", "ex", "exs", "erl", "hrl", "clj", "cljs",
];

/// Scan a directory for source files
pub fn scan_directory(dir: &Path, excludes: &[String]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !should_exclude(e.path(), excludes))
    {
        let entry = entry?;
        let path = entry.path();

        // Skip directories
        if path.is_dir() {
            continue;
        }

        // Check if the file has a scannable extension
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                if SCANNABLE_EXTENSIONS.contains(&ext_str.to_lowercase().as_str()) {
                    files.push(path.to_path_buf());
                }
            }
        }
    }

    Ok(files)
}

/// Check if a path should be excluded
fn should_exclude(path: &Path, excludes: &[String]) -> bool {
    for component in path.components() {
        if let Some(name) = component.as_os_str().to_str() {
            // Skip hidden directories (except the root if it starts with .)
            if name.starts_with('.') && name != "." && name != ".." {
                return true;
            }

            // Check against exclude patterns
            for exclude in excludes {
                if name == exclude {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_exclude() {
        let excludes = vec!["node_modules".to_string(), "target".to_string()];
        
        assert!(should_exclude(Path::new("project/node_modules/package"), &excludes));
        assert!(should_exclude(Path::new("project/target/debug"), &excludes));
        assert!(should_exclude(Path::new("project/.git/config"), &excludes));
        assert!(!should_exclude(Path::new("project/src/main.rs"), &excludes));
    }
}

