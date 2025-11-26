//! GitHub API commands for Tauri
//!
//! These commands handle fetching data from the GitHub API.

use chrono::Utc;
use tauri::{command, Emitter, State};

use super::auth::AppState;
use crate::database::{badge, level, streak, xp, UserStats, XpActionType};
use crate::github::{GitHubClient, GitHubStats, GitHubUser};

/// Get GitHub user profile
#[command]
pub async fn get_github_user(state: State<'_, AppState>) -> Result<GitHubUser, String> {
    let token = state
        .token_manager
        .get_access_token()
        .await
        .map_err(|e| e.to_string())?;

    let client = GitHubClient::new(token);
    client.get_user().await.map_err(|e| e.to_string())
}

/// Get GitHub stats for the current user
#[command]
pub async fn get_github_stats(state: State<'_, AppState>) -> Result<GitHubStats, String> {
    let token = state
        .token_manager
        .get_access_token()
        .await
        .map_err(|e| e.to_string())?;

    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Not logged in")?;

    let client = GitHubClient::new(token);
    client
        .get_user_stats(&user.username)
        .await
        .map_err(|e| e.to_string())
}

/// Get local user stats (gamification data)
#[command]
pub async fn get_user_stats(state: State<'_, AppState>) -> Result<Option<UserStats>, String> {
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(u) = user {
        state.db.get_user_stats(u.id).await.map_err(|e| e.to_string())
    } else {
        Ok(None)
    }
}

/// Result of GitHub stats sync with XP details
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub user_stats: UserStats,
    pub xp_gained: i32,
    pub old_level: u32,
    pub new_level: u32,
    pub level_up: bool,
    pub xp_breakdown: XpBreakdownResult,
    pub streak_bonus: StreakBonusInfo,
    pub new_badges: Vec<NewBadgeInfo>,
}

/// Information about newly earned badge
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewBadgeInfo {
    pub badge_id: String,
    pub badge_type: String,
    pub name: String,
    pub description: String,
    pub rarity: String,
    pub icon: String,
}

/// XP breakdown details for frontend display
#[derive(Debug, Clone, serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XpBreakdownResult {
    pub commits_xp: i32,
    pub prs_created_xp: i32,
    pub prs_merged_xp: i32,
    pub issues_created_xp: i32,
    pub issues_closed_xp: i32,
    pub reviews_xp: i32,
    pub stars_xp: i32,
    pub streak_bonus_xp: i32,
    pub total_xp: i32,
}

/// Streak bonus information for frontend
#[derive(Debug, Clone, serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StreakBonusInfo {
    pub daily_bonus: i32,
    pub milestone_bonus: i32,
    pub total_bonus: i32,
    pub milestone_reached: Option<i32>,
    pub current_streak: i32,
    pub next_milestone_days: Option<i32>,
    pub days_to_next_milestone: Option<i32>,
}

/// Event emitted when XP is gained
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XpGainedEvent {
    pub xp_gained: i32,
    pub total_xp: u32,
    pub old_level: u32,
    pub new_level: u32,
    pub level_up: bool,
    pub xp_breakdown: XpBreakdownResult,
    pub streak_bonus: StreakBonusInfo,
}

/// Event emitted when a streak milestone is reached
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreakMilestoneEvent {
    pub milestone_days: i32,
    pub bonus_xp: i32,
    pub current_streak: i32,
}

/// Event emitted when a badge is earned
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgeEarnedEvent {
    pub badge_id: String,
    pub badge_type: String,
    pub name: String,
    pub description: String,
    pub rarity: String,
    pub icon: String,
}

