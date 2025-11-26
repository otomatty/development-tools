//! Authentication commands for Tauri
//!
//! These commands handle GitHub OAuth authentication flow.

use std::sync::Arc;
use tauri::{command, State};
use tokio::sync::Mutex;

use crate::auth::{AuthState, OAuthConfig, OAuthFlow, TokenManager, UserInfo};
use crate::database::Database;
use crate::github::GitHubClient;

/// Shared application state
pub struct AppState {
    pub db: Database,
    pub token_manager: TokenManager,
    pub oauth_flow: Arc<Mutex<Option<OAuthFlow>>>,
    pub oauth_config: Option<OAuthConfig>,
}

impl AppState {
    pub async fn new() -> Result<Self, String> {
        let db = Database::new()
            .await
            .map_err(|e| format!("Failed to initialize database: {}", e))?;

        let token_manager = TokenManager::new(db.clone())
            .map_err(|e| format!("Failed to initialize token manager: {}", e))?;

        Ok(Self {
            db,
            token_manager,
            oauth_flow: Arc::new(Mutex::new(None)),
            oauth_config: None,
        })
    }

    pub fn with_oauth_config(mut self, config: OAuthConfig) -> Self {
        self.oauth_config = Some(config);
        self
    }
}

/// Get current authentication state
#[command]
pub async fn get_auth_state(state: State<'_, AppState>) -> Result<AuthState, String> {
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?;

    Ok(AuthState {
        is_logged_in: user.is_some(),
        user: user.map(UserInfo::from),
    })
}

/// Start OAuth login flow - returns authorization URL
#[command]
pub async fn start_oauth_login(state: State<'_, AppState>) -> Result<String, String> {
    let config = state
        .oauth_config
        .as_ref()
        .ok_or("OAuth not configured. Please set GITHUB_CLIENT_ID and GITHUB_CLIENT_SECRET")?
        .clone();

    let mut flow = OAuthFlow::new(config);
    let auth_url = flow.get_authorization_url();

    // Store the flow for later
    let mut oauth_flow = state.oauth_flow.lock().await;
    *oauth_flow = Some(flow);

    Ok(auth_url)
}

/// Handle OAuth callback - exchange code for token
#[command]
pub async fn handle_oauth_callback(
    state: State<'_, AppState>,
    code: String,
    callback_state: String,
) -> Result<AuthState, String> {
    let mut oauth_flow_guard = state.oauth_flow.lock().await;
    let flow = oauth_flow_guard
        .take()
        .ok_or("No OAuth flow in progress")?;

    // Exchange code for token
    let token = flow
        .exchange_code(&code, &callback_state)
        .await
        .map_err(|e| format!("Failed to exchange code: {}", e))?;

    // Get user info from GitHub
    let github_client = GitHubClient::new(token.access_token.clone());
    let github_user = github_client
        .get_user()
        .await
        .map_err(|e| format!("Failed to get user info: {}", e))?;

    // Check if user already exists
    let existing_user = state
        .db
        .get_user_by_github_id(github_user.id)
        .await
        .map_err(|e| e.to_string())?;

    let user = if let Some(existing) = existing_user {
        // Update tokens for existing user
        state
            .token_manager
            .save_tokens(existing.id, &token)
            .await
            .map_err(|e| e.to_string())?;

        state
            .db
            .get_user_by_id(existing.id)
            .await
            .map_err(|e| e.to_string())?
    } else {
        // Create new user
        state
            .token_manager
            .create_user_from_token(
                github_user.id,
                &github_user.login,
                Some(&github_user.avatar_url),
                &token,
            )
            .await
            .map_err(|e| e.to_string())?
    };

    Ok(AuthState {
        is_logged_in: true,
        user: Some(UserInfo::from(user)),
    })
}

/// Logout current user
#[command]
pub async fn logout(state: State<'_, AppState>) -> Result<(), String> {
    state
        .token_manager
        .logout()
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Get current user info
#[command]
pub async fn get_current_user(state: State<'_, AppState>) -> Result<Option<UserInfo>, String> {
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?;

    Ok(user.map(UserInfo::from))
}

