//! Frontend type definitions
//!
//! This module defines the data structures used in the frontend application.
//! Split into submodules for better maintainability.

mod auth;
mod challenge;
mod gamification;
pub mod issue;
mod mock_server;
mod network;
mod settings;

// Re-export all types
pub use auth::*;
pub use challenge::*;
pub use gamification::*;
pub use issue::*;
pub use mock_server::*;
pub use network::*;
pub use settings::*;

/// アプリのページ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppPage {
    #[default]
    Home,
    Projects,
    ProjectDetail(i64),
    MockServer,
    Settings,
    XpHistory,
}
