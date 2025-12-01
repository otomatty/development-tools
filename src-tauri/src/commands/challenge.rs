//! Challenge commands for Tauri
//!
//! These commands handle challenge-related operations: CRUD, progress tracking, etc.

use chrono::Utc;
use tauri::{command, State};

use super::auth::AppState;
use crate::database::challenge;
use crate::database::models::Challenge;

/// Challenge info for frontend with additional computed fields
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeInfo {
    pub id: i64,
    pub user_id: i64,
    pub challenge_type: String,
    pub target_metric: String,
    pub target_value: i32,
    pub current_value: i32,
    pub reward_xp: i32,
    pub start_date: String,
    pub end_date: String,
    pub status: String,
    pub completed_at: Option<String>,
    // Computed fields
    pub progress_percent: f32,
    pub remaining_time_hours: i64,
    pub is_completed: bool,
    pub is_expired: bool,
}

impl From<Challenge> for ChallengeInfo {
    fn from(c: Challenge) -> Self {
        let now = Utc::now();
        let progress = if c.target_value > 0 {
            (c.current_value as f32 / c.target_value as f32 * 100.0).min(100.0)
        } else {
            0.0
        };
        let remaining_hours = (c.end_date - now).num_hours().max(0);
        let is_completed = c.status == "completed";
        let is_expired = c.end_date < now && c.status == "active";

        ChallengeInfo {
            id: c.id,
            user_id: c.user_id,
            challenge_type: c.challenge_type,
            target_metric: c.target_metric,
            target_value: c.target_value,
            current_value: c.current_value,
            reward_xp: c.reward_xp,
            start_date: c.start_date.to_rfc3339(),
            end_date: c.end_date.to_rfc3339(),
            status: c.status,
            completed_at: c.completed_at.map(|dt| dt.to_rfc3339()),
            progress_percent: progress,
            remaining_time_hours: remaining_hours,
            is_completed,
            is_expired,
        }
    }
}

/// Request to create a new challenge
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChallengeRequest {
    pub challenge_type: String,
    pub target_metric: String,
    pub target_value: i32,
    pub reward_xp: Option<i32>,
}

