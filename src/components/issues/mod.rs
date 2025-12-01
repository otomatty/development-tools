//! Issue Management Components
//!
//! This module contains components for the GitHub Issue management feature
//! with a Linear-style kanban board interface.
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this module):
//!   └─ src/components/mod.rs
//!   └─ src/app.rs
//! Dependencies (What this module uses):
//!   ├─ src/types/issue.rs
//!   ├─ src/tauri_api.rs
//!   └─ src/components/icons.rs
//! Related Documentation:
//!   └─ docs/03_plans/issue-management/20251201_implementation_plan.md

mod create_issue_modal;
mod issue_card;
mod issue_detail_modal;
mod kanban_board;
mod link_repository_modal;
mod project_dashboard;
mod projects_page;

pub use create_issue_modal::CreateIssueModal;
pub use issue_card::{IssueCard, IssueClickEvent, StatusChangeEvent};
pub use issue_detail_modal::{IssueDetailModal, IssueDetailStatusChange};
pub use kanban_board::KanbanBoard;
pub use link_repository_modal::LinkRepositoryModal;
pub use project_dashboard::ProjectDashboard;
pub use projects_page::ProjectsPage;
