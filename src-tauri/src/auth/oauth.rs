//! GitHub OAuth flow implementation
//!
//! Handles the GitHub Device Flow authentication.
//! Device Flow is recommended for desktop apps as it doesn't require client_secret.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OAuthError {
    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest::Error),

    #[error("Token exchange failed: {0}")]
    TokenExchange(String),

    #[error("Device flow authorization pending")]
    AuthorizationPending,

    #[error("Device flow slow down")]
    SlowDown,

    #[error("Device code expired")]
    ExpiredToken,

    #[error("Access denied by user")]
    AccessDenied,
}

pub type OAuthResult<T> = Result<T, OAuthError>;

/// Device Flow configuration (does NOT require client_secret)
#[derive(Debug, Clone)]
pub struct DeviceFlowConfig {
    pub client_id: String,
    pub scopes: Vec<String>,
}

impl DeviceFlowConfig {
    /// Create a new Device Flow config
    pub fn new(client_id: String) -> Self {
        Self {
            client_id,
            scopes: vec![
                "read:user".to_string(),
                "repo".to_string(),
                "read:org".to_string(),
            ],
        }
    }

    /// Get scopes as a space-separated string
    pub fn scopes_string(&self) -> String {
        self.scopes.join(" ")
    }
}

/// Device code response from GitHub
/// Note: GitHub returns snake_case, but we serialize to camelCase for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodeResponse {
    #[serde(alias = "device_code")]
    pub device_code: String,
    #[serde(alias = "user_code")]
    pub user_code: String,
    #[serde(alias = "verification_uri")]
    pub verification_uri: String,
    #[serde(alias = "expires_in")]
    pub expires_in: i64,
    #[serde(alias = "interval")]
    pub interval: i64,
}

/// Device token polling status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DeviceTokenStatus {
    /// Authorization is still pending - user hasn't completed authorization yet
    Pending,
    /// Token was successfully obtained
    Success {
        auth_state: crate::auth::token::AuthState,
    },
    /// An error occurred
    Error { message: String },
}

/// Token response from GitHub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub token_type: String,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub expires_in: Option<i64>,
}

/// Processed token with expiration time
#[derive(Debug, Clone)]
pub struct AuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl From<TokenResponse> for AuthToken {
    fn from(response: TokenResponse) -> Self {
        let expires_at = response
            .expires_in
            .map(|secs| Utc::now() + Duration::seconds(secs));

        Self {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            expires_at,
        }
    }
}

/// Device Flow manager - does NOT require client_secret
pub struct DeviceFlow {
    config: DeviceFlowConfig,
    client: reqwest::Client,
}

impl DeviceFlow {
    /// Create a new Device Flow with a new HTTP client
    pub fn new(config: DeviceFlowConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Create a new Device Flow with a shared HTTP client
    /// This is more efficient when making multiple requests (e.g., polling)
    /// as it reuses the connection pool
    pub fn with_client(config: DeviceFlowConfig, client: reqwest::Client) -> Self {
        Self { config, client }
    }

    /// Start the device flow - returns device code and user code
    pub async fn start(&self) -> OAuthResult<DeviceCodeResponse> {
        let response = self
            .client
            .post("https://github.com/login/device/code")
            .header("Accept", "application/json")
            .form(&[
                ("client_id", self.config.client_id.as_str()),
                ("scope", self.config.scopes_string().as_str()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OAuthError::TokenExchange(format!(
                "Failed to start device flow: {}",
                error_text
            )));
        }

        let device_response: DeviceCodeResponse = response.json().await?;
        Ok(device_response)
    }

    /// Poll for the access token
    /// Returns Ok(token) if authorized, Err(AuthorizationPending) if still pending
    pub async fn poll_token(&self, device_code: &str) -> OAuthResult<AuthToken> {
        #[derive(Deserialize)]
        struct PollResponse {
            #[serde(default)]
            access_token: Option<String>,
            #[serde(default)]
            error: Option<String>,
            #[serde(default)]
            error_description: Option<String>,
        }

        let response = self
            .client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&[
                ("client_id", self.config.client_id.as_str()),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await?;

        let poll_response: PollResponse = response.json().await?;

        // Check for errors first
        if let Some(error) = poll_response.error {
            return match error.as_str() {
                "authorization_pending" => Err(OAuthError::AuthorizationPending),
                "slow_down" => Err(OAuthError::SlowDown),
                "expired_token" => Err(OAuthError::ExpiredToken),
                "access_denied" => Err(OAuthError::AccessDenied),
                _ => Err(OAuthError::TokenExchange(
                    poll_response
                        .error_description
                        .unwrap_or_else(|| error.clone()),
                )),
            };
        }

        // Check if we got a token
        if let Some(access_token) = poll_response.access_token {
            if !access_token.is_empty() {
                return Ok(AuthToken {
                    access_token,
                    refresh_token: None, // Device flow doesn't provide refresh tokens by default
                    expires_at: None,    // GitHub tokens don't expire by default
                });
            }
        }

        Err(OAuthError::TokenExchange(
            "Unexpected response from GitHub".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_flow_config() {
        let config = DeviceFlowConfig::new("client_id".to_string());

        assert_eq!(config.client_id, "client_id");
        assert!(config.scopes.contains(&"read:user".to_string()));
        assert!(config.scopes.contains(&"repo".to_string()));
        assert_eq!(config.scopes_string(), "read:user repo read:org");
    }

    #[test]
    fn test_device_flow_new() {
        let config = DeviceFlowConfig::new("test_client".to_string());
        let _flow = DeviceFlow::new(config);
        // Just ensure it compiles and creates without panic
    }
}
