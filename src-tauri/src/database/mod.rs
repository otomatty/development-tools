//! Database module for SQLite operations
//!
//! This module provides database connection management, migrations,
//! and CRUD operations for the gamification system.

pub mod challenge;
pub mod connection;
pub mod migrations;
pub mod models;
pub mod repository;

// Re-export challenge types used by commands/github.rs
#[allow(unused_imports)]
pub use challenge::{
    calculate_challenge_period, calculate_recommended_targets, calculate_reward_xp,
    generate_daily_challenges, generate_weekly_challenges, should_generate_daily_challenges,
    should_generate_weekly_challenges, ChallengeGeneratorConfig, ChallengeStats,
    ChallengeTemplate, HistoricalStats, RecommendedTargets,
};
pub use connection::{Database, DatabaseError, DbResult};
pub use models::*;

