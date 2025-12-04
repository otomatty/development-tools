//! Project Dashboard Utility Functions
//!
//! Helper functions for the project dashboard page.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   └─ src/components/pages/project_dashboard/mod.rs
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

use crate::types::issue::{CachedIssue, IssueStatus, KanbanBoard};

/// Add issue to kanban board based on its status
pub fn add_issue_to_board(board: &mut KanbanBoard, issue: CachedIssue) {
    let status = issue.get_status();
    match status {
        IssueStatus::Backlog => board.backlog.push(issue),
        IssueStatus::Todo => board.todo.push(issue),
        IssueStatus::InProgress => board.in_progress.push(issue),
        IssueStatus::InReview => board.in_review.push(issue),
        IssueStatus::Done => board.done.push(issue),
        IssueStatus::Cancelled => board.cancelled.push(issue),
    }
}

/// Get status column name for display
pub fn get_status_column_name(status: &IssueStatus) -> &'static str {
    match status {
        IssueStatus::Backlog => "Backlog",
        IssueStatus::Todo => "Todo",
        IssueStatus::InProgress => "In Progress",
        IssueStatus::InReview => "In Review",
        IssueStatus::Done => "Done",
        IssueStatus::Cancelled => "Cancelled",
    }
}

/// Get status color class
pub fn get_status_color(status: &IssueStatus) -> &'static str {
    match status {
        IssueStatus::Backlog => "text-slate-400",
        IssueStatus::Todo => "text-blue-400",
        IssueStatus::InProgress => "text-yellow-400",
        IssueStatus::InReview => "text-purple-400",
        IssueStatus::Done => "text-green-400",
        IssueStatus::Cancelled => "text-red-400",
    }
}

/// Get issue priority color
pub fn get_priority_color(priority: Option<&str>) -> &'static str {
    match priority {
        Some("critical") => "text-red-500",
        Some("high") => "text-orange-400",
        Some("medium") => "text-yellow-400",
        Some("low") => "text-blue-400",
        _ => "text-slate-400",
    }
}
