//! Frontend type definitions
//!
//! This module defines the data structures used in the frontend application.
//! Split into submodules for better maintainability.

mod auth;
mod challenge;
mod gamification;
mod mock_server;
mod settings;
mod tool;

// Re-export all types
pub use auth::*;
pub use challenge::*;
pub use gamification::*;
pub use mock_server::*;
pub use settings::*;
pub use tool::*;

use std::collections::HashMap;

/// オプション値のマップ
pub type OptionValues = HashMap<String, serde_json::Value>;

/// アプリのページ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppPage {
    #[default]
    Home,
    Tools,
    MockServer,
    Settings,
}
