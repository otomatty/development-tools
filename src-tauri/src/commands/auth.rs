//! Authentication commands for Tauri
//!
//! These commands handle GitHub Device Flow authentication.
//! Device Flow is recommended for desktop apps as it doesn't require client_secret.

use std::sync::Arc;
use tauri::{command, AppHandle, State};
use tokio::sync::Mutex;

use crate::auth::{
    handle_unauthorized, reasons, AuthState, AuthToken, DeviceCodeResponse, DeviceFlow,
    DeviceFlowConfig, DeviceTokenStatus, OAuthError, TokenManager, UserInfo,
};
use crate::database::Database;
use crate::github::GitHubClient;

/// Shared application state
pub struct AppState {
    pub db: Database,
    pub token_manager: TokenManager,
    /// Device Flow config - only requires client_id (no client_secret needed)
    pub device_flow_config: Option<DeviceFlowConfig>,
    /// Current device flow state (device_code for polling)
    pub device_flow_state: Arc<Mutex<Option<String>>>,
    /// Shared HTTP client for reuse across requests (improves performance)
    pub http_client: reqwest::Client,
    /// Serializes `run_github_sync` invocations so a manual "sync now" never
    /// races a scheduler-driven sync. Two concurrent runs would read the same
    /// pre-sync snapshot and double-apply XP / badges / challenge progress.
    pub sync_lock: Arc<Mutex<()>>,
}

impl AppState {
    pub async fn new() -> Result<Self, String> {
        let db = Database::new()
            .await
            .map_err(|e| format!("Failed to initialize database: {}", e))?;

        let token_manager = TokenManager::new(db.clone())
            .map_err(|e| format!("Failed to initialize token manager: {}", e))?;

        // Create a shared HTTP client for reuse across all requests
        // This improves performance by reusing connection pools
        let http_client = reqwest::Client::new();

        Ok(Self {
            db,
            token_manager,
            device_flow_config: None,
            device_flow_state: Arc::new(Mutex::new(None)),
            http_client,
            sync_lock: Arc::new(Mutex::new(())),
        })
    }

