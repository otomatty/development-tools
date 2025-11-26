//! Token management
//!
//! Handles token storage, retrieval, and secure token storage.
//! Note: GitHub Device Flow tokens don't expire and don't support refresh.

use thiserror::Error;

use super::crypto::{Crypto, CryptoError};
use super::oauth::{AuthToken, OAuthError};
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
}

pub type TokenResult<T> = Result<T, TokenError>;

/// Token manager handles secure token storage and retrieval
pub struct TokenManager {
    crypto: Crypto,
    db: Database,
}

impl TokenManager {
    /// Create a new token manager
    pub fn new(db: Database) -> TokenResult<Self> {
        let crypto = Crypto::from_app_key()?;
        Ok(Self { crypto, db })
    }

    /// Save tokens for a user
    pub async fn save_tokens(&self, user_id: i64, token: &AuthToken) -> TokenResult<()> {
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

    /// Get the current access token
    /// Note: GitHub tokens from Device Flow don't expire, so no refresh logic is needed
    pub async fn get_access_token(&self) -> TokenResult<String> {
        let user = self
            .db
            .get_current_user()
            .await?
            .ok_or(TokenError::NotLoggedIn)?;

        // Decrypt and return the token
        self.crypto
            .decrypt(&user.access_token_encrypted)
            .map_err(|e| e.into())
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

    /// Logout current user (clears token but preserves user data)
    pub async fn logout(&self) -> TokenResult<()> {
        if let Some(user) = self.db.get_current_user().await? {
            // Only clear the token, preserve all user data (XP, badges, etc.)
            self.db.clear_user_tokens(user.id).await?;
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
    pub created_at: Option<String>,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            github_id: user.github_id,
            username: user.username,
            avatar_url: user.avatar_url,
            created_at: Some(user.created_at.to_rfc3339()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests would require a running database
    // Unit tests for the crypto layer are in crypto.rs
}
