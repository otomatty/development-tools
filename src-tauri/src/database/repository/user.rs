//! User repository operations

use chrono::{DateTime, Utc};
use sqlx::FromRow;

use crate::database::connection::{Database, DatabaseError, DbResult};
use crate::database::models::User;

/// User row from database
#[derive(Debug, FromRow)]
pub(crate) struct UserRow {
    pub id: i64,
    pub github_id: i64,
    pub username: String,
    pub avatar_url: Option<String>,
    pub access_token_encrypted: String,
    pub refresh_token_encrypted: Option<String>,
    pub token_expires_at: Option<String>,
    pub encryption_version: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl TryFrom<UserRow> for User {
    type Error = DatabaseError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        Ok(User {
            id: row.id,
            github_id: row.github_id,
            username: row.username,
            avatar_url: row.avatar_url,
            access_token_encrypted: row.access_token_encrypted,
            refresh_token_encrypted: row.refresh_token_encrypted,
            token_expires_at: row
                .token_expires_at
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            encryption_version: row.encryption_version,
            created_at: DateTime::parse_from_rfc3339(&row.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&row.updated_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

/// User repository operations
impl Database {
    /// Create a new user
    pub async fn create_user(
        &self,
        github_id: i64,
        username: &str,
        avatar_url: Option<&str>,
        access_token_encrypted: &str,
        refresh_token_encrypted: Option<&str>,
        token_expires_at: Option<DateTime<Utc>>,
    ) -> DbResult<User> {
        let now = Utc::now().to_rfc3339();
        let expires_at = token_expires_at.map(|dt| dt.to_rfc3339());

        // New rows are always written with the keystore-managed key
        // (Issue #196 / Audit §9.3); only pre-existing rows carry version 1.
        let id = sqlx::query(
            r#"
            INSERT INTO users (github_id, username, avatar_url, access_token_encrypted,
                              refresh_token_encrypted, token_expires_at, encryption_version,
                              created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(github_id)
        .bind(username)
        .bind(avatar_url)
        .bind(access_token_encrypted)
        .bind(refresh_token_encrypted)
        .bind(&expires_at)
        .bind(crate::auth::token::ENCRYPTION_VERSION_KEYSTORE)
        .bind(&now)
        .bind(&now)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?
        .last_insert_rowid();

        // Create default user stats
        self.create_user_stats(id).await?;

        self.get_user_by_id(id).await
    }

    /// Get user by ID
    pub async fn get_user_by_id(&self, id: i64) -> DbResult<User> {
        let row: UserRow = sqlx::query_as("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_one(self.pool())
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        row.try_into()
    }

    /// Get user by GitHub ID
    pub async fn get_user_by_github_id(&self, github_id: i64) -> DbResult<Option<User>> {
        let row: Option<UserRow> = sqlx::query_as("SELECT * FROM users WHERE github_id = ?")
            .bind(github_id)
            .fetch_optional(self.pool())
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }

    /// Update user tokens
    pub async fn update_user_tokens(
        &self,
        user_id: i64,
        access_token_encrypted: &str,
        refresh_token_encrypted: Option<&str>,
        token_expires_at: Option<DateTime<Utc>>,
    ) -> DbResult<()> {
        let now = Utc::now().to_rfc3339();
        let expires_at = token_expires_at.map(|dt| dt.to_rfc3339());

        // Any token write goes through the keystore-managed key, so bump
        // the per-row version tag in lockstep. Keeping the bump here means
        // a one-off `update_user_tokens` (e.g. lazy re-encryption) doesn't
        // need a separate "mark migrated" call.
        sqlx::query(
            r#"
            UPDATE users
            SET access_token_encrypted = ?, refresh_token_encrypted = ?,
                token_expires_at = ?, encryption_version = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(access_token_encrypted)
        .bind(refresh_token_encrypted)
        .bind(expires_at)
        .bind(crate::auth::token::ENCRYPTION_VERSION_KEYSTORE)
        .bind(now)
        .bind(user_id)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }

    /// Delete user (and cascade delete related data)
    pub async fn delete_user(&self, user_id: i64) -> DbResult<()> {
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(user_id)
            .execute(self.pool())
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }

    /// Clear user tokens (logout without deleting data)
    /// Sets tokens to empty string to indicate logged out state
    pub async fn clear_user_tokens(&self, user_id: i64) -> DbResult<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE users 
            SET access_token_encrypted = '', refresh_token_encrypted = NULL, 
                token_expires_at = NULL, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(now)
        .bind(user_id)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(())
    }

    /// Get current logged in user (user with valid token)
    /// Returns None if no user exists or if the user has logged out (empty token)
    pub async fn get_current_user(&self) -> DbResult<Option<User>> {
        // Only return user if they have a non-empty token (logged in)
        let row: Option<UserRow> =
            sqlx::query_as("SELECT * FROM users WHERE access_token_encrypted != '' LIMIT 1")
                .fetch_optional(self.pool())
                .await
                .map_err(|e| DatabaseError::Query(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }

    /// Fetch every user row still tagged with the legacy
    /// (`Crypto::from_app_key`) encryption version.
    ///
    /// Used by `TokenManager::migrate_legacy_tokens_if_needed` to sweep
    /// existing installations onto the keystore-managed key (Issue #196).
    /// Returns the empty vec on a fresh install — no users yet means nothing
    /// to migrate.
    pub async fn list_users_with_legacy_encryption(&self) -> DbResult<Vec<User>> {
        let rows: Vec<UserRow> = sqlx::query_as("SELECT * FROM users WHERE encryption_version = ?")
            .bind(crate::auth::token::ENCRYPTION_VERSION_LEGACY)
            .fetch_all(self.pool())
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;
        rows.into_iter().map(User::try_from).collect()
    }

    /// Count rows whose tokens were encrypted under the keystore-managed
    /// key and still hold a non-empty ciphertext.
    ///
    /// Used by `TokenManager::with_keystore` to detect the "OS keystore
    /// was wiped but our SQLite DB survived" recovery case — see Issue
    /// #196 / Codex review. A non-zero count means a freshly generated
    /// master key would orphan those rows, so the caller wipes the
    /// ciphertext and forces re-authentication.
    pub async fn count_keystore_token_rows(&self) -> DbResult<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM users \
             WHERE encryption_version = ? AND access_token_encrypted != ''",
        )
        .bind(crate::auth::token::ENCRYPTION_VERSION_KEYSTORE)
        .fetch_one(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;
        Ok(row.0)
    }

    /// Wipe access / refresh tokens on every keystore-encrypted row.
    ///
    /// Used in the lost-master-key recovery path: the ciphertext is no
    /// longer decryptable, so we treat those users as logged out (same
    /// contract as `clear_user_tokens` — non-token user data is
    /// preserved). The encryption version stays at
    /// `ENCRYPTION_VERSION_KEYSTORE` because the fresh master key
    /// generated immediately after is also a v2 key.
    pub async fn clear_keystore_orphan_tokens(&self) -> DbResult<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            UPDATE users
            SET access_token_encrypted = '',
                refresh_token_encrypted = NULL,
                token_expires_at = NULL,
                updated_at = ?
            WHERE encryption_version = ? AND access_token_encrypted != ''
            "#,
        )
        .bind(now)
        .bind(crate::auth::token::ENCRYPTION_VERSION_KEYSTORE)
        .execute(self.pool())
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;
        Ok(())
    }

    /// Set `encryption_version` for a user without rewriting the ciphertext.
    ///
    /// Used by the migration to tag rows that have an empty token blob
    /// (logged-out users) so a future read doesn't keep bouncing through
    /// the legacy decryption path.
    pub async fn set_user_encryption_version(
        &self,
        user_id: i64,
        encryption_version: i32,
    ) -> DbResult<()> {
        sqlx::query("UPDATE users SET encryption_version = ? WHERE id = ?")
            .bind(encryption_version)
            .bind(user_id)
            .execute(self.pool())
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;
        Ok(())
    }

    /// Get user by ID regardless of login state
    ///
    /// Unlike `get_current_user()` which only returns logged-in users,
    /// this function returns the user even if they have logged out.
    ///
    /// **Intended use cases:**
    /// - Data recovery scenarios
    /// - Admin/maintenance operations
    /// - Checking if user data exists before re-login
    /// - Future multi-account support where we need to access
    ///   non-current user accounts
    pub async fn get_user_by_id_any_state(&self, id: i64) -> DbResult<Option<User>> {
        let row: Option<UserRow> = sqlx::query_as("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool())
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }
}
