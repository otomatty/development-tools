//! XP History repository operations

use chrono::{DateTime, Utc};
use sqlx::Row;

use crate::database::connection::{Database, DatabaseError, DbResult};
use crate::database::models::XpHistoryEntry;

/// XP History repository operations
impl Database {
    /// Record XP gain
    pub async fn record_xp_gain(
        &self,
        user_id: i64,
        action_type: &str,
        xp_amount: i32,
        description: Option<&str>,
        github_event_id: Option<&str>,
    ) -> DbResult<i64> {
        let id = sqlx::query(
            r#"
            INSERT INTO xp_history (user_id, action_type, xp_amount, description, github_event_id)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(action_type)
        .bind(xp_amount)
        .bind(description)
        .bind(github_event_id)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?
        .last_insert_rowid();

        Ok(id)
    }

    /// Check if XP was already recorded for a GitHub event
    pub async fn is_xp_recorded_for_event(&self, github_event_id: &str) -> DbResult<bool> {
        let count: i32 =
            sqlx::query_scalar("SELECT COUNT(*) FROM xp_history WHERE github_event_id = ?")
                .bind(github_event_id)
                .fetch_one(self.pool())
                .await
                .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(count > 0)
    }

    /// Get recent XP history
    pub async fn get_recent_xp_history(
        &self,
        user_id: i64,
        limit: i32,
    ) -> DbResult<Vec<XpHistoryEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, action_type, xp_amount, description, github_event_id, created_at
            FROM xp_history
            WHERE user_id = ?
            ORDER BY created_at DESC
            LIMIT ?
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        let entries: Vec<XpHistoryEntry> = rows
            .iter()
            .map(|row| XpHistoryEntry {
                id: row.get("id"),
                user_id: row.get("user_id"),
                action_type: row.get("action_type"),
                xp_amount: row.get("xp_amount"),
                description: row.get("description"),
                github_event_id: row.get("github_event_id"),
                created_at: DateTime::parse_from_rfc3339(row.get::<&str, _>("created_at"))
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
            .collect();

        Ok(entries)
    }
}
