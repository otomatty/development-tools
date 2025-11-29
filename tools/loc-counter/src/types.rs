//! Type definitions for LOC Counter
//!
//! This module contains all the data structures used throughout the application.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Statistics for a single file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileStats {
    /// File path
    pub path: PathBuf,
    /// Detected language
    pub language: String,
    /// Number of code lines
    pub code: usize,
    /// Number of comment lines
    pub comments: usize,
    /// Number of blank lines
    pub blanks: usize,
    /// Total number of lines
    pub lines: usize,
}

impl FileStats {
    /// Create new FileStats
    pub fn new(path: PathBuf, language: String) -> Self {
        Self {
            path,
            language,
            ..Default::default()
        }
    }

    /// Add line counts
    pub fn add_counts(&mut self, code: usize, comments: usize, blanks: usize) {
        self.code += code;
        self.comments += comments;
        self.blanks += blanks;
        self.lines = self.code + self.comments + self.blanks;
    }
}

/// Statistics aggregated by language
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LanguageStats {
    /// Language name
    pub language: String,
    /// Number of files
    pub files: usize,
    /// Number of code lines
    pub code: usize,
    /// Number of comment lines
    pub comments: usize,
    /// Number of blank lines
    pub blanks: usize,
    /// Total number of lines
    pub lines: usize,
    /// Percentage of total lines
    pub percentage: f64,
}

impl LanguageStats {
    /// Create new LanguageStats
    pub fn new(language: String) -> Self {
        Self {
            language,
            ..Default::default()
        }
    }

    /// Add file stats to language stats
    pub fn add_file(&mut self, file_stats: &FileStats) {
        self.files += 1;
        self.code += file_stats.code;
        self.comments += file_stats.comments;
        self.blanks += file_stats.blanks;
        self.lines += file_stats.lines;
    }

    /// Calculate percentage based on total lines
    pub fn calculate_percentage(&mut self, total_lines: usize) {
        if total_lines > 0 {
            self.percentage = (self.lines as f64 / total_lines as f64) * 100.0;
        }
    }
}

/// Summary statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Summary {
    /// Total number of files
    pub total_files: usize,
    /// Total number of lines
    pub total_lines: usize,
    /// Total code lines
    pub code_lines: usize,
    /// Total comment lines
    pub comment_lines: usize,
    /// Total blank lines
    pub blank_lines: usize,
}

impl Summary {
    /// Create summary from language stats
    pub fn from_language_stats(stats: &[LanguageStats]) -> Self {
        let mut summary = Self::default();
        for lang in stats {
            summary.total_files += lang.files;
            summary.total_lines += lang.lines;
            summary.code_lines += lang.code;
            summary.comment_lines += lang.comments;
            summary.blank_lines += lang.blanks;
        }
        summary
    }
}

/// Complete LOC result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocResult {
    /// Summary statistics
    pub summary: Summary,
    /// Statistics by language
    pub by_language: Vec<LanguageStats>,
    /// Individual file statistics (when --by-file is used)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<FileStats>>,
}

impl LocResult {
    /// Create LocResult from file stats
    pub fn from_file_stats(file_stats: Vec<FileStats>, include_files: bool) -> Self {
        // Aggregate by language
        let mut lang_map: HashMap<String, LanguageStats> = HashMap::new();

        for fs in &file_stats {
            let entry = lang_map
                .entry(fs.language.clone())
                .or_insert_with(|| LanguageStats::new(fs.language.clone()));
            entry.add_file(fs);
        }

        // Convert to vec and calculate total
        let mut by_language: Vec<LanguageStats> = lang_map.into_values().collect();
        let total_lines: usize = by_language.iter().map(|l| l.lines).sum();

        // Calculate percentages
        for lang in &mut by_language {
            lang.calculate_percentage(total_lines);
        }

        // Sort by lines (default)
        by_language.sort_by(|a, b| b.lines.cmp(&a.lines));

        // Create summary
        let summary = Summary::from_language_stats(&by_language);

        Self {
            summary,
            by_language,
            files: if include_files {
                Some(file_stats)
            } else {
                None
            },
        }
    }

