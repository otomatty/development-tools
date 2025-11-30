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

// Re-export all repository operations
// Note: Repository operations are implemented as methods on Database,
// so they're automatically available when Database is imported.
