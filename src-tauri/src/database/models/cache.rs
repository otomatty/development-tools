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
    /// GitHub Notifications list (cached so a 304 response can still serve
    /// the UI without forcing a re-fetch). See Issue #186.
    pub const GITHUB_NOTIFICATIONS: &str = "github_notifications";
    /// Recent activity feed payload backing the home timeline
    /// (`/users/{u}/events`). See Issue #187.
    pub const ACTIVITY_FEED: &str = "activity_feed";
    /// Realtime "今日のコミット数" — thin commit-count query used by the
    /// gamification today-quest UI. See Issue #188.
    pub const TODAY_COMMITS: &str = "today_commits";
    /// Language / repository breakdown payload (per-language byte share +
    /// per-repository additions/deletions). See Issue #193.
    pub const LANGUAGE_BREAKDOWN: &str = "language_breakdown";
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
    /// GitHub Notifications cache duration. Set very high (≈1 year)
    /// because the ETag-backed flow needs the cache to outlive the
    /// `clear_expired_cache` startup sweep — a deleted cache row paired
    /// with a still-current ETag would otherwise force a transparent
    /// re-fetch on every cold start (handled by the 304+empty-cache
    /// recovery path in `get_notifications`, which is fine but visible).
    /// Notifications data isn't sensitive enough to justify a short TTL,
    /// and the row is overwritten on every successful sync.
    pub const GITHUB_NOTIFICATIONS: i64 = 60 * 24 * 365;
    /// Activity timeline cache duration (5 minutes).
    /// REST budget is 5000 req/hr — even with revalidate-on-focus the
    /// 5-minute TTL keeps refresh load negligible.
    pub const ACTIVITY_FEED: i64 = 5;
    /// Realtime today-commit-count cache duration (3 minutes).
    /// The query is a thin GraphQL `history(since:) { totalCount }` — the
    /// 3-minute TTL gives the gamification today-quest UI near-realtime
    /// updates while keeping GraphQL spend trivial. See Issue #188.
    pub const TODAY_COMMITS: i64 = 3;
    /// Language / repository breakdown cache duration (24 hours).
    /// The underlying GraphQL query scans up to 100 repos with language
    /// edges and 100 commits each — keeping a 24h TTL matches the issue
    /// #193 requirement and prevents focus revalidations from burning
    /// the GraphQL budget on what is fundamentally slow-moving data.
    pub const LANGUAGE_BREAKDOWN: i64 = 1440;
}
