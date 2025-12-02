//! Project and Issue models
//!
//! This module defines data structures for the GitHub Issue management feature.
//! Related Issue: GitHub Issue #59 - GitHub Issue管理機能（Linear風カンバン）
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this module):
//!   ├─ src-tauri/src/database/models/mod.rs
//!   ├─ src-tauri/src/commands/issues.rs
//!   └─ src-tauri/src/github/issues.rs
//! Related Documentation:
//!   └─ docs/03_plans/issue-management/20251201_implementation_plan.md

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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
    /// Convert to GitHub label name
    pub fn to_label(&self) -> &'static str {
        match self {
            IssueStatus::Backlog => "status:backlog",
            IssueStatus::Todo => "status:todo",
            IssueStatus::InProgress => "status:in-progress",
            IssueStatus::InReview => "status:in-review",
            IssueStatus::Done => "status:done",
            IssueStatus::Cancelled => "status:cancelled",
        }
    }

    /// Parse from GitHub label name
    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "status:backlog" => Some(IssueStatus::Backlog),
            "status:todo" => Some(IssueStatus::Todo),
            "status:in-progress" => Some(IssueStatus::InProgress),
            "status:in-review" => Some(IssueStatus::InReview),
            "status:done" => Some(IssueStatus::Done),
            "status:cancelled" => Some(IssueStatus::Cancelled),
            _ => None,
        }
    }

    /// Get all status labels
    pub fn all_labels() -> Vec<&'static str> {
        vec![
            "status:backlog",
            "status:todo",
            "status:in-progress",
            "status:in-review",
            "status:done",
            "status:cancelled",
        ]
    }

    /// Get label color
    pub fn label_color(&self) -> &'static str {
        match self {
            IssueStatus::Backlog => "E2E2E2",
            IssueStatus::Todo => "0052CC",
            IssueStatus::InProgress => "FBCA04",
            IssueStatus::InReview => "7C3AED",
            IssueStatus::Done => "0E8A16",
            IssueStatus::Cancelled => "6A737D",
        }
    }

    /// Get label description
    pub fn label_description(&self) -> &'static str {
        match self {
            IssueStatus::Backlog => "Issue is in backlog",
            IssueStatus::Todo => "Issue is planned",
            IssueStatus::InProgress => "Issue is being worked on",
            IssueStatus::InReview => "Issue is in review",
            IssueStatus::Done => "Issue is completed",
            IssueStatus::Cancelled => "Issue is cancelled",
        }
    }
}

impl std::fmt::Display for IssueStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueStatus::Backlog => write!(f, "backlog"),
            IssueStatus::Todo => write!(f, "todo"),
            IssueStatus::InProgress => write!(f, "in-progress"),
            IssueStatus::InReview => write!(f, "in-review"),
            IssueStatus::Done => write!(f, "done"),
            IssueStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for IssueStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "backlog" => Ok(IssueStatus::Backlog),
            "todo" => Ok(IssueStatus::Todo),
            "in-progress" => Ok(IssueStatus::InProgress),
            "in-review" => Ok(IssueStatus::InReview),
            "done" => Ok(IssueStatus::Done),
            "cancelled" => Ok(IssueStatus::Cancelled),
            _ => Err(format!("Unknown status: {}", s)),
        }
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
    /// Convert to GitHub label name
    pub fn to_label(&self) -> &'static str {
        match self {
            IssuePriority::High => "priority:high",
            IssuePriority::Medium => "priority:medium",
            IssuePriority::Low => "priority:low",
        }
    }

    /// Parse from GitHub label name
    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "priority:high" => Some(IssuePriority::High),
            "priority:medium" => Some(IssuePriority::Medium),
            "priority:low" => Some(IssuePriority::Low),
            _ => None,
        }
    }

    /// Get label color
    pub fn label_color(&self) -> &'static str {
        match self {
            IssuePriority::High => "D73A4A",
            IssuePriority::Medium => "FBCA04",
            IssuePriority::Low => "0E8A16",
        }
    }

    /// Get label description
    pub fn label_description(&self) -> &'static str {
        match self {
            IssuePriority::High => "High priority",
            IssuePriority::Medium => "Medium priority",
            IssuePriority::Low => "Low priority",
        }
    }
}

impl std::fmt::Display for IssuePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssuePriority::High => write!(f, "high"),
            IssuePriority::Medium => write!(f, "medium"),
            IssuePriority::Low => write!(f, "low"),
        }
    }
}

impl std::str::FromStr for IssuePriority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "high" => Ok(IssuePriority::High),
            "medium" => Ok(IssuePriority::Medium),
            "low" => Ok(IssuePriority::Low),
            _ => Err(format!("Unknown priority: {}", s)),
        }
    }
}

/// Project model (1 project = 1 repository)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub github_repo_id: Option<i64>,
    pub repo_owner: Option<String>,
    pub repo_name: Option<String>,
    pub repo_full_name: Option<String>,
    #[sqlx(rename = "is_actions_setup")]
    pub is_actions_setup: bool,
    pub last_synced_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Project with additional info for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectWithStats {
    #[serde(flatten)]
    pub project: Project,
    pub open_issues_count: i32,
    pub total_issues_count: i32,
}

/// Request to create a new project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Request to update a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Request to link a repository to a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkRepositoryRequest {
    pub project_id: i64,
    pub owner: String,
    pub repo: String,
}

