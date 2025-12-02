//! Issue management types for frontend
//!
//! This module defines the data structures used in the frontend
//! for the GitHub Issue management feature.
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this module):
//!   ├─ src/types/mod.rs
//!   ├─ src/components/issues/*.rs
//!   └─ src/tauri_api.rs
//! Related Documentation:
//!   └─ docs/03_plans/issue-management/20251201_implementation_plan.md

use serde::{Deserialize, Serialize};

/// Issue status (maps to GitHub labels)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum IssueStatus {
    #[default]
    Backlog,
    Todo,
    InProgress,
    InReview,
    Done,
    Cancelled,
}

impl IssueStatus {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            IssueStatus::Backlog => "Backlog",
            IssueStatus::Todo => "Todo",
            IssueStatus::InProgress => "In Progress",
            IssueStatus::InReview => "In Review",
            IssueStatus::Done => "Done",
            IssueStatus::Cancelled => "Cancelled",
        }
    }

    /// Get CSS color class
    pub fn color_class(&self) -> &'static str {
        match self {
            IssueStatus::Backlog => "bg-gray-400",
            IssueStatus::Todo => "bg-blue-500",
            IssueStatus::InProgress => "bg-yellow-500",
            IssueStatus::InReview => "bg-purple-500",
            IssueStatus::Done => "bg-green-500",
            IssueStatus::Cancelled => "bg-gray-500",
        }
    }

    /// Get all statuses in order
    pub fn all() -> Vec<IssueStatus> {
        vec![
            IssueStatus::Backlog,
            IssueStatus::Todo,
            IssueStatus::InProgress,
            IssueStatus::InReview,
            IssueStatus::Done,
            IssueStatus::Cancelled,
        ]
    }

    /// Get visible statuses (for kanban board display)
    pub fn visible() -> Vec<IssueStatus> {
        vec![
            IssueStatus::Backlog,
            IssueStatus::Todo,
            IssueStatus::InProgress,
            IssueStatus::InReview,
            IssueStatus::Done,
            IssueStatus::Cancelled,
        ]
    }
}

impl std::fmt::Display for IssueStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Issue priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssuePriority {
    High,
    Medium,
    Low,
}

impl IssuePriority {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            IssuePriority::High => "High",
            IssuePriority::Medium => "Medium",
            IssuePriority::Low => "Low",
        }
    }

    /// Get emoji indicator
    pub fn emoji(&self) -> &'static str {
        match self {
            IssuePriority::High => "🔴",
            IssuePriority::Medium => "🟡",
            IssuePriority::Low => "🟢",
        }
    }

    /// Get CSS color class
    pub fn color_class(&self) -> &'static str {
        match self {
            IssuePriority::High => "text-red-500",
            IssuePriority::Medium => "text-yellow-500",
            IssuePriority::Low => "text-green-500",
        }
    }
}

/// Project model (1 project = 1 repository)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub github_repo_id: Option<i64>,
    pub repo_owner: Option<String>,
    pub repo_name: Option<String>,
    pub repo_full_name: Option<String>,
    pub is_actions_setup: bool,
    pub last_synced_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Project {
    /// Check if repository is linked
    pub fn is_linked(&self) -> bool {
        self.github_repo_id.is_some()
    }

    /// Get repository display name
    pub fn repo_display_name(&self) -> Option<&str> {
        self.repo_full_name.as_deref()
    }
}

/// Project with additional stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectWithStats {
    #[serde(flatten)]
    pub project: Project,
    pub open_issues_count: i32,
    pub total_issues_count: i32,
}

/// Cached issue model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedIssue {
    pub id: i64,
    pub project_id: i64,
    pub github_issue_id: i64,
    pub number: i32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub status: String,
    pub priority: Option<String>,
    pub assignee_login: Option<String>,
    pub assignee_avatar_url: Option<String>,
    pub labels_json: Option<String>,
    pub html_url: Option<String>,
    pub github_created_at: Option<String>,
    pub github_updated_at: Option<String>,
    pub cached_at: String,
}

