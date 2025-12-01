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

        let id = sqlx::query(
            r#"
            INSERT INTO users (github_id, username, avatar_url, access_token_encrypted, 
                              refresh_token_encrypted, token_expires_at, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(github_id)
        .bind(username)
        .bind(avatar_url)
        .bind(access_token_encrypted)
        .bind(refresh_token_encrypted)
        .bind(&expires_at)
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

        sqlx::query(
            r#"
            UPDATE users 
            SET access_token_encrypted = ?, refresh_token_encrypted = ?, 
                token_expires_at = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(access_token_encrypted)
        .bind(refresh_token_encrypted)
        .bind(expires_at)
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