/// Cached issue model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
        self.status.parse().unwrap_or_default()
    }

    /// Get parsed priority
    pub fn get_priority(&self) -> Option<IssuePriority> {
        self.priority.as_ref().and_then(|p| p.parse().ok())
    }

    /// Get parsed labels
    pub fn get_labels(&self) -> Vec<String> {
        self.labels_json
            .as_ref()
            .and_then(|json| serde_json::from_str(json).ok())
            .unwrap_or_default()
    }
}

/// Request to update issue status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIssueStatusRequest {
    pub project_id: i64,
    pub issue_number: i32,
    pub status: IssueStatus,
}

/// Request to create a new issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssueRequest {
    pub project_id: i64,
    pub title: String,
    pub body: Option<String>,
    pub status: Option<IssueStatus>,
    pub priority: Option<IssuePriority>,
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
}

/// Label definition for creating status/priority labels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelDefinition {
    pub name: String,
    pub color: String,
    pub description: String,
}

impl LabelDefinition {
    /// Get all status label definitions
    pub fn status_labels() -> Vec<Self> {
        vec![
            LabelDefinition {
                name: "status:backlog".to_string(),
                color: "E2E2E2".to_string(),
                description: "Issue is in backlog".to_string(),
            },
            LabelDefinition {
                name: "status:todo".to_string(),
                color: "0052CC".to_string(),
                description: "Issue is planned".to_string(),
            },
            LabelDefinition {
                name: "status:in-progress".to_string(),
                color: "FBCA04".to_string(),
                description: "Issue is being worked on".to_string(),
            },
            LabelDefinition {
                name: "status:in-review".to_string(),
                color: "7C3AED".to_string(),
                description: "Issue is in review".to_string(),
            },
            LabelDefinition {
                name: "status:done".to_string(),
                color: "0E8A16".to_string(),
                description: "Issue is completed".to_string(),
            },
            LabelDefinition {
                name: "status:cancelled".to_string(),
                color: "6A737D".to_string(),
                description: "Issue is cancelled".to_string(),
            },
        ]
    }

    /// Get all priority label definitions
    pub fn priority_labels() -> Vec<Self> {
        vec![
            LabelDefinition {
                name: "priority:high".to_string(),
                color: "D73A4A".to_string(),
                description: "High priority".to_string(),
            },
            LabelDefinition {
                name: "priority:medium".to_string(),
                color: "FBCA04".to_string(),
                description: "Medium priority".to_string(),
            },
            LabelDefinition {
                name: "priority:low".to_string(),
                color: "0E8A16".to_string(),
                description: "Low priority".to_string(),
            },
        ]
    }

    /// Get all label definitions (status + priority)
    pub fn all_labels() -> Vec<Self> {
        let mut labels = Self::status_labels();
        labels.extend(Self::priority_labels());
        labels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_status_to_label() {
        assert_eq!(IssueStatus::Backlog.to_label(), "status:backlog");
        assert_eq!(IssueStatus::InProgress.to_label(), "status:in-progress");
        assert_eq!(IssueStatus::Done.to_label(), "status:done");
    }

    #[test]
    fn test_issue_status_from_label() {
        assert_eq!(
            IssueStatus::from_label("status:backlog"),
            Some(IssueStatus::Backlog)
        );
        assert_eq!(
            IssueStatus::from_label("status:in-progress"),
            Some(IssueStatus::InProgress)
        );
        assert_eq!(IssueStatus::from_label("unknown"), None);
    }

    #[test]
    fn test_issue_status_parse() {
        assert_eq!(
            "backlog".parse::<IssueStatus>().unwrap(),
            IssueStatus::Backlog
        );
        assert_eq!(
            "in-progress".parse::<IssueStatus>().unwrap(),
            IssueStatus::InProgress
        );
        assert!("unknown".parse::<IssueStatus>().is_err());
    }

    #[test]
    fn test_issue_priority_to_label() {
        assert_eq!(IssuePriority::High.to_label(), "priority:high");
        assert_eq!(IssuePriority::Medium.to_label(), "priority:medium");
        assert_eq!(IssuePriority::Low.to_label(), "priority:low");
    }

    #[test]
    fn test_kanban_board_from_issues() {
        let issues = vec![
            CachedIssue {
                id: 1,
                project_id: 1,
                github_issue_id: 100,
                number: 1,
                title: "Test 1".to_string(),
                body: None,
                state: "open".to_string(),
                status: "backlog".to_string(),
                priority: None,
                assignee_login: None,
                assignee_avatar_url: None,
                labels_json: None,
                html_url: None,
                github_created_at: None,
                github_updated_at: None,
                cached_at: "2025-01-01".to_string(),
            },
            CachedIssue {
                id: 2,
                project_id: 1,
                github_issue_id: 101,
                number: 2,
                title: "Test 2".to_string(),
                body: None,
                state: "open".to_string(),
                status: "in-progress".to_string(),
                priority: Some("high".to_string()),
                assignee_login: None,
                assignee_avatar_url: None,
                labels_json: None,
                html_url: None,
                github_created_at: None,
                github_updated_at: None,
                cached_at: "2025-01-01".to_string(),
            },
        ];

        let board = KanbanBoard::from_issues(issues);
        assert_eq!(board.backlog.len(), 1);
        assert_eq!(board.in_progress.len(), 1);
        assert_eq!(board.todo.len(), 0);
    }
}