    /// Sort by specified field
    pub fn sort_by(&mut self, sort_key: &str) {
        match sort_key {
            "files" => self.by_language.sort_by(|a, b| b.files.cmp(&a.files)),
            "name" => self.by_language.sort_by(|a, b| a.language.cmp(&b.language)),
            "code" => self.by_language.sort_by(|a, b| b.code.cmp(&a.code)),
            _ => self.by_language.sort_by(|a, b| b.lines.cmp(&a.lines)), // default: lines
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_stats_new() {
        let fs = FileStats::new(PathBuf::from("test.rs"), "Rust".to_string());
        assert_eq!(fs.path, PathBuf::from("test.rs"));
        assert_eq!(fs.language, "Rust");
        assert_eq!(fs.code, 0);
        assert_eq!(fs.comments, 0);
        assert_eq!(fs.blanks, 0);
        assert_eq!(fs.lines, 0);
    }

    #[test]
    fn test_file_stats_add_counts() {
        let mut fs = FileStats::new(PathBuf::from("test.rs"), "Rust".to_string());
        fs.add_counts(100, 20, 10);
        assert_eq!(fs.code, 100);
        assert_eq!(fs.comments, 20);
        assert_eq!(fs.blanks, 10);
        assert_eq!(fs.lines, 130);
    }

    #[test]
    fn test_language_stats_add_file() {
        let mut lang = LanguageStats::new("Rust".to_string());
        let mut fs1 = FileStats::new(PathBuf::from("a.rs"), "Rust".to_string());
        fs1.add_counts(50, 10, 5);
        let mut fs2 = FileStats::new(PathBuf::from("b.rs"), "Rust".to_string());
        fs2.add_counts(30, 5, 3);

        lang.add_file(&fs1);
        lang.add_file(&fs2);

        assert_eq!(lang.files, 2);
        assert_eq!(lang.code, 80);
        assert_eq!(lang.comments, 15);
        assert_eq!(lang.blanks, 8);
        assert_eq!(lang.lines, 103);
    }

    #[test]
    fn test_language_stats_calculate_percentage() {
        let mut lang = LanguageStats::new("Rust".to_string());
        lang.lines = 50;
        lang.calculate_percentage(100);
        assert!((lang.percentage - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_summary_from_language_stats() {
        let mut rust = LanguageStats::new("Rust".to_string());
        rust.files = 3;
        rust.code = 100;
        rust.comments = 20;
        rust.blanks = 10;
        rust.lines = 130;

        let mut ts = LanguageStats::new("TypeScript".to_string());
        ts.files = 2;
        ts.code = 50;
        ts.comments = 10;
        ts.blanks = 5;
        ts.lines = 65;

        let summary = Summary::from_language_stats(&[rust, ts]);

        assert_eq!(summary.total_files, 5);
        assert_eq!(summary.total_lines, 195);
        assert_eq!(summary.code_lines, 150);
        assert_eq!(summary.comment_lines, 30);
        assert_eq!(summary.blank_lines, 15);
    }

    #[test]
    fn test_loc_result_from_file_stats() {
        let mut fs1 = FileStats::new(PathBuf::from("a.rs"), "Rust".to_string());
        fs1.add_counts(100, 20, 10);
        let mut fs2 = FileStats::new(PathBuf::from("b.ts"), "TypeScript".to_string());
        fs2.add_counts(50, 10, 5);

        let result = LocResult::from_file_stats(vec![fs1, fs2], false);

        assert_eq!(result.summary.total_files, 2);
        assert_eq!(result.summary.total_lines, 195);
        assert_eq!(result.by_language.len(), 2);
        assert!(result.files.is_none());
    }

    #[test]
    fn test_loc_result_with_files() {
        let mut fs1 = FileStats::new(PathBuf::from("a.rs"), "Rust".to_string());
        fs1.add_counts(100, 20, 10);

        let result = LocResult::from_file_stats(vec![fs1], true);

        assert!(result.files.is_some());
        assert_eq!(result.files.unwrap().len(), 1);
    }

    #[test]
    fn test_loc_result_sort_by() {
        let mut fs1 = FileStats::new(PathBuf::from("a.rs"), "Rust".to_string());
        fs1.add_counts(100, 20, 10);
        let mut fs2 = FileStats::new(PathBuf::from("b.ts"), "TypeScript".to_string());
        fs2.add_counts(200, 30, 15);
        let mut fs3 = FileStats::new(PathBuf::from("c.py"), "Python".to_string());
        fs3.add_counts(50, 5, 3);

        let mut result = LocResult::from_file_stats(vec![fs1, fs2, fs3], false);

        // Sort by name
        result.sort_by("name");
        assert_eq!(result.by_language[0].language, "Python");
        assert_eq!(result.by_language[1].language, "Rust");
        assert_eq!(result.by_language[2].language, "TypeScript");

        // Sort by lines (default)
        result.sort_by("lines");
        assert_eq!(result.by_language[0].language, "TypeScript");
    }
}
