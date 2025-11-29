//! CLI argument parsing for LOC Counter
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

/// Sort order options
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum SortOrder {
    /// Sort by total lines (default)
    #[default]
    Lines,
    /// Sort by number of files
    Files,
    /// Sort by language name
    Name,
    /// Sort by code lines
    Code,
}

impl SortOrder {
    /// Convert to string key for LocResult::sort_by
    pub fn as_str(&self) -> &'static str {
        match self {
            SortOrder::Lines => "lines",
            SortOrder::Files => "files",
            SortOrder::Name => "name",
            SortOrder::Code => "code",
        }
    }
}

/// LOC Counter - Count lines of code by language
#[derive(Parser, Debug)]
#[command(name = "loc-counter")]
#[command(author = "otomatty")]
#[command(version = "1.0.0")]
#[command(about = "Count lines of code by language with code/comment/blank line statistics")]
pub struct Args {
    /// Directory to scan (default: current directory)
    #[arg(short = 's', long = "scan-dir")]
    pub scan_dir: Option<PathBuf>,

    /// Exclude patterns (comma-separated, glob format)
    #[arg(short = 'e', long = "exclude", default_value = "node_modules,target,.git,vendor,dist,build,__pycache__,.venv")]
    pub exclude: String,

    /// Output format
    #[arg(short = 'o', long = "output", default_value = "json")]
    pub output: OutputFormat,

    /// Sort order
    #[arg(long = "sort", default_value = "lines")]
    pub sort: SortOrder,

    /// Show individual file statistics
    #[arg(long = "by-file")]
    pub by_file: bool,
}

impl Args {
    /// Get exclude patterns as a vector
    pub fn get_excludes(&self) -> Vec<String> {
        self.exclude
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_excludes() {
        let args = Args {
            scan_dir: None,
            exclude: "node_modules,target,.git".to_string(),
            output: OutputFormat::Json,
            sort: SortOrder::Lines,
            by_file: false,
        };

        let excludes = args.get_excludes();
        assert_eq!(excludes.len(), 3);
        assert!(excludes.contains(&"node_modules".to_string()));
        assert!(excludes.contains(&"target".to_string()));
        assert!(excludes.contains(&".git".to_string()));
    }

    #[test]
    fn test_get_excludes_with_spaces() {
        let args = Args {
            scan_dir: None,
            exclude: " node_modules , target , .git ".to_string(),
            output: OutputFormat::Json,
            sort: SortOrder::Lines,
            by_file: false,
        };

        let excludes = args.get_excludes();
        assert_eq!(excludes.len(), 3);
        assert!(excludes.contains(&"node_modules".to_string()));
    }

    #[test]
    fn test_get_excludes_empty() {
        let args = Args {
            scan_dir: None,
            exclude: "".to_string(),
            output: OutputFormat::Json,
            sort: SortOrder::Lines,
            by_file: false,
        };

        let excludes = args.get_excludes();
        assert!(excludes.is_empty());
    }

    #[test]
    fn test_sort_order_as_str() {
        assert_eq!(SortOrder::Lines.as_str(), "lines");
        assert_eq!(SortOrder::Files.as_str(), "files");
        assert_eq!(SortOrder::Name.as_str(), "name");
        assert_eq!(SortOrder::Code.as_str(), "code");
    }
}
