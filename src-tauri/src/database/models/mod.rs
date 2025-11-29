//! Database models
//!
//! This module defines the data structures that map to database tables.
//! Split into submodules for better maintainability.

pub mod badge;
mod cache;
pub mod challenge;
pub mod code_stats;
pub mod level;
mod settings;
pub mod streak;
mod user;
pub mod xp;

// Re-export all models and utilities
pub use badge::*;
pub use cache::*;
pub use challenge::*;
pub use code_stats::*;
pub use level::*;
pub use settings::*;
pub use streak::*;
pub use user::*;
pub use xp::*;
