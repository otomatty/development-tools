//! Code statistics repository operations
//!
//! CRUD operations for daily code statistics and sync metadata.

use chrono::{NaiveDate, Utc};
use sqlx::Row;

use crate::database::connection::{Database, DbResult};
use crate::database::models::code_stats::{
    CodeStatsResponse, CodeStatsSummary, DailyCodeStats, StatsPeriod, SyncMetadata,
};

impl Database {
    // ========================================================================
    // Daily Code Stats Operations
    // ========================================================================

    /// Save or update daily code statistics
    pub async fn upsert_daily_code_stats(
        &self,
        user_id: i64,
        date: NaiveDate,
        additions: i32,
        deletions: i32,
        commits_count: i32,
        repositories: Option<Vec<String>>,
    ) -> DbResult<DailyCodeStats> {
        let date_str = date.format("%Y-%m-%d").to_string();
        let repositories_json = repositories.map(|r| serde_json::to_string(&r).unwrap_or_default());

        sqlx::query(
            r#"
            INSERT INTO daily_code_stats (user_id, date, additions, deletions, commits_count, repositories_json, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(user_id, date) DO UPDATE SET
                additions = excluded.additions,
                deletions = excluded.deletions,
                commits_count = excluded.commits_count,
                repositories_json = excluded.repositories_json,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(user_id)
        .bind(&date_str)
        .bind(additions)
        .bind(deletions)
        .bind(commits_count)
        .bind(&repositories_json)
        .execute(self.pool())
        .await
        .map_err(|e| crate::database::connection::DatabaseError::Query(e.to_string()))?;

        // Fetch the upserted record
        self.get_daily_code_stats(user_id, date).await.map(|opt| {
            opt.expect("Record should exist after upsert")
        })
    }

    /// Get daily code statistics for a specific date
    pub async fn get_daily_code_stats(
        &self,
        user_id: i64,
        date: NaiveDate,
    ) -> DbResult<Option<DailyCodeStats>> {
        let date_str = date.format("%Y-%m-%d").to_string();
        let row = sqlx::query(
            r#"
            SELECT id, user_id, date, additions, deletions, commits_count, 
                   repositories_json, created_at, updated_at
            FROM daily_code_stats
            WHERE user_id = ? AND date = ?
            "#,
        )
        .bind(user_id)
        .bind(&date_str)
        .fetch_optional(self.pool())
        .await
        .map_err(|e| crate::database::connection::DatabaseError::Query(e.to_string()))?;

        Ok(row.map(|r| DailyCodeStats {
            id: r.get("id"),
            user_id: r.get("user_id"),
            date: r.get("date"),
            additions: r.get("additions"),
            deletions: r.get("deletions"),
            commits_count: r.get("commits_count"),
            repositories_json: r.get("repositories_json"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    /// Get daily code statistics for a date range
    pub async fn get_daily_code_stats_range(
        &self,
        user_id: i64,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> DbResult<Vec<DailyCodeStats>> {
        let start_str = start_date.format("%Y-%m-%d").to_string();
        let end_str = end_date.format("%Y-%m-%d").to_string();
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, date, additions, deletions, commits_count,
                   repositories_json, created_at, updated_at
            FROM daily_code_stats
            WHERE user_id = ? AND date >= ? AND date <= ?
            ORDER BY date DESC
            "#,
        )
        .bind(user_id)
        .bind(&start_str)
        .bind(&end_str)
        .fetch_all(self.pool())
        .await
        .map_err(|e| crate::database::connection::DatabaseError::Query(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| DailyCodeStats {
                id: r.get("id"),
                user_id: r.get("user_id"),
                date: r.get("date"),
                additions: r.get("additions"),
                deletions: r.get("deletions"),
                commits_count: r.get("commits_count"),
                repositories_json: r.get("repositories_json"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }

    /// Get code statistics response with summaries for a period
    pub async fn get_code_stats_response(
        &self,
        user_id: i64,
        period: StatsPeriod,
    ) -> DbResult<CodeStatsResponse> {
        let today = Utc::now().date_naive();
        let period_days = period.days() as i64;
        
        // Calculate date ranges
        let start_date = today - chrono::Duration::days(period_days);
        let week_start = today - chrono::Duration::days(7);
        let month_start = today - chrono::Duration::days(30);
        
        let week_start_str = week_start.format("%Y-%m-%d").to_string();
        let month_start_str = month_start.format("%Y-%m-%d").to_string();

        // Get all daily stats for the period
        let daily = self.get_daily_code_stats_range(user_id, start_date, today).await?;

        // Calculate weekly summary
        let weekly_stats: Vec<_> = daily.iter().filter(|s| s.date >= week_start_str).cloned().collect();
        let weekly_total = CodeStatsSummary::from_daily_stats(&weekly_stats);

        // Calculate monthly summary
        let monthly_stats: Vec<_> = daily.iter().filter(|s| s.date >= month_start_str).cloned().collect();
        let monthly_total = CodeStatsSummary::from_daily_stats(&monthly_stats);

        Ok(CodeStatsResponse {
            daily,
            weekly_total,
            monthly_total,
            period,
        })
    }

    // ========================================================================
    // Sync Metadata Operations
    // ========================================================================

    /// Get or create sync metadata for a user and sync type
    pub async fn get_or_create_sync_metadata(
        &self,
        user_id: i64,
        sync_type: &str,
    ) -> DbResult<SyncMetadata> {
        // Try to get existing
        if let Some(metadata) = self.get_sync_metadata(user_id, sync_type).await? {
            return Ok(metadata);
        }

        // Create new
        sqlx::query(
            r#"
            INSERT INTO sync_metadata (user_id, sync_type)
            VALUES (?, ?)
            "#,
        )
        .bind(user_id)
        .bind(sync_type)
        .execute(self.pool())
        .await
        .map_err(|e| crate::database::connection::DatabaseError::Query(e.to_string()))?;

        self.get_sync_metadata(user_id, sync_type)
            .await
            .map(|opt| opt.expect("Record should exist after insert"))
    }

    /// Get sync metadata
    pub async fn get_sync_metadata(
        &self,
        user_id: i64,
        sync_type: &str,
    ) -> DbResult<Option<SyncMetadata>> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, sync_type, last_sync_at, last_sync_cursor,
                   etag, rate_limit_remaining, rate_limit_reset_at
            FROM sync_metadata
            WHERE user_id = ? AND sync_type = ?
            "#,
        )
        .bind(user_id)
        .bind(sync_type)
        .fetch_optional(self.pool())
        .await
        .map_err(|e| crate::database::connection::DatabaseError::Query(e.to_string()))?;

        Ok(row.map(|r| SyncMetadata {
            id: r.get("id"),
            user_id: r.get("user_id"),
            sync_type: r.get("sync_type"),
            last_sync_at: r.get("last_sync_at"),
            last_sync_cursor: r.get("last_sync_cursor"),
            etag: r.get("etag"),
            rate_limit_remaining: r.get("rate_limit_remaining"),
            rate_limit_reset_at: r.get("rate_limit_reset_at"),
        }))
    }

    /// Update sync metadata after a sync operation
    pub async fn update_sync_metadata(
        &self,
        user_id: i64,
        sync_type: &str,
        last_sync_at: Option<String>,
        cursor: Option<&str>,
        etag: Option<&str>,
        rate_limit_remaining: Option<i32>,
        rate_limit_reset_at: Option<String>,
    ) -> DbResult<()> {
        sqlx::query(
            r#"
            UPDATE sync_metadata
            SET last_sync_at = COALESCE(?, last_sync_at),
                last_sync_cursor = COALESCE(?, last_sync_cursor),
                etag = COALESCE(?, etag),
                rate_limit_remaining = COALESCE(?, rate_limit_remaining),
                rate_limit_reset_at = COALESCE(?, rate_limit_reset_at),
                updated_at = CURRENT_TIMESTAMP
            WHERE user_id = ? AND sync_type = ?
            "#,
        )
        .bind(&last_sync_at)
        .bind(cursor)
        .bind(etag)
        .bind(rate_limit_remaining)
        .bind(&rate_limit_reset_at)
        .bind(user_id)
        .bind(sync_type)
        .execute(self.pool())
        .await
        .map_err(|e| crate::database::connection::DatabaseError::Query(e.to_string()))?;

        Ok(())
    }

    /// Check if a sync is needed (based on last sync time and cache duration)
    pub async fn is_sync_needed(
        &self,
        user_id: i64,
        sync_type: &str,
        cache_duration_hours: i64,
    ) -> DbResult<bool> {
        use chrono::DateTime;
        
        let metadata = self.get_sync_metadata(user_id, sync_type).await?;

        match metadata {
            None => Ok(true), // No sync metadata, need initial sync
            Some(m) => {
                match m.last_sync_at_parsed() {
                    None => Ok(true), // Never synced
                    Some(last_sync) => {
                        let cache_duration = chrono::Duration::hours(cache_duration_hours);
                        let is_stale = Utc::now() - last_sync > cache_duration;
                        Ok(is_stale)
                    }
                }
            }
        }
    }

    /// Get the date to sync from (for incremental sync)
    pub async fn get_sync_start_date(
        &self,
        user_id: i64,
        default_days_back: i64,
    ) -> DbResult<NaiveDate> {
        // Check if we have existing data
        let row = sqlx::query(
            r#"
            SELECT MAX(date) as last_date
            FROM daily_code_stats
            WHERE user_id = ?
            "#,
        )
        .bind(user_id)
        .fetch_optional(self.pool())
        .await
        .map_err(|e| crate::database::connection::DatabaseError::Query(e.to_string()))?;

        let today = Utc::now().date_naive();
        
        if let Some(r) = row {
            if let Some(last_date_str) = r.get::<Option<String>, _>("last_date") {
                // Parse the date string and sync from last known date (with some overlap)
                if let Ok(last_date) = NaiveDate::parse_from_str(&last_date_str, "%Y-%m-%d") {
                    let sync_from = last_date - chrono::Duration::days(1);
                    return Ok(sync_from);
                }
            }
        }

        // No existing data, sync from default days back
        Ok(today - chrono::Duration::days(default_days_back))
    }
}
