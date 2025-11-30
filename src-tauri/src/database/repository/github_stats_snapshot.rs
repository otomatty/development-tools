//! GitHub Stats Snapshot Repository
//!
//! Database operations for storing and retrieving GitHub stats snapshots.
//! Used for calculating day-over-day differences in statistics.
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this repository):
//!   └─ src-tauri/src/commands/github.rs
//! Dependencies (Files this repository imports):
//!   └─ src-tauri/src/database/models/github_stats_snapshot.rs
//! Related Documentation:
//!   ├─ Spec: ../models/github_stats_snapshot.spec.md
//!   └─ Issue: docs/01_issues/open/2025_11/20251129_02_github-stats-daily-comparison.md

use sqlx::Row;

use crate::database::connection::{Database, DbResult};
use crate::database::models::github_stats_snapshot::GitHubStatsSnapshot;

impl Database {
    /// Save or update a GitHub stats snapshot for a user on a specific date
    ///
    /// Uses UPSERT semantics: inserts a new snapshot if none exists for the date,
    /// or updates the existing one if it does.
    pub async fn save_github_stats_snapshot(
        &self,
        snapshot: &GitHubStatsSnapshot,
    ) -> DbResult<()> {
        sqlx::query(
            r#"
            INSERT INTO github_stats_snapshots (
                user_id, total_commits, total_prs, total_reviews, 
                total_issues, total_stars_received, total_contributions, snapshot_date
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(user_id, snapshot_date) DO UPDATE SET
                total_commits = excluded.total_commits,
                total_prs = excluded.total_prs,
                total_reviews = excluded.total_reviews,
                total_issues = excluded.total_issues,
                total_stars_received = excluded.total_stars_received,
                total_contributions = excluded.total_contributions
            "#,
        )
        .bind(snapshot.user_id)
        .bind(snapshot.total_commits)
        .bind(snapshot.total_prs)
        .bind(snapshot.total_reviews)
        .bind(snapshot.total_issues)
        .bind(snapshot.total_stars_received)
        .bind(snapshot.total_contributions)
        .bind(&snapshot.snapshot_date)
        .execute(self.pool())
        .await?;

        Ok(())
    }

    /// Get the most recent snapshot before a given date
    ///
    /// Used for calculating day-over-day differences.
    /// Returns None if no previous snapshot exists.
    pub async fn get_previous_github_stats_snapshot(
        &self,
        user_id: i64,
        before_date: &str,
    ) -> DbResult<Option<GitHubStatsSnapshot>> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, total_commits, total_prs, total_reviews,
                   total_issues, total_stars_received, total_contributions,
                   snapshot_date, created_at
            FROM github_stats_snapshots
            WHERE user_id = ? AND snapshot_date < ?
            ORDER BY snapshot_date DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(before_date)
        .fetch_optional(self.pool())
        .await?;

        Ok(row.map(|r| GitHubStatsSnapshot {
            id: r.get("id"),
            user_id: r.get("user_id"),
            total_commits: r.get("total_commits"),
            total_prs: r.get("total_prs"),
            total_reviews: r.get("total_reviews"),
            total_issues: r.get("total_issues"),
            total_stars_received: r.get("total_stars_received"),
            total_contributions: r.get("total_contributions"),
            snapshot_date: r.get("snapshot_date"),
            created_at: r.get("created_at"),
        }))
    }

