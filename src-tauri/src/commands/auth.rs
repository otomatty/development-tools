//! Authentication commands for Tauri
//!
//! These commands handle GitHub Device Flow authentication.
//! Device Flow is recommended for desktop apps as it doesn't require client_secret.

use std::sync::Arc;
use tauri::{command, State};
use tokio::sync::Mutex;

use crate::auth::{
    AuthState, AuthToken, DeviceCodeResponse, DeviceFlow, DeviceFlowConfig, DeviceTokenStatus,
    OAuthError, TokenManager, UserInfo,
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
            device_flow_config: None,
            device_flow_state: Arc::new(Mutex::new(None)),
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

    let flow = DeviceFlow::new(config);
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

    let flow = DeviceFlow::new(config);

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
