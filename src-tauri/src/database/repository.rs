//! Database repository layer
//!
//! This module provides CRUD operations for database models.

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{FromRow, Row};

use super::connection::{Database, DatabaseError, DbResult};
use super::models::*;

/// User row from database
#[derive(Debug, FromRow)]
struct UserRow {
    id: i64,
    github_id: i64,
    username: String,
    avatar_url: Option<String>,
    access_token_encrypted: String,
    refresh_token_encrypted: Option<String>,
    token_expires_at: Option<String>,
    created_at: String,
    updated_at: String,
}

impl TryFrom<UserRow> for User {
    type Error = DatabaseError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        Ok(User {
            id: row.id,
            github_id: row.github_id,
            username: row.username,
            avatar_url: row.avatar_url,
            access_token_encrypted: row.access_token_encrypted,
            refresh_token_encrypted: row.refresh_token_encrypted,
            token_expires_at: row
                .token_expires_at
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            created_at: DateTime::parse_from_rfc3339(&row.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&row.updated_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

/// User repository operations
impl Database {
    /// Create a new user
    pub async fn create_user(
        &self,
        github_id: i64,
        username: &str,
        avatar_url: Option<&str>,
        access_token_encrypted: &str,
        refresh_token_encrypted: Option<&str>,
        token_expires_at: Option<DateTime<Utc>>,
    ) -> DbResult<User> {
        let now = Utc::now().to_rfc3339();
        let expires_at = token_expires_at.map(|dt| dt.to_rfc3339());

        let id = sqlx::query(
            r#"
            INSERT INTO users (github_id, username, avatar_url, access_token_encrypted, 
                              refresh_token_encrypted, token_expires_at, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(github_id)
        .bind(username)
        .bind(avatar_url)
        .bind(access_token_encrypted)
        .bind(refresh_token_encrypted)
        .bind(&expires_at)
        .bind(&now)
        .bind(&now)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?
        .last_insert_rowid();

        // Create default user stats
        self.create_user_stats(id).await?;

        self.get_user_by_id(id).await
    }

    /// Get user by ID
    pub async fn get_user_by_id(&self, id: i64) -> DbResult<User> {
        let row: UserRow = sqlx::query_as("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_one(self.pool())
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        row.try_into()
    }

    /// Get user by GitHub ID
    pub async fn get_user_by_github_id(&self, github_id: i64) -> DbResult<Option<User>> {
        let row: Option<UserRow> = sqlx::query_as("SELECT * FROM users WHERE github_id = ?")
            .bind(github_id)
            .fetch_optional(self.pool())
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }

    /// Update user tokens
    pub async fn update_user_tokens(
        &self,
        user_id: i64,
        access_token_encrypted: &str,
        refresh_token_encrypted: Option<&str>,
        token_expires_at: Option<DateTime<Utc>>,
    ) -> DbResult<()> {
        let now = Utc::now().to_rfc3339();
        let expires_at = token_expires_at.map(|dt| dt.to_rfc3339());

        sqlx::query(
            r#"
            UPDATE users 
            SET access_token_encrypted = ?, refresh_token_encrypted = ?, 
                token_expires_at = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(access_token_encrypted)
        .bind(refresh_token_encrypted)
        .bind(expires_at)
        .bind(now)
        .bind(user_id)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }

    /// Delete user (and cascade delete related data)
    pub async fn delete_user(&self, user_id: i64) -> DbResult<()> {
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(user_id)
            .execute(self.pool())
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }

    /// Clear user tokens (logout without deleting data)
    /// Sets tokens to empty string to indicate logged out state
    pub async fn clear_user_tokens(&self, user_id: i64) -> DbResult<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE users 
            SET access_token_encrypted = '', refresh_token_encrypted = NULL, 
                token_expires_at = NULL, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(now)
        .bind(user_id)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }

    /// Get current logged in user (user with valid token)
    /// Returns None if no user exists or if the user has logged out (empty token)
    pub async fn get_current_user(&self) -> DbResult<Option<User>> {
        // Only return user if they have a non-empty token (logged in)
        let row: Option<UserRow> = sqlx::query_as(
            "SELECT * FROM users WHERE access_token_encrypted != '' LIMIT 1"
        )
        .fetch_optional(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }

    /// Get user by ID regardless of login state
    /// 
    /// Unlike `get_current_user()` which only returns logged-in users,
    /// this function returns the user even if they have logged out.
    /// 
    /// **Intended use cases:**
    /// - Data recovery scenarios
    /// - Admin/maintenance operations
    /// - Checking if user data exists before re-login
    /// - Future multi-account support where we need to access
    ///   non-current user accounts
    pub async fn get_user_by_id_any_state(&self, id: i64) -> DbResult<Option<User>> {
        let row: Option<UserRow> = sqlx::query_as("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool())
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }
}

/// User stats row from database
#[derive(Debug, FromRow)]
struct UserStatsRow {
    id: i64,
    user_id: i64,
    total_xp: i32,
    current_level: i32,
    current_streak: i32,
    longest_streak: i32,
    last_activity_date: Option<String>,
    total_commits: i32,
    total_prs: i32,
    total_reviews: i32,
    total_issues: i32,
    updated_at: String,
}

impl TryFrom<UserStatsRow> for UserStats {
    type Error = DatabaseError;

    fn try_from(row: UserStatsRow) -> Result<Self, Self::Error> {
        Ok(UserStats {
            id: row.id,
            user_id: row.user_id,
            total_xp: row.total_xp,
            current_level: row.current_level,
            current_streak: row.current_streak,
            longest_streak: row.longest_streak,
            last_activity_date: row
                .last_activity_date
                .and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
            total_commits: row.total_commits,
            total_prs: row.total_prs,
            total_reviews: row.total_reviews,
            total_issues: row.total_issues,
            updated_at: DateTime::parse_from_rfc3339(&row.updated_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

/// User stats repository operations
impl Database {
    /// Create user stats (called automatically when creating user)
    async fn create_user_stats(&self, user_id: i64) -> DbResult<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO user_stats (user_id, total_xp, current_level, current_streak, 
                                   longest_streak, total_commits, total_prs, 
                                   total_reviews, total_issues, updated_at)
            VALUES (?, 0, 1, 0, 0, 0, 0, 0, 0, ?)
            "#,
        )
        .bind(user_id)
        .bind(now)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }

    /// Get user stats by user ID
    pub async fn get_user_stats(&self, user_id: i64) -> DbResult<Option<UserStats>> {
        let row: Option<UserStatsRow> =
            sqlx::query_as("SELECT * FROM user_stats WHERE user_id = ?")
                .bind(user_id)
                .fetch_optional(self.pool())
                .await
                .map_err(|e| DatabaseError::Query(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }

    /// Update user XP and level
    pub async fn add_xp(&self, user_id: i64, xp_amount: i32) -> DbResult<UserStats> {
        let now = Utc::now().to_rfc3339();

        // Get current stats
        let current = self
            .get_user_stats(user_id)
            .await?
            .ok_or_else(|| DatabaseError::Query("User stats not found".to_string()))?;

        let new_total_xp = current.total_xp + xp_amount;
        let new_level = level::level_from_xp(new_total_xp as u32) as i32;

        sqlx::query(
            r#"
            UPDATE user_stats 
            SET total_xp = ?, current_level = ?, updated_at = ?
            WHERE user_id = ?
            "#,
        )
        .bind(new_total_xp)
        .bind(new_level)
        .bind(now)
        .bind(user_id)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        self.get_user_stats(user_id)
            .await?
            .ok_or_else(|| DatabaseError::Query("User stats not found after update".to_string()))
    }

    /// Update streak
    pub async fn update_streak(&self, user_id: i64, activity_date: NaiveDate) -> DbResult<UserStats> {
        let now = Utc::now().to_rfc3339();

        let current = self
            .get_user_stats(user_id)
            .await?
            .ok_or_else(|| DatabaseError::Query("User stats not found".to_string()))?;

        let (new_streak, new_longest) = if let Some(last_date) = current.last_activity_date {
            let days_diff = (activity_date - last_date).num_days();
            if days_diff == 1 {
                // Consecutive day
                let new_streak = current.current_streak + 1;
                let new_longest = new_streak.max(current.longest_streak);
                (new_streak, new_longest)
            } else if days_diff == 0 {
                // Same day, no change
                (current.current_streak, current.longest_streak)
            } else {
                // Streak broken
                (1, current.longest_streak)
            }
        } else {
            // First activity
            (1, 1)
        };

        sqlx::query(
            r#"
            UPDATE user_stats 
            SET current_streak = ?, longest_streak = ?, last_activity_date = ?, updated_at = ?
            WHERE user_id = ?
            "#,
        )
        .bind(new_streak)
        .bind(new_longest)
        .bind(activity_date.to_string())
        .bind(now)
        .bind(user_id)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        self.get_user_stats(user_id)
            .await?
            .ok_or_else(|| DatabaseError::Query("User stats not found after update".to_string()))
    }

    /// Update streak from GitHub contribution calendar data
    ///
    /// This method takes pre-calculated streak information from the GitHub contribution
    /// calendar and updates the database accordingly. Unlike `update_streak`, this method
    /// uses the actual GitHub contribution history rather than app usage dates.
    ///
    /// Uses an atomic UPDATE with RETURNING to avoid race conditions and reduce
    /// database round-trips. The MAX() function ensures longest_streak never decreases.
    ///
    /// # Arguments
    /// * `user_id` - The user's database ID
    /// * `current_streak` - Current consecutive days with contributions (from GitHub)
    /// * `longest_streak` - Longest consecutive days ever (from GitHub)
    /// * `last_activity_date` - The last date with contributions (YYYY-MM-DD format)
    pub async fn update_streak_from_github(
        &self,
        user_id: i64,
        current_streak: i32,
        longest_streak: i32,
        last_activity_date: Option<&str>,
    ) -> DbResult<UserStats> {
        let now = Utc::now().to_rfc3339();

        // Use atomic UPDATE with RETURNING to avoid race conditions
        // MAX() ensures longest_streak never decreases
        let updated_row: UserStatsRow = sqlx::query_as(
            r#"
            UPDATE user_stats 
            SET 
                current_streak = ?,
                longest_streak = MAX(longest_streak, ?),
                last_activity_date = ?,
                updated_at = ?
            WHERE user_id = ?
            RETURNING *
            "#,
        )
        .bind(current_streak)
        .bind(longest_streak)
        .bind(last_activity_date)
        .bind(&now)
        .bind(user_id)
        .fetch_one(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        updated_row.try_into()
    }

    /// Increment activity counts
    pub async fn increment_activity_count(
        &self,
        user_id: i64,
        commits: i32,
        prs: i32,
        reviews: i32,
        issues: i32,
    ) -> DbResult<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE user_stats 
            SET total_commits = total_commits + ?,
                total_prs = total_prs + ?,
                total_reviews = total_reviews + ?,
                total_issues = total_issues + ?,
                updated_at = ?
            WHERE user_id = ?
            "#,
        )
        .bind(commits)
        .bind(prs)
        .bind(reviews)
        .bind(issues)
        .bind(now)
        .bind(user_id)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }
}

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
        let count: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM xp_history WHERE github_event_id = ?"
        )
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
}

/// User settings row from database
#[derive(Debug, FromRow)]
struct UserSettingsRow {
    id: i64,
    user_id: i64,
    notification_method: String,
    notify_xp_gain: i32,
    notify_level_up: i32,
    notify_badge_earned: i32,
    notify_streak_update: i32,
    notify_streak_milestone: i32,
    sync_interval_minutes: i32,
    background_sync: i32,
    sync_on_startup: i32,
    animations_enabled: i32,
    created_at: String,
    updated_at: String,
}

impl TryFrom<UserSettingsRow> for UserSettings {
    type Error = DatabaseError;

    fn try_from(row: UserSettingsRow) -> Result<Self, Self::Error> {
        Ok(UserSettings {
            id: row.id,
            user_id: row.user_id,
            notification_method: NotificationMethod::from_str(&row.notification_method),
            notify_xp_gain: row.notify_xp_gain != 0,
            notify_level_up: row.notify_level_up != 0,
            notify_badge_earned: row.notify_badge_earned != 0,
            notify_streak_update: row.notify_streak_update != 0,
            notify_streak_milestone: row.notify_streak_milestone != 0,
            sync_interval_minutes: row.sync_interval_minutes,
            background_sync: row.background_sync != 0,
            sync_on_startup: row.sync_on_startup != 0,
            animations_enabled: row.animations_enabled != 0,
            created_at: DateTime::parse_from_rfc3339(&row.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&row.updated_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

/// User settings repository operations
impl Database {
    /// Get user settings by user ID
    pub async fn get_user_settings(&self, user_id: i64) -> DbResult<Option<UserSettings>> {
        let row: Option<UserSettingsRow> =
            sqlx::query_as("SELECT * FROM user_settings WHERE user_id = ?")
                .bind(user_id)
                .fetch_optional(self.pool())
                .await
                .map_err(|e| DatabaseError::Query(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }

    /// Create default user settings
    pub async fn create_user_settings(&self, user_id: i64) -> DbResult<UserSettings> {
        let now = Utc::now().to_rfc3339();
        let defaults = settings_defaults::NOTIFICATION_METHOD;

        sqlx::query(
            r#"
            INSERT INTO user_settings (
                user_id, notification_method, 
                notify_xp_gain, notify_level_up, notify_badge_earned, 
                notify_streak_update, notify_streak_milestone,
                sync_interval_minutes, background_sync, sync_on_startup,
                animations_enabled, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(defaults.as_str())
        .bind(settings_defaults::NOTIFY_XP_GAIN as i32)
        .bind(settings_defaults::NOTIFY_LEVEL_UP as i32)
        .bind(settings_defaults::NOTIFY_BADGE_EARNED as i32)
        .bind(settings_defaults::NOTIFY_STREAK_UPDATE as i32)
        .bind(settings_defaults::NOTIFY_STREAK_MILESTONE as i32)
        .bind(settings_defaults::SYNC_INTERVAL_MINUTES)
        .bind(settings_defaults::BACKGROUND_SYNC as i32)
        .bind(settings_defaults::SYNC_ON_STARTUP as i32)
        .bind(settings_defaults::ANIMATIONS_ENABLED as i32)
        .bind(&now)
        .bind(&now)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        self.get_user_settings(user_id)
            .await?
            .ok_or_else(|| DatabaseError::Query("Settings not found after creation".to_string()))
    }

    /// Get or create user settings
    pub async fn get_or_create_user_settings(&self, user_id: i64) -> DbResult<UserSettings> {
        match self.get_user_settings(user_id).await? {
            Some(settings) => Ok(settings),
            None => self.create_user_settings(user_id).await,
        }
    }

    /// Update user settings
    pub async fn update_user_settings(&self, user_id: i64, settings: &UserSettings) -> DbResult<UserSettings> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE user_settings SET
                notification_method = ?,
                notify_xp_gain = ?,
                notify_level_up = ?,
                notify_badge_earned = ?,
                notify_streak_update = ?,
                notify_streak_milestone = ?,
                sync_interval_minutes = ?,
                background_sync = ?,
                sync_on_startup = ?,
                animations_enabled = ?,
                updated_at = ?
            WHERE user_id = ?
            "#,
        )
        .bind(settings.notification_method.as_str())
        .bind(settings.notify_xp_gain as i32)
        .bind(settings.notify_level_up as i32)
        .bind(settings.notify_badge_earned as i32)
        .bind(settings.notify_streak_update as i32)
        .bind(settings.notify_streak_milestone as i32)
        .bind(settings.sync_interval_minutes)
        .bind(settings.background_sync as i32)
        .bind(settings.sync_on_startup as i32)
        .bind(settings.animations_enabled as i32)
        .bind(&now)
        .bind(user_id)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        self.get_user_settings(user_id)
            .await?
            .ok_or_else(|| DatabaseError::Query("Settings not found after update".to_string()))
    }

    /// Reset user settings to defaults
    pub async fn reset_user_settings(&self, user_id: i64) -> DbResult<UserSettings> {
        let now = Utc::now().to_rfc3339();
        let defaults = settings_defaults::NOTIFICATION_METHOD;

        sqlx::query(
            r#"
            UPDATE user_settings SET
                notification_method = ?,
                notify_xp_gain = ?,
                notify_level_up = ?,
                notify_badge_earned = ?,
                notify_streak_update = ?,
                notify_streak_milestone = ?,
                sync_interval_minutes = ?,
                background_sync = ?,
                sync_on_startup = ?,
                animations_enabled = ?,
                updated_at = ?
            WHERE user_id = ?
            "#,
        )
        .bind(defaults.as_str())
        .bind(settings_defaults::NOTIFY_XP_GAIN as i32)
        .bind(settings_defaults::NOTIFY_LEVEL_UP as i32)
        .bind(settings_defaults::NOTIFY_BADGE_EARNED as i32)
        .bind(settings_defaults::NOTIFY_STREAK_UPDATE as i32)
        .bind(settings_defaults::NOTIFY_STREAK_MILESTONE as i32)
        .bind(settings_defaults::SYNC_INTERVAL_MINUTES)
        .bind(settings_defaults::BACKGROUND_SYNC as i32)
        .bind(settings_defaults::SYNC_ON_STARTUP as i32)
        .bind(settings_defaults::ANIMATIONS_ENABLED as i32)
        .bind(&now)
        .bind(user_id)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        self.get_user_settings(user_id)
            .await?
            .ok_or_else(|| DatabaseError::Query("Settings not found after reset".to_string()))
    }

    /// Clear all cache for a user
    pub async fn clear_user_cache(&self, user_id: i64) -> DbResult<ClearCacheResult> {
        let result = sqlx::query("DELETE FROM activity_cache WHERE user_id = ?")
            .bind(user_id)
            .execute(self.pool())
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(ClearCacheResult {
            cleared_entries: result.rows_affected() as i32,
            freed_bytes: 0, // SQLite doesn't easily report freed bytes
        })
    }

    /// Reset all user data (XP, badges, stats, challenges, cache)
    /// Does NOT delete user account or settings
    pub async fn reset_all_user_data(&self, user_id: i64) -> DbResult<()> {
        let now = Utc::now().to_rfc3339();

        // Delete XP history
        sqlx::query("DELETE FROM xp_history WHERE user_id = ?")
            .bind(user_id)
            .execute(self.pool())
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        // Delete badges
        sqlx::query("DELETE FROM badges WHERE user_id = ?")
            .bind(user_id)
            .execute(self.pool())
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        // Delete challenges
        sqlx::query("DELETE FROM challenges WHERE user_id = ?")
            .bind(user_id)
            .execute(self.pool())
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        // Delete cache
        sqlx::query("DELETE FROM activity_cache WHERE user_id = ?")
            .bind(user_id)
            .execute(self.pool())
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        // Reset user_stats to defaults
        sqlx::query(
            r#"
            UPDATE user_stats SET
                total_xp = 0,
                current_level = 1,
                current_streak = 0,
                longest_streak = 0,
                last_activity_date = NULL,
                total_commits = 0,
                total_prs = 0,
                total_reviews = 0,
                total_issues = 0,
                updated_at = ?
            WHERE user_id = ?
            "#,
        )
        .bind(&now)
        .bind(user_id)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
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
        let result: Option<String> = sqlx::query_scalar(
            "SELECT start_stats_json FROM challenges WHERE id = ?"
        )
        .bind(challenge_id)
        .fetch_one(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result)
    }

    /// Get the most recent daily challenge date for a user
    pub async fn get_last_daily_challenge_date(&self, user_id: i64) -> DbResult<Option<chrono::NaiveDate>> {
        let result: Option<String> = sqlx::query_scalar(
            r#"
            SELECT DATE(start_date) FROM challenges 
            WHERE user_id = ? AND challenge_type = 'daily'
            ORDER BY start_date DESC
            LIMIT 1
            "#
        )
        .bind(user_id)
        .fetch_optional(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?
        .flatten();

        Ok(result.and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()))
    }

    /// Get the most recent weekly challenge date for a user
    pub async fn get_last_weekly_challenge_date(&self, user_id: i64) -> DbResult<Option<chrono::NaiveDate>> {
        let result: Option<String> = sqlx::query_scalar(
            r#"
            SELECT DATE(start_date) FROM challenges 
            WHERE user_id = ? AND challenge_type = 'weekly'
            ORDER BY start_date DESC
            LIMIT 1
            "#
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

        Ok(Challenge {
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
            "SELECT COUNT(*) FROM challenges WHERE user_id = ? AND status = 'completed'"
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

        // Calculate consecutive weeks
        let mut consecutive = 0;
        let mut last_week: Option<i64> = None;

        for row in &rows {
            if let Some(completed_at_str) = row.get::<Option<&str>, _>("completed_at") {
                if let Ok(completed_at) = DateTime::parse_from_rfc3339(completed_at_str) {
                    let week_number = completed_at.timestamp() / (7 * 24 * 60 * 60);
                    
                    match last_week {
                        None => {
                            consecutive = 1;
                            last_week = Some(week_number);
                        }
                        Some(last) => {
                            if week_number == last - 1 {
                                consecutive += 1;
                                last_week = Some(week_number);
                            } else if week_number != last {
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

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> Database {
        Database::in_memory().await.expect("Failed to create test database")
    }

    #[tokio::test]
    async fn test_create_and_get_user() {
        let db = setup_test_db().await;

        let user = db
            .create_user(12345, "testuser", Some("https://avatar.url"), "encrypted_token", None, None)
            .await
            .expect("Should create user");

        assert_eq!(user.github_id, 12345);
        assert_eq!(user.username, "testuser");

        let fetched = db
            .get_user_by_github_id(12345)
            .await
            .expect("Should fetch user")
            .expect("User should exist");

        assert_eq!(fetched.id, user.id);
    }

    #[tokio::test]
    async fn test_user_stats_created_with_user() {
        let db = setup_test_db().await;

        let user = db
            .create_user(12345, "testuser", None, "token", None, None)
            .await
            .expect("Should create user");

        let stats = db
            .get_user_stats(user.id)
            .await
            .expect("Should get stats")
            .expect("Stats should exist");

        assert_eq!(stats.total_xp, 0);
        assert_eq!(stats.current_level, 1);
    }

    #[tokio::test]
    async fn test_add_xp() {
        let db = setup_test_db().await;

        let user = db
            .create_user(12345, "testuser", None, "token", None, None)
            .await
            .expect("Should create user");

        let stats = db.add_xp(user.id, 100).await.expect("Should add XP");

        assert_eq!(stats.total_xp, 100);
        assert_eq!(stats.current_level, 2); // 100 XP = level 2
    }

    #[tokio::test]
    async fn test_streak_tracking() {
        let db = setup_test_db().await;

        let user = db
            .create_user(12345, "testuser", None, "token", None, None)
            .await
            .expect("Should create user");

        let today = Utc::now().date_naive();

        let stats = db
            .update_streak(user.id, today)
            .await
            .expect("Should update streak");

        assert_eq!(stats.current_streak, 1);
        assert_eq!(stats.longest_streak, 1);
    }

    #[tokio::test]
    async fn test_badge_operations() {
        let db = setup_test_db().await;

        let user = db
            .create_user(12345, "testuser", None, "token", None, None)
            .await
            .expect("Should create user");

        db.award_badge(user.id, "milestone", "first_blood")
            .await
            .expect("Should award badge");

        let has_badge = db
            .has_badge(user.id, "first_blood")
            .await
            .expect("Should check badge");

        assert!(has_badge);

        let badges = db
            .get_user_badges(user.id)
            .await
            .expect("Should get badges");

        assert_eq!(badges.len(), 1);
        assert_eq!(badges[0].badge_id, "first_blood");
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let db = setup_test_db().await;

        let user = db
            .create_user(12345, "testuser", None, "token", None, None)
            .await
            .expect("Should create user");

        let expires = Utc::now() + chrono::Duration::hours(1);
        db.save_cache(user.id, "test_data", r#"{"test": true}"#, expires)
            .await
            .expect("Should save cache");

        let cached = db
            .get_valid_cache(user.id, "test_data")
            .await
            .expect("Should get cache")
            .expect("Cache should exist");

        assert_eq!(cached, r#"{"test": true}"#);
    }

    #[tokio::test]
    async fn test_create_challenge() {
        let db = setup_test_db().await;

        let user = db
            .create_user(12345, "testuser", None, "token", None, None)
            .await
            .expect("Should create user");

        let start = Utc::now();
        let end = start + chrono::Duration::days(7);

        let challenge = db
            .create_challenge(user.id, "weekly", "commits", 10, 100, start, end)
            .await
            .expect("Should create challenge");

        assert_eq!(challenge.user_id, user.id);
        assert_eq!(challenge.challenge_type, "weekly");
        assert_eq!(challenge.target_metric, "commits");
        assert_eq!(challenge.target_value, 10);
        assert_eq!(challenge.current_value, 0);
        assert_eq!(challenge.reward_xp, 100);
        assert_eq!(challenge.status, "active");
    }

    #[tokio::test]
    async fn test_get_active_challenges() {
        let db = setup_test_db().await;

        let user = db
            .create_user(12345, "testuser", None, "token", None, None)
            .await
            .expect("Should create user");

        let start = Utc::now();
        let end = start + chrono::Duration::days(7);

        db.create_challenge(user.id, "weekly", "commits", 10, 100, start, end)
            .await
            .expect("Should create challenge 1");

        db.create_challenge(user.id, "daily", "prs", 2, 50, start, start + chrono::Duration::days(1))
            .await
            .expect("Should create challenge 2");

        let challenges = db
            .get_active_challenges(user.id)
            .await
            .expect("Should get active challenges");

        assert_eq!(challenges.len(), 2);
    }

    #[tokio::test]
    async fn test_update_challenge_progress() {
        let db = setup_test_db().await;

        let user = db
            .create_user(12345, "testuser", None, "token", None, None)
            .await
            .expect("Should create user");

        let start = Utc::now();
        let end = start + chrono::Duration::days(7);

        let challenge = db
            .create_challenge(user.id, "weekly", "commits", 10, 100, start, end)
            .await
            .expect("Should create challenge");

        let updated = db
            .update_challenge_progress(challenge.id, 5)
            .await
            .expect("Should update progress");

        assert_eq!(updated.current_value, 5);
        assert_eq!(updated.status, "active");
    }

    #[tokio::test]
    async fn test_complete_challenge() {
        let db = setup_test_db().await;

        let user = db
            .create_user(12345, "testuser", None, "token", None, None)
            .await
            .expect("Should create user");

        let start = Utc::now();
        let end = start + chrono::Duration::days(7);

        let challenge = db
            .create_challenge(user.id, "weekly", "commits", 10, 100, start, end)
            .await
            .expect("Should create challenge");

        let completed = db
            .complete_challenge(challenge.id)
            .await
            .expect("Should complete challenge");

        assert_eq!(completed.status, "completed");
        assert!(completed.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_fail_challenge() {
        let db = setup_test_db().await;

        let user = db
            .create_user(12345, "testuser", None, "token", None, None)
            .await
            .expect("Should create user");

        let start = Utc::now();
        let end = start + chrono::Duration::days(7);

        let challenge = db
            .create_challenge(user.id, "weekly", "commits", 10, 100, start, end)
            .await
            .expect("Should create challenge");

        let failed = db
            .fail_challenge(challenge.id)
            .await
            .expect("Should fail challenge");

        assert_eq!(failed.status, "failed");
    }

    #[tokio::test]
    async fn test_has_active_challenge() {
        let db = setup_test_db().await;

        let user = db
            .create_user(12345, "testuser", None, "token", None, None)
            .await
            .expect("Should create user");

        let start = Utc::now();
        let end = start + chrono::Duration::days(7);

        // No challenge yet
        let has = db
            .has_active_challenge(user.id, "weekly", "commits")
            .await
            .expect("Should check");
        assert!(!has);

        // Create challenge
        db.create_challenge(user.id, "weekly", "commits", 10, 100, start, end)
            .await
            .expect("Should create challenge");

        // Now should have one
        let has = db
            .has_active_challenge(user.id, "weekly", "commits")
            .await
            .expect("Should check");
        assert!(has);
    }

    #[tokio::test]
    async fn test_delete_challenge() {
        let db = setup_test_db().await;

        let user = db
            .create_user(12345, "testuser", None, "token", None, None)
            .await
            .expect("Should create user");

        let start = Utc::now();
        let end = start + chrono::Duration::days(7);

        let challenge = db
            .create_challenge(user.id, "weekly", "commits", 10, 100, start, end)
            .await
            .expect("Should create challenge");

        db.delete_challenge(challenge.id)
            .await
            .expect("Should delete challenge");

        let challenges = db
            .get_active_challenges(user.id)
            .await
            .expect("Should get challenges");

        assert_eq!(challenges.len(), 0);
    }

    #[tokio::test]
    async fn test_challenge_completion_count() {
        let db = setup_test_db().await;

        let user = db
            .create_user(12345, "testuser", None, "token", None, None)
            .await
            .expect("Should create user");

        let start = Utc::now();
        let end = start + chrono::Duration::days(7);

        let challenge1 = db
            .create_challenge(user.id, "weekly", "commits", 10, 100, start, end)
            .await
            .expect("Should create challenge 1");

        let challenge2 = db
            .create_challenge(user.id, "daily", "prs", 2, 50, start, start + chrono::Duration::days(1))
            .await
            .expect("Should create challenge 2");

        // Complete one challenge
        db.complete_challenge(challenge1.id)
            .await
            .expect("Should complete challenge 1");

        let count = db
            .get_challenge_completion_count(user.id)
            .await
            .expect("Should get count");

        assert_eq!(count, 1);

        // Complete second challenge
        db.complete_challenge(challenge2.id)
            .await
            .expect("Should complete challenge 2");

        let count = db
            .get_challenge_completion_count(user.id)
            .await
            .expect("Should get count");

        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_create_challenge_with_stats() {
        let db = setup_test_db().await;

        let user = db
            .create_user(12345, "testuser", None, "token", None, None)
            .await
            .expect("Should create user");

        let start = Utc::now();
        let end = start + chrono::Duration::days(7);
        let start_stats = r#"{"commits":100,"prs":10,"reviews":5,"issues":3}"#;

        let challenge = db
            .create_challenge_with_stats(
                user.id,
                "weekly",
                "commits",
                10,
                100,
                start,
                end,
                start_stats,
            )
            .await
            .expect("Should create challenge with stats");

        assert_eq!(challenge.target_metric, "commits");
        assert_eq!(challenge.target_value, 10);

        // Verify start stats can be retrieved
        let retrieved_stats = db
            .get_challenge_start_stats(challenge.id)
            .await
            .expect("Should get start stats");

        assert!(retrieved_stats.is_some());
        assert_eq!(retrieved_stats.unwrap(), start_stats);
    }

    #[tokio::test]
    async fn test_get_last_daily_challenge_date() {
        let db = setup_test_db().await;

        let user = db
            .create_user(12345, "testuser", None, "token", None, None)
            .await
            .expect("Should create user");

        // Initially no challenges
        let last_date = db
            .get_last_daily_challenge_date(user.id)
            .await
            .expect("Should check last date");
        assert!(last_date.is_none());

        // Create a daily challenge
        let start = Utc::now();
        let end = start + chrono::Duration::days(1);

        db.create_challenge(user.id, "daily", "commits", 5, 50, start, end)
            .await
            .expect("Should create daily challenge");

        // Now should have a date
        let last_date = db
            .get_last_daily_challenge_date(user.id)
            .await
            .expect("Should check last date");
        assert!(last_date.is_some());
    }

    #[tokio::test]
    async fn test_get_last_weekly_challenge_date() {
        let db = setup_test_db().await;

        let user = db
            .create_user(12345, "testuser", None, "token", None, None)
            .await
            .expect("Should create user");

        // Initially no challenges
        let last_date = db
            .get_last_weekly_challenge_date(user.id)
            .await
            .expect("Should check last date");
        assert!(last_date.is_none());

        // Create a weekly challenge
        let start = Utc::now();
        let end = start + chrono::Duration::days(7);

        db.create_challenge(user.id, "weekly", "commits", 10, 100, start, end)
            .await
            .expect("Should create weekly challenge");

        // Now should have a date
        let last_date = db
            .get_last_weekly_challenge_date(user.id)
            .await
            .expect("Should check last date");
        assert!(last_date.is_some());
    }

    #[tokio::test]
    async fn test_progress_capped_at_target() {
        let db = setup_test_db().await;

        let user = db
            .create_user(12345, "testuser", None, "token", None, None)
            .await
            .expect("Should create user");

        let start = Utc::now();
        let end = start + chrono::Duration::days(7);

        let challenge = db
            .create_challenge(user.id, "weekly", "commits", 10, 100, start, end)
            .await
            .expect("Should create challenge");

        // Update progress to more than target
        let updated = db
            .update_challenge_progress(challenge.id, 15)
            .await
            .expect("Should update progress");

        // Progress should be capped at target
        assert_eq!(updated.current_value, 10);
        assert_eq!(updated.status, "completed");
    }
}
