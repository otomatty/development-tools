//! Preset definitions for Large File Finder
//!
//! This module defines built-in exclusion presets for common
//! languages and frameworks.

use std::path::Path;

/// Available preset names
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PresetName {
    /// Common patterns for all projects
    Common,
    /// Rust-specific patterns
    Rust,
    /// Node.js/JavaScript patterns
    Node,
    /// Python patterns
    Python,
    /// Go patterns
    Go,
}

impl PresetName {
    /// Parse a preset name from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "common" => Some(Self::Common),
            "rust" => Some(Self::Rust),
            "node" | "nodejs" | "javascript" | "js" => Some(Self::Node),
            "python" | "py" => Some(Self::Python),
            "go" | "golang" => Some(Self::Go),
            _ => None,
        }
    }

    /// Get the display name
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Common => "common",
            Self::Rust => "rust",
            Self::Node => "node",
            Self::Python => "python",
            Self::Go => "go",
        }
    }
}

/// Get exclusion patterns for a preset
pub fn get_preset_patterns(preset: PresetName) -> Vec<&'static str> {
    match preset {
        PresetName::Common => vec![
            ".git",
            "node_modules",
            "dist",
            "build",
            "vendor",
            "__pycache__",
            "*.min.js",
            "*.min.css",
            ".DS_Store",
            "*.lock",
        ],
        PresetName::Rust => vec![
            "target",
            "Cargo.lock",
            "*.rlib",
            "*.rmeta",
        ],
        PresetName::Node => vec![
            "node_modules",
            "package-lock.json",
            "yarn.lock",
            "pnpm-lock.yaml",
            ".npm",
            ".yarn",
            "*.min.js",
            "*.bundle.js",
        ],
        PresetName::Python => vec![
            "venv",
            ".venv",
            "__pycache__",
            "*.pyc",
            "*.pyo",
            "*.egg-info",
            ".eggs",
            ".tox",
            ".pytest_cache",
        ],
        PresetName::Go => vec![
            "vendor",
            "go.sum",
        ],
    }
}

/// Detect which presets should be applied based on project files
pub fn detect_presets(root: &Path) -> Vec<PresetName> {
    let mut presets = vec![PresetName::Common];

    // Detect Rust
    if root.join("Cargo.toml").exists() {
        presets.push(PresetName::Rust);
    }

    // Detect Node.js
    if root.join("package.json").exists() {
        presets.push(PresetName::Node);
    }

    // Detect Python
    if root.join("pyproject.toml").exists()
        || root.join("setup.py").exists()
        || root.join("requirements.txt").exists()
    {
        presets.push(PresetName::Python);
    }

    // Detect Go
    if root.join("go.mod").exists() {
        presets.push(PresetName::Go);
    }

    presets
}

/// Get all exclusion patterns for multiple presets
pub fn get_all_patterns(presets: &[PresetName]) -> Vec<String> {
    let mut patterns: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();

    for preset in presets {
        for pattern in get_preset_patterns(*preset) {
            if !seen.contains(pattern) {
                seen.insert(pattern);
                patterns.push(pattern.to_string());
            }
        }
    }

    patterns
}

