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
    Migration {
        version: 2,
        name: "add_user_settings",
        sql: r#"
-- User settings table: stores user preferences
CREATE TABLE IF NOT EXISTS user_settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER UNIQUE NOT NULL,
    
    -- Notification settings
    notification_method TEXT DEFAULT 'both', -- 'app_only' | 'os_only' | 'both' | 'none'
    notify_xp_gain INTEGER DEFAULT 1,
    notify_level_up INTEGER DEFAULT 1,
    notify_badge_earned INTEGER DEFAULT 1,
    notify_streak_update INTEGER DEFAULT 1,
    notify_streak_milestone INTEGER DEFAULT 1,
    
    -- Sync settings
    sync_interval_minutes INTEGER DEFAULT 60, -- 5, 15, 30, 60, 180, 0 (manual only)
    background_sync INTEGER DEFAULT 1,
    sync_on_startup INTEGER DEFAULT 1,
    
    -- Appearance settings
    animations_enabled INTEGER DEFAULT 1,
    
    -- Metadata
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create index for user_settings
CREATE INDEX IF NOT EXISTS idx_user_settings_user_id ON user_settings(user_id);
"#,
    },
    Migration {
        version: 3,
        name: "add_challenge_start_stats",
        sql: r#"
-- Add start_stats column to challenges table for tracking progress
-- This stores the GitHub stats at the time the challenge was created
ALTER TABLE challenges ADD COLUMN start_stats_json TEXT;
"#,
    },
    Migration {
        version: 4,
        name: "add_mock_server_tables",
        sql: r#"
-- Mock Server configuration
CREATE TABLE IF NOT EXISTS mock_server_config (
    id INTEGER PRIMARY KEY DEFAULT 1,
    port INTEGER NOT NULL DEFAULT 9876,
    cors_mode TEXT NOT NULL DEFAULT 'simple', -- 'simple' | 'advanced'
    cors_origins TEXT, -- JSON array for advanced mode
    cors_methods TEXT, -- JSON array for advanced mode
    cors_headers TEXT, -- JSON array for advanced mode
    cors_max_age INTEGER DEFAULT 86400,
    show_directory_listing INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Initialize default config
INSERT OR IGNORE INTO mock_server_config (id, port, cors_mode, cors_max_age) 
VALUES (1, 9876, 'simple', 86400);

-- Directory mappings for Mock Server
CREATE TABLE IF NOT EXISTS mock_server_mappings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    virtual_path TEXT NOT NULL UNIQUE,
    local_path TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_mock_server_mappings_virtual_path ON mock_server_mappings(virtual_path);
CREATE INDEX IF NOT EXISTS idx_mock_server_mappings_enabled ON mock_server_mappings(enabled);
"#,
    },
    Migration {
        version: 5,
        name: "add_code_stats_tables",
        sql: r#"
-- Daily code statistics table: stores additions/deletions per day
CREATE TABLE IF NOT EXISTS daily_code_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    date DATE NOT NULL,
    additions INTEGER NOT NULL DEFAULT 0,
    deletions INTEGER NOT NULL DEFAULT 0,
    commits_count INTEGER NOT NULL DEFAULT 0,
    repositories_json TEXT, -- JSON array: ["repo1", "repo2"]
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, date)
);

-- Sync metadata table: tracks incremental sync state
CREATE TABLE IF NOT EXISTS sync_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    sync_type TEXT NOT NULL, -- 'code_stats', 'contributions', etc.
    last_sync_at DATETIME,
    last_sync_cursor TEXT, -- GraphQL cursor for pagination
    etag TEXT, -- For conditional requests (ETag/If-Modified-Since)
    rate_limit_remaining INTEGER,
    rate_limit_reset_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, sync_type)
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_daily_code_stats_user_date ON daily_code_stats(user_id, date DESC);
CREATE INDEX IF NOT EXISTS idx_daily_code_stats_summary ON daily_code_stats(user_id, date, additions, deletions);
CREATE INDEX IF NOT EXISTS idx_sync_metadata_user_type ON sync_metadata(user_id, sync_type);
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

