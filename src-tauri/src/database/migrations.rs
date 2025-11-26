//! Database migrations
//!
//! This module contains all database schema migrations.
//! Migrations are run in order and tracked in a migrations table.

use sqlx::{Pool, Row, Sqlite};

use super::connection::{DatabaseError, DbResult};

/// Migration definition
struct Migration {
    version: i32,
    name: &'static str,
    sql: &'static str,
}

/// All migrations in order
const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "initial_schema",
        sql: r#"
-- Users table: stores GitHub user information
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    github_id INTEGER UNIQUE NOT NULL,
    username TEXT NOT NULL,
    avatar_url TEXT,
    access_token_encrypted TEXT NOT NULL,
    refresh_token_encrypted TEXT,
    token_expires_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- User statistics table: stores gamification data
CREATE TABLE IF NOT EXISTS user_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER UNIQUE NOT NULL,
    total_xp INTEGER DEFAULT 0,
    current_level INTEGER DEFAULT 1,
    current_streak INTEGER DEFAULT 0,
    longest_streak INTEGER DEFAULT 0,
    last_activity_date DATE,
    total_commits INTEGER DEFAULT 0,
    total_prs INTEGER DEFAULT 0,
    total_reviews INTEGER DEFAULT 0,
    total_issues INTEGER DEFAULT 0,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Badges table: stores earned badges
CREATE TABLE IF NOT EXISTS badges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    badge_type TEXT NOT NULL,
    badge_id TEXT NOT NULL,
    earned_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, badge_id)
);

-- Challenges table: stores active and completed challenges
CREATE TABLE IF NOT EXISTS challenges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    challenge_type TEXT NOT NULL,
    target_metric TEXT NOT NULL,
    target_value INTEGER NOT NULL,
    current_value INTEGER DEFAULT 0,
    reward_xp INTEGER NOT NULL,
    start_date DATETIME NOT NULL,
    end_date DATETIME NOT NULL,
    status TEXT DEFAULT 'active',
    completed_at DATETIME,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- XP history table: tracks XP gains
CREATE TABLE IF NOT EXISTS xp_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    action_type TEXT NOT NULL,
    xp_amount INTEGER NOT NULL,
    description TEXT,
    github_event_id TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Activity cache table: caches GitHub API responses
CREATE TABLE IF NOT EXISTS activity_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    data_type TEXT NOT NULL,
    data_json TEXT NOT NULL,
    fetched_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, data_type)
);

-- App settings table: stores application settings
CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_badges_user_id ON badges(user_id);
CREATE INDEX IF NOT EXISTS idx_challenges_user_id ON challenges(user_id);
CREATE INDEX IF NOT EXISTS idx_challenges_status ON challenges(status);
CREATE INDEX IF NOT EXISTS idx_xp_history_user_id ON xp_history(user_id);
CREATE INDEX IF NOT EXISTS idx_xp_history_created_at ON xp_history(created_at);
CREATE INDEX IF NOT EXISTS idx_activity_cache_expires ON activity_cache(expires_at);
CREATE INDEX IF NOT EXISTS idx_activity_cache_user_type ON activity_cache(user_id, data_type);
"#,
    },
];

/// Create the migrations tracking table
async fn ensure_migrations_table(pool: &Pool<Sqlite>) -> DbResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS _migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| DatabaseError::Migration(e.to_string()))?;

    Ok(())
}

/// Get the current migration version
async fn get_current_version(pool: &Pool<Sqlite>) -> DbResult<i32> {
    let result = sqlx::query("SELECT COALESCE(MAX(version), 0) as version FROM _migrations")
        .fetch_one(pool)
        .await
        .map_err(|e| DatabaseError::Migration(e.to_string()))?;

    Ok(result.get::<i32, _>("version"))
}

/// Record a migration as applied
async fn record_migration(pool: &Pool<Sqlite>, version: i32, name: &str) -> DbResult<()> {
    sqlx::query("INSERT INTO _migrations (version, name) VALUES (?, ?)")
        .bind(version)
        .bind(name)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Migration(e.to_string()))?;

    Ok(())
}

/// Run all pending migrations
pub async fn run_migrations(pool: &Pool<Sqlite>) -> DbResult<()> {
    ensure_migrations_table(pool).await?;

    let current_version = get_current_version(pool).await?;

    for migration in MIGRATIONS {
        if migration.version > current_version {
            tracing_log(&format!(
                "Running migration {}: {}",
                migration.version, migration.name
            ));

            // Execute migration SQL
            sqlx::query(migration.sql)
                .execute(pool)
                .await
                .map_err(|e| {
                    DatabaseError::Migration(format!(
                        "Failed to run migration {}: {}",
                        migration.version, e
                    ))
                })?;

            // Record migration
            record_migration(pool, migration.version, migration.name).await?;

            tracing_log(&format!(
                "Migration {} completed successfully",
                migration.version
            ));
        }
    }

    Ok(())
}

/// Simple logging function (will be replaced with proper tracing later)
fn tracing_log(message: &str) {
    #[cfg(debug_assertions)]
    eprintln!("[DB Migration] {}", message);
    let _ = message; // Suppress unused warning in release
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn create_test_pool() -> Pool<Sqlite> {
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create test pool")
    }

    #[tokio::test]
    async fn test_migrations_run_successfully() {
        let pool = create_test_pool().await;

        let result = run_migrations(&pool).await;
        assert!(result.is_ok(), "Migrations should run successfully");

        // Verify tables exist
        let tables: Vec<String> = sqlx::query_scalar(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
        )
        .fetch_all(&pool)
        .await
        .expect("Should query tables");

        assert!(tables.contains(&"users".to_string()));
        assert!(tables.contains(&"user_stats".to_string()));
        assert!(tables.contains(&"badges".to_string()));
        assert!(tables.contains(&"challenges".to_string()));
        assert!(tables.contains(&"xp_history".to_string()));
        assert!(tables.contains(&"activity_cache".to_string()));
        assert!(tables.contains(&"app_settings".to_string()));
        assert!(tables.contains(&"_migrations".to_string()));
    }

    #[tokio::test]
    async fn test_migrations_are_idempotent() {
        let pool = create_test_pool().await;

        // Run migrations twice
        run_migrations(&pool).await.expect("First run should succeed");
        run_migrations(&pool)
            .await
            .expect("Second run should also succeed");

        // Check migration count
        let count: i32 =
            sqlx::query_scalar("SELECT COUNT(*) FROM _migrations")
                .fetch_one(&pool)
                .await
                .expect("Should count migrations");

        assert_eq!(
            count,
            MIGRATIONS.len() as i32,
            "Should have correct number of migrations"
        );
    }
}

