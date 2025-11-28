//! Settings commands
//!
//! Tauri commands for managing user settings.

use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::database::models::{ClearCacheResult, DatabaseInfo, NotificationMethod, UserSettings};

use super::AppState;

/// Application information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub build_date: String,
    pub tauri_version: String,
    pub leptos_version: String,
    pub rust_version: String,
}

/// Settings update request (without id and user_id)
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

/// Get user settings
#[tauri::command]
pub async fn get_settings(state: tauri::State<'_, AppState>) -> Result<UserSettings, String> {
    // Get current user
    let user = state.db
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Not logged in".to_string())?;

    // Get or create settings
    let settings = state.db
        .get_or_create_user_settings(user.id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(settings)
}

/// Update user settings
#[tauri::command]
pub async fn update_settings(
    state: tauri::State<'_, AppState>,
    settings: UpdateSettingsRequest,
) -> Result<UserSettings, String> {
    // Get current user
    let user = state.db
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Not logged in".to_string())?;

    // Get existing settings
    let mut existing = state.db
        .get_or_create_user_settings(user.id)
        .await
        .map_err(|e| e.to_string())?;

    // Update fields
    existing.notification_method = NotificationMethod::from_str(&settings.notification_method);
    existing.notify_xp_gain = settings.notify_xp_gain;
    existing.notify_level_up = settings.notify_level_up;
    existing.notify_badge_earned = settings.notify_badge_earned;
    existing.notify_streak_update = settings.notify_streak_update;
    existing.notify_streak_milestone = settings.notify_streak_milestone;
    existing.sync_interval_minutes = settings.sync_interval_minutes;
    existing.background_sync = settings.background_sync;
    existing.sync_on_startup = settings.sync_on_startup;
    existing.animations_enabled = settings.animations_enabled;

    // Save
    let updated = state.db
        .update_user_settings(user.id, &existing)
        .await
        .map_err(|e| e.to_string())?;

    Ok(updated)
}

/// Reset settings to defaults
#[tauri::command]
pub async fn reset_settings(state: tauri::State<'_, AppState>) -> Result<UserSettings, String> {
    // Get current user
    let user = state.db
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Not logged in".to_string())?;

    // Reset settings
    let settings = state.db
        .reset_user_settings(user.id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(settings)
}

/// Clear cache
#[tauri::command]
pub async fn clear_cache(state: tauri::State<'_, AppState>) -> Result<ClearCacheResult, String> {
    // Get current user
    let user = state.db
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Not logged in".to_string())?;

    // Clear cache
    let result = state.db
        .clear_user_cache(user.id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result)
}

/// Get database info
#[tauri::command]
pub async fn get_database_info(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<DatabaseInfo, String> {
    // Get current user
    let user = state.db
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Not logged in".to_string())?;

    // Get database path
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let db_path = app_data_dir.join("app.db");

    // Get database size
    let size_bytes = std::fs::metadata(&db_path)
        .map(|m| m.len())
        .unwrap_or(0);

    // Get cache size
    let cache_size_bytes = state.db
        .get_user_cache_size(user.id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(DatabaseInfo {
        path: db_path.to_string_lossy().to_string(),
        size_bytes,
        cache_size_bytes,
    })
}

/// Reset all user data (XP, badges, stats, challenges, cache)
#[tauri::command]
pub async fn reset_all_data(state: tauri::State<'_, AppState>) -> Result<(), String> {
    // Get current user
    let user = state.db
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Not logged in".to_string())?;

    // Reset all data
    state.db
        .reset_all_user_data(user.id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Sync interval option for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncIntervalOption {
    pub value: i32,
    pub label: String,
}

/// Get available sync interval options
/// This is the single source of truth for sync interval configuration
#[tauri::command]
pub fn get_sync_intervals() -> Vec<SyncIntervalOption> {
    use crate::database::models::settings_defaults::SYNC_INTERVALS;
    
    SYNC_INTERVALS
        .iter()
        .map(|(value, label)| SyncIntervalOption {
            value: *value,
            label: label.to_string(),
        })
        .collect()
}

/// Export user data as JSON
#[tauri::command]
pub async fn export_data(state: tauri::State<'_, AppState>) -> Result<String, String> {
    use crate::database::models::{ExportData, ExportUser};
    
    // Get current user
    let user = state.db
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Not logged in".to_string())?;

    // Get user stats
    let stats = state.db
        .get_user_stats(user.id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "User stats not found".to_string())?;

    // Get badges
    let badges = state.db
        .get_user_badges(user.id)
        .await
        .map_err(|e| e.to_string())?;

    // Get XP history (last 1000 entries)
    let xp_history = state.db
        .get_recent_xp_history(user.id, 1000)
        .await
        .map_err(|e| e.to_string())?;

    // Create export data
    let export = ExportData {
        exported_at: chrono::Utc::now().to_rfc3339(),
        version: "1.0".to_string(),
        user: ExportUser {
            github_id: user.github_id,
            username: user.username.clone(),
        },
        stats,
        badges,
        xp_history,
    };

    // Serialize to JSON
    serde_json::to_string_pretty(&export)
        .map_err(|e| format!("Failed to serialize data: {}", e))
}

/// Get application information
#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_date: env!("BUILD_DATE").to_string(),
        tauri_version: tauri::VERSION.to_string(),
        leptos_version: env!("LEPTOS_VERSION").to_string(),
        rust_version: env!("RUST_VERSION").to_string(),
    }
}

/// Open URL in external browser
#[tauri::command]
pub async fn open_external_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    
    app.opener()
        .open_url(&url, None::<String>)
        .map_err(|e| format!("Failed to open URL: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_method_conversion() {
        assert_eq!(NotificationMethod::Both.as_str(), "both");
        assert_eq!(NotificationMethod::AppOnly.as_str(), "app_only");
        assert_eq!(NotificationMethod::OsOnly.as_str(), "os_only");
        assert_eq!(NotificationMethod::None.as_str(), "none");

        assert_eq!(NotificationMethod::from_str("both"), NotificationMethod::Both);
        assert_eq!(NotificationMethod::from_str("app_only"), NotificationMethod::AppOnly);
        assert_eq!(NotificationMethod::from_str("os_only"), NotificationMethod::OsOnly);
        assert_eq!(NotificationMethod::from_str("none"), NotificationMethod::None);
        assert_eq!(NotificationMethod::from_str("invalid"), NotificationMethod::Both); // default
    }
}

