//! Token management
//!
//! Handles token storage, retrieval, and refresh logic.

use chrono::{Duration, Utc};
use thiserror::Error;

use super::crypto::{Crypto, CryptoError};
use super::oauth::{AuthToken, OAuthConfig, OAuthError, OAuthFlow};
use crate::database::{Database, DatabaseError, User};

#[derive(Error, Debug)]
pub enum TokenError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Crypto error: {0}")]
    Crypto(#[from] CryptoError),

    #[error("OAuth error: {0}")]
    OAuth(#[from] OAuthError),

    #[error("No user logged in")]
    NotLoggedIn,

    #[error("Token expired and refresh failed")]
    RefreshFailed,
}

pub type TokenResult<T> = Result<T, TokenError>;

/// Token manager handles secure token storage and retrieval
pub struct TokenManager {
    crypto: Crypto,
    db: Database,
    oauth_config: Option<OAuthConfig>,
}

impl TokenManager {
    /// Create a new token manager
    pub fn new(db: Database) -> TokenResult<Self> {
        let crypto = Crypto::from_app_key()?;
        Ok(Self {
            crypto,
            db,
            oauth_config: None,
        })
    }

    /// Set OAuth config for token refresh
    pub fn with_oauth_config(mut self, config: OAuthConfig) -> Self {
        self.oauth_config = Some(config);
        self
    }

    /// Save tokens for a user
    pub async fn save_tokens(
        &self,
        user_id: i64,
        token: &AuthToken,
    ) -> TokenResult<()> {
        let encrypted_access = self.crypto.encrypt(&token.access_token)?;
        let encrypted_refresh = token
            .refresh_token
            .as_ref()
            .map(|rt| self.crypto.encrypt(rt))
            .transpose()?;

        self.db
            .update_user_tokens(
                user_id,
                &encrypted_access,
                encrypted_refresh.as_deref(),
                token.expires_at,
            )
            .await?;

        Ok(())
    }

    /// Get the current access token, refreshing if needed
    pub async fn get_access_token(&self) -> TokenResult<String> {
        let user = self
            .db
            .get_current_user()
            .await?
            .ok_or(TokenError::NotLoggedIn)?;

        // Check if token is expired or about to expire (within 5 minutes)
        if let Some(expires_at) = user.token_expires_at {
            let buffer = Duration::minutes(5);
            if Utc::now() + buffer >= expires_at {
                // Token is expired or expiring soon, try to refresh
                return self.refresh_and_get_token(&user).await;
            }
        }

        // Token is valid, decrypt and return
        self.crypto.decrypt(&user.access_token_encrypted).map_err(|e| e.into())
    }

    /// Refresh token and return new access token
    async fn refresh_and_get_token(&self, user: &User) -> TokenResult<String> {
        // Check if we have refresh token and OAuth config
        let refresh_token_encrypted = user
            .refresh_token_encrypted
            .as_ref()
            .ok_or(TokenError::RefreshFailed)?;

        let oauth_config = self
            .oauth_config
            .as_ref()
            .ok_or(TokenError::RefreshFailed)?;

        // Decrypt refresh token
        let refresh_token = self.crypto.decrypt(refresh_token_encrypted)?;

        // Exchange refresh token for new access token
        let flow = OAuthFlow::new(oauth_config.clone());
        let new_token = flow.refresh_token(&refresh_token).await?;

        // Save new tokens
        self.save_tokens(user.id, &new_token).await?;

        Ok(new_token.access_token)
    }

    /// Create a new user from OAuth token
    pub async fn create_user_from_token(
        &self,
        github_id: i64,
        username: &str,
        avatar_url: Option<&str>,
        token: &AuthToken,
    ) -> TokenResult<User> {
        let encrypted_access = self.crypto.encrypt(&token.access_token)?;
        let encrypted_refresh = token
            .refresh_token
            .as_ref()
            .map(|rt| self.crypto.encrypt(rt))
            .transpose()?;

        let user = self
            .db
            .create_user(
                github_id,
                username,
                avatar_url,
                &encrypted_access,
                encrypted_refresh.as_deref(),
                token.expires_at,
            )
            .await?;

        Ok(user)
    }

    /// Check if a user is logged in
    pub async fn is_logged_in(&self) -> TokenResult<bool> {
        Ok(self.db.get_current_user().await?.is_some())
    }

    /// Get current user if logged in
    pub async fn get_current_user(&self) -> TokenResult<Option<User>> {
        Ok(self.db.get_current_user().await?)
    }

    /// Logout current user
    pub async fn logout(&self) -> TokenResult<()> {
        if let Some(user) = self.db.get_current_user().await? {
            self.db.delete_user(user.id).await?;
        }
        Ok(())
    }

    /// Validate that a token is working by making a test API call
    pub async fn validate_token(&self, access_token: &str) -> TokenResult<bool> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "development-tools")
            .send()
            .await
            .map_err(OAuthError::from)?;

        Ok(response.status().is_success())
    }
}

/// Auth state that can be sent to frontend
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthState {
    pub is_logged_in: bool,
    pub user: Option<UserInfo>,
}

/// User info for frontend (without sensitive data)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: i64,
    pub github_id: i64,
    pub username: String,
    pub avatar_url: Option<String>,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            github_id: user.github_id,
            username: user.username,
            avatar_url: user.avatar_url,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests would require a running database
    // Unit tests for the crypto layer are in crypto.rs
}

