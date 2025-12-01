//! Challenge repository operations

use chrono::{DateTime, Datelike, Utc};
use sqlx::Row;

use crate::database::connection::{Database, DatabaseError, DbResult};
use crate::database::models::Challenge;

/// Challenge repository operations
impl Database {
    /// Create a new challenge
    pub async fn create_challenge(
        &self,
        user_id: i64,
        challenge_type: &str,
        target_metric: &str,
        target_value: i32,
        reward_xp: i32,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> DbResult<Challenge> {
        let id = sqlx::query(
            r#"
            INSERT INTO challenges (user_id, challenge_type, target_metric, target_value, 
                                   current_value, reward_xp, start_date, end_date, status)
            VALUES (?, ?, ?, ?, 0, ?, ?, ?, 'active')
            "#,
        )
        .bind(user_id)
        .bind(challenge_type)
        .bind(target_metric)
        .bind(target_value)
        .bind(reward_xp)
        .bind(start_date.to_rfc3339())
        .bind(end_date.to_rfc3339())
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?
        .last_insert_rowid();

        self.get_challenge_by_id(id).await
    }

    /// Create a new challenge with start stats for progress tracking
    pub async fn create_challenge_with_stats(
        &self,
        user_id: i64,
        challenge_type: &str,
        target_metric: &str,
        target_value: i32,
        reward_xp: i32,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        start_stats_json: &str,
    ) -> DbResult<Challenge> {
        let id = sqlx::query(
            r#"
            INSERT INTO challenges (user_id, challenge_type, target_metric, target_value, 
                                   current_value, reward_xp, start_date, end_date, status, start_stats_json)
            VALUES (?, ?, ?, ?, 0, ?, ?, ?, 'active', ?)
            "#,
        )
        .bind(user_id)
        .bind(challenge_type)
        .bind(target_metric)
        .bind(target_value)
        .bind(reward_xp)
        .bind(start_date.to_rfc3339())
        .bind(end_date.to_rfc3339())
        .bind(start_stats_json)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?
        .last_insert_rowid();

        self.get_challenge_by_id(id).await
    }

    /// Get start stats JSON for a challenge
    pub async fn get_challenge_start_stats(&self, challenge_id: i64) -> DbResult<Option<String>> {
        let result: Option<String> =
            sqlx::query_scalar("SELECT start_stats_json FROM challenges WHERE id = ?")
                .bind(challenge_id)
                .fetch_one(self.pool())
                .await
                .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result)
    }

    /// Get the most recent daily challenge date for a user
    pub async fn get_last_daily_challenge_date(
        &self,
        user_id: i64,
    ) -> DbResult<Option<chrono::NaiveDate>> {
        let result: Option<String> = sqlx::query_scalar(
            r#"
            SELECT DATE(start_date) FROM challenges 
            WHERE user_id = ? AND challenge_type = 'daily'
            ORDER BY start_date DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?
        .flatten();

        Ok(result.and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()))
    }

    /// Get the most recent weekly challenge date for a user
    pub async fn get_last_weekly_challenge_date(
        &self,
        user_id: i64,
    ) -> DbResult<Option<chrono::NaiveDate>> {
        let result: Option<String> = sqlx::query_scalar(
            r#"
            SELECT DATE(start_date) FROM challenges 
            WHERE user_id = ? AND challenge_type = 'weekly'
            ORDER BY start_date DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?
        .flatten();

        Ok(result.and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()))
    }

