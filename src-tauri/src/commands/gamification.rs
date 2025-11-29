//! Gamification commands for Tauri
//!
//! These commands handle the gamification features: XP, levels, badges, etc.

use tauri::{command, State};

use super::auth::AppState;
use crate::database::{badge, level, Badge, UserStats, XpHistoryEntry};

/// Level info for frontend
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelInfo {
    pub current_level: i32,
    pub total_xp: i32,
    pub xp_for_current_level: i32,
    pub xp_for_next_level: i32,
    pub xp_to_next_level: i32,
    pub progress_percent: f32,
}

/// Get level info for current user
#[command]
pub async fn get_level_info(state: State<'_, AppState>) -> Result<Option<LevelInfo>, String> {
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(u) = user {
        let stats = state
            .db
            .get_user_stats(u.id)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(s) = stats {
            let total_xp = s.total_xp;
            let current_level = level::level_from_xp(total_xp);

            Ok(Some(LevelInfo {
                current_level,
                total_xp,
                xp_for_current_level: level::xp_for_level(current_level),
                xp_for_next_level: level::xp_for_level(current_level + 1),
                xp_to_next_level: level::xp_to_next_level(total_xp),
                progress_percent: level::progress_to_next_level(total_xp),
            }))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

/// Add XP to current user (for testing/admin purposes)
#[command]
pub async fn add_xp(
    state: State<'_, AppState>,
    amount: i32,
    action_type: String,
    description: Option<String>,
) -> Result<UserStats, String> {
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Not logged in")?;

    // Record XP gain
    state
        .db
        .record_xp_gain(
            user.id,
            &action_type,
            amount,
            description.as_deref(),
            None,
        )
        .await
        .map_err(|e| e.to_string())?;

    // Update user stats
    state
        .db
        .add_xp(user.id, amount)
        .await
        .map_err(|e| e.to_string())
}

/// Get user's badges
#[command]
pub async fn get_badges(state: State<'_, AppState>) -> Result<Vec<Badge>, String> {
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Not logged in")?;

    state
        .db
        .get_user_badges(user.id)
        .await
        .map_err(|e| e.to_string())
}

/// Award a badge to current user
#[command]
pub async fn award_badge(
    state: State<'_, AppState>,
    badge_type: String,
    badge_id: String,
) -> Result<bool, String> {
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Not logged in")?;

    // Check if already has badge
    let has_badge = state
        .db
        .has_badge(user.id, &badge_id)
        .await
        .map_err(|e| e.to_string())?;

    if has_badge {
        return Ok(false);
    }

    state
        .db
        .award_badge(user.id, &badge_type, &badge_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(true)
}

/// Get recent XP history
#[command]
pub async fn get_xp_history(
    state: State<'_, AppState>,
    limit: Option<i32>,
) -> Result<Vec<XpHistoryEntry>, String> {
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Not logged in")?;

    state
        .db
        .get_recent_xp_history(user.id, limit.unwrap_or(10))
        .await
        .map_err(|e| e.to_string())
}

/// Badge definition for frontend
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgeDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub badge_type: String,
    pub rarity: String,
    pub icon: String,
}

/// Get all available badge definitions
#[command]
pub fn get_badge_definitions() -> Vec<BadgeDefinition> {
    badge::get_all_badge_definitions()
        .into_iter()
        .map(|def| BadgeDefinition {
            id: def.id,
            name: def.name,
            description: def.description,
            badge_type: def.badge_type,
            rarity: def.rarity,
            icon: def.icon,
        })
        .collect()
}

