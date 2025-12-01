//! Code statistics models
//!
//! Data structures for tracking daily code changes (additions/deletions).
//! Used for visualizing code activity over time.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// Daily code statistics for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyCodeStats {
    pub id: i64,
    pub user_id: i64,
    /// Date in YYYY-MM-DD format
    pub date: String,
    pub additions: i32,
    pub deletions: i32,
    pub commits_count: i32,
    /// JSON array of repository names that had commits on this day
    pub repositories_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl DailyCodeStats {
    /// Parse date as NaiveDate
    pub fn date_as_naive(&self) -> Option<NaiveDate> {
        NaiveDate::parse_from_str(&self.date, "%Y-%m-%d").ok()
    }

    /// Get net change (additions - deletions)
    pub fn net_change(&self) -> i32 {
        self.additions - self.deletions
    }

    /// Parse repositories from JSON
    pub fn repositories(&self) -> Vec<String> {
        self.repositories_json
            .as_ref()
            .and_then(|json| serde_json::from_str(json).ok())
            .unwrap_or_default()
    }
}

/// Sync metadata for tracking incremental data fetching
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncMetadata {
    pub id: i64,
    pub user_id: i64,
    /// Type of sync: 'code_stats', 'contributions', etc.
    pub sync_type: String,
    /// Last sync time in RFC3339 format
    pub last_sync_at: Option<String>,
    /// GraphQL cursor or pagination token
    pub last_sync_cursor: Option<String>,
    /// ETag for conditional requests
    pub etag: Option<String>,
    /// Remaining API rate limit
    pub rate_limit_remaining: Option<i32>,
    /// When rate limit resets (RFC3339)
    pub rate_limit_reset_at: Option<String>,
}

impl SyncMetadata {
    /// Parse last_sync_at as DateTime<Utc>
    pub fn last_sync_at_parsed(&self) -> Option<DateTime<Utc>> {
        self.last_sync_at.as_ref().and_then(|s| {
            DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        })
    }

    /// Parse rate_limit_reset_at as DateTime<Utc>
    pub fn rate_limit_reset_at_parsed(&self) -> Option<DateTime<Utc>> {
        self.rate_limit_reset_at.as_ref().and_then(|s| {
            DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        })
    }
}

/// Summary of code statistics for a period
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CodeStatsSummary {
    pub additions: i32,
    pub deletions: i32,
    pub net_change: i32,
    pub commits_count: i32,
    pub active_days: i32,
}

impl CodeStatsSummary {
    /// Create summary from daily stats
    pub fn from_daily_stats(stats: &[DailyCodeStats]) -> Self {
        let additions: i32 = stats.iter().map(|s| s.additions).sum();
        let deletions: i32 = stats.iter().map(|s| s.deletions).sum();
        let commits_count: i32 = stats.iter().map(|s| s.commits_count).sum();
        let active_days = stats.iter().filter(|s| s.commits_count > 0).count() as i32;

        Self {
            additions,
            deletions,
            net_change: additions - deletions,
            commits_count,
            active_days,
        }
    }
}

/// Response containing code statistics for display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeStatsResponse {
    /// Daily statistics for the requested period
    pub daily: Vec<DailyCodeStats>,
    /// Weekly summary
    pub weekly_total: CodeStatsSummary,
    /// Monthly summary
    pub monthly_total: CodeStatsSummary,
    /// Period type requested
    pub period: StatsPeriod,
}

/// Statistics period for queries
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum StatsPeriod {
    #[default]
    Week,
    Month,
    Quarter,
    Year,
}

impl StatsPeriod {
    /// Get number of days for this period
    pub fn days(&self) -> i64 {
        match self {
            StatsPeriod::Week => 7,
            StatsPeriod::Month => 30,
            StatsPeriod::Quarter => 90,
            StatsPeriod::Year => 365,
        }
    }
}

/// Rate limit information for API calls
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitInfo {
    /// REST API rate limit
    pub rest_remaining: i32,
    pub rest_limit: i32,
    /// Reset time in RFC3339 format
    pub rest_reset_at: Option<String>,
    /// GraphQL API rate limit
    pub graphql_remaining: i32,
    pub graphql_limit: i32,
    /// Reset time in RFC3339 format
    pub graphql_reset_at: Option<String>,
    /// Search API rate limit (stricter)
    pub search_remaining: i32,
    pub search_limit: i32,
    /// Reset time in RFC3339 format
    pub search_reset_at: Option<String>,
    /// Is any rate limit critical (below 20%)?
    pub is_critical: bool,
}

