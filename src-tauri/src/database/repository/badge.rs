//! Badge repository operations

use chrono::{DateTime, Utc};
use sqlx::Row;

use crate::database::connection::{Database, DatabaseError, DbResult};
use crate::database::models::Badge;

/// Badge repository operations
impl Database {
    /// Award a badge to user
    pub async fn award_badge(
        &self,
        user_id: i64,
        badge_type: &str,
        badge_id: &str,
    ) -> DbResult<i64> {
        let id = sqlx::query(
            r#"
            INSERT OR IGNORE INTO badges (user_id, badge_type, badge_id)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(badge_type)
        .bind(badge_id)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?
        .last_insert_rowid();

        Ok(id)
    }

    /// Check if user has a badge
    pub async fn has_badge(&self, user_id: i64, badge_id: &str) -> DbResult<bool> {
        let count: i32 =
            sqlx::query_scalar("SELECT COUNT(*) FROM badges WHERE user_id = ? AND badge_id = ?")
                .bind(user_id)
                .bind(badge_id)
                .fetch_one(self.pool())
                .await
                .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(count > 0)
    }

    /// Get all badges for a user
    pub async fn get_user_badges(&self, user_id: i64) -> DbResult<Vec<Badge>> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, badge_type, badge_id, earned_at
            FROM badges
            WHERE user_id = ?
            ORDER BY earned_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        let badges: Vec<Badge> = rows
            .iter()
            .map(|row| Badge {
                id: row.get("id"),
                user_id: row.get("user_id"),
                badge_type: row.get("badge_type"),
                badge_id: row.get("badge_id"),
                earned_at: DateTime::parse_from_rfc3339(row.get::<&str, _>("earned_at"))
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
            .collect();

        Ok(badges)
    }
}