/// Parse preset names from a comma-separated string
pub fn parse_presets(input: &str) -> Vec<PresetName> {
    input
        .split(',')
        .filter_map(|s| PresetName::from_str(s.trim()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    // ==================== PresetName Tests ====================

    #[test]
    fn test_preset_name_from_str_common() {
        assert_eq!(PresetName::from_str("common"), Some(PresetName::Common));
        assert_eq!(PresetName::from_str("COMMON"), Some(PresetName::Common));
    }

    #[test]
    fn test_preset_name_from_str_rust() {
        assert_eq!(PresetName::from_str("rust"), Some(PresetName::Rust));
        assert_eq!(PresetName::from_str("Rust"), Some(PresetName::Rust));
    }

    #[test]
    fn test_preset_name_from_str_node_aliases() {
        assert_eq!(PresetName::from_str("node"), Some(PresetName::Node));
        assert_eq!(PresetName::from_str("nodejs"), Some(PresetName::Node));
        assert_eq!(PresetName::from_str("javascript"), Some(PresetName::Node));
        assert_eq!(PresetName::from_str("js"), Some(PresetName::Node));
    }

    #[test]
    fn test_preset_name_from_str_python_aliases() {
        assert_eq!(PresetName::from_str("python"), Some(PresetName::Python));
        assert_eq!(PresetName::from_str("py"), Some(PresetName::Python));
    }

    #[test]
    fn test_preset_name_from_str_go_aliases() {
        assert_eq!(PresetName::from_str("go"), Some(PresetName::Go));
        assert_eq!(PresetName::from_str("golang"), Some(PresetName::Go));
    }

    #[test]
    fn test_preset_name_from_str_invalid() {
        assert_eq!(PresetName::from_str("invalid"), None);
        assert_eq!(PresetName::from_str(""), None);
    }

    #[test]
    fn test_preset_name_as_str() {
        assert_eq!(PresetName::Common.as_str(), "common");
        assert_eq!(PresetName::Rust.as_str(), "rust");
        assert_eq!(PresetName::Node.as_str(), "node");
        assert_eq!(PresetName::Python.as_str(), "python");
        assert_eq!(PresetName::Go.as_str(), "go");
    }

    // ==================== get_preset_patterns Tests ====================

    #[test]
    fn test_get_preset_patterns_common_contains_expected() {
        let patterns = get_preset_patterns(PresetName::Common);
        assert!(patterns.contains(&".git"));
        assert!(patterns.contains(&"node_modules"));
        assert!(patterns.contains(&"*.min.js"));
    }

    #[test]
    fn test_get_preset_patterns_rust_contains_expected() {
        let patterns = get_preset_patterns(PresetName::Rust);
        assert!(patterns.contains(&"target"));
        assert!(patterns.contains(&"Cargo.lock"));
    }

    #[test]
    fn test_get_preset_patterns_node_contains_expected() {
        let patterns = get_preset_patterns(PresetName::Node);
        assert!(patterns.contains(&"node_modules"));
        assert!(patterns.contains(&"package-lock.json"));
        assert!(patterns.contains(&"yarn.lock"));
    }

    #[test]
    fn test_get_preset_patterns_python_contains_expected() {
        let patterns = get_preset_patterns(PresetName::Python);
        assert!(patterns.contains(&"venv"));
        assert!(patterns.contains(&"__pycache__"));
        assert!(patterns.contains(&"*.pyc"));
    }

    #[test]
    fn test_get_preset_patterns_go_contains_expected() {
        let patterns = get_preset_patterns(PresetName::Go);
        assert!(patterns.contains(&"vendor"));
        assert!(patterns.contains(&"go.sum"));
    }

    // ==================== detect_presets Tests ====================

    #[test]
    fn test_detect_presets_empty_dir() {
        let dir = tempdir().unwrap();
        let presets = detect_presets(dir.path());
        
        // Should always include common
        assert!(presets.contains(&PresetName::Common));
        assert_eq!(presets.len(), 1);
    }

    #[test]
    fn test_detect_presets_rust_project() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("Cargo.toml")).unwrap();
        
        let presets = detect_presets(dir.path());
        
        assert!(presets.contains(&PresetName::Common));
        assert!(presets.contains(&PresetName::Rust));
    }

    #[test]
    fn test_detect_presets_node_project() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("package.json")).unwrap();
        
        let presets = detect_presets(dir.path());
        
        assert!(presets.contains(&PresetName::Common));
        assert!(presets.contains(&PresetName::Node));
    }

    #[test]
    fn test_detect_presets_python_pyproject() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("pyproject.toml")).unwrap();
        
        let presets = detect_presets(dir.path());
        
        assert!(presets.contains(&PresetName::Common));
        assert!(presets.contains(&PresetName::Python));
    }

    #[test]
    fn test_detect_presets_python_setup_py() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("setup.py")).unwrap();
        
        let presets = detect_presets(dir.path());
        
        assert!(presets.contains(&PresetName::Python));
    }

    #[test]
    fn test_detect_presets_python_requirements() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("requirements.txt")).unwrap();
        
        let presets = detect_presets(dir.path());
        
        assert!(presets.contains(&PresetName::Python));
    }

    #[test]
    fn test_detect_presets_go_project() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("go.mod")).unwrap();
        
        let presets = detect_presets(dir.path());
        
        assert!(presets.contains(&PresetName::Common));
        assert!(presets.contains(&PresetName::Go));
    }

    #[test]
    fn test_detect_presets_multi_language_project() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("Cargo.toml")).unwrap();
        File::create(dir.path().join("package.json")).unwrap();
        
        let presets = detect_presets(dir.path());
        
        assert!(presets.contains(&PresetName::Common));
        assert!(presets.contains(&PresetName::Rust));
        assert!(presets.contains(&PresetName::Node));
    }

    // ==================== get_all_patterns Tests ====================

    #[test]
    fn test_get_all_patterns_single_preset() {
        let patterns = get_all_patterns(&[PresetName::Rust]);
        
        assert!(patterns.contains(&"target".to_string()));
        assert!(patterns.contains(&"Cargo.lock".to_string()));
    }

    #[test]
    fn test_get_all_patterns_multiple_presets() {
        let patterns = get_all_patterns(&[PresetName::Common, PresetName::Rust]);
        
        // Common patterns
        assert!(patterns.contains(&".git".to_string()));
        // Rust patterns
        assert!(patterns.contains(&"target".to_string()));
    }

    #[test]
    fn test_get_all_patterns_no_duplicates() {
        // Both common and node have node_modules
        let patterns = get_all_patterns(&[PresetName::Common, PresetName::Node]);
        
        let node_modules_count = patterns.iter().filter(|p| *p == "node_modules").count();
        assert_eq!(node_modules_count, 1);
    }

    // ==================== parse_presets Tests ====================

    #[test]
    fn test_parse_presets_single() {
        let presets = parse_presets("rust");
        assert_eq!(presets, vec![PresetName::Rust]);
    }

    #[test]
    fn test_parse_presets_multiple() {
        let presets = parse_presets("rust,node,python");
        assert_eq!(presets.len(), 3);
        assert!(presets.contains(&PresetName::Rust));
        assert!(presets.contains(&PresetName::Node));
        assert!(presets.contains(&PresetName::Python));
    }

    #[test]
    fn test_parse_presets_with_spaces() {
        let presets = parse_presets("rust, node, python");
        assert_eq!(presets.len(), 3);
    }

    #[test]
    fn test_parse_presets_with_invalid() {
        let presets = parse_presets("rust,invalid,node");
        assert_eq!(presets.len(), 2);
        assert!(presets.contains(&PresetName::Rust));
        assert!(presets.contains(&PresetName::Node));
    }

    #[test]
    fn test_parse_presets_empty() {
        let presets = parse_presets("");
        assert!(presets.is_empty());
    }
}
