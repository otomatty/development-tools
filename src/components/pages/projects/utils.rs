//! Projects Page Utility Functions
//!
//! Helper functions for the projects page.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   └─ src/components/pages/projects/mod.rs
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

use crate::types::issue::Project;

/// Get project status text
pub fn get_project_status_text(project: &Project) -> &'static str {
    if project.is_linked() {
        "Linked"
    } else {
        "Not linked"
    }
}

/// Get project status color class
pub fn get_project_status_color(project: &Project) -> &'static str {
    if project.is_linked() {
        "bg-green-500"
    } else {
        "bg-yellow-500"
    }
}

/// Format last synced time
pub fn format_last_synced(last_synced_at: Option<&str>) -> String {
    match last_synced_at {
        Some(synced) => format!("Synced: {}", synced),
        None => "Never synced".to_string(),
    }
}
