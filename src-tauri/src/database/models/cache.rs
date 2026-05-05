//! Cache-related models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Cached GitHub activity data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityCache {
    pub id: i64,
    pub user_id: i64,
    pub data_type: String,
    pub data: String, // JSON-encoded data
    pub fetched_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl ActivityCache {
    /// Check if cache is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Parse cached data as JSON
    pub fn parse_data<T: for<'de> Deserialize<'de>>(&self) -> Option<T> {
        serde_json::from_str(&self.data).ok()
    }
}

/// Cache data types
pub mod cache_types {
    pub const CONTRIBUTION_GRAPH: &str = "contribution_graph";
    pub const GITHUB_STATS: &str = "github_stats";
    pub const USER_STATS: &str = "user_stats";
    pub const REPOSITORIES: &str = "repositories";
    pub const LANGUAGES: &str = "languages";
    /// Cross-repo "Today / Inbox": assigned Open Issues + Review Requested
    /// PRs combined into a single payload. See Issue #183.
    pub const MY_OPEN_WORK: &str = "my_open_work";
    /// PR progress dashboard payload (mergeable / checks / reviewDecision)
    /// for the viewer's open PRs. See Issue #185.
    pub const MY_PR_PROGRESS: &str = "my_pr_progress";
}

/// Default cache durations in minutes
pub mod cache_durations {
    /// Contribution graph cache duration (1 hour)
    pub const CONTRIBUTION_GRAPH: i64 = 60;
    /// GitHub stats cache duration (30 minutes)
    pub const GITHUB_STATS: i64 = 30;
    /// User stats (gamification) cache duration (1 hour)
    pub const USER_STATS: i64 = 60;
    /// Repositories cache duration (2 hours)
    pub const REPOSITORIES: i64 = 120;
    /// Languages cache duration (24 hours)
    pub const LANGUAGES: i64 = 1440;
    /// Cross-repo "Today / Inbox" cache duration (5 minutes).
    /// Search API budget is 30 req/min — even at one user with one foreground
    /// + revalidate-on-focus, 5 minutes keeps us well under that.
    pub const MY_OPEN_WORK: i64 = 5;
    /// PR progress dashboard cache duration (5 minutes).
    /// Backed by GraphQL (5000 points/hour) — short TTL keeps the panel
    /// responsive without burning the budget on focus revalidations.
    pub const MY_PR_PROGRESS: i64 = 5;
}