    /// Set Device Flow config (only requires client_id)
    pub fn with_device_flow_config(mut self, config: DeviceFlowConfig) -> Self {
        self.device_flow_config = Some(config);
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

/// Validate current token (check if token is still valid)
///
/// Note: GitHub Device Flow tokens don't expire, but users can revoke them manually.
/// This command checks if the current token is still valid by making a test API call.
/// Returns `Ok(false)` if the user is not logged in.
///
/// Side effect: when GitHub responds with 401 (token revoked), the stored
/// credential is cleared and the `auth-expired` event is emitted so the
/// frontend can surface a re-login prompt without polling.
#[command]
pub async fn validate_token(app: AppHandle, state: State<'_, AppState>) -> Result<bool, String> {
    // Check if user is logged in first
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?;

    if user.is_none() {
        return Ok(false);
    }

    let access_token = state
        .token_manager
        .get_access_token()
        .await
        .map_err(|e| e.to_string())?;

    let is_valid = state
        .token_manager
        .validate_token(&access_token)
        .await
        .map_err(|e| e.to_string())?;

    if !is_valid {
        // Token is definitively rejected by GitHub — clear it and notify the
        // UI. Transport / network errors take the `Err` branch above and
        // intentionally do NOT trigger a forced logout (see
        // `TokenManager::validate_token`'s contract).
        handle_unauthorized(&app, state.inner(), reasons::MANUAL_VALIDATION_FAILED).await;
    }

    Ok(is_valid)
}

/// Best-effort startup probe of the persisted token.
///
/// Called from `lib.rs` setup *after* the Tauri runtime is ready so the
/// `auth-expired` emit has a working app handle. Runs in a background task
/// so it never delays splash → first paint, and silently no-ops when:
/// - no user is logged in, or
/// - GitHub is unreachable (transport error) — we don't want a flaky network
///   on launch to log the user out.
///
/// Only a confirmed 401 from GitHub clears the token.
pub async fn run_startup_token_validation(app: AppHandle, state: &AppState) {
    let user = match state.token_manager.get_current_user().await {
        Ok(Some(u)) => u,
        Ok(None) => return,
        Err(e) => {
            // TODO: [INFRA] logクレートに置換（ログ基盤整備時に一括対応）
            eprintln!("Startup auth check: failed to read current user: {}", e);
            return;
        }
    };

    let access_token = match state.token_manager.get_access_token().await {
        Ok(t) => t,
        Err(e) => {
            eprintln!(
                "Startup auth check: failed to load token for {}: {}",
                user.id, e
            );
            return;
        }
    };

    match state.token_manager.validate_token(&access_token).await {
        Ok(true) => {
            // Token still works — nothing to do.
        }
        Ok(false) => {
            // Re-check the active credential before clearing it. The user
            // could have re-authenticated (or logged out, then back in as a
            // different account) while the GitHub probe was in flight; the
            // 401 we just observed applies to a token that's already been
            // replaced. Forcing a logout here would wipe the freshly saved
            // session and immediately undo the user's re-login.
            //
            // Compare the decrypted token rather than just the user id so an
            // account-swap to the *same* GitHub user (rare, but possible
            // after a manual revoke + re-grant) is also caught.
            let still_same = match state.token_manager.get_access_token().await {
                Ok(current) => current == access_token,
                // NotLoggedIn / decrypt failure → user state changed under
                // us; either way, don't clobber it.
                Err(_) => false,
            };
            if !still_same {
                eprintln!(
                    "Startup auth check: stored token for user {} changed during validation; skipping logout",
                    user.id
                );
                return;
            }

            eprintln!(
                "Startup auth check: GitHub rejected the stored token for user {}; clearing session",
                user.id
            );
            handle_unauthorized(&app, state, reasons::STARTUP_VALIDATION_FAILED).await;
        }
        Err(e) => {
            // Transport failure — leave the session alone so a flaky
            // network on launch doesn't sign the user out.
            eprintln!(
                "Startup auth check: validate_token failed for user {} ({}); leaving session intact",
                user.id, e
            );
        }
    }
}

// ============================================
// Device Flow Commands
// ============================================

/// Start Device Flow - returns device code info for user to authorize
///
/// The user should:
/// 1. Go to the verification_uri (https://github.com/login/device)
/// 2. Enter the user_code shown
/// 3. The app will poll for completion using poll_device_token
#[command]
pub async fn start_device_flow(state: State<'_, AppState>) -> Result<DeviceCodeResponse, String> {
    let config = state
        .device_flow_config
        .as_ref()
        .ok_or("Device Flow not configured. Please set GITHUB_CLIENT_ID")?
        .clone();

    // Use shared HTTP client for better performance
    let flow = DeviceFlow::with_client(config, state.http_client.clone());
    let device_response = flow
        .start()
        .await
        .map_err(|e| format!("Failed to start device flow: {}", e))?;

    // Store the device_code for polling
    let mut device_state = state.device_flow_state.lock().await;
    *device_state = Some(device_response.device_code.clone());

    Ok(device_response)
}

/// Poll for device token - call this periodically after start_device_flow
///
/// Returns:
/// - DeviceTokenStatus::Pending - user hasn't completed authorization yet
/// - DeviceTokenStatus::Success - authorization complete, user is logged in
/// - DeviceTokenStatus::Error - an error occurred
#[command]
pub async fn poll_device_token(state: State<'_, AppState>) -> Result<DeviceTokenStatus, String> {
    let config = state
        .device_flow_config
        .as_ref()
        .ok_or("Device Flow not configured")?
        .clone();

    let device_code = {
        let device_state = state.device_flow_state.lock().await;
        device_state.clone().ok_or("No device flow in progress")?
    };

    // Use shared HTTP client for better performance during polling
    let flow = DeviceFlow::with_client(config, state.http_client.clone());

    match flow.poll_token(&device_code).await {
        Ok(token) => {
            // Successfully got token - complete the login
            let auth_state = complete_device_login(&state, token).await?;

            // Clear device flow state
            let mut device_state = state.device_flow_state.lock().await;
            *device_state = None;

            Ok(DeviceTokenStatus::Success { auth_state })
        }
        Err(OAuthError::AuthorizationPending) => {
            // User hasn't completed authorization yet
            Ok(DeviceTokenStatus::Pending)
        }
        Err(OAuthError::SlowDown) => {
            // Too many requests - return pending (caller should slow down)
            Ok(DeviceTokenStatus::Pending)
        }
        Err(OAuthError::ExpiredToken) => {
            // Device code expired - clear state and return error
            let mut device_state = state.device_flow_state.lock().await;
            *device_state = None;
            Ok(DeviceTokenStatus::Error {
                message: "Device code expired. Please start over.".to_string(),
            })
        }
        Err(OAuthError::AccessDenied) => {
            // User denied access
            let mut device_state = state.device_flow_state.lock().await;
            *device_state = None;
            Ok(DeviceTokenStatus::Error {
                message: "Access denied by user.".to_string(),
            })
        }
        Err(e) => Ok(DeviceTokenStatus::Error {
            message: format!("Token exchange failed: {}", e),
        }),
    }
}

/// Cancel the current device flow
#[command]
pub async fn cancel_device_flow(state: State<'_, AppState>) -> Result<(), String> {
    let mut device_state = state.device_flow_state.lock().await;
    *device_state = None;
    Ok(())
}

/// Open a URL in the system's default browser
#[command]
pub async fn open_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    use tauri_plugin_shell::ShellExt;
    app.shell()
        .open(&url, None)
        .map_err(|e| format!("Failed to open URL: {}", e))
}

/// Helper function to complete login after getting token from device flow
async fn complete_device_login(
    state: &State<'_, AppState>,
    token: AuthToken,
) -> Result<AuthState, String> {
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

        // Retrieve the updated user - get_user_by_id uses fetch_one, so it will
        // return an error if the user doesn't exist (which shouldn't happen here
        // since we just saved tokens for this user)
        state
            .db
            .get_user_by_id(existing.id)
            .await
            .map_err(|e| format!(
                "Failed to retrieve user {} after saving tokens: {}. This may indicate a database inconsistency.",
                existing.id, e
            ))?
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_current_user_includes_created_at() {
        // This test would require a test database setup
        // For now, we verify that UserInfo::from includes created_at
        use crate::database::models::User;
        use chrono::Utc;

        let test_user = User {
            id: 1,
            github_id: 12345678,
            username: "testuser".to_string(),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
            access_token_encrypted: "encrypted_token".to_string(),
            refresh_token_encrypted: None,
            token_expires_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let user_info = UserInfo::from(test_user);
        assert!(user_info.created_at.is_some());
    }
}
