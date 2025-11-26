//! Database connection management
//!
//! Provides SQLite database connection pool and initialization.

use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Failed to create database directory: {0}")]
    DirectoryCreation(#[from] std::io::Error),

    #[error("Database connection error: {0}")]
    Connection(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Query error: {0}")]
    Query(String),
}

pub type DbResult<T> = Result<T, DatabaseError>;

/// Database wrapper that manages SQLite connection pool
#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    /// Create a new database connection
    ///
    /// This will:
    /// 1. Create the database directory if it doesn't exist
    /// 2. Create the database file if it doesn't exist
    /// 3. Run all pending migrations
    pub async fn new() -> DbResult<Self> {
        let db_path = Self::get_database_path()?;
        Self::from_path(&db_path).await
    }

    /// Create a database connection from a specific path
    pub async fn from_path(path: &PathBuf) -> DbResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let database_url = format!("sqlite:{}?mode=rwc", path.display());

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        let db = Self { pool };

        // Run migrations
        db.run_migrations().await?;

        Ok(db)
    }

    /// Create an in-memory database (for testing)
    #[cfg(test)]
    pub async fn in_memory() -> DbResult<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;

        let db = Self { pool };
        db.run_migrations().await?;

        Ok(db)
    }

    /// Get the database pool
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    /// Run all pending migrations
    async fn run_migrations(&self) -> DbResult<()> {
        super::migrations::run_migrations(&self.pool).await
    }

    /// Get the default database path
    fn get_database_path() -> DbResult<PathBuf> {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| {
                DatabaseError::DirectoryCreation(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not find local data directory",
                ))
            })?
            .join("development-tools");

        Ok(data_dir.join("gamification.db"))
    }

    /// Close the database connection
    pub async fn close(&self) {
        self.pool.close().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_creation() {
        let db = Database::in_memory().await;
        assert!(db.is_ok(), "Database should be created successfully");
    }

    #[tokio::test]
    async fn test_database_path() {
        let path = Database::get_database_path();
        assert!(path.is_ok(), "Database path should be resolvable");

        let path = path.unwrap();
        assert!(
            path.to_string_lossy().contains("development-tools"),
            "Path should contain app name"
        );
        assert!(
            path.to_string_lossy().ends_with("gamification.db"),
            "Path should end with database filename"
        );
    }
}

