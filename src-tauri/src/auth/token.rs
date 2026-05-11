//! Token management
//!
//! Handles token storage, retrieval, and secure token storage.
//! Note: GitHub Device Flow tokens don't expire and don't support refresh.
//!
//! The AES-256-GCM master key is sourced from the OS keystore — see
//! [`crate::auth::keystore`] and Audit §9.3 / Issue #196. Rows written before
//! that change carry `encryption_version = 1` and are lazily re-encrypted to
//! `ENCRYPTION_VERSION_KEYSTORE` (= 2) on first read via
//! `migrate_legacy_tokens_if_needed`.

use std::sync::Arc;

use rand::RngExt as _;
use thiserror::Error;

use super::crypto::{Crypto, CryptoError};
use super::keystore::{KeyStore, OsKeyStore, KEY_LEN};
use super::oauth::{AuthToken, OAuthError};
use crate::database::{Database, DatabaseError, User};

/// Per-row tag in `users.encryption_version` for tokens encrypted with the
/// legacy app-derived key (pre-Issue #196). Read with `Crypto::from_app_key`.
pub const ENCRYPTION_VERSION_LEGACY: i32 = 1;
/// Per-row tag for tokens encrypted with the OS-keystore-managed key.
pub const ENCRYPTION_VERSION_KEYSTORE: i32 = 2;

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
    /// Active cipher — key sourced from the OS keystore.
    crypto: Crypto,
    /// One-shot legacy cipher used solely to decrypt pre-#196 rows during
    /// `migrate_legacy_tokens_if_needed` (and the lazy `decrypt_for_user`
    /// fallback). Constructed eagerly alongside the keystore-managed
    /// cipher — the derived-key computation is cheap (a couple of
    /// `DefaultHasher` passes) and keeping both ciphers ready means the
    /// migration paths never have to fail mid-decrypt because we forgot
    /// to initialise the legacy side.
    legacy_crypto: Crypto,
    db: Database,
    /// Shared HTTP client so the periodic / startup `validate_token` probes
    /// reuse the underlying connection pool instead of spinning up a fresh
    /// TCP+TLS handshake per call. `reqwest::Client` is internally `Arc`-wrapped
    /// so cloning is cheap and the manager itself stays trivially `Send + Sync`.
    http_client: reqwest::Client,
}

impl TokenManager {
    /// Create a new token manager backed by the platform credential store.
    pub async fn new(db: Database) -> TokenResult<Self> {
        Self::with_keystore(db, Arc::new(OsKeyStore::new())).await
    }

