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
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StatsPeriod {
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
    /// Check if any rate limit is below the critical threshold (20%)
    pub fn check_critical(&mut self) {
        let rest_critical = self.rest_limit > 0 
            && (self.rest_remaining as f32 / self.rest_limit as f32) < 0.2;
        let graphql_critical = self.graphql_limit > 0 
            && (self.graphql_remaining as f32 / self.graphql_limit as f32) < 0.2;
        let search_critical = self.search_limit > 0 
            && (self.search_remaining as f32 / self.search_limit as f32) < 0.2;
        
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

    #[test]
    fn test_code_stats_summary_from_daily_stats() {
        let stats = vec![
            DailyCodeStats {
                id: 1,
                user_id: 1,
                date: "2025-11-28".to_string(),
                additions: 100,
                deletions: 50,
                commits_count: 5,
                repositories_json: None,
                created_at: Utc::now().to_rfc3339(),
                updated_at: Utc::now().to_rfc3339(),
            },
            DailyCodeStats {
                id: 2,
                user_id: 1,
                date: "2025-11-29".to_string(),
                additions: 200,
                deletions: 30,
                commits_count: 3,
                repositories_json: None,
                created_at: Utc::now().to_rfc3339(),
                updated_at: Utc::now().to_rfc3339(),
            },
        ];

        let summary = CodeStatsSummary::from_daily_stats(&stats);

        assert_eq!(summary.additions, 300);
        assert_eq!(summary.deletions, 80);
        assert_eq!(summary.net_change, 220);
        assert_eq!(summary.commits_count, 8);
        assert_eq!(summary.active_days, 2);
    }

    #[test]
    fn test_stats_period_days() {
        assert_eq!(StatsPeriod::Week.days(), 7);
        assert_eq!(StatsPeriod::Month.days(), 30);
        assert_eq!(StatsPeriod::Quarter.days(), 90);
        assert_eq!(StatsPeriod::Year.days(), 365);
    }

    #[test]
    fn test_rate_limit_critical_check() {
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
        
        let mut info2 = RateLimitInfo {
            rest_remaining: 4000,
            rest_limit: 5000,
            graphql_remaining: 4000,
            graphql_limit: 5000,
            search_remaining: 25,
            search_limit: 30,
            ..Default::default()
        };
        
        info2.check_critical();
        assert!(!info2.is_critical); // all above 20%
    }
}