/// Get all active challenges for current user
#[command]
pub async fn get_active_challenges(
    state: State<'_, AppState>,
) -> Result<Vec<ChallengeInfo>, String> {
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Not logged in")?;

    // First, fail any expired challenges
    state
        .db
        .fail_expired_challenges(user.id)
        .await
        .map_err(|e| e.to_string())?;

    let challenges = state
        .db
        .get_active_challenges(user.id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(challenges.into_iter().map(ChallengeInfo::from).collect())
}

/// Get all challenges (including completed and failed)
#[command]
pub async fn get_all_challenges(state: State<'_, AppState>) -> Result<Vec<ChallengeInfo>, String> {
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Not logged in")?;

    let challenges = state
        .db
        .get_all_challenges(user.id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(challenges.into_iter().map(ChallengeInfo::from).collect())
}

/// Get challenges by type (daily/weekly)
#[command]
pub async fn get_challenges_by_type(
    state: State<'_, AppState>,
    challenge_type: String,
) -> Result<Vec<ChallengeInfo>, String> {
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Not logged in")?;

    let challenges = state
        .db
        .get_challenges_by_type(user.id, &challenge_type)
        .await
        .map_err(|e| e.to_string())?;

    Ok(challenges.into_iter().map(ChallengeInfo::from).collect())
}

/// Create a custom challenge
#[command]
pub async fn create_challenge(
    state: State<'_, AppState>,
    request: CreateChallengeRequest,
) -> Result<ChallengeInfo, String> {
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Not logged in")?;

    // Validate challenge type
    if request.challenge_type != "daily" && request.challenge_type != "weekly" {
        return Err("Invalid challenge type. Must be 'daily' or 'weekly'".to_string());
    }

    // Validate target metric
    let valid_metrics = ["commits", "prs", "reviews", "issues"];
    if !valid_metrics.contains(&request.target_metric.as_str()) {
        return Err(
            "Invalid target metric. Must be one of: commits, prs, reviews, issues".to_string(),
        );
    }

    // Check if there's already an active challenge of this type and metric
    if state
        .db
        .has_active_challenge(user.id, &request.challenge_type, &request.target_metric)
        .await
        .map_err(|e| e.to_string())?
    {
        return Err("An active challenge of this type and metric already exists".to_string());
    }

    let now = Utc::now();
    let (start_date, end_date) =
        challenge::calculate_challenge_period(&request.challenge_type, now);

    // Calculate reward XP if not provided
    let reward_xp = request.reward_xp.unwrap_or_else(|| {
        challenge::calculate_reward_xp(&request.target_metric, request.target_value)
    });

    let challenge = state
        .db
        .create_challenge(
            user.id,
            &request.challenge_type,
            &request.target_metric,
            request.target_value,
            reward_xp,
            start_date,
            end_date,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(ChallengeInfo::from(challenge))
}

/// Delete a challenge
#[command]
pub async fn delete_challenge(state: State<'_, AppState>, challenge_id: i64) -> Result<(), String> {
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Not logged in")?;

    // Verify the challenge belongs to the user
    let challenge = state
        .db
        .get_challenge_by_id(challenge_id)
        .await
        .map_err(|e| e.to_string())?;

    if challenge.user_id != user.id {
        return Err("Challenge not found".to_string());
    }

    state
        .db
        .delete_challenge(challenge_id)
        .await
        .map_err(|e| e.to_string())
}

/// Update challenge progress manually (for testing/admin)
#[command]
pub async fn update_challenge_progress(
    state: State<'_, AppState>,
    challenge_id: i64,
    current_value: i32,
) -> Result<ChallengeInfo, String> {
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Not logged in")?;

    // Verify the challenge belongs to the user and get original state
    let original_challenge = state
        .db
        .get_challenge_by_id(challenge_id)
        .await
        .map_err(|e| e.to_string())?;

    if original_challenge.user_id != user.id {
        return Err("Challenge not found".to_string());
    }

    let updated = state
        .db
        .update_challenge_progress(challenge_id, current_value)
        .await
        .map_err(|e| e.to_string())?;

    // Check if challenge was just completed (active -> completed transition)
    if updated.status == "completed" && original_challenge.status == "active" {
        // Award XP for completing the challenge
        state
            .db
            .record_xp_gain(
                user.id,
                "challenge_completed",
                updated.reward_xp,
                Some(&format!("Completed {} challenge", updated.challenge_type)),
                None,
            )
            .await
            .map_err(|e| e.to_string())?;

        state
            .db
            .add_xp(user.id, updated.reward_xp)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(ChallengeInfo::from(updated))
}

/// Get challenge completion stats
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeStats {
    pub total_completed: i32,
    pub consecutive_weekly_completions: i32,
    pub active_count: i32,
}

#[command]
pub async fn get_challenge_stats(state: State<'_, AppState>) -> Result<ChallengeStats, String> {
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Not logged in")?;

    let total_completed = state
        .db
        .get_challenge_completion_count(user.id)
        .await
        .map_err(|e| e.to_string())?;

    let consecutive_weekly = state
        .db
        .get_consecutive_weekly_completions(user.id)
        .await
        .map_err(|e| e.to_string())?;

    let active_challenges = state
        .db
        .get_active_challenges(user.id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ChallengeStats {
        total_completed,
        consecutive_weekly_completions: consecutive_weekly,
        active_count: active_challenges.len() as i32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_calculate_reward_xp() {
        // Test through challenge module
        assert_eq!(challenge::calculate_reward_xp("commits", 5), 50);
        assert_eq!(challenge::calculate_reward_xp("prs", 2), 80);
        assert_eq!(challenge::calculate_reward_xp("reviews", 3), 60);
        assert_eq!(challenge::calculate_reward_xp("issues", 4), 100);
    }

    #[test]
    fn test_calculate_challenge_period_daily() {
        let now = Utc::now();
        let (start, end) = challenge::calculate_challenge_period("daily", now);

        assert_eq!(start, now);
        assert!(end > now);
        assert!(end <= now + Duration::days(1));
    }

    #[test]
    fn test_calculate_challenge_period_weekly() {
        let now = Utc::now();
        let (start, end) = challenge::calculate_challenge_period("weekly", now);

        assert_eq!(start, now);
        assert!(end > now);
        assert!(end <= now + Duration::days(7));
    }
}