impl RateLimitInfo {
    /// Calculate REST API usage percentage
    pub fn rest_usage_percent(&self) -> f32 {
        if self.rest_limit > 0 {
            (1.0 - self.rest_remaining as f32 / self.rest_limit as f32) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate GraphQL API usage percentage
    pub fn graphql_usage_percent(&self) -> f32 {
        if self.graphql_limit > 0 {
            (1.0 - self.graphql_remaining as f32 / self.graphql_limit as f32) * 100.0
        } else {
            0.0
        }
    }

    /// Check if any rate limit is critical (below 20%) and update is_critical flag
    /// Used internally for tests
    #[cfg(test)]
    pub fn check_critical(&mut self) {
        const CRITICAL_THRESHOLD: f32 = 0.20;

        let rest_critical = self.rest_limit > 0
            && (self.rest_remaining as f32 / self.rest_limit as f32) < CRITICAL_THRESHOLD;
        let graphql_critical = self.graphql_limit > 0
            && (self.graphql_remaining as f32 / self.graphql_limit as f32) < CRITICAL_THRESHOLD;
        let search_critical = self.search_limit > 0
            && (self.search_remaining as f32 / self.search_limit as f32) < CRITICAL_THRESHOLD;

        self.is_critical = rest_critical || graphql_critical || search_critical;
    }
}

/// Commit statistics from a single repository
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitStats {
    pub repository: String,
    pub commits: Vec<CommitDailyStats>,
}

/// Daily commit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitDailyStats {
    /// Date in YYYY-MM-DD format
    pub date: String,
    pub additions: i32,
    pub deletions: i32,
    pub commit_count: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;
    use chrono::Timelike;

    // ========================================================================
    // DailyCodeStats Tests
    // ========================================================================

    fn create_daily_stats(
        id: i64,
        date: &str,
        additions: i32,
        deletions: i32,
        commits: i32,
    ) -> DailyCodeStats {
        DailyCodeStats {
            id,
            user_id: 1,
            date: date.to_string(),
            additions,
            deletions,
            commits_count: commits,
            repositories_json: None,
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn test_daily_code_stats_date_as_naive() {
        let stats = create_daily_stats(1, "2025-11-30", 100, 50, 5);
        let date = stats.date_as_naive();

        assert!(date.is_some());
        let naive_date = date.unwrap();
        assert_eq!(naive_date.year(), 2025);
        assert_eq!(naive_date.month(), 11);
        assert_eq!(naive_date.day(), 30);
    }

    #[test]
    fn test_daily_code_stats_date_as_naive_invalid() {
        let stats = DailyCodeStats {
            id: 1,
            user_id: 1,
            date: "invalid-date".to_string(),
            additions: 100,
            deletions: 50,
            commits_count: 5,
            repositories_json: None,
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };

        assert!(stats.date_as_naive().is_none());
    }

    #[test]
    fn test_daily_code_stats_net_change() {
        let stats = create_daily_stats(1, "2025-11-30", 150, 50, 5);
        assert_eq!(stats.net_change(), 100);

        // Negative net change (more deletions)
        let stats2 = create_daily_stats(2, "2025-11-29", 30, 100, 3);
        assert_eq!(stats2.net_change(), -70);
    }

    #[test]
    fn test_daily_code_stats_repositories_parsing() {
        let mut stats = create_daily_stats(1, "2025-11-30", 100, 50, 5);
        stats.repositories_json = Some(r#"["repo1", "repo2", "repo3"]"#.to_string());

        let repos = stats.repositories();
        assert_eq!(repos.len(), 3);
        assert_eq!(repos[0], "repo1");
        assert_eq!(repos[1], "repo2");
        assert_eq!(repos[2], "repo3");
    }

    #[test]
    fn test_daily_code_stats_repositories_empty() {
        let stats = create_daily_stats(1, "2025-11-30", 100, 50, 5);
        let repos = stats.repositories();
        assert!(repos.is_empty());
    }

    #[test]
    fn test_daily_code_stats_repositories_invalid_json() {
        let mut stats = create_daily_stats(1, "2025-11-30", 100, 50, 5);
        stats.repositories_json = Some("invalid json".to_string());

        let repos = stats.repositories();
        assert!(repos.is_empty());
    }

    // ========================================================================
    // SyncMetadata Tests
    // ========================================================================

    #[test]
    fn test_sync_metadata_last_sync_at_parsed() {
        let metadata = SyncMetadata {
            id: 1,
            user_id: 1,
            sync_type: "code_stats".to_string(),
            last_sync_at: Some("2025-11-30T12:00:00Z".to_string()),
            last_sync_cursor: None,
            etag: None,
            rate_limit_remaining: Some(4500),
            rate_limit_reset_at: None,
        };

        let parsed = metadata.last_sync_at_parsed();
        assert!(parsed.is_some());
        let dt = parsed.unwrap();
        assert_eq!(dt.hour(), 12);
        assert_eq!(dt.minute(), 0);
    }

    #[test]
    fn test_sync_metadata_last_sync_at_none() {
        let metadata = SyncMetadata {
            id: 1,
            user_id: 1,
            sync_type: "code_stats".to_string(),
            last_sync_at: None,
            last_sync_cursor: None,
            etag: None,
            rate_limit_remaining: None,
            rate_limit_reset_at: None,
        };

        assert!(metadata.last_sync_at_parsed().is_none());
    }

    #[test]
    fn test_sync_metadata_rate_limit_reset_parsed() {
        let metadata = SyncMetadata {
            id: 1,
            user_id: 1,
            sync_type: "code_stats".to_string(),
            last_sync_at: None,
            last_sync_cursor: None,
            etag: None,
            rate_limit_remaining: Some(100),
            rate_limit_reset_at: Some("2025-11-30T13:00:00Z".to_string()),
        };

        let parsed = metadata.rate_limit_reset_at_parsed();
        assert!(parsed.is_some());
        assert_eq!(parsed.unwrap().hour(), 13);
    }

    // ========================================================================
    // CodeStatsSummary Tests
    // ========================================================================

    #[test]
    fn test_code_stats_summary_from_daily_stats() {
        let stats = vec![
            create_daily_stats(1, "2025-11-28", 100, 50, 5),
            create_daily_stats(2, "2025-11-29", 200, 30, 3),
        ];

        let summary = CodeStatsSummary::from_daily_stats(&stats);

        assert_eq!(summary.additions, 300);
        assert_eq!(summary.deletions, 80);
        assert_eq!(summary.net_change, 220);
        assert_eq!(summary.commits_count, 8);
        assert_eq!(summary.active_days, 2);
    }

    #[test]
    fn test_code_stats_summary_empty_stats() {
        let stats: Vec<DailyCodeStats> = vec![];
        let summary = CodeStatsSummary::from_daily_stats(&stats);

        assert_eq!(summary.additions, 0);
        assert_eq!(summary.deletions, 0);
        assert_eq!(summary.net_change, 0);
        assert_eq!(summary.commits_count, 0);
        assert_eq!(summary.active_days, 0);
    }

    #[test]
    fn test_code_stats_summary_inactive_days_not_counted() {
        let stats = vec![
            create_daily_stats(1, "2025-11-28", 100, 50, 5),
            create_daily_stats(2, "2025-11-29", 0, 0, 0), // No activity
            create_daily_stats(3, "2025-11-30", 50, 20, 2),
        ];

        let summary = CodeStatsSummary::from_daily_stats(&stats);

        assert_eq!(summary.active_days, 2); // Only 2 days with commits
        assert_eq!(summary.commits_count, 7);
    }

    // ========================================================================
    // StatsPeriod Tests
    // ========================================================================

    #[test]
    fn test_stats_period_days() {
        assert_eq!(StatsPeriod::Week.days(), 7);
        assert_eq!(StatsPeriod::Month.days(), 30);
        assert_eq!(StatsPeriod::Quarter.days(), 90);
        assert_eq!(StatsPeriod::Year.days(), 365);
    }

    // ========================================================================
    // RateLimitInfo Tests
    // ========================================================================

    #[test]
    fn test_rate_limit_critical_check_search_critical() {
        let mut info = RateLimitInfo {
            rest_remaining: 100,
            rest_limit: 5000,
            graphql_remaining: 100,
            graphql_limit: 5000,
            search_remaining: 5,
            search_limit: 30,
            ..Default::default()
        };

        info.check_critical();
        assert!(info.is_critical); // search is below 20%
    }

    #[test]
    fn test_rate_limit_critical_check_rest_critical() {
        let mut info = RateLimitInfo {
            rest_remaining: 500,
            rest_limit: 5000,
            graphql_remaining: 4000,
            graphql_limit: 5000,
            search_remaining: 25,
            search_limit: 30,
            ..Default::default()
        };

        info.check_critical();
        assert!(info.is_critical); // REST is 10%, below 20%
    }

    #[test]
    fn test_rate_limit_critical_check_graphql_critical() {
        let mut info = RateLimitInfo {
            rest_remaining: 4000,
            rest_limit: 5000,
            graphql_remaining: 800,
            graphql_limit: 5000,
            search_remaining: 25,
            search_limit: 30,
            ..Default::default()
        };

        info.check_critical();
        assert!(info.is_critical); // GraphQL is 16%, below 20%
    }

    #[test]
    fn test_rate_limit_critical_check_all_ok() {
        let mut info = RateLimitInfo {
            rest_remaining: 4000,
            rest_limit: 5000,
            graphql_remaining: 4000,
            graphql_limit: 5000,
            search_remaining: 25,
            search_limit: 30,
            ..Default::default()
        };

        info.check_critical();
        assert!(!info.is_critical); // all above 20%
    }

    #[test]
    fn test_rate_limit_critical_check_zero_limits() {
        let mut info = RateLimitInfo {
            rest_remaining: 0,
            rest_limit: 0,
            graphql_remaining: 0,
            graphql_limit: 0,
            search_remaining: 0,
            search_limit: 0,
            ..Default::default()
        };

        info.check_critical();
        assert!(!info.is_critical); // Zero limits don't trigger critical
    }
}
