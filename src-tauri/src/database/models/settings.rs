//! User settings models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Notification method options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NotificationMethod {
    AppOnly,
    OsOnly,
    #[default]
    Both,
    None,
}

impl NotificationMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationMethod::AppOnly => "app_only",
            NotificationMethod::OsOnly => "os_only",
            NotificationMethod::Both => "both",
            NotificationMethod::None => "none",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "app_only" => NotificationMethod::AppOnly,
            "os_only" => NotificationMethod::OsOnly,
            "both" => NotificationMethod::Both,
            "none" => NotificationMethod::None,
            _ => NotificationMethod::Both, // default
        }
    }
}

/// User settings model - stores user preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSettings {
    pub id: i64,
    pub user_id: i64,

    // Notification settings
    pub notification_method: NotificationMethod,
    pub notify_xp_gain: bool,
    pub notify_level_up: bool,
    pub notify_badge_earned: bool,
    pub notify_streak_update: bool,
    pub notify_streak_milestone: bool,

    // Sync settings
    pub sync_interval_minutes: i32,
    pub background_sync: bool,
    pub sync_on_startup: bool,

    // Appearance settings
    pub animations_enabled: bool,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            id: 0,
            user_id: 0,
            notification_method: NotificationMethod::Both,
            notify_xp_gain: true,
            notify_level_up: true,
            notify_badge_earned: true,
            notify_streak_update: true,
            notify_streak_milestone: true,
            sync_interval_minutes: 60,
            background_sync: true,
            sync_on_startup: true,
            animations_enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

/// Settings defaults as constants
pub mod settings_defaults {
    use super::NotificationMethod;

    pub const NOTIFICATION_METHOD: NotificationMethod = NotificationMethod::Both;
    pub const NOTIFY_XP_GAIN: bool = true;
    pub const NOTIFY_LEVEL_UP: bool = true;
    pub const NOTIFY_BADGE_EARNED: bool = true;
    pub const NOTIFY_STREAK_UPDATE: bool = true;
    pub const NOTIFY_STREAK_MILESTONE: bool = true;
    pub const SYNC_INTERVAL_MINUTES: i32 = 60;
    pub const BACKGROUND_SYNC: bool = true;
    pub const SYNC_ON_STARTUP: bool = true;
    pub const ANIMATIONS_ENABLED: bool = true;

    /// Available sync interval options (minutes, label)
    /// This is the single source of truth - frontend should fetch this via command
    pub const SYNC_INTERVALS: &[(i32, &str)] = &[
        (5, "5分"),
        (15, "15分"),
        (30, "30分"),
        (60, "1時間"),
        (180, "3時間"),
        (0, "手動のみ"),
    ];
}

/// Database info for display in settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseInfo {
    pub path: String,
    pub size_bytes: u64,
    pub cache_size_bytes: u64,
}

/// Result of clearing cache
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearCacheResult {
    pub cleared_entries: i32,
    pub freed_bytes: u64,
}
