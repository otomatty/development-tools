//! Gamification commands for Tauri
//!
//! These commands handle the gamification features: XP, levels, badges, etc.

use tauri::{command, State};

use super::auth::AppState;
use crate::database::{level, Badge, UserStats, XpHistoryEntry};

/// Level info for frontend
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelInfo {
    pub current_level: u32,
    pub total_xp: u32,
    pub xp_for_current_level: u32,
    pub xp_for_next_level: u32,
    pub xp_to_next_level: u32,
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
            let total_xp = s.total_xp as u32;
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
    vec![
        // Milestone badges
        BadgeDefinition {
            id: "first_blood".to_string(),
            name: "First Blood".to_string(),
            description: "Make your first commit".to_string(),
            badge_type: "milestone".to_string(),
            rarity: "bronze".to_string(),
            icon: "🎯".to_string(),
        },
        BadgeDefinition {
            id: "century".to_string(),
            name: "Century".to_string(),
            description: "Reach 100 commits".to_string(),
            badge_type: "milestone".to_string(),
            rarity: "silver".to_string(),
            icon: "💯".to_string(),
        },
        BadgeDefinition {
            id: "thousand_cuts".to_string(),
            name: "Thousand Cuts".to_string(),
            description: "Reach 1,000 commits".to_string(),
            badge_type: "milestone".to_string(),
            rarity: "gold".to_string(),
            icon: "⚔️".to_string(),
        },
        BadgeDefinition {
            id: "legendary".to_string(),
            name: "Legendary".to_string(),
            description: "Reach 10,000 commits".to_string(),
            badge_type: "milestone".to_string(),
            rarity: "platinum".to_string(),
            icon: "🏆".to_string(),
        },
        // Streak badges
        BadgeDefinition {
            id: "on_fire".to_string(),
            name: "On Fire".to_string(),
            description: "7 day commit streak".to_string(),
            badge_type: "streak".to_string(),
            rarity: "bronze".to_string(),
            icon: "🔥".to_string(),
        },
        BadgeDefinition {
            id: "unstoppable".to_string(),
            name: "Unstoppable".to_string(),
            description: "30 day commit streak".to_string(),
            badge_type: "streak".to_string(),
            rarity: "silver".to_string(),
            icon: "💪".to_string(),
        },
        BadgeDefinition {
            id: "immortal".to_string(),
            name: "Immortal".to_string(),
            description: "365 day commit streak".to_string(),
            badge_type: "streak".to_string(),
            rarity: "platinum".to_string(),
            icon: "👑".to_string(),
        },
        // Collaboration badges
        BadgeDefinition {
            id: "team_player".to_string(),
            name: "Team Player".to_string(),
            description: "Complete your first review".to_string(),
            badge_type: "collaboration".to_string(),
            rarity: "bronze".to_string(),
            icon: "🤝".to_string(),
        },
        BadgeDefinition {
            id: "mentor".to_string(),
            name: "Mentor".to_string(),
            description: "Complete 50 reviews".to_string(),
            badge_type: "collaboration".to_string(),
            rarity: "silver".to_string(),
            icon: "🎓".to_string(),
        },
        BadgeDefinition {
            id: "guardian".to_string(),
            name: "Guardian".to_string(),
            description: "Merge 100 PRs".to_string(),
            badge_type: "collaboration".to_string(),
            rarity: "gold".to_string(),
            icon: "🛡️".to_string(),
        },
        // Quality badges
        BadgeDefinition {
            id: "clean_coder".to_string(),
            name: "Clean Coder".to_string(),
            description: "90%+ PR merge rate (10+ PRs)".to_string(),
            badge_type: "quality".to_string(),
            rarity: "gold".to_string(),
            icon: "✨".to_string(),
        },
        BadgeDefinition {
            id: "bug_hunter".to_string(),
            name: "Bug Hunter".to_string(),
            description: "Close 50 issues".to_string(),
            badge_type: "quality".to_string(),
            rarity: "silver".to_string(),
            icon: "🐛".to_string(),
        },
        BadgeDefinition {
            id: "polyglot".to_string(),
            name: "Polyglot".to_string(),
            description: "Use 5+ programming languages".to_string(),
            badge_type: "quality".to_string(),
            rarity: "silver".to_string(),
            icon: "🌍".to_string(),
        },
    ]
}