    /// Get a snapshot for a specific date
    ///
    /// Returns None if no snapshot exists for that date.
    pub async fn get_github_stats_snapshot_for_date(
        &self,
        user_id: i64,
        date: &str,
    ) -> DbResult<Option<GitHubStatsSnapshot>> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, total_commits, total_prs, total_reviews,
                   total_issues, total_stars_received, total_contributions,
                   snapshot_date, created_at
            FROM github_stats_snapshots
            WHERE user_id = ? AND snapshot_date = ?
            "#,
        )
        .bind(user_id)
        .bind(date)
        .fetch_optional(self.pool())
        .await?;

        Ok(row.map(|r| GitHubStatsSnapshot {
            id: r.get("id"),
            user_id: r.get("user_id"),
            total_commits: r.get("total_commits"),
            total_prs: r.get("total_prs"),
            total_reviews: r.get("total_reviews"),
            total_issues: r.get("total_issues"),
            total_stars_received: r.get("total_stars_received"),
            total_contributions: r.get("total_contributions"),
            snapshot_date: r.get("snapshot_date"),
            created_at: r.get("created_at"),
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::database::connection::Database;
    use crate::database::models::github_stats_snapshot::GitHubStatsSnapshot;

    async fn setup_test_db() -> Database {
        let db = Database::in_memory().await.expect("Failed to create test database");

        // Create a test user
        db.create_user(12345, "testuser", None, "encrypted_token", None, None)
            .await
            .expect("Failed to create test user");

        db
    }

    fn create_snapshot(user_id: i64, commits: i32, prs: i32, reviews: i32, issues: i32, stars: i32, contributions: i32, date: &str) -> GitHubStatsSnapshot {
        GitHubStatsSnapshot::new(user_id, commits, prs, reviews, issues, stars, contributions, date)
    }

    // TC-101: Save new snapshot
    #[tokio::test]
    async fn test_save_new_snapshot() {
        let db = setup_test_db().await;
        let snapshot = create_snapshot(1, 100, 20, 30, 15, 50, 200, "2025-11-30");

        let result = db.save_github_stats_snapshot(&snapshot).await;

        assert!(result.is_ok(), "Should save snapshot successfully");

        // Verify the snapshot was saved
        let saved = db
            .get_github_stats_snapshot_for_date(1, "2025-11-30")
            .await
            .expect("Should query snapshot");

        assert!(saved.is_some(), "Snapshot should exist");
        let s = saved.unwrap();
        assert_eq!(s.total_commits, 100);
        assert_eq!(s.total_prs, 20);
        assert_eq!(s.total_reviews, 30);
        assert_eq!(s.total_issues, 15);
        assert_eq!(s.total_stars_received, 50);
        assert_eq!(s.total_contributions, 200);
    }

    // TC-102: Update existing snapshot (UPSERT)
    #[tokio::test]
    async fn test_update_existing_snapshot() {
        let db = setup_test_db().await;

        // Save initial snapshot
        let initial = create_snapshot(1, 100, 20, 30, 15, 50, 200, "2025-11-30");
        db.save_github_stats_snapshot(&initial)
            .await
            .expect("Should save initial snapshot");

        // Update with new values
        let updated = create_snapshot(1, 110, 22, 35, 18, 55, 220, "2025-11-30");
        db.save_github_stats_snapshot(&updated)
            .await
            .expect("Should update snapshot");

        // Verify the snapshot was updated
        let snapshot = db
            .get_github_stats_snapshot_for_date(1, "2025-11-30")
            .await
            .expect("Should query snapshot")
            .expect("Snapshot should exist");

        assert_eq!(snapshot.total_commits, 110);
        assert_eq!(snapshot.total_prs, 22);
        assert_eq!(snapshot.total_reviews, 35);
        assert_eq!(snapshot.total_issues, 18);
        assert_eq!(snapshot.total_stars_received, 55);
        assert_eq!(snapshot.total_contributions, 220);
    }

    // TC-103: Get previous snapshot
    #[tokio::test]
    async fn test_get_previous_snapshot() {
        let db = setup_test_db().await;

        // Save snapshots for multiple days
        let snap1 = create_snapshot(1, 90, 18, 25, 12, 45, 180, "2025-11-28");
        db.save_github_stats_snapshot(&snap1)
            .await
            .expect("Should save first snapshot");

        let snap2 = create_snapshot(1, 95, 19, 27, 14, 47, 190, "2025-11-29");
        db.save_github_stats_snapshot(&snap2)
            .await
            .expect("Should save second snapshot");

        let snap3 = create_snapshot(1, 100, 20, 30, 15, 50, 200, "2025-11-30");
        db.save_github_stats_snapshot(&snap3)
            .await
            .expect("Should save third snapshot");

        // Get previous snapshot (before 2025-11-30)
        let previous = db
            .get_previous_github_stats_snapshot(1, "2025-11-30")
            .await
            .expect("Should query previous snapshot")
            .expect("Previous snapshot should exist");

        assert_eq!(previous.snapshot_date, "2025-11-29");
        assert_eq!(previous.total_commits, 95);
    }

    // TC-104: Get previous snapshot when none exists
    #[tokio::test]
    async fn test_get_previous_snapshot_none_exists() {
        let db = setup_test_db().await;

        // Save only one snapshot
        let snapshot = create_snapshot(1, 100, 20, 30, 15, 50, 200, "2025-11-30");
        db.save_github_stats_snapshot(&snapshot)
            .await
            .expect("Should save snapshot");

        // Try to get previous (should be None)
        let previous = db
            .get_previous_github_stats_snapshot(1, "2025-11-30")
            .await
            .expect("Should query without error");

        assert!(previous.is_none(), "Should have no previous snapshot");
    }

    // TC-105: Get snapshot for specific date when not exists
    #[tokio::test]
    async fn test_get_snapshot_for_nonexistent_date() {
        let db = setup_test_db().await;

        let snapshot = db
            .get_github_stats_snapshot_for_date(1, "2025-11-30")
            .await
            .expect("Should query without error");

        assert!(snapshot.is_none(), "Should have no snapshot for this date");
    }

    // TC-106: Multiple users have separate snapshots
    #[tokio::test]
    async fn test_multiple_users_separate_snapshots() {
        let db = setup_test_db().await;

        // Create second user
        db.create_user(67890, "testuser2", None, "encrypted_token2", None, None)
            .await
            .expect("Failed to create second user");

        // Save snapshots for both users on same date
        let user1_snap = create_snapshot(1, 100, 20, 30, 15, 50, 200, "2025-11-30");
        db.save_github_stats_snapshot(&user1_snap)
            .await
            .expect("Should save user 1 snapshot");

        let user2_snap = create_snapshot(2, 50, 10, 15, 7, 25, 100, "2025-11-30");
        db.save_github_stats_snapshot(&user2_snap)
            .await
            .expect("Should save user 2 snapshot");

        // Verify each user has their own data
        let user1_snapshot = db
            .get_github_stats_snapshot_for_date(1, "2025-11-30")
            .await
            .expect("Should query")
            .expect("Should have snapshot");

        let user2_snapshot = db
            .get_github_stats_snapshot_for_date(2, "2025-11-30")
            .await
            .expect("Should query")
            .expect("Should have snapshot");

        assert_eq!(user1_snapshot.total_commits, 100);
        assert_eq!(user2_snapshot.total_commits, 50);
    }
}
