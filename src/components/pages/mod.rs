//! Page Components Module
//!
//! This module contains page-level components that serve as entry points for different
//! routes/views in the application. Page components are responsible for:
//! - Layout definition
//! - Feature component composition
//! - Page-specific state management
//! - Routing-related props handling
//!
//! Page components should NOT contain:
//! - Business logic (delegate to feature components)
//! - API calls (delegate to feature components or service layer)
//! - Complex UI implementations (use feature/ui components)
//!
//! DEPENDENCY MAP:
//! Children (files that this module exports):
//!   ├─ home_page.rs - Home page
//!   ├─ xp_history/ - XP history page (directory)
//!   ├─ projects_page.rs - Projects list page
//!   ├─ project_dashboard_page.rs - Project dashboard page
//!   ├─ settings_page.rs - Settings page
//!   └─ mock_server_page.rs - Mock server page
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

pub mod home_page;
pub mod mock_server_page;
pub mod project_dashboard_page;
pub mod projects_page;
pub mod settings_page;
pub mod xp_history;

pub use home_page::HomePage;
pub use mock_server_page::MockServerPage;
pub use project_dashboard_page::ProjectDashboardPage;
pub use projects_page::ProjectsPage;
pub use settings_page::SettingsPage;
pub use xp_history::XpHistoryPage;
