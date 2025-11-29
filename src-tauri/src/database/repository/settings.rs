//! User settings repository operations

use chrono::{DateTime, Utc};
use sqlx::FromRow;

use crate::database::connection::{Database, DatabaseError, DbResult};
use crate::database::models::{
    settings_defaults, ClearCacheResult, NotificationMethod, UserSettings,
};

/// User settings row from database
#[derive(Debug, FromRow)]
pub(crate) struct UserSettingsRow {
    pub id: i64,
    pub user_id: i64,
    pub notification_method: String,
    pub notify_xp_gain: i32,
    pub notify_level_up: i32,
    pub notify_badge_earned: i32,
    pub notify_streak_update: i32,
    pub notify_streak_milestone: i32,
    pub sync_interval_minutes: i32,
    pub background_sync: i32,
    pub sync_on_startup: i32,
    pub animations_enabled: i32,
    pub created_at: String,
    pub updated_at: String,
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
}
