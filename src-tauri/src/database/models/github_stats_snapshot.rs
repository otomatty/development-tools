//! GitHub Stats Snapshot models
//!
//! Data structures for storing daily snapshots of GitHub statistics.
//! Used for calculating day-over-day differences.
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this model):
//!   ├─ src-tauri/src/database/repository/github_stats_snapshot.rs (planned)
//!   └─ src-tauri/src/commands/github.rs (planned)
//! Related Documentation:
//!   ├─ Spec: ./github_stats_snapshot.spec.md
//!   └─ Issue: docs/01_issues/open/2025_11/20251129_02_github-stats-daily-comparison.md

use serde::{Deserialize, Serialize};

/// GitHub statistics snapshot for a specific date
///
/// Stores a point-in-time snapshot of user's GitHub statistics.
/// Used for calculating day-over-day differences.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubStatsSnapshot {
    pub id: i64,
    pub user_id: i64,
    pub total_commits: i32,
    pub total_prs: i32,
    pub total_reviews: i32,
    pub total_issues: i32,
    pub total_stars_received: i32,
    pub total_contributions: i32,
    /// Snapshot date in YYYY-MM-DD format
    pub snapshot_date: String,
    pub created_at: String,
}

/// Statistics difference between two snapshots
///
/// Represents the change in GitHub statistics compared to a previous snapshot.
/// Positive values indicate increase, negative indicate decrease.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StatsDiff {
    pub commits_diff: i32,
    pub prs_diff: i32,
    pub reviews_diff: i32,
    pub issues_diff: i32,
    pub stars_diff: i32,
    pub contributions_diff: i32,
    /// The date of the snapshot being compared against (None if first sync)
    pub comparison_date: Option<String>,
}

impl GitHubStatsSnapshot {
    /// Create a new snapshot from current GitHub stats
    pub fn new(
        user_id: i64,
        total_commits: i32,
        total_prs: i32,
        total_reviews: i32,
        total_issues: i32,
        total_stars_received: i32,
        total_contributions: i32,
        snapshot_date: &str,
    ) -> Self {
        Self {
            id: 0, // Will be set by database
            user_id,
            total_commits,
            total_prs,
            total_reviews,
            total_issues,
            total_stars_received,
            total_contributions,
            snapshot_date: snapshot_date.to_string(),
            created_at: String::new(), // Will be set by database
        }
    }

    /// Calculate the difference between this snapshot and a previous one
    ///
    /// If `previous` is None (first sync), returns a default diff with all zeros.
    pub fn calculate_diff(&self, previous: Option<&GitHubStatsSnapshot>) -> StatsDiff {
        match previous {
            Some(prev) => StatsDiff {
                commits_diff: self.total_commits - prev.total_commits,
                prs_diff: self.total_prs - prev.total_prs,
                reviews_diff: self.total_reviews - prev.total_reviews,
                issues_diff: self.total_issues - prev.total_issues,
                stars_diff: self.total_stars_received - prev.total_stars_received,
                contributions_diff: self.total_contributions - prev.total_contributions,
                comparison_date: Some(prev.snapshot_date.clone()),
            },
            None => StatsDiff::default(),
        }
    }
}

impl StatsDiff {
    /// Check if there are any changes in the diff
    pub fn has_changes(&self) -> bool {
        self.commits_diff != 0
            || self.prs_diff != 0
            || self.reviews_diff != 0
            || self.issues_diff != 0
            || self.stars_diff != 0
            || self.contributions_diff != 0
    }

