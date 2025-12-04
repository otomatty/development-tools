//! Issue Management Feature Components
//!
//! Components for issue tracking, project management, and kanban board functionality.

pub mod create_issue_modal;
pub mod create_project_modal;
pub mod issue_card;
pub mod issue_detail_modal;
pub mod kanban_board;
pub mod link_repository_modal;
pub mod project_dashboard;
pub mod projects_page;

pub use create_issue_modal::CreateIssueModal;
pub use create_project_modal::CreateProjectModal;
pub use issue_card::{IssueCard, IssueClickEvent, StatusChangeEvent};
pub use issue_detail_modal::{IssueDetailModal, IssueDetailStatusChange};
pub use kanban_board::KanbanBoard;
pub use link_repository_modal::LinkRepositoryModal;
pub use project_dashboard::ProjectDashboard;
pub use projects_page::ProjectsPage;
