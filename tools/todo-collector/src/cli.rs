//! CLI argument definitions for TODO Collector

use clap::Parser;
use std::path::PathBuf;

/// TODO Collector - Collect and list TODO/FIXME/HACK comments from codebase
#[derive(Parser, Debug)]
#[command(name = "todo-collector")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Directory to scan for TODO comments
    #[arg(short = 's', long = "scan-dir")]
    pub scan_dir: Option<PathBuf>,

    /// Patterns to detect (comma-separated, e.g., "TODO,FIXME,HACK")
    #[arg(short = 'p', long = "pattern", default_value = "TODO,FIXME,HACK,XXX,NOTE")]
    pub pattern: String,

    /// Directories to exclude (comma-separated, e.g., "node_modules,target")
    #[arg(short = 'e', long = "exclude", default_value = "node_modules,target,.git,vendor,dist,build")]
    pub exclude: String,

    /// Output format (text or json)
    #[arg(short = 'o', long = "output", default_value = "text")]
    pub output: OutputFormat,

    /// Sort by priority (FIXME > TODO > HACK > XXX > NOTE)
    #[arg(long = "priority")]
    pub priority: bool,
}

/// Output format enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

impl Args {
    /// Get the list of patterns to detect
    pub fn get_patterns(&self) -> Vec<String> {
        self.pattern
            .split(',')
            .map(|s| s.trim().to_uppercase())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Get the list of directories to exclude
    pub fn get_excludes(&self) -> Vec<String> {
        self.exclude
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