/// Sync GitHub stats to local database
#[command]
pub async fn sync_github_stats(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<SyncResult, String> {
    let token = state
        .token_manager
        .get_access_token()
        .await
        .map_err(|e| e.to_string())?;

    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Not logged in")?;

    let client = GitHubClient::new(token);
    let github_stats = client
        .get_user_stats(&user.username)
        .await
        .map_err(|e| e.to_string())?;

    // Get previous stats for diff calculation
    let previous_stats_json = state
        .db
        .get_previous_github_stats(user.id)
        .await
        .map_err(|e| e.to_string())?;

    // Calculate diff and XP
    let (xp_breakdown, xp_gained) = if let Some(prev_json) = previous_stats_json {
        if let Ok(prev_stats) = serde_json::from_str::<GitHubStats>(&prev_json) {
            let breakdown = xp::XpBreakdown::calculate(
                github_stats.total_commits - prev_stats.total_commits,
                github_stats.total_prs - prev_stats.total_prs,
                github_stats.total_prs_merged - prev_stats.total_prs_merged,
                github_stats.total_issues - prev_stats.total_issues,
                github_stats.total_issues_closed - prev_stats.total_issues_closed,
                github_stats.total_reviews - prev_stats.total_reviews,
                github_stats.total_stars_received - prev_stats.total_stars_received,
            );
            let total = breakdown.total_xp;
            (breakdown, total)
        } else {
            // First sync - calculate full XP
            let breakdown = xp::XpBreakdown::calculate(
                github_stats.total_commits,
                github_stats.total_prs,
                github_stats.total_prs_merged,
                github_stats.total_issues,
                github_stats.total_issues_closed,
                github_stats.total_reviews,
                github_stats.total_stars_received,
            );
            let total = breakdown.total_xp;
            (breakdown, total)
        }
    } else {
        // First sync - calculate full XP
        let breakdown = xp::XpBreakdown::calculate(
            github_stats.total_commits,
            github_stats.total_prs,
            github_stats.total_prs_merged,
            github_stats.total_issues,
            github_stats.total_issues_closed,
            github_stats.total_reviews,
            github_stats.total_stars_received,
        );
        let total = breakdown.total_xp;
        (breakdown, total)
    };

    // Get current stats for level comparison and streak update
    let current_stats = state
        .db
        .get_user_stats(user.id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("User stats not found")?;

    let old_level = level::level_from_xp(current_stats.total_xp as u32);
    let old_streak = current_stats.current_streak;

    // Update streak if there was activity today
    let today = Utc::now().date_naive();
    let streak_result = if github_stats.total_contributions > 0 {
        // Update streak and get the result
        let updated_stats = state
            .db
            .update_streak(user.id, today)
            .await
            .map_err(|e| e.to_string())?;
        
        let new_streak = updated_stats.current_streak;
        let bonus = streak::calculate_streak_bonus(old_streak, new_streak);
        
        Some((bonus, new_streak))
    } else {
        None
    };

    // Calculate streak bonus XP
    let (streak_bonus_result, streak_bonus_xp) = if let Some((bonus, new_streak)) = &streak_result {
        let next_milestone = streak::get_next_milestone(*new_streak);
        let days_to_next = streak::days_to_next_milestone(*new_streak);
        
        (
            StreakBonusInfo {
                daily_bonus: bonus.daily_bonus,
                milestone_bonus: bonus.milestone_bonus,
                total_bonus: bonus.total_bonus,
                milestone_reached: bonus.milestone_reached,
                current_streak: *new_streak,
                next_milestone_days: next_milestone.map(|m| m.days),
                days_to_next_milestone: days_to_next,
            },
            bonus.total_bonus,
        )
    } else {
        (
            StreakBonusInfo {
                current_streak: old_streak,
                next_milestone_days: streak::get_next_milestone(old_streak).map(|m| m.days),
                days_to_next_milestone: streak::days_to_next_milestone(old_streak),
                ..Default::default()
            },
            0,
        )
    };

    // Total XP gained (activity XP + streak bonus)
    let total_xp_gained = xp_gained + streak_bonus_xp;

    // Add XP if there's any gain
    let updated_stats = if total_xp_gained > 0 {
        // Record activity XP gain
        if xp_gained > 0 {
            state
                .db
                .record_xp_gain(
                    user.id,
                    "github_sync",
                    xp_gained,
                    Some("GitHub stats sync"),
                    None,
                )
                .await
                .map_err(|e| e.to_string())?;
        }

        // Record streak bonus XP
        if streak_bonus_xp > 0 {
            let description = if let Some(milestone) = streak_bonus_result.milestone_reached {
                format!("{}日連続達成ボーナス！", milestone)
            } else {
                "連続コミットボーナス".to_string()
            };
            
            state
                .db
                .record_xp_gain(
                    user.id,
                    &XpActionType::StreakBonus.to_string(),
                    streak_bonus_xp,
                    Some(&description),
                    None,
                )
                .await
                .map_err(|e| e.to_string())?;
        }

        // Add total XP to user stats
        state
            .db
            .add_xp(user.id, total_xp_gained)
            .await
            .map_err(|e| e.to_string())?
    } else {
        // Refresh stats (might have been updated by streak)
        state
            .db
            .get_user_stats(user.id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("User stats not found")?
    };

    let new_level = level::level_from_xp(updated_stats.total_xp as u32);
    let level_up = new_level > old_level;

    // Save current GitHub stats as previous for next diff
    let stats_json = serde_json::to_string(&github_stats).map_err(|e| e.to_string())?;
    state
        .db
        .save_previous_github_stats(user.id, &stats_json)
        .await
        .map_err(|e| e.to_string())?;

    let xp_breakdown_result = XpBreakdownResult {
        commits_xp: xp_breakdown.commits_xp,
        prs_created_xp: xp_breakdown.prs_created_xp,
        prs_merged_xp: xp_breakdown.prs_merged_xp,
        issues_created_xp: xp_breakdown.issues_created_xp,
        issues_closed_xp: xp_breakdown.issues_closed_xp,
        reviews_xp: xp_breakdown.reviews_xp,
        stars_xp: xp_breakdown.stars_xp,
        streak_bonus_xp,
        total_xp: total_xp_gained,
    };

    // Emit XP gained event for frontend
    if total_xp_gained > 0 {
        let event = XpGainedEvent {
            xp_gained: total_xp_gained,
            total_xp: updated_stats.total_xp as u32,
            old_level,
            new_level,
            level_up,
            xp_breakdown: xp_breakdown_result.clone(),
            streak_bonus: streak_bonus_result.clone(),
        };
        let _ = app.emit("xp-gained", &event);

        // Emit level up event if level increased
        if level_up {
            let _ = app.emit("level-up", &event);
        }

        // Emit streak milestone event if milestone reached
        if let Some(milestone_days) = streak_bonus_result.milestone_reached {
            let milestone_event = StreakMilestoneEvent {
                milestone_days,
                bonus_xp: streak_bonus_result.milestone_bonus,
                current_streak: streak_bonus_result.current_streak,
            };
            let _ = app.emit("streak-milestone", &milestone_event);
        }
    }

    // Badge evaluation
    let badge_context = badge::BadgeEvalContext {
        total_commits: github_stats.total_commits,
        current_streak: updated_stats.current_streak,
        longest_streak: updated_stats.longest_streak,
        total_reviews: github_stats.total_reviews,
        total_prs: github_stats.total_prs,
        total_prs_merged: github_stats.total_prs_merged,
        total_issues_closed: github_stats.total_issues_closed,
        languages_count: github_stats.languages_count,
    };

    // Get already earned badges
    let earned_badges = state
        .db
        .get_user_badges(user.id)
        .await
        .map_err(|e| e.to_string())?;
    let earned_badge_ids: Vec<String> = earned_badges.iter().map(|b| b.badge_id.clone()).collect();

    // Evaluate badges
    let new_badge_results = badge::evaluate_badges(&badge_context, &earned_badge_ids);
    let badge_definitions = badge::get_all_badge_definitions();

    let mut new_badges: Vec<NewBadgeInfo> = Vec::new();
    for badge_result in new_badge_results {
        // Award the badge
        state
            .db
            .award_badge(user.id, &badge_result.badge_type, &badge_result.badge_id)
            .await
            .map_err(|e| e.to_string())?;

        // Find badge definition for event
        if let Some(def) = badge_definitions.iter().find(|d| d.id == badge_result.badge_id) {
            let badge_info = NewBadgeInfo {
                badge_id: def.id.clone(),
                badge_type: def.badge_type.clone(),
                name: def.name.clone(),
                description: def.description.clone(),
                rarity: def.rarity.clone(),
                icon: def.icon.clone(),
            };

            // Emit badge earned event
            let badge_event = BadgeEarnedEvent {
                badge_id: badge_info.badge_id.clone(),
                badge_type: badge_info.badge_type.clone(),
                name: badge_info.name.clone(),
                description: badge_info.description.clone(),
                rarity: badge_info.rarity.clone(),
                icon: badge_info.icon.clone(),
            };
            let _ = app.emit("badge-earned", &badge_event);

            new_badges.push(badge_info);
        }
    }

    Ok(SyncResult {
        user_stats: updated_stats,
        xp_gained: total_xp_gained,
        old_level,
        new_level,
        level_up,
        xp_breakdown: xp_breakdown_result,
        streak_bonus: streak_bonus_result,
        new_badges,
    })
}

/// Get contribution calendar
#[command]
pub async fn get_contribution_calendar(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let token = state
        .token_manager
        .get_access_token()
        .await
        .map_err(|e| e.to_string())?;

    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Not logged in")?;

    let client = GitHubClient::new(token);
    let contributions = client
        .get_contribution_calendar(&user.username)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_value(contributions.contribution_calendar).map_err(|e| e.to_string())
}