    /// Get challenge by ID
    pub async fn get_challenge_by_id(&self, id: i64) -> DbResult<Challenge> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, challenge_type, target_metric, target_value, 
                   current_value, reward_xp, start_date, end_date, status, completed_at
            FROM challenges
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        let start_date = DateTime::parse_from_rfc3339(row.get::<&str, _>("start_date"))
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| DatabaseError::Query(format!("Failed to parse start_date: {}", e)))?;
        let end_date = DateTime::parse_from_rfc3339(row.get::<&str, _>("end_date"))
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| DatabaseError::Query(format!("Failed to parse end_date: {}", e)))?;

        Ok(Challenge {
            id: row.get("id"),
            user_id: row.get("user_id"),
            challenge_type: row.get("challenge_type"),
            target_metric: row.get("target_metric"),
            target_value: row.get("target_value"),
            current_value: row.get("current_value"),
            reward_xp: row.get("reward_xp"),
            start_date,
            end_date,
            status: row.get("status"),
            completed_at: row
                .get::<Option<&str>, _>("completed_at")
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
        })
    }

    /// Get all active challenges for a user
    pub async fn get_active_challenges(&self, user_id: i64) -> DbResult<Vec<Challenge>> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, challenge_type, target_metric, target_value, 
                   current_value, reward_xp, start_date, end_date, status, completed_at
            FROM challenges
            WHERE user_id = ? AND status = 'active'
            ORDER BY end_date ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        let challenges: Vec<Challenge> = rows
            .iter()
            .map(|row| Challenge {
                id: row.get("id"),
                user_id: row.get("user_id"),
                challenge_type: row.get("challenge_type"),
                target_metric: row.get("target_metric"),
                target_value: row.get("target_value"),
                current_value: row.get("current_value"),
                reward_xp: row.get("reward_xp"),
                start_date: DateTime::parse_from_rfc3339(row.get::<&str, _>("start_date"))
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                end_date: DateTime::parse_from_rfc3339(row.get::<&str, _>("end_date"))
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                status: row.get("status"),
                completed_at: row
                    .get::<Option<&str>, _>("completed_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
            })
            .collect();

        Ok(challenges)
    }

    /// Get challenges by type (daily/weekly) for a user
    pub async fn get_challenges_by_type(
        &self,
        user_id: i64,
        challenge_type: &str,
    ) -> DbResult<Vec<Challenge>> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, challenge_type, target_metric, target_value, 
                   current_value, reward_xp, start_date, end_date, status, completed_at
            FROM challenges
            WHERE user_id = ? AND challenge_type = ?
            ORDER BY start_date DESC
            "#,
        )
        .bind(user_id)
        .bind(challenge_type)
        .fetch_all(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        let challenges: Vec<Challenge> = rows
            .iter()
            .map(|row| Challenge {
                id: row.get("id"),
                user_id: row.get("user_id"),
                challenge_type: row.get("challenge_type"),
                target_metric: row.get("target_metric"),
                target_value: row.get("target_value"),
                current_value: row.get("current_value"),
                reward_xp: row.get("reward_xp"),
                start_date: DateTime::parse_from_rfc3339(row.get::<&str, _>("start_date"))
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                end_date: DateTime::parse_from_rfc3339(row.get::<&str, _>("end_date"))
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                status: row.get("status"),
                completed_at: row
                    .get::<Option<&str>, _>("completed_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
            })
            .collect();

        Ok(challenges)
    }

    /// Get all challenges for a user (including completed and failed)
    pub async fn get_all_challenges(&self, user_id: i64) -> DbResult<Vec<Challenge>> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, challenge_type, target_metric, target_value, 
                   current_value, reward_xp, start_date, end_date, status, completed_at
            FROM challenges
            WHERE user_id = ?
            ORDER BY start_date DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        let challenges: Vec<Challenge> = rows
            .iter()
            .map(|row| Challenge {
                id: row.get("id"),
                user_id: row.get("user_id"),
                challenge_type: row.get("challenge_type"),
                target_metric: row.get("target_metric"),
                target_value: row.get("target_value"),
                current_value: row.get("current_value"),
                reward_xp: row.get("reward_xp"),
                start_date: DateTime::parse_from_rfc3339(row.get::<&str, _>("start_date"))
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                end_date: DateTime::parse_from_rfc3339(row.get::<&str, _>("end_date"))
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                status: row.get("status"),
                completed_at: row
                    .get::<Option<&str>, _>("completed_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
            })
            .collect();

        Ok(challenges)
    }

    /// Update challenge progress
    ///
    /// Caps progress at target_value and automatically completes the challenge
    /// when the target is reached.
    pub async fn update_challenge_progress(
        &self,
        challenge_id: i64,
        current_value: i32,
    ) -> DbResult<Challenge> {
        // Get current challenge to check target
        let challenge = self.get_challenge_by_id(challenge_id).await?;

        // Cap at target value
        let capped_value = current_value.min(challenge.target_value);

        // Update progress
        sqlx::query(
            r#"
            UPDATE challenges SET current_value = ?
            WHERE id = ?
            "#,
        )
        .bind(capped_value)
        .bind(challenge_id)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        // Auto-complete if target reached
        if capped_value >= challenge.target_value && challenge.status == "active" {
            return self.complete_challenge(challenge_id).await;
        }

        self.get_challenge_by_id(challenge_id).await
    }

    /// Complete a challenge (mark as completed and set completed_at)
    pub async fn complete_challenge(&self, challenge_id: i64) -> DbResult<Challenge> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE challenges SET status = 'completed', completed_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&now)
        .bind(challenge_id)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        self.get_challenge_by_id(challenge_id).await
    }

    /// Fail a challenge (mark as failed)
    pub async fn fail_challenge(&self, challenge_id: i64) -> DbResult<Challenge> {
        sqlx::query(
            r#"
            UPDATE challenges SET status = 'failed'
            WHERE id = ?
            "#,
        )
        .bind(challenge_id)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        self.get_challenge_by_id(challenge_id).await
    }

    /// Delete a challenge
    pub async fn delete_challenge(&self, challenge_id: i64) -> DbResult<()> {
        sqlx::query("DELETE FROM challenges WHERE id = ?")
            .bind(challenge_id)
            .execute(self.pool())
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }

    /// Check and fail expired challenges
    pub async fn fail_expired_challenges(&self, user_id: i64) -> DbResult<Vec<Challenge>> {
        let now = Utc::now().to_rfc3339();

        // Get expired challenges first
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, challenge_type, target_metric, target_value, 
                   current_value, reward_xp, start_date, end_date, status, completed_at
            FROM challenges
            WHERE user_id = ? AND status = 'active' AND end_date < ?
            "#,
        )
        .bind(user_id)
        .bind(&now)
        .fetch_all(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        let expired_challenges: Vec<Challenge> = rows
            .iter()
            .map(|row| Challenge {
                id: row.get("id"),
                user_id: row.get("user_id"),
                challenge_type: row.get("challenge_type"),
                target_metric: row.get("target_metric"),
                target_value: row.get("target_value"),
                current_value: row.get("current_value"),
                reward_xp: row.get("reward_xp"),
                start_date: DateTime::parse_from_rfc3339(row.get::<&str, _>("start_date"))
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                end_date: DateTime::parse_from_rfc3339(row.get::<&str, _>("end_date"))
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                status: "failed".to_string(), // Will be updated
                completed_at: None,
            })
            .collect();

        // Mark as failed
        sqlx::query(
            r#"
            UPDATE challenges SET status = 'failed'
            WHERE user_id = ? AND status = 'active' AND end_date < ?
            "#,
        )
        .bind(user_id)
        .bind(&now)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(expired_challenges)
    }

    /// Check if there's an active challenge of a specific type and metric
    pub async fn has_active_challenge(
        &self,
        user_id: i64,
        challenge_type: &str,
        target_metric: &str,
    ) -> DbResult<bool> {
        let count: i32 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM challenges 
            WHERE user_id = ? AND challenge_type = ? AND target_metric = ? AND status = 'active'
            "#,
        )
        .bind(user_id)
        .bind(challenge_type)
        .bind(target_metric)
        .fetch_one(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(count > 0)
    }

    /// Get challenge completion count for badge tracking
    pub async fn get_challenge_completion_count(&self, user_id: i64) -> DbResult<i32> {
        let count: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM challenges WHERE user_id = ? AND status = 'completed'",
        )
        .bind(user_id)
        .fetch_one(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(count)
    }

    /// Get consecutive weeks with completed weekly challenges (for 'consistent' badge)
    pub async fn get_consecutive_weekly_completions(&self, user_id: i64) -> DbResult<i32> {
        // Get all completed weekly challenges ordered by completion date
        let rows = sqlx::query(
            r#"
            SELECT completed_at
            FROM challenges
            WHERE user_id = ? AND challenge_type = 'weekly' AND status = 'completed'
            ORDER BY completed_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        if rows.is_empty() {
            return Ok(0);
        }

        // Calculate consecutive weeks using ISO week numbers
        let mut consecutive = 0;
        let mut last_week: Option<(i32, u32)> = None; // (year, week_number)

        for row in &rows {
            if let Some(completed_at_str) = row.get::<Option<&str>, _>("completed_at") {
                if let Ok(completed_at) = DateTime::parse_from_rfc3339(completed_at_str) {
                    let completed_at_utc = completed_at.with_timezone(&Utc);
                    let iso_week = completed_at_utc.iso_week();
                    let week_tuple = (iso_week.year(), iso_week.week());

                    match last_week {
                        None => {
                            consecutive = 1;
                            last_week = Some(week_tuple);
                        }
                        Some((last_year, last_wk)) => {
                            // Check if this is the previous week
                            let is_previous_week = if week_tuple.0 == last_year {
                                // Same year: check if week number is exactly one less
                                week_tuple.1 + 1 == last_wk
                            } else if week_tuple.0 + 1 == last_year && last_wk == 1 {
                                // Year boundary: last week of previous year to first week of next year
                                week_tuple.1 >= 52 // Week 52 or 53
                            } else {
                                false
                            };

                            if is_previous_week {
                                consecutive += 1;
                                last_week = Some(week_tuple);
                            } else if week_tuple != (last_year, last_wk) {
                                // Gap in weeks, stop counting
                                break;
                            }
                            // If same week, skip (multiple challenges in same week)
                        }
                    }
                }
            }
        }

        Ok(consecutive)
    }
}
