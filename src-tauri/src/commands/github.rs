//! GitHub API commands for Tauri
//!
//! These commands handle fetching data from the GitHub API.

use tauri::{command, State};

use super::auth::AppState;
use crate::database::UserStats;
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

/// Sync GitHub stats to local database
#[command]
pub async fn sync_github_stats(state: State<'_, AppState>) -> Result<UserStats, String> {
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

    // Update local stats with GitHub data
    // This would be expanded to properly track XP based on new activity
    state
        .db
        .increment_activity_count(
            user.id,
            github_stats.total_commits,
            github_stats.total_prs,
            github_stats.total_reviews,
            github_stats.total_issues,
        )
        .await
        .map_err(|e| e.to_string())?;

    state
        .db
        .get_user_stats(user.id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Failed to get updated stats".to_string())
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

