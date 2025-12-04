//! Mock Server Page Utility Functions
//!
//! Helper functions for the mock server page.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   └─ src/components/pages/mock_server/mod.rs
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

use crate::types::ServerStatus;

/// Get status color class based on server status
pub fn get_status_color(status: &ServerStatus) -> &'static str {
    match status {
        ServerStatus::Running => "text-green-400",
        ServerStatus::Stopped => "text-slate-400",
    }
}

/// Get status background class based on server status
pub fn get_status_bg(status: &ServerStatus) -> &'static str {
    match status {
        ServerStatus::Running => "bg-green-500/20",
        ServerStatus::Stopped => "bg-slate-500/20",
    }
}

/// Get status text based on server status
pub fn get_status_text(status: &ServerStatus) -> &'static str {
    match status {
        ServerStatus::Running => "Running",
        ServerStatus::Stopped => "Stopped",
    }
}

/// Format file size for display
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

/// Get file icon based on extension
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