    /// Check if the overall trend is positive (more increases than decreases)
    pub fn is_positive(&self) -> bool {
        let total = self.commits_diff
            + self.prs_diff
            + self.reviews_diff
            + self.issues_diff
            + self.stars_diff
            + self.contributions_diff;
        total > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TC-001: GitHubStatsSnapshot::new
    #[test]
    fn test_new_snapshot() {
        let snapshot = GitHubStatsSnapshot::new(
            1,    // user_id
            100,  // commits
            20,   // prs
            30,   // reviews
            15,   // issues
            50,   // stars
            200,  // contributions
            "2025-11-30",
        );

        assert_eq!(snapshot.user_id, 1);
        assert_eq!(snapshot.total_commits, 100);
        assert_eq!(snapshot.total_prs, 20);
        assert_eq!(snapshot.total_reviews, 30);
        assert_eq!(snapshot.total_issues, 15);
        assert_eq!(snapshot.total_stars_received, 50);
        assert_eq!(snapshot.total_contributions, 200);
        assert_eq!(snapshot.snapshot_date, "2025-11-30");
    }

    // TC-002: calculate_diff with previous snapshot
    #[test]
    fn test_calculate_diff_with_previous() {
        let previous = GitHubStatsSnapshot {
            id: 1,
            user_id: 1,
            total_commits: 90,
            total_prs: 18,
            total_reviews: 25,
            total_issues: 12,
            total_stars_received: 45,
            total_contributions: 180,
            snapshot_date: "2025-11-29".to_string(),
            created_at: "2025-11-29T00:00:00Z".to_string(),
        };

        let current = GitHubStatsSnapshot::new(
            1,
            100,  // +10 commits
            20,   // +2 prs
            30,   // +5 reviews
            15,   // +3 issues
            50,   // +5 stars
            200,  // +20 contributions
            "2025-11-30",
        );

        let diff = current.calculate_diff(Some(&previous));

        assert_eq!(diff.commits_diff, 10);
        assert_eq!(diff.prs_diff, 2);
        assert_eq!(diff.reviews_diff, 5);
        assert_eq!(diff.issues_diff, 3);
        assert_eq!(diff.stars_diff, 5);
        assert_eq!(diff.contributions_diff, 20);
        assert_eq!(diff.comparison_date, Some("2025-11-29".to_string()));
    }

    // TC-003: calculate_diff without previous snapshot
    #[test]
    fn test_calculate_diff_without_previous() {
        let current = GitHubStatsSnapshot::new(1, 100, 20, 30, 15, 50, 200, "2025-11-30");

        let diff = current.calculate_diff(None);

        assert_eq!(diff, StatsDiff::default());
        assert_eq!(diff.commits_diff, 0);
        assert_eq!(diff.comparison_date, None);
    }

    // TC-004: StatsDiff has_changes (with changes)
    #[test]
    fn test_stats_diff_has_changes() {
        let diff = StatsDiff {
            commits_diff: 5,
            prs_diff: 0,
            reviews_diff: 0,
            issues_diff: 0,
            stars_diff: 0,
            contributions_diff: 0,
            comparison_date: None,
        };

        assert!(diff.has_changes());
    }

    // TC-005: StatsDiff has_changes (no changes)
    #[test]
    fn test_stats_diff_no_changes() {
        let diff = StatsDiff::default();

        assert!(!diff.has_changes());
    }

    // Additional: Test is_positive
    #[test]
    fn test_stats_diff_is_positive() {
        let positive_diff = StatsDiff {
            commits_diff: 10,
            prs_diff: 2,
            reviews_diff: -1,
            issues_diff: 0,
            stars_diff: 0,
            contributions_diff: 5,
            comparison_date: None,
        };
        assert!(positive_diff.is_positive());

        let negative_diff = StatsDiff {
            commits_diff: -10,
            prs_diff: -2,
            reviews_diff: 1,
            issues_diff: 0,
            stars_diff: 0,
            contributions_diff: -5,
            comparison_date: None,
        };
        assert!(!negative_diff.is_positive());
    }

    // Additional: Test negative diff values
    #[test]
    fn test_calculate_diff_with_decrease() {
        let previous = GitHubStatsSnapshot {
            id: 1,
            user_id: 1,
            total_commits: 100,
            total_prs: 20,
            total_reviews: 30,
            total_issues: 15,
            total_stars_received: 50,
            total_contributions: 200,
            snapshot_date: "2025-11-29".to_string(),
            created_at: "2025-11-29T00:00:00Z".to_string(),
        };

        // Stars decreased (e.g., repository deleted)
        let current = GitHubStatsSnapshot::new(
            1,
            100,  // no change
            20,   // no change
            30,   // no change
            15,   // no change
            45,   // -5 stars
            200,  // no change
            "2025-11-30",
        );

        let diff = current.calculate_diff(Some(&previous));

        assert_eq!(diff.stars_diff, -5);
        assert!(diff.has_changes());
        assert!(!diff.is_positive());
    }
}