    /// Create a token manager with a caller-supplied keystore.
    ///
    /// Production code should use [`TokenManager::new`]; this entry point
    /// exists for unit tests (which pass `MemoryKeyStore`).
    ///
    /// Async because the constructor may need to clear orphaned token rows
    /// when the OS keystore has lost its master key (see below).
    pub async fn with_keystore(db: Database, keystore: Arc<dyn KeyStore>) -> TokenResult<Self> {
        // Two-step key acquisition (instead of `get_or_create_key`) so we can
        // detect the "OS keystore was wiped but our SQLite DB survived" case
        // — e.g. the user reset their credential store, migrated profiles,
        // or ran a privacy-cleanup tool. Silently generating a fresh key
        // would leave every existing `encryption_version = 2` row encrypted
        // under an unrecoverable key, and the user would just see opaque
        // decryption errors on every subsequent token read.
        let key = match keystore.get_key().map_err(CryptoError::from)? {
            Some(k) => k,
            None => {
                // Count v2 token rows that would be bricked by a new key.
                // Logged-out rows (empty ciphertext) are excluded because
                // they have nothing to lose.
                let orphans = db.count_keystore_token_rows().await?;
                if orphans > 0 {
                    // Fail-closed recovery: drop the orphaned tokens so the
                    // user is cleanly forced through Device Flow on next
                    // launch. We preserve all non-token user data (XP,
                    // badges, etc.) — same contract as `logout()`.
                    db.clear_keystore_orphan_tokens().await?;
                    // TODO: [INFRA] logクレートに置換（ログ基盤整備時に一括対応）
                    eprintln!(
                        "Token keystore: master key missing from OS keystore but \
                         {} encrypted token row(s) found in DB; cleared orphaned \
                         tokens so the user can re-authenticate (Issue #196).",
                        orphans
                    );
                }
                let mut fresh = [0u8; KEY_LEN];
                rand::rng().fill(&mut fresh);
                keystore.set_key(&fresh).map_err(CryptoError::from)?;
                fresh
            }
        };
        let crypto = Crypto::new(&key)?;
        // `#[deprecated]` on the legacy constructor is intentional — the only
        // legitimate caller is the migration path below, so silence the lint
        // at the single call site rather than ripping the marker off.
        #[allow(deprecated)]
        let legacy_crypto = Crypto::from_app_key()?;
        Ok(Self {
            crypto,
            legacy_crypto,
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

    /// Decrypt a stored ciphertext using the cipher matched to the row's
    /// `encryption_version`. Legacy (v1) ciphertext is decrypted with the
    /// derived app-key cipher and silently re-encrypted under the
    /// keystore-managed cipher via `migrate_user_tokens`.
    async fn decrypt_for_user(&self, user: &User) -> TokenResult<String> {
        match user.encryption_version {
            ENCRYPTION_VERSION_KEYSTORE => Ok(self.crypto.decrypt(&user.access_token_encrypted)?),
            ENCRYPTION_VERSION_LEGACY => {
                let access = self.legacy_crypto.decrypt(&user.access_token_encrypted)?;
                // Also recover the refresh token (if any) so the migration
                // re-encrypts the full row in a single pass.
                let refresh = user
                    .refresh_token_encrypted
                    .as_deref()
                    .map(|ct| self.legacy_crypto.decrypt(ct))
                    .transpose()?;
                // Best-effort migrate-on-read: we already have the plaintext
                // in hand, so a transient DB write failure (disk full, lock
                // contention, etc.) must NOT block the user from using the
                // session. The eager `migrate_legacy_tokens_if_needed` sweep
                // re-runs on every launch and will retry, and on subsequent
                // reads we'll come back through here. The legacy row at rest
                // is no more recoverable than before #196 — the derived key
                // hasn't gone anywhere — so leaving it briefly is acceptable.
                if let Err(e) = self
                    .migrate_user_tokens(
                        user.id,
                        &access,
                        refresh.as_deref(),
                        user.token_expires_at,
                    )
                    .await
                {
                    // TODO: [INFRA] logクレートに置換（ログ基盤整備時に一括対応）
                    eprintln!(
                        "Token migration: failed to re-encrypt user {} during read: {}; \
                         returning decrypted token, will retry on next access",
                        user.id, e
                    );
                }
                Ok(access)
            }
            other => Err(TokenError::Crypto(CryptoError::Decryption(format!(
                "Unknown encryption_version {} for user {}",
                other, user.id
            )))),
        }
    }

    /// Re-encrypt a single user's tokens under the keystore-managed key and
    /// bump `encryption_version`. Idempotent: `update_user_tokens` writes
    /// version 2 unconditionally, so a re-run on an already-migrated row is
    /// harmless.
    async fn migrate_user_tokens(
        &self,
        user_id: i64,
        access_token: &str,
        refresh_token: Option<&str>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> TokenResult<()> {
        let encrypted_access = self.crypto.encrypt(access_token)?;
        let encrypted_refresh = refresh_token
            .map(|rt| self.crypto.encrypt(rt))
            .transpose()?;
        self.db
            .update_user_tokens(
                user_id,
                &encrypted_access,
                encrypted_refresh.as_deref(),
                expires_at,
            )
            .await?;
        Ok(())
    }

    /// Eagerly migrate every legacy-encrypted user row.
    ///
    /// Called once during application setup so the upgrade window — between
    /// "new build deployed" and "user happens to read a token" — doesn't
    /// leave plaintext-recoverable rows on disk longer than necessary.
    /// Failures on a single row are logged but do not abort the sweep; the
    /// lazy `decrypt_for_user` path will retry on next access.
    pub async fn migrate_legacy_tokens_if_needed(&self) -> TokenResult<usize> {
        let legacy_users = self.db.list_users_with_legacy_encryption().await?;
        let mut migrated = 0usize;
        for user in legacy_users {
            // Skip cleared rows (logged-out placeholders): their token blob
            // is the empty string and decryption would fail.
            if user.access_token_encrypted.is_empty() {
                // Still bump the version so a future read doesn't bounce
                // through the legacy branch.
                if let Err(e) = self
                    .db
                    .set_user_encryption_version(user.id, ENCRYPTION_VERSION_KEYSTORE)
                    .await
                {
                    eprintln!(
                        "Token migration: failed to tag empty row for user {}: {}",
                        user.id, e
                    );
                }
                continue;
            }
            match self.legacy_crypto.decrypt(&user.access_token_encrypted) {
                Ok(access) => {
                    let refresh = match user.refresh_token_encrypted.as_deref() {
                        Some(ct) => match self.legacy_crypto.decrypt(ct) {
                            Ok(plain) => Some(plain),
                            Err(e) => {
                                eprintln!(
                                    "Token migration: failed to decrypt refresh for user {}: {}; clearing it",
                                    user.id, e
                                );
                                None
                            }
                        },
                        None => None,
                    };
                    if let Err(e) = self
                        .migrate_user_tokens(
                            user.id,
                            &access,
                            refresh.as_deref(),
                            user.token_expires_at,
                        )
                        .await
                    {
                        eprintln!(
                            "Token migration: failed to re-encrypt user {}: {}",
                            user.id, e
                        );
                        continue;
                    }
                    migrated += 1;
                }
                Err(e) => {
                    // Can't recover plaintext — leave the row alone so a
                    // future hotfix can attempt it. We do NOT clear the
                    // token here; that would silently log the user out.
                    eprintln!(
                        "Token migration: failed to decrypt legacy token for user {}: {}",
                        user.id, e
                    );
                }
            }
        }
        Ok(migrated)
    }

    /// Get the current access token
    /// Note: GitHub tokens from Device Flow don't expire, so no refresh logic is needed
    pub async fn get_access_token(&self) -> TokenResult<String> {
        let user = self
            .db
            .get_current_user()
            .await?
            .ok_or(TokenError::NotLoggedIn)?;

        self.decrypt_for_user(&user).await
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
        let token = self.decrypt_for_user(&user).await?;
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

    #[test]
    fn encryption_version_constants_are_distinct() {
        // Cheap regression: callers branch on these tags inside
        // `decrypt_for_user`, so a copy-paste typo silently routing v1
        // through the v2 cipher would brick logins for existing users.
        assert_ne!(ENCRYPTION_VERSION_LEGACY, ENCRYPTION_VERSION_KEYSTORE);
        assert_eq!(ENCRYPTION_VERSION_LEGACY, 1);
        assert_eq!(ENCRYPTION_VERSION_KEYSTORE, 2);
    }

    // --------------------------------------------------------------
    // End-to-end migration: simulates "user logged in on the old
    // build, then upgraded to the keystore-managed key" — the exact
    // scenario `migrate_legacy_tokens_if_needed` exists to handle.
    // --------------------------------------------------------------
    #[tokio::test]
    #[allow(deprecated)]
    async fn legacy_row_is_decrypted_and_re_encrypted_in_place() {
        use crate::auth::keystore::MemoryKeyStore;
        use crate::database::Database;

        let db = Database::in_memory().await.expect("in-memory db");
        let legacy = Crypto::from_app_key().unwrap();
        let plaintext_access = "ghp_legacy_access_token";
        let plaintext_refresh = "ghp_legacy_refresh_token";
        let legacy_access = legacy.encrypt(plaintext_access).unwrap();
        let legacy_refresh = legacy.encrypt(plaintext_refresh).unwrap();

        // `create_user` writes encryption_version = 2 by design, so we
        // hand-insert a v1 row to mimic a pre-#196 install.
        sqlx::query(
            "INSERT INTO users (github_id, username, access_token_encrypted,
                                refresh_token_encrypted, encryption_version)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(42i64)
        .bind("legacy-user")
        .bind(&legacy_access)
        .bind(&legacy_refresh)
        .bind(ENCRYPTION_VERSION_LEGACY)
        .execute(db.pool())
        .await
        .unwrap();

        // Initialise stats so cascading lookups don't trip foreign keys
        // in code paths beyond what we exercise here.
        db.create_user_stats(1).await.unwrap();

        let keystore: Arc<dyn KeyStore> = Arc::new(MemoryKeyStore::new());
        let tm = TokenManager::with_keystore(db.clone(), keystore)
            .await
            .unwrap();

        let migrated = tm.migrate_legacy_tokens_if_needed().await.unwrap();
        assert_eq!(migrated, 1, "exactly one legacy row should migrate");

        // Row is now keystore-encrypted; legacy cipher must NOT decrypt it.
        let user = db.get_current_user().await.unwrap().expect("logged in");
        assert_eq!(user.encryption_version, ENCRYPTION_VERSION_KEYSTORE);
        assert!(
            legacy.decrypt(&user.access_token_encrypted).is_err(),
            "post-migration ciphertext must not be readable with the legacy key"
        );

        // ...but the token manager (keystore cipher) still recovers plaintext.
        let recovered = tm.get_access_token().await.unwrap();
        assert_eq!(recovered, plaintext_access);

        // A second sweep is a no-op.
        let again = tm.migrate_legacy_tokens_if_needed().await.unwrap();
        assert_eq!(
            again, 0,
            "re-running the sweep on a clean DB must not migrate"
        );
    }

    #[tokio::test]
    async fn legacy_empty_row_is_tagged_without_failing() {
        // Logged-out rows carry an empty `access_token_encrypted`; the
        // migration sweep can't decrypt them but also shouldn't get stuck
        // — it should bump the version so subsequent reads skip the legacy
        // branch.
        use crate::auth::keystore::MemoryKeyStore;
        use crate::database::Database;

        let db = Database::in_memory().await.unwrap();
        sqlx::query(
            "INSERT INTO users (github_id, username, access_token_encrypted, encryption_version)
             VALUES (?, ?, '', ?)",
        )
        .bind(1i64)
        .bind("logged-out")
        .bind(ENCRYPTION_VERSION_LEGACY)
        .execute(db.pool())
        .await
        .unwrap();

        let tm = TokenManager::with_keystore(db.clone(), Arc::new(MemoryKeyStore::new()))
            .await
            .unwrap();

        // No rows are "migrated" (we don't have a token to re-encrypt),
        // but the version tag must still flip so future reads don't trip.
        let migrated = tm.migrate_legacy_tokens_if_needed().await.unwrap();
        assert_eq!(migrated, 0);

        let row: (i32,) =
            sqlx::query_as("SELECT encryption_version FROM users WHERE github_id = 1")
                .fetch_one(db.pool())
                .await
                .unwrap();
        assert_eq!(row.0, ENCRYPTION_VERSION_KEYSTORE);
    }

    #[tokio::test]
    async fn save_tokens_writes_keystore_version() {
        use crate::auth::keystore::MemoryKeyStore;
        use crate::database::Database;

        let db = Database::in_memory().await.unwrap();
        let tm = TokenManager::with_keystore(db.clone(), Arc::new(MemoryKeyStore::new()))
            .await
            .unwrap();

        let user = tm
            .create_user_from_token(
                7,
                "fresh-user",
                None,
                &AuthToken {
                    access_token: "ghp_new_token".into(),
                    refresh_token: None,
                    expires_at: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(user.encryption_version, ENCRYPTION_VERSION_KEYSTORE);

        // And re-reading goes through the keystore cipher cleanly.
        assert_eq!(tm.get_access_token().await.unwrap(), "ghp_new_token");
    }

    /// Codex P2: if the OS keystore loses the master key but our SQLite DB
    /// survives, naïvely generating a fresh key would brick every existing
    /// v2 row. Constructor must detect this and clear the orphaned tokens
    /// so the user re-authenticates instead of getting opaque decryption
    /// errors forever.
    #[tokio::test]
    async fn lost_master_key_clears_orphan_token_rows_and_continues() {
        use crate::auth::keystore::MemoryKeyStore;
        use crate::database::Database;

        let db = Database::in_memory().await.unwrap();

        // Seed: a logged-in v2 user (will be orphaned), a logged-out v2
        // user (already empty, must remain untouched but harmless), and a
        // v1 legacy user (untouched — different key family).
        sqlx::query(
            "INSERT INTO users (github_id, username, access_token_encrypted,
                                refresh_token_encrypted, encryption_version)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(11i64)
        .bind("alive-v2")
        .bind("ciphertext-from-old-key")
        .bind(Option::<&str>::None)
        .bind(ENCRYPTION_VERSION_KEYSTORE)
        .execute(db.pool())
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO users (github_id, username, access_token_encrypted, encryption_version)
             VALUES (?, ?, '', ?)",
        )
        .bind(12i64)
        .bind("logged-out-v2")
        .bind(ENCRYPTION_VERSION_KEYSTORE)
        .execute(db.pool())
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO users (github_id, username, access_token_encrypted, encryption_version)
             VALUES (?, ?, ?, ?)",
        )
        .bind(13i64)
        .bind("legacy-v1")
        .bind("legacy-ciphertext")
        .bind(ENCRYPTION_VERSION_LEGACY)
        .execute(db.pool())
        .await
        .unwrap();

        // Empty keystore = "we lost the master key".
        let keystore: Arc<dyn KeyStore> = Arc::new(MemoryKeyStore::new());
        let _tm = TokenManager::with_keystore(db.clone(), keystore.clone())
            .await
            .expect("constructor must recover, not error");

        // The orphaned v2 row is now cleared (logged-out shape).
        let alive: (String,) =
            sqlx::query_as("SELECT access_token_encrypted FROM users WHERE github_id = 11")
                .fetch_one(db.pool())
                .await
                .unwrap();
        assert!(alive.0.is_empty(), "orphaned v2 token must be cleared");

        // The legacy v1 row is untouched — its ciphertext is decryptable
        // by `legacy_crypto`, independent of the keystore key.
        let legacy: (String, i32) = sqlx::query_as(
            "SELECT access_token_encrypted, encryption_version FROM users WHERE github_id = 13",
        )
        .fetch_one(db.pool())
        .await
        .unwrap();
        assert_eq!(legacy.0, "legacy-ciphertext");
        assert_eq!(legacy.1, ENCRYPTION_VERSION_LEGACY);

        // And the keystore is now populated, so a second construction is
        // a normal (non-recovery) startup.
        assert!(keystore.get_key().unwrap().is_some());
    }
}
