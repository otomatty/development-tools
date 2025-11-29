//! CLI argument parsing for Large File Finder
//!
//! This module defines the command-line interface using clap.

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Output format options
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Plain text output with colors
    Text,
    /// JSON output for programmatic use
    #[default]
    Json,
}

/// Large File Finder - Find files with more than a specified number of lines
#[derive(Parser, Debug)]
#[command(name = "large-file-finder")]
#[command(author = "otomatty")]
#[command(version = "1.0.0")]
#[command(about = "Find files with more than a specified number of lines, sorted by line count")]
#[command(long_about = "A CLI tool to detect files that exceed a specified line count threshold.\n\nFeatures:\n- Automatically respects .gitignore patterns\n- Sorts results by line count in descending order\n- Supports text and JSON output formats\n- Built-in presets for common languages (rust, node, python, go)")]
pub struct Args {
    /// Minimum number of lines to consider a file as "large"
    #[arg(short = 'm', long = "min-lines")]
    pub min_lines: Option<usize>,

    /// Show only top N files
    #[arg(short = 't', long = "top")]
    pub top: Option<usize>,

    /// Output format
    #[arg(short = 'o', long = "output", default_value = "json")]
    pub output: OutputFormat,

    /// Presets to use (comma-separated: common, rust, node, python, go)
    #[arg(short = 'p', long = "preset")]
    pub preset: Option<String>,

    /// Disable auto-detection of presets based on project files
    #[arg(long = "no-auto-detect")]
    pub no_auto_detect: bool,

    /// Additional exclude patterns (comma-separated, glob format)
    #[arg(short = 'e', long = "exclude")]
    pub exclude: Option<String>,

    /// Directory to scan (default: current directory)
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,
}

impl Args {
    /// Get the path to scan, defaulting to current directory
    pub fn get_scan_path(&self) -> PathBuf {
        match &self.path {
            Some(p) => {
                // Expand ~ to home directory
                if p.starts_with("~") {
                    match dirs::home_dir() {
                        Some(home) => {
                            let rest = p.strip_prefix("~").unwrap();
                            return home.join(rest.strip_prefix("/").unwrap_or(rest));
                        }
                        None => {
                            eprintln!("Warning: Could not determine home directory. Using path as-is.");
                            return p.clone();
                        }
                    }
                }
                p.clone()
            }
            None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }

    /// Get exclude patterns as a vector
    pub fn get_excludes(&self) -> Vec<String> {
        match &self.exclude {
            Some(e) => e.split(',').map(|s| s.trim().to_string()).collect(),
            None => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_min_lines() {
        let args = Args::parse_from(["large-file-finder"]);
        assert_eq!(args.min_lines, None);
    }

    #[test]
    fn test_custom_min_lines() {
        let args = Args::parse_from(["large-file-finder", "--min-lines", "1000"]);
        assert_eq!(args.min_lines, Some(1000));
    }

    #[test]
    fn test_top_option() {
        let args = Args::parse_from(["large-file-finder", "--top", "10"]);
        assert_eq!(args.top, Some(10));
    }

    #[test]
    fn test_output_format() {
        let args = Args::parse_from(["large-file-finder", "--output", "text"]);
        assert_eq!(args.output, OutputFormat::Text);
    }

    #[test]
    fn test_exclude_patterns() {
        let args = Args::parse_from(["large-file-finder", "--exclude", "generated/**,*.g.dart"]);
        let excludes = args.get_excludes();
        assert_eq!(excludes.len(), 2);
        assert_eq!(excludes[0], "generated/**");
        assert_eq!(excludes[1], "*.g.dart");
    }

    #[test]
    fn test_path_argument() {
        let args = Args::parse_from(["large-file-finder", "/some/path"]);
        assert_eq!(args.path, Some(PathBuf::from("/some/path")));
    }

    #[test]
    fn test_preset_option() {
        let args = Args::parse_from(["large-file-finder", "--preset", "rust,node"]);
        assert_eq!(args.preset, Some("rust,node".to_string()));
    }

    #[test]
    fn test_auto_detect_default() {
        let args = Args::parse_from(["large-file-finder"]);
        assert!(!args.no_auto_detect); // default: auto-detect enabled
    }

    #[test]
    fn test_auto_detect_disabled() {
        let args = Args::parse_from(["large-file-finder", "--no-auto-detect"]);
        assert!(args.no_auto_detect); // auto-detect disabled
    }
}
