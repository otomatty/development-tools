//! Type definitions for Large File Finder
//!
//! This module defines the core types used throughout the application.

use serde::Serialize;
use std::path::PathBuf;

/// Information about a single file with its line count
#[derive(Debug, Clone, Serialize)]
pub struct FileInfo {
    /// Relative path from the scan root
    pub path: String,
    /// Absolute path to the file
    #[serde(skip)]
    #[allow(dead_code)]
    pub absolute_path: PathBuf,
    /// Number of lines in the file
    pub lines: usize,
}

impl FileInfo {
    /// Create a new FileInfo
    pub fn new(path: String, absolute_path: PathBuf, lines: usize) -> Self {
        Self {
            path,
            absolute_path,
            lines,
        }
    }
}

/// Result of scanning for large files
#[derive(Debug, Clone, Serialize)]
pub struct ScanResult {
    /// Directory that was scanned
    pub scan_directory: String,
    /// Minimum line threshold used
    pub min_lines: usize,
    /// Presets used for scanning
    pub presets: Vec<String>,
    /// Files that exceed the threshold, sorted by line count descending
    pub files: Vec<FileInfo>,
    /// Total number of files scanned
    pub total_files_scanned: usize,
    /// Total lines in all large files
    pub total_lines: usize,
}

impl ScanResult {
    /// Create a new ScanResult
    pub fn new(scan_directory: String, min_lines: usize) -> Self {
        Self {
            scan_directory,
            min_lines,
            presets: Vec::new(),
            files: Vec::new(),
            total_files_scanned: 0,
            total_lines: 0,
        }
    }

    /// Add a file to the result if it exceeds the threshold
    pub fn add_file(&mut self, file: FileInfo) {
        if file.lines >= self.min_lines {
            self.total_lines += file.lines;
            self.files.push(file);
        }
    }

    /// Increment the total files scanned counter
    pub fn increment_scanned(&mut self) {
        self.total_files_scanned += 1;
    }

    /// Sort files by line count in descending order
    pub fn sort_by_lines(&mut self) {
        self.files.sort_by(|a, b| b.lines.cmp(&a.lines));
    }

    /// Limit the results to top N files
    pub fn limit(&mut self, top: usize) {
        if self.files.len() > top {
            self.files.truncate(top);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_info_creation() {
        let info = FileInfo::new(
            "src/main.rs".to_string(),
            PathBuf::from("/project/src/main.rs"),
            100,
        );
        assert_eq!(info.path, "src/main.rs");
        assert_eq!(info.lines, 100);
    }

    #[test]
    fn test_scan_result_add_file() {
        let mut result = ScanResult::new("/project".to_string(), 50);
        
        // File below threshold should not be added
        let small_file = FileInfo::new("small.rs".to_string(), PathBuf::from("/project/small.rs"), 30);
        result.add_file(small_file);
        assert_eq!(result.files.len(), 0);
        
        // File at threshold should be added
        let threshold_file = FileInfo::new("threshold.rs".to_string(), PathBuf::from("/project/threshold.rs"), 50);
        result.add_file(threshold_file);
        assert_eq!(result.files.len(), 1);
        
        // File above threshold should be added
        let large_file = FileInfo::new("large.rs".to_string(), PathBuf::from("/project/large.rs"), 100);
        result.add_file(large_file);
        assert_eq!(result.files.len(), 2);
        assert_eq!(result.total_lines, 150);
    }

    #[test]
    fn test_scan_result_sort() {
        let mut result = ScanResult::new("/project".to_string(), 10);
        result.add_file(FileInfo::new("a.rs".to_string(), PathBuf::from("/a.rs"), 50));
        result.add_file(FileInfo::new("b.rs".to_string(), PathBuf::from("/b.rs"), 100));
        result.add_file(FileInfo::new("c.rs".to_string(), PathBuf::from("/c.rs"), 75));
        
        result.sort_by_lines();
        
        assert_eq!(result.files[0].lines, 100);
        assert_eq!(result.files[1].lines, 75);
        assert_eq!(result.files[2].lines, 50);
    }

    #[test]
    fn test_scan_result_limit() {
        let mut result = ScanResult::new("/project".to_string(), 10);
        for i in 0..10 {
            result.add_file(FileInfo::new(format!("{}.rs", i), PathBuf::from(format!("/{}.rs", i)), 20 + i));
        }
        
        result.limit(5);
        assert_eq!(result.files.len(), 5);
    }
}
