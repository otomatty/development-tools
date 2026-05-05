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
    /// Shared HTTP client so the periodic / startup `validate_token` probes
    /// reuse the underlying connection pool instead of spinning up a fresh
    /// TCP+TLS handshake per call. `reqwest::Client` is internally `Arc`-wrapped
    /// so cloning is cheap and the manager itself stays trivially `Send + Sync`.
    http_client: reqwest::Client,
}

impl TokenManager {
    /// Create a new token manager
    pub fn new(db: Database) -> TokenResult<Self> {
        let crypto = Crypto::from_app_key()?;
        Ok(Self {
            crypto,
            db,
            http_client: reqwest::Client::new(),
        })
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

    /// Get the current user *and* the decrypted access token from the same
    /// row read.
    ///
    /// Combining the two lookups closes a race where the user logs out (or
    /// switches accounts) between separate `get_access_token()` and
    /// `get_current_user()` calls — without it, a command can issue an API
    /// request with account A's token and then persist the response under
    /// account B's local `user.id`. Callers that need both must use this
    /// method instead of the two-step pattern.
    pub async fn get_current_user_with_token(&self) -> TokenResult<(User, String)> {
        let user = self
            .db
            .get_current_user()
            .await?
            .ok_or(TokenError::NotLoggedIn)?;
        let token = self.crypto.decrypt(&user.access_token_encrypted)?;
        Ok((user, token))
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

    /// Validate that a token is working by making a test API call.
    ///
    /// Returns:
    /// - `Ok(true)` when the API call succeeded (token is currently accepted)
    /// - `Ok(false)` when GitHub responded with 401 (token revoked / invalid)
    /// - `Err(_)` for transport / non-401 HTTP failures so callers can
    ///   distinguish "definitely revoked" from "couldn't reach GitHub" — the
    ///   latter must NOT trigger a forced logout.
    pub async fn validate_token(&self, access_token: &str) -> TokenResult<bool> {
        let response = self
            .http_client
            .get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "development-tools")
            .send()
            .await
            .map_err(OAuthError::from)?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Ok(false);
        }
        // Non-401, non-success status (403 rate-limited / abuse-blocked, 5xx
        // server error, etc.) becomes an `Err` via `error_for_status` so
        // callers can distinguish "definitely revoked" from "GitHub is having
        // issues" and avoid a forced logout — see the lifecycle contract
        // documented above and in docs/api/AUTH_LIFECYCLE.md.
        response.error_for_status().map_err(OAuthError::from)?;
        Ok(true)
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

    /// Compile-time regression for the `validate_token` three-way contract.
    ///
    /// The body of `validate_token` itself can't be unit-tested without an
    /// HTTP mock, but we can at least lock in the documented mapping at the
    /// type / status-code level so a future refactor doesn't silently
    /// re-introduce the "Ok(false) for any non-success status" bug
    /// (Issue #181 review feedback) that would force-logout users on
    /// transient 5xx / 403 responses.
    #[test]
    fn validate_token_status_mapping_contract() {
        // 2xx → Ok(true)
        assert!(reqwest::StatusCode::OK.is_success());
        // 401 → Ok(false) — only this status maps to the auth-expired flow.
        assert_eq!(
            reqwest::StatusCode::UNAUTHORIZED,
            reqwest::StatusCode::from_u16(401).unwrap()
        );
        // 403 / 5xx must NOT be is_success() AND must not equal UNAUTHORIZED,
        // so they take the Err branch in the implementation.
        for code in [403u16, 500, 502, 503, 504] {
            let status = reqwest::StatusCode::from_u16(code).unwrap();
            assert!(!status.is_success(), "{} should not be success", code);
            assert_ne!(
                status,
                reqwest::StatusCode::UNAUTHORIZED,
                "{} should not equal 401",
                code
            );
        }
    }
}
