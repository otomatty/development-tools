//! Configuration file support for Large File Finder
//!
//! This module handles reading and parsing `.largefile.toml` configuration files.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

/// Configuration file structure
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Additional exclude patterns
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Minimum lines threshold (overrides CLI default)
    #[serde(default)]
    pub min_lines: Option<usize>,

    /// Preset configuration
    #[serde(default = "PresetConfig::default")]
    pub presets: PresetConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            exclude: Vec::new(),
            min_lines: None,
            presets: PresetConfig::default(),
        }
    }
}

/// Preset configuration in config file
#[derive(Debug, Clone, Deserialize)]
pub struct PresetConfig {
    /// Auto-detect presets based on project files
    #[serde(default = "default_auto_detect")]
    pub auto_detect: bool,

    /// Manual preset list
    #[serde(default)]
    pub manual: Vec<String>,
}

impl Default for PresetConfig {
    fn default() -> Self {
        Self {
            auto_detect: true,
            manual: Vec::new(),
        }
    }
}

fn default_auto_detect() -> bool {
    true
}

/// Configuration file name
pub const CONFIG_FILE_NAME: &str = ".largefile.toml";

/// Load configuration from a directory
/// Returns None if no config file exists, Some(Config) if found and parsed
pub fn load_config(dir: &Path) -> Result<Option<Config>> {
    let config_path = dir.join(CONFIG_FILE_NAME);

    if !config_path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

    let config: Config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;

    Ok(Some(config))
}

/// Check if a config file exists in the given directory
#[allow(dead_code)]
pub fn config_exists(dir: &Path) -> bool {
    dir.join(CONFIG_FILE_NAME).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // ==================== Config Parsing Tests ====================

    #[test]
    fn test_parse_empty_config() {
        let config: Config = toml::from_str("").unwrap();
        assert!(config.exclude.is_empty());
        assert!(config.min_lines.is_none());
        assert!(config.presets.auto_detect);
        assert!(config.presets.manual.is_empty());
    }

    #[test]
    fn test_parse_exclude_only() {
        let toml = r#"
exclude = ["generated/**", "*.min.js"]
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.exclude.len(), 2);
        assert!(config.exclude.contains(&"generated/**".to_string()));
        assert!(config.exclude.contains(&"*.min.js".to_string()));
    }

    #[test]
    fn test_parse_min_lines() {
        let toml = r#"
min_lines = 300
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.min_lines, Some(300));
    }

    #[test]
    fn test_parse_presets_auto_detect_true() {
        let toml = r#"
[presets]
auto_detect = true
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.presets.auto_detect);
    }

    #[test]
    fn test_parse_presets_auto_detect_false() {
        let toml = r#"
[presets]
auto_detect = false
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(!config.presets.auto_detect);
    }

    #[test]
    fn test_parse_presets_manual() {
        let toml = r#"
[presets]
manual = ["rust", "node"]
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.presets.manual.len(), 2);
        assert!(config.presets.manual.contains(&"rust".to_string()));
        assert!(config.presets.manual.contains(&"node".to_string()));
    }

    #[test]
    fn test_parse_full_config() {
        let toml = r#"
# Additional exclude patterns
exclude = [
    "generated/**",
    "*.min.js",
    "vendor/legacy/**",
]

# Override default min_lines
min_lines = 400

[presets]
auto_detect = false
manual = ["rust", "node", "python"]
"#;
        let config: Config = toml::from_str(toml).unwrap();

        assert_eq!(config.exclude.len(), 3);
        assert!(config.exclude.contains(&"generated/**".to_string()));
        assert!(config.exclude.contains(&"*.min.js".to_string()));
        assert!(config.exclude.contains(&"vendor/legacy/**".to_string()));

        assert_eq!(config.min_lines, Some(400));

        assert!(!config.presets.auto_detect);
        assert_eq!(config.presets.manual.len(), 3);
    }

    // ==================== File Loading Tests ====================

    #[test]
    fn test_load_config_no_file() {
        let dir = tempdir().unwrap();
        let result = load_config(dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_load_config_exists() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join(CONFIG_FILE_NAME);

        fs::write(
            &config_path,
            r#"
exclude = ["test/**"]
min_lines = 250
"#,
        )
        .unwrap();

        let result = load_config(dir.path()).unwrap();
        assert!(result.is_some());

        let config = result.unwrap();
        assert_eq!(config.exclude, vec!["test/**"]);
        assert_eq!(config.min_lines, Some(250));
    }

    #[test]
    fn test_load_config_invalid_toml() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join(CONFIG_FILE_NAME);

        fs::write(&config_path, "this is not valid toml [[[").unwrap();

        let result = load_config(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_config_exists_true() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join(CONFIG_FILE_NAME);
        fs::write(&config_path, "").unwrap();

        assert!(config_exists(dir.path()));
    }

    #[test]
    fn test_config_exists_false() {
        let dir = tempdir().unwrap();
        assert!(!config_exists(dir.path()));
    }

    // ==================== Default Values Tests ====================

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.exclude.is_empty());
        assert!(config.min_lines.is_none());
        assert!(config.presets.auto_detect); // Default is true
        assert!(config.presets.manual.is_empty());
    }

    #[test]
    fn test_preset_config_default() {
        let preset_config = PresetConfig::default();
        assert!(preset_config.auto_detect); // Default is true
        assert!(preset_config.manual.is_empty());
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_parse_config_with_comments() {
        let toml = r#"
# This is a comment
exclude = ["foo"]  # inline comment

# Another comment
[presets]
auto_detect = true  # yet another comment
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.exclude, vec!["foo"]);
        assert!(config.presets.auto_detect);
    }

    #[test]
    fn test_parse_empty_exclude_array() {
        let toml = r#"
exclude = []
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.exclude.is_empty());
    }

    #[test]
    fn test_parse_min_lines_zero() {
        let toml = r#"
min_lines = 0
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.min_lines, Some(0));
    }
}
