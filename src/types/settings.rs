//! Settings-related types

use serde::{Deserialize, Serialize};

/// ユーザー設定
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserSettings {
    pub id: i64,
    pub user_id: i64,
    pub notification_method: String,
    pub notify_xp_gain: bool,
    pub notify_level_up: bool,
    pub notify_badge_earned: bool,
    pub notify_streak_update: bool,
    pub notify_streak_milestone: bool,
    pub sync_interval_minutes: i32,
    pub background_sync: bool,
    pub sync_on_startup: bool,
    pub animations_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 設定更新リクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingsRequest {
    pub notification_method: String,
    pub notify_xp_gain: bool,
    pub notify_level_up: bool,
    pub notify_badge_earned: bool,
    pub notify_streak_update: bool,
    pub notify_streak_milestone: bool,
    pub sync_interval_minutes: i32,
    pub background_sync: bool,
    pub sync_on_startup: bool,
    pub animations_enabled: bool,
}

impl From<&UserSettings> for UpdateSettingsRequest {
    fn from(settings: &UserSettings) -> Self {
        Self {
            notification_method: settings.notification_method.clone(),
            notify_xp_gain: settings.notify_xp_gain,
            notify_level_up: settings.notify_level_up,
            notify_badge_earned: settings.notify_badge_earned,
            notify_streak_update: settings.notify_streak_update,
            notify_streak_milestone: settings.notify_streak_milestone,
            sync_interval_minutes: settings.sync_interval_minutes,
            background_sync: settings.background_sync,
            sync_on_startup: settings.sync_on_startup,
            animations_enabled: settings.animations_enabled,
        }
    }
}

/// データベース情報
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseInfo {
    pub path: String,
    pub size_bytes: u64,
    pub cache_size_bytes: u64,
}

/// キャッシュクリア結果
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClearCacheResult {
    pub cleared_entries: i32,
    pub freed_bytes: u64,
}

/// アプリケーション情報
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub build_date: String,
    pub tauri_version: String,
    pub leptos_version: String,
    pub rust_version: String,
}

/// 通知方法の選択肢
/// 
/// **IMPORTANT**: This enum must be kept in sync with the backend definition at:
/// `src-tauri/src/database/models.rs::NotificationMethod`
/// 
/// Both implementations use the same string values (app_only, os_only, both, none)
/// for serialization to ensure compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
            _ => NotificationMethod::Both,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            NotificationMethod::AppOnly => "アプリ内のみ",
            NotificationMethod::OsOnly => "OSネイティブのみ",
            NotificationMethod::Both => "両方",
            NotificationMethod::None => "通知なし",
        }
    }
}

/// 同期間隔の選択肢
/// 
/// **IMPORTANT**: This constant must be kept in sync with the backend definition at:
/// `src-tauri/src/database/models.rs::settings_defaults::SYNC_INTERVALS`
/// 
/// Alternatively, use `tauri_api::get_sync_intervals()` to fetch the authoritative
/// list from the backend, which is the recommended approach for dynamic UI.
pub const SYNC_INTERVALS: &[(i32, &str)] = &[
    (5, "5分"),
    (15, "15分"),
    (30, "30分"),
    (60, "1時間"),
    (180, "3時間"),
    (0, "手動のみ"),
];

/// 同期間隔のラベルを取得
pub fn get_sync_interval_label(minutes: i32) -> &'static str {
    SYNC_INTERVALS
        .iter()
        .find(|(m, _)| *m == minutes)
        .map(|(_, label)| *label)
        .unwrap_or("不明")
}

/// 同期間隔オプション（バックエンドから取得）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncIntervalOption {
    pub value: i32,
    pub label: String,
}
