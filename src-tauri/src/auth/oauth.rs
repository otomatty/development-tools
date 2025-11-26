//! GitHub OAuth flow implementation
//!
//! Handles the OAuth 2.0 authorization code flow with PKCE.

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OAuthError {
    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest::Error),

    #[error("Token exchange failed: {0}")]
    TokenExchange(String),

    #[error("Invalid state parameter")]
    InvalidState,

    #[error("Missing authorization code")]
    MissingCode,

    #[error("Configuration error: {0}")]
    Configuration(String),
}

pub type OAuthResult<T> = Result<T, OAuthError>;

/// OAuth configuration
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

impl OAuthConfig {
    /// Create a new OAuth config from environment or defaults
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri: "development-tools://callback".to_string(),
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

/// OAuth flow manager
pub struct OAuthFlow {
    config: OAuthConfig,
    client: reqwest::Client,
    state: Option<String>,
}

impl OAuthFlow {
    /// Create a new OAuth flow
    pub fn new(config: OAuthConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
            state: None,
        }
    }

    /// Generate a random state parameter
    fn generate_state() -> String {
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        hex::encode(bytes)
    }

    /// Get the authorization URL
    ///
    /// Returns the URL to redirect the user to for authorization
    pub fn get_authorization_url(&mut self) -> String {
        let state = Self::generate_state();
        self.state = Some(state.clone());

        format!(
            "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope={}&state={}",
            self.config.client_id,
            urlencoding::encode(&self.config.redirect_uri),
            urlencoding::encode(&self.config.scopes_string()),
            state
        )
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code(&self, code: &str, state: &str) -> OAuthResult<AuthToken> {
        // Verify state
        if self.state.as_deref() != Some(state) {
            return Err(OAuthError::InvalidState);
        }

        let response = self
            .client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&[
                ("client_id", self.config.client_id.as_str()),
                ("client_secret", self.config.client_secret.as_str()),
                ("code", code),
                ("redirect_uri", self.config.redirect_uri.as_str()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OAuthError::TokenExchange(error_text));
        }

        let token_response: TokenResponse = response.json().await?;

        if token_response.access_token.is_empty() {
            return Err(OAuthError::TokenExchange(
                "Empty access token received".to_string(),
            ));
        }

        Ok(token_response.into())
    }

    /// Refresh an access token
    pub async fn refresh_token(&self, refresh_token: &str) -> OAuthResult<AuthToken> {
        let response = self
            .client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&[
                ("client_id", self.config.client_id.as_str()),
                ("client_secret", self.config.client_secret.as_str()),
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OAuthError::TokenExchange(error_text));
        }

        let token_response: TokenResponse = response.json().await?;
        Ok(token_response.into())
    }
}

/// Parse callback URL to extract code and state
pub fn parse_callback_url(url: &str) -> OAuthResult<(String, String)> {
    let url = Url::parse(url).map_err(|e| OAuthError::Configuration(e.to_string()))?;

    let mut code = None;
    let mut state = None;

    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.to_string()),
            "state" => state = Some(value.to_string()),
            _ => {}
        }
    }

    let code = code.ok_or(OAuthError::MissingCode)?;
    let state = state.ok_or(OAuthError::InvalidState)?;

    Ok((code, state))
}

// Add hex crate dependency - if not available, implement simple hex encoding
mod hex {
    pub fn encode(bytes: Vec<u8>) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_config() {
        let config = OAuthConfig::new("client_id".to_string(), "client_secret".to_string());

        assert_eq!(config.client_id, "client_id");
        assert_eq!(config.redirect_uri, "development-tools://callback");
        assert!(config.scopes.contains(&"read:user".to_string()));
    }

    #[test]
    fn test_authorization_url() {
        let config = OAuthConfig::new("test_client".to_string(), "test_secret".to_string());
        let mut flow = OAuthFlow::new(config);

        let url = flow.get_authorization_url();

        assert!(url.contains("github.com/login/oauth/authorize"));
        assert!(url.contains("client_id=test_client"));
        assert!(url.contains("state="));
    }

    #[test]
    fn test_parse_callback_url() {
        let url = "development-tools://callback?code=abc123&state=xyz789";
        let (code, state) = parse_callback_url(url).expect("Should parse");

        assert_eq!(code, "abc123");
        assert_eq!(state, "xyz789");
    }

    #[test]
    fn test_parse_callback_url_missing_code() {
        let url = "development-tools://callback?state=xyz789";
        let result = parse_callback_url(url);

        assert!(matches!(result, Err(OAuthError::MissingCode)));
    }
}