impl CachedIssue {
    /// Get parsed status
    pub fn get_status(&self) -> IssueStatus {
        match self.status.as_str() {
            "backlog" => IssueStatus::Backlog,
            "todo" => IssueStatus::Todo,
            "in-progress" => IssueStatus::InProgress,
            "in-review" => IssueStatus::InReview,
            "done" => IssueStatus::Done,
            "cancelled" => IssueStatus::Cancelled,
            _ => IssueStatus::Backlog,
        }
    }

    /// Get parsed priority
    pub fn get_priority(&self) -> Option<IssuePriority> {
        self.priority.as_ref().and_then(|p| match p.as_str() {
            "high" => Some(IssuePriority::High),
            "medium" => Some(IssuePriority::Medium),
            "low" => Some(IssuePriority::Low),
            _ => None,
        })
    }

    /// Get parsed labels
    pub fn get_labels(&self) -> Vec<String> {
        self.labels_json
            .as_ref()
            .and_then(|json| serde_json::from_str(json).ok())
            .unwrap_or_default()
    }

    /// Check if issue is open
    pub fn is_open(&self) -> bool {
        self.state == "open"
    }

    /// Check if issue was updated within the specified number of days
    /// Uses JavaScript Date for WASM compatibility
    pub fn is_updated_within_days(&self, days: i64) -> bool {
        let updated_at = match &self.github_updated_at {
            Some(s) => s,
            None => return true, // If no date, show it (conservative approach)
        };

        // Parse RFC3339 date using JavaScript Date
        let updated_ms = js_sys::Date::parse(updated_at);
        if updated_ms.is_nan() {
            return true; // If parse fails, show it
        }

        let now_ms = js_sys::Date::now();
        let days_ms = (days as f64) * 24.0 * 60.0 * 60.0 * 1000.0;
        
        (now_ms - updated_ms) <= days_ms
    }

    /// Check if issue is a completed status (Done or Cancelled)
    pub fn is_completed_status(&self) -> bool {
        matches!(self.status.as_str(), "done" | "cancelled")
    }
}

/// Issues grouped by status for kanban display
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KanbanBoard {
    pub backlog: Vec<CachedIssue>,
    pub todo: Vec<CachedIssue>,
    pub in_progress: Vec<CachedIssue>,
    pub in_review: Vec<CachedIssue>,
    pub done: Vec<CachedIssue>,
    pub cancelled: Vec<CachedIssue>,
}

impl KanbanBoard {
    /// Create kanban board from issues list
    pub fn from_issues(issues: Vec<CachedIssue>) -> Self {
        let mut board = KanbanBoard::default();

        for issue in issues {
            match issue.get_status() {
                IssueStatus::Backlog => board.backlog.push(issue),
                IssueStatus::Todo => board.todo.push(issue),
                IssueStatus::InProgress => board.in_progress.push(issue),
                IssueStatus::InReview => board.in_review.push(issue),
                IssueStatus::Done => board.done.push(issue),
                IssueStatus::Cancelled => board.cancelled.push(issue),
            }
        }

        board
    }

    /// Get issues for a specific status
    pub fn get_issues(&self, status: IssueStatus) -> &Vec<CachedIssue> {
        match status {
            IssueStatus::Backlog => &self.backlog,
            IssueStatus::Todo => &self.todo,
            IssueStatus::InProgress => &self.in_progress,
            IssueStatus::InReview => &self.in_review,
            IssueStatus::Done => &self.done,
            IssueStatus::Cancelled => &self.cancelled,
        }
    }

    /// Get count for a specific status
    pub fn count(&self, status: IssueStatus) -> usize {
        self.get_issues(status).len()
    }

    /// Get total count
    pub fn total(&self) -> usize {
        self.backlog.len()
            + self.todo.len()
            + self.in_progress.len()
            + self.in_review.len()
            + self.done.len()
            + self.cancelled.len()
    }
}

/// GitHub repository info for linking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub owner: String,
    pub description: Option<String>,
    pub html_url: String,
    pub private: bool,
    pub open_issues_count: i32,
}
