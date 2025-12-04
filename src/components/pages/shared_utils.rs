//! Shared Page Utility Functions
//!
//! Common utility functions shared across multiple page components.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   ├─ src/components/pages/mock_server/utils.rs
//!   ├─ src/components/pages/project_dashboard/utils.rs
//!   ├─ src/components/pages/projects/utils.rs
//!   └─ src/components/pages/settings/utils.rs
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

/// Format file size for display
///
/// Converts bytes to human-readable format (B, KB, MB, GB)
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Get file icon based on file extension
///
/// Returns Lucide icon name based on the file extension
pub fn get_file_icon(filename: &str) -> &'static str {
    let ext = filename.split('.').last().unwrap_or("");
    match ext.to_lowercase().as_str() {
        "html" | "htm" => "file-code",
        "css" => "file-code",
        "js" | "ts" | "jsx" | "tsx" => "file-code",
        "json" => "file-json",
        "md" => "file-text",
        "txt" => "file-text",
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" => "image",
        "pdf" => "file-text",
        _ => "file",
    }
}

/// Format timestamp for display
///
/// Converts a timestamp to a human-readable relative time format
pub fn format_last_synced(timestamp: i64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let diff = now - timestamp;
    let seconds = 60;
    let minutes = 60 * seconds;
    let hours = 24 * minutes;
    let days = 30 * hours;

    if diff < seconds {
        "just now".to_string()
    } else if diff < minutes {
        format!("{} seconds ago", diff / seconds)
    } else if diff < hours {
        format!("{} minutes ago", diff / minutes)
    } else if diff < days {
        format!("{} hours ago", diff / hours)
    } else {
        format!("{} days ago", diff / days)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(100), "100 B");
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_get_file_icon() {
        assert_eq!(get_file_icon("test.html"), "file-code");
        assert_eq!(get_file_icon("style.css"), "file-code");
        assert_eq!(get_file_icon("script.ts"), "file-code");
        assert_eq!(get_file_icon("data.json"), "file-json");
        assert_eq!(get_file_icon("readme.md"), "file-text");
        assert_eq!(get_file_icon("image.png"), "image");
        assert_eq!(get_file_icon("unknown.xyz"), "file");
    }
}
