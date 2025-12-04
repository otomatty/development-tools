//! Mock Server Page Utility Functions
//!
//! Helper functions for the mock server page.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   └─ src/components/pages/mock_server/mod.rs
//! Imports (shared modules):
//!   └─ crate::components::pages::shared_utils::{format_file_size, get_file_icon}
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

pub use crate::components::pages::shared_utils::{format_file_size, get_file_icon};
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
