//! Page Components Module
//!
//! This module contains page-level components that serve as entry points for different
//! routes/views in the application. Page components are responsible for:
//! - Layout definition
//! - Feature component composition
//! - Page-specific state management
//! - Routing-related props handling
//!
//! Each page is organized as a directory with:
//! - mod.rs - Main page component
//! - loading.rs - Loading skeleton and spinner components
//! - utils.rs - Utility functions
//!
//! Page components should NOT contain:
//! - Business logic (delegate to feature components)
//! - API calls (delegate to feature components or service layer)
//! - Complex UI implementations (use feature/ui components)
//!
//! DEPENDENCY MAP:
//! Children (directories that this module exports):
//!   ├─ home/ - Home page
//!   ├─ xp_history/ - XP history page
//!   ├─ projects/ - Projects list page
//!   ├─ project_dashboard/ - Project dashboard page
//!   ├─ settings/ - Settings page
//!   └─ mock_server/ - Mock server page
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

pub mod home;
pub mod mock_server;
pub mod project_dashboard;
pub mod projects;
pub mod settings;
pub mod xp_history;

pub use home::HomePage;
pub use mock_server::MockServerPage;
pub use project_dashboard::ProjectDashboardPage;
pub use projects::ProjectsPage;
pub use settings::SettingsPage;
pub use xp_history::XpHistoryPage;
