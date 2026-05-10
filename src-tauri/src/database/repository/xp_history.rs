//! XP History repository operations

use chrono::{DateTime, Utc};
use sqlx::Row;

use crate::database::connection::{Database, DatabaseError, DbResult};
use crate::database::models::{XpBreakdown, XpHistoryEntry};

/// `xp_history.source` value for entries produced by the live sync stream
/// (`run_github_sync`, streak bonus, manual `add_xp`, …). This is the default
/// for every pre-Issue #194 row.
pub const XP_HISTORY_SOURCE_LIVE: &str = "live";

/// `xp_history.source` value for entries produced by `recalculate_xp_history`
/// (Issue #194). These rows are intentionally NOT folded into
/// `user_stats.total_xp` — they exist for audit / comparison only, so a
/// recalculation never overwrites or double-counts the live XP stream.
pub const XP_HISTORY_SOURCE_RECALCULATED: &str = "recalculated";

/// XP History repository operations
impl Database {
    /// Record an XP gain produced by the live sync stream.
    ///
    /// Thin wrapper over [`record_xp_gain_with_source`] that pins the new
    /// `xp_history.source` column to [`XP_HISTORY_SOURCE_LIVE`] so existing
    /// callers don't need to thread the source through. Recalculation
    /// entries must go through [`record_xp_recalculation`] instead.
    pub async fn record_xp_gain(
        &self,
        user_id: i64,
        action_type: &str,
        xp_amount: i32,
        description: Option<&str>,
        github_event_id: Option<&str>,
        breakdown: Option<&XpBreakdown>,
    ) -> DbResult<i64> {
        self.record_xp_gain_with_source(
            user_id,
            action_type,
            xp_amount,
            description,
            github_event_id,
            breakdown,
            XP_HISTORY_SOURCE_LIVE,
        )
        .await
    }

    /// Record an XP gain, tagging it with the supplied `source`.
    ///
    /// Centralises the INSERT so the live path and the recalculation path
    /// can't drift apart on serialization or column ordering.
    pub async fn record_xp_gain_with_source(
        &self,
        user_id: i64,
        action_type: &str,
        xp_amount: i32,
        description: Option<&str>,
        github_event_id: Option<&str>,
        breakdown: Option<&XpBreakdown>,
        source: &str,
    ) -> DbResult<i64> {
        let breakdown_json = breakdown
            .map(|b| serde_json::to_string(b))
            .transpose()
            .map_err(|e| {
                DatabaseError::Query(format!("Failed to serialize XP breakdown: {}", e))
            })?;

        // Bind `created_at` explicitly as RFC3339 (with microseconds)
        // instead of falling back to SQLite's CURRENT_TIMESTAMP default,
        // which uses 'YYYY-MM-DD HH:MM:SS'. Both formats sort
        // lexicographically against UTC RFC3339, but pinning the column
        // to a single format makes the range queries in
        // `get_xp_total_in_range` and the rate-limit guard in
        // `get_last_recalculation_at` deterministic and lets the
        // recalculation feature reuse RFC3339 throughout.
        let now = Utc::now().to_rfc3339();
        let id = sqlx::query(
            r#"
            INSERT INTO xp_history (user_id, action_type, xp_amount, description, github_event_id, breakdown_json, source, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(action_type)
        .bind(xp_amount)
        .bind(description)
        .bind(github_event_id)
        .bind(breakdown_json)
        .bind(source)
        .bind(now)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?
        .last_insert_rowid();

        Ok(id)
    }

    /// Record a past-year XP recalculation (Issue #194).
    ///
    /// Always writes with `source = 'recalculated'` so `user_stats.total_xp`
    /// is left untouched — the row is for audit / before-after comparison.
    pub async fn record_xp_recalculation(
        &self,
        user_id: i64,
        xp_amount: i32,
        description: Option<&str>,
        breakdown: Option<&XpBreakdown>,
    ) -> DbResult<i64> {
        self.record_xp_gain_with_source(
            user_id,
            "recalculation",
            xp_amount,
            description,
            None,
            breakdown,
            XP_HISTORY_SOURCE_RECALCULATED,
        )
        .await
    }

    /// Sum `xp_history.xp_amount` for a user, scoped to a `source` value and
    /// a `[since, now]` window. Used by `recalculate_xp_history` to render
    /// "previous live XP earned in this window" alongside the recalculated
    /// value (DoD: 計算前後の値を比較表示できる).
    pub async fn get_xp_total_in_range(
        &self,
        user_id: i64,
        since: DateTime<Utc>,
        source: &str,
    ) -> DbResult<i32> {
        let total: Option<i32> = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(xp_amount), 0)
            FROM xp_history
            WHERE user_id = ?
              AND source = ?
              AND created_at >= ?
            "#,
        )
        .bind(user_id)
        .bind(source)
        .bind(since.to_rfc3339())
        .fetch_one(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(total.unwrap_or(0))
    }

    /// Most-recent recalculation timestamp for a user, used by the
    /// rate-limit guard in `recalculate_xp_history`. `None` means the user
    /// has never run a recalculation.
    pub async fn get_last_recalculation_at(
        &self,
        user_id: i64,
    ) -> DbResult<Option<DateTime<Utc>>> {
        let row: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT created_at
            FROM xp_history
            WHERE user_id = ? AND source = ?
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(XP_HISTORY_SOURCE_RECALCULATED)
        .fetch_optional(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(row.and_then(|(s,)| parse_xp_history_timestamp(&s)))
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
            SELECT id, user_id, action_type, xp_amount, description, github_event_id, breakdown_json, created_at, source
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
            .map(|row| {
                let breakdown_json: Option<String> = row.get("breakdown_json");
                let breakdown = breakdown_json.and_then(|json| {
                    match serde_json::from_str::<XpBreakdown>(&json) {
                        Ok(b) => Some(b),
                        Err(e) => {
                            // TODO: [INFRA] logクレートに置換（ログ基盤整備時に一括対応）
                            eprintln!("[WARN] Failed to deserialize XP breakdown: {}", e);
                            None
                        }
                    }
                });

                XpHistoryEntry {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    action_type: row.get("action_type"),
                    xp_amount: row.get("xp_amount"),
                    description: row.get("description"),
                    github_event_id: row.get("github_event_id"),
                    breakdown,
                    created_at: parse_xp_history_timestamp(row.get::<&str, _>("created_at"))
                        .unwrap_or_else(Utc::now),
                    source: row.get("source"),
                }
            })
            .collect();

        Ok(entries)
    }
}

/// Parse a `xp_history.created_at` cell into a UTC `DateTime`.
///
/// Older rows (pre-Issue #194) were inserted without an explicit
/// `created_at` bind and use SQLite's `CURRENT_TIMESTAMP` default
/// (`YYYY-MM-DD HH:MM:SS`). New rows are written as RFC3339 (see
/// `record_xp_gain_with_source`). This helper accepts either, so the
/// rate-limit guard works for both legacy and fresh entries.
fn parse_xp_history_timestamp(raw: &str) -> Option<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S") {
        return Some(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc));
    }
    None
}
