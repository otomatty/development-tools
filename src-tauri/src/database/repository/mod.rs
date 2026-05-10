//! Database repository layer
//!
//! This module provides CRUD operations for database models.
//! Split into submodules for better maintainability.

mod badge;
mod cache;
mod challenge;
mod code_stats;
mod github_stats_snapshot;
mod settings;
mod user;
mod user_stats;
mod xp_history;

#[cfg(test)]
mod tests;

// Re-export repository-owned types that callers need to construct directly.
// (Most repository operations are methods on `Database` and become
// available automatically when `Database` is imported.)
pub use user_stats::UserStatsGitHubAggregates;
pub use xp_history::{XP_HISTORY_SOURCE_LIVE, XP_HISTORY_SOURCE_RECALCULATED};
