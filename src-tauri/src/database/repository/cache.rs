//! Activity cache repository operations

use chrono::{DateTime, Utc};

use crate::database::connection::{Database, DatabaseError, DbResult};

/// Activity cache repository operations
impl Database {
    /// Save or update cache entry
    pub async fn save_cache(
        &self,
        user_id: i64,
        data_type: &str,
        data_json: &str,
        expires_at: DateTime<Utc>,
    ) -> DbResult<()> {
        let now = Utc::now().to_rfc3339();
        let expires = expires_at.to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO activity_cache (user_id, data_type, data_json, fetched_at, expires_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(user_id, data_type) DO UPDATE SET
                data_json = excluded.data_json,
                fetched_at = excluded.fetched_at,
                expires_at = excluded.expires_at
            "#,
        )
        .bind(user_id)
        .bind(data_type)
        .bind(data_json)
        .bind(now)
        .bind(expires)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }

    /// Get valid cache entry (not expired)
    pub async fn get_valid_cache(
        &self,
        user_id: i64,
        data_type: &str,
    ) -> DbResult<Option<String>> {
        let now = Utc::now().to_rfc3339();

        let result: Option<String> = sqlx::query_scalar(
            r#"
            SELECT data_json FROM activity_cache
            WHERE user_id = ? AND data_type = ? AND expires_at > ?
            "#,
        )
        .bind(user_id)
        .bind(data_type)
        .bind(now)
        .fetch_optional(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result)
    }

    /// Clear expired cache entries
    pub async fn clear_expired_cache(&self) -> DbResult<u64> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query("DELETE FROM activity_cache WHERE expires_at <= ?")
            .bind(now)
            .execute(self.pool())
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result.rows_affected())
    }

    /// Save previous GitHub stats (for diff calculation)
    /// Uses activity_cache with a very long expiry to persist stats
    pub async fn save_previous_github_stats(
        &self,
        user_id: i64,
        stats_json: &str,
    ) -> DbResult<()> {
        // Set expiry to 100 years in the future (effectively permanent)
        let expires = Utc::now() + chrono::Duration::days(36500);
        self.save_cache(user_id, "previous_github_stats", stats_json, expires)
            .await
    }

    /// Get previous GitHub stats (for diff calculation)
    pub async fn get_previous_github_stats(&self, user_id: i64) -> DbResult<Option<String>> {
        // Get cache without expiry check (we use a very long expiry)
        let result: Option<String> = sqlx::query_scalar(
            r#"
            SELECT data_json FROM activity_cache
            WHERE user_id = ? AND data_type = 'previous_github_stats'
            "#,
        )
        .bind(user_id)
        .fetch_optional(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result)
    }

    /// Get cache size for a user
    pub async fn get_user_cache_size(&self, user_id: i64) -> DbResult<u64> {
        let result: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(LENGTH(data_json)), 0) FROM activity_cache WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_one(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result as u64)
    }
}
