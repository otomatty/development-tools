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
    Migration {
        version: 6,
        name: "add_github_stats_snapshots",
        sql: r#"
-- GitHub stats snapshots table: stores daily snapshots for diff calculation
-- Related Issue: GitHub Issue #35 - GitHub統計の前日比表示機能
CREATE TABLE IF NOT EXISTS github_stats_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    total_commits INTEGER NOT NULL DEFAULT 0,
    total_prs INTEGER NOT NULL DEFAULT 0,
    total_reviews INTEGER NOT NULL DEFAULT 0,
    total_issues INTEGER NOT NULL DEFAULT 0,
    total_stars_received INTEGER NOT NULL DEFAULT 0,
    total_contributions INTEGER NOT NULL DEFAULT 0,
    snapshot_date DATE NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, snapshot_date)
);

-- Create indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_github_stats_snapshots_user_date 
    ON github_stats_snapshots(user_id, snapshot_date DESC);
"#,
    },
    Migration {
        version: 7,
        name: "add_issue_management_tables",
        sql: r#"
-- Projects table: 1 project = 1 repository
-- Related Issue: GitHub Issue #59 - GitHub Issue管理機能（Linear風カンバン）
CREATE TABLE IF NOT EXISTS projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    -- Repository info (1:1 mapping)
    github_repo_id INTEGER,
    repo_owner TEXT,
    repo_name TEXT,
    repo_full_name TEXT,
    is_actions_setup INTEGER DEFAULT 0,
    last_synced_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, github_repo_id)
);

-- Cached Issues table: local cache of GitHub issues
CREATE TABLE IF NOT EXISTS cached_issues (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    github_issue_id INTEGER NOT NULL,
    number INTEGER NOT NULL,
    title TEXT NOT NULL,
    body TEXT,
    state TEXT NOT NULL DEFAULT 'open',
    status TEXT NOT NULL DEFAULT 'backlog',
    priority TEXT,
    assignee_login TEXT,
    assignee_avatar_url TEXT,
    labels_json TEXT,
    html_url TEXT,
    github_created_at DATETIME,
    github_updated_at DATETIME,
    cached_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    UNIQUE(project_id, github_issue_id)
);

-- Create indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_projects_user ON projects(user_id);
CREATE INDEX IF NOT EXISTS idx_cached_issues_project ON cached_issues(project_id);
CREATE INDEX IF NOT EXISTS idx_cached_issues_status ON cached_issues(project_id, status);
CREATE INDEX IF NOT EXISTS idx_cached_issues_number ON cached_issues(project_id, number);
"#,
    },
    Migration {
        version: 8,
        name: "add_xp_history_breakdown",
        sql: r#"
-- Add breakdown_json column to xp_history table
-- This stores the detailed XP breakdown for each history entry
-- Related: XP履歴の詳細内訳保存機能
ALTER TABLE xp_history ADD COLUMN breakdown_json TEXT;
"#,
    },
    Migration {
        version: 9,
        name: "drop_legacy_static_file_server_tables",
        sql: r#"
-- Drop tables from a removed feature (see GitHub issue #175).
-- Safe for fresh installs because IF EXISTS guards the drops.
DROP INDEX IF EXISTS idx_mock_server_mappings_virtual_path;
DROP INDEX IF EXISTS idx_mock_server_mappings_enabled;
DROP TABLE IF EXISTS mock_server_mappings;
DROP TABLE IF EXISTS mock_server_config;
"#,
    },
    Migration {
        version: 10,
        name: "add_sync_metadata_last_skipped",
        sql: r#"
-- Track the most recent skip event per sync_type for the scheduler.
-- Related Issue: GitHub Issue #180 - 同期スケジューラの実装
ALTER TABLE sync_metadata ADD COLUMN last_skipped_at DATETIME;
ALTER TABLE sync_metadata ADD COLUMN last_skipped_reason TEXT;
"#,
    },
    Migration {
        version: 11,
        name: "add_sync_metadata_scheduler_baseline",
        sql: r#"
-- Distinct from `last_sync_at` (which records actual sync completions),
-- `scheduler_baseline_at` records when the scheduler decided to start
-- counting the interval for `sync_on_startup=false` users with no real sync
-- history. Keeping them separate avoids lying to UI consumers that read
-- `last_sync_at` as "last real sync".
-- Related Issue: GitHub Issue #180 - 同期スケジューラの実装
ALTER TABLE sync_metadata ADD COLUMN scheduler_baseline_at DATETIME;
"#,
    },
    Migration {
        version: 12,
        name: "consolidate_previous_github_stats_into_snapshots",
        sql: r#"
-- Consolidate the two stores of "previous GitHub stats" into
-- `github_stats_snapshots` so XP diffs and the daily-comparison UI share
-- a single source of truth.
-- Related Issue: GitHub Issue #189 / Audit §9.2
--
-- 1. Extend `github_stats_snapshots` so it can serve as the XP base — we
--    need `total_prs_merged` and `total_issues_closed` to compute the
--    same XP breakdown the legacy KV path produced.
ALTER TABLE github_stats_snapshots ADD COLUMN total_prs_merged INTEGER NOT NULL DEFAULT 0;
ALTER TABLE github_stats_snapshots ADD COLUMN total_issues_closed INTEGER NOT NULL DEFAULT 0;

-- 2. Seed a snapshot for users who only have the legacy KV but no
--    snapshot row. Without this fallback, step 4's DELETE would strip
--    their only baseline and the first post-migration sync would treat
--    them as a fresh user — re-awarding XP for their entire lifetime
--    of activity. The JSON keys are camelCase (`#[serde(rename_all =
--    "camelCase")]` on `GitHubStats`); `DATE(ac.fetched_at)` anchors
--    the seed row to when the KV was last written, which approximates
--    the user's last sync.
INSERT INTO github_stats_snapshots (
    user_id, total_commits, total_prs, total_prs_merged, total_reviews,
    total_issues, total_issues_closed, total_stars_received,
    total_contributions, snapshot_date
)
SELECT
    ac.user_id,
    COALESCE(CAST(json_extract(ac.data_json, '$.totalCommits')        AS INTEGER), 0),
    COALESCE(CAST(json_extract(ac.data_json, '$.totalPrs')            AS INTEGER), 0),
    COALESCE(CAST(json_extract(ac.data_json, '$.totalPrsMerged')      AS INTEGER), 0),
    COALESCE(CAST(json_extract(ac.data_json, '$.totalReviews')        AS INTEGER), 0),
    COALESCE(CAST(json_extract(ac.data_json, '$.totalIssues')         AS INTEGER), 0),
    COALESCE(CAST(json_extract(ac.data_json, '$.totalIssuesClosed')   AS INTEGER), 0),
    COALESCE(CAST(json_extract(ac.data_json, '$.totalStarsReceived')  AS INTEGER), 0),
    COALESCE(CAST(json_extract(ac.data_json, '$.totalContributions')  AS INTEGER), 0),
    DATE(ac.fetched_at)
FROM activity_cache ac
WHERE ac.data_type = 'previous_github_stats'
  AND NOT EXISTS (
      SELECT 1 FROM github_stats_snapshots s WHERE s.user_id = ac.user_id
  );

-- 3. Backfill `prs_merged` / `issues_closed` on each user's *latest*
--    snapshot from the legacy KV. Picking by `MAX(snapshot_date)` (not
--    `MAX(id)`) is required because `id` only reflects insert order —
--    if a user backfilled an older date later, `MAX(id)` would land on
--    that backdated row while the actual latest row stayed at default
--    `0`, leaving the next sync to re-award XP for every historical
--    merged PR / closed issue.
--    Older snapshots stay at the default 0 because they are only used
--    when even more recent rows are absent, and the diff UI doesn't
--    surface these two metrics anyway.
UPDATE github_stats_snapshots
SET total_prs_merged = COALESCE(
        (SELECT CAST(json_extract(ac.data_json, '$.totalPrsMerged') AS INTEGER)
         FROM activity_cache ac
         WHERE ac.user_id = github_stats_snapshots.user_id
           AND ac.data_type = 'previous_github_stats'),
        total_prs_merged
    ),
    total_issues_closed = COALESCE(
        (SELECT CAST(json_extract(ac.data_json, '$.totalIssuesClosed') AS INTEGER)
         FROM activity_cache ac
         WHERE ac.user_id = github_stats_snapshots.user_id
           AND ac.data_type = 'previous_github_stats'),
        total_issues_closed
    )
WHERE snapshot_date = (
    SELECT MAX(s2.snapshot_date)
    FROM github_stats_snapshots s2
    WHERE s2.user_id = github_stats_snapshots.user_id
);

-- 4. Drop the legacy `previous_github_stats` KV now that snapshots carry
--    the same information. Safe because steps 2–3 guarantee every user
--    that had a KV now has at least one snapshot row populated with
--    that KV's `prs_merged` / `issues_closed` values.
DELETE FROM activity_cache WHERE data_type = 'previous_github_stats';
"#,
    },
    Migration {
        version: 13,
        name: "add_project_archive_columns",
        sql: r#"
-- Track linked-repo deletion / rename so `sync_project_issues` can mark a
-- single project unreachable instead of stopping the whole sync run, and so
-- the UI can offer "re-link" or "delete" without the user having to dig
-- through the database. See Issue #190 / Audit §7.3 G-09.
--
-- `is_archived` is the durable flag (1 = repository gone, 0 = healthy).
-- `archived_at` records when we first observed the 404 and is null while
-- the project is healthy. `archived_reason` is a short tag we expose to
-- the UI ("repository_gone", or future variants).
--
-- We deliberately keep `repo_*` columns populated on archive so the
-- "re-link" flow can prefill the dialog with the previous owner / repo,
-- and so audit logs remain meaningful after the fact.
ALTER TABLE projects ADD COLUMN is_archived INTEGER NOT NULL DEFAULT 0;
ALTER TABLE projects ADD COLUMN archived_at DATETIME;
ALTER TABLE projects ADD COLUMN archived_reason TEXT;

-- Mirror flag on `cached_issues` so the UI can dim / segregate orphaned
-- issues without dropping them (the original GitHub URL still works for
-- a renamed repo, and historical context is sometimes valuable even
-- after a hard delete). Physical deletion is intentionally avoided —
-- `delete_project` already cascades when the user opts in.
ALTER TABLE cached_issues ADD COLUMN is_archived INTEGER NOT NULL DEFAULT 0;
ALTER TABLE cached_issues ADD COLUMN archived_at DATETIME;

CREATE INDEX IF NOT EXISTS idx_projects_archived ON projects(user_id, is_archived);
"#,
    },
    Migration {
        version: 14,
        name: "add_user_stats_badge_eval_fields",
        sql: r#"
-- Extend `user_stats` with the aggregate fields the badge evaluator needs
-- so `get_badges_with_progress` can run entirely against the local DB
-- instead of re-fetching `client.get_user_stats` (REST 4 + GraphQL 1) on
-- every render. See Issue #191 / Audit §6.3 / §9.1.
--
-- Older rows default to 0 — the next `sync_github_stats` run rewrites
-- these from the freshly fetched `GitHubStats`, so the badge UI catches
-- up after the first post-migration sync. Until then, badges that
-- depend on these new metrics show 0 progress (never *spurious* progress),
-- which is the same behaviour a brand-new user gets.
ALTER TABLE user_stats ADD COLUMN weekly_streak INTEGER NOT NULL DEFAULT 0;
ALTER TABLE user_stats ADD COLUMN monthly_streak INTEGER NOT NULL DEFAULT 0;
ALTER TABLE user_stats ADD COLUMN total_prs_merged INTEGER NOT NULL DEFAULT 0;
ALTER TABLE user_stats ADD COLUMN total_issues_closed INTEGER NOT NULL DEFAULT 0;
ALTER TABLE user_stats ADD COLUMN languages_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE user_stats ADD COLUMN total_stars_received INTEGER NOT NULL DEFAULT 0;
"#,
    },
    Migration {
        version: 15,
        name: "add_xp_history_source",
        sql: r#"
-- Distinguish how each xp_history row was produced so the past-year
-- recalculation feature (Issue #194 / Audit §6.2 / §8 G-13) can coexist
-- with the live sync stream without overwriting it.
--
-- 'live'         — recorded by run_github_sync / streak bonus / manual add_xp
--                  (default for all existing rows so this migration is a no-op
--                  for live behaviour and `total_xp` is unchanged).
-- 'recalculated' — written by recalculate_xp_history; these rows are NOT
--                  added to `user_stats.total_xp`. They exist purely as an
--                  audit / comparison surface so the UI can render
--                  "live vs recalculated" without destroying the original
--                  XP record.
--
-- Keeping the original 'live' rows intact preserves the audit trail and
-- makes the recalculation idempotent: re-running just inserts another
-- 'recalculated' row, never mutates past 'live' entries.
ALTER TABLE xp_history ADD COLUMN source TEXT NOT NULL DEFAULT 'live';

-- Index supports both (a) the rate-limit guard's "most recent recalc per
-- user" lookup and (b) the per-window total used for before/after diffing.
CREATE INDEX IF NOT EXISTS idx_xp_history_user_source_created
    ON xp_history(user_id, source, created_at);
"#,
    },
    Migration {
        version: 16,
        name: "normalize_xp_history_created_at_to_rfc3339",
        sql: r#"
-- One-shot canonicalisation of `xp_history.created_at`.
--
-- Rows recorded before Issue #194 were inserted via SQLite's
-- `CURRENT_TIMESTAMP` default and stored as `YYYY-MM-DD HH:MM:SS` (UTC,
-- whole seconds). Rows recorded after Issue #194 are written as RFC3339
-- with sub-second precision (e.g. `2026-05-10T22:11:33.482912+00:00`).
--
-- The mixed format forced range queries to wrap `created_at` in
-- `datetime(...)` for correctness, which (a) prevented the
-- `(user_id, source, created_at)` index from being used for the range
-- portion and (b) silently rounded both sides to whole seconds, so a
-- sub-second-precision live row written near the bound could be
-- mis-classified by `recalculate_xp_history`'s before/after diff
-- (Codex P2 review on PR #217).
--
-- Backfilling the legacy rows to `YYYY-MM-DDTHH:MM:SS+00:00` gives the
-- whole table a single canonical lexicographic ordering that:
--   1. matches the new RFC3339 inserts byte-for-byte for the same
--      whole-second instant;
--   2. preserves sub-second precision (no `datetime()` truncation);
--   3. lets the v15 composite index drive range scans directly.
--
-- Idempotent guard: the `WHERE` clause skips any row that already has
-- a 'T' (i.e. is already RFC3339), so re-running the migration on a
-- fresh install (with no legacy rows) or on a partially-migrated DB
-- is a no-op.
UPDATE xp_history
   SET created_at = strftime('%Y-%m-%dT%H:%M:%S+00:00', created_at)
 WHERE created_at NOT LIKE '%T%';
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

    /// Issue #189 / migration v12: when an existing user has both a
    /// `previous_github_stats` KV and a `github_stats_snapshots` row at
    /// the time the migration runs, the new `total_prs_merged` and
    /// `total_issues_closed` columns must be backfilled from the KV's
    /// JSON onto the user's most-recent snapshot — otherwise the next
    /// sync would compute the diff against `0` and award a one-time XP
    /// burst for every historical merged PR / closed issue.
    ///
    /// The test re-runs the v12 backfill+delete block (which is
    /// idempotent) on a freshly migrated DB seeded with a v11-shape
    /// snapshot (prs_merged=0, issues_closed=0) plus a legacy KV.
    #[tokio::test]
    async fn test_migration_v12_backfills_prs_merged_and_issues_closed() {
        let pool = create_test_pool().await;
        run_migrations(&pool)
            .await
            .expect("Migrations should run cleanly");

        sqlx::query(
            "INSERT INTO users (github_id, username, access_token_encrypted) VALUES (?, ?, ?)",
        )
        .bind(12345_i64)
        .bind("testuser")
        .bind("encrypted")
        .execute(&pool)
        .await
        .expect("Should insert user");

        sqlx::query(
            r#"INSERT INTO github_stats_snapshots
               (user_id, total_commits, total_prs, total_prs_merged, total_reviews,
                total_issues, total_issues_closed, total_stars_received,
                total_contributions, snapshot_date)
               VALUES (1, 100, 20, 0, 30, 15, 0, 50, 200, '2025-11-30')"#,
        )
        .execute(&pool)
        .await
        .expect("Should insert pre-v12-shape snapshot");

        sqlx::query(
            r#"INSERT INTO activity_cache
               (user_id, data_type, data_json, fetched_at, expires_at)
               VALUES (1, 'previous_github_stats',
                       '{"totalPrsMerged":12,"totalIssuesClosed":8}',
                       '2025-11-30T00:00:00Z', '2125-11-30T00:00:00Z')"#,
        )
        .execute(&pool)
        .await
        .expect("Should insert legacy KV");

        // Re-apply the v12 backfill+delete tail. Both statements are
        // idempotent (UPDATE COALESCEs to current value when the
        // subquery returns NULL; DELETE on an empty result set is a
        // no-op), so this also proves the v12 SQL itself is safe to
        // re-run accidentally.
        sqlx::query(
            r#"
            UPDATE github_stats_snapshots
            SET total_prs_merged = COALESCE(
                    (SELECT CAST(json_extract(ac.data_json, '$.totalPrsMerged') AS INTEGER)
                     FROM activity_cache ac
                     WHERE ac.user_id = github_stats_snapshots.user_id
                       AND ac.data_type = 'previous_github_stats'),
                    total_prs_merged
                ),
                total_issues_closed = COALESCE(
                    (SELECT CAST(json_extract(ac.data_json, '$.totalIssuesClosed') AS INTEGER)
                     FROM activity_cache ac
                     WHERE ac.user_id = github_stats_snapshots.user_id
                       AND ac.data_type = 'previous_github_stats'),
                    total_issues_closed
                )
            WHERE snapshot_date = (
                SELECT MAX(s2.snapshot_date)
                FROM github_stats_snapshots s2
                WHERE s2.user_id = github_stats_snapshots.user_id
            );
            "#,
        )
        .execute(&pool)
        .await
        .expect("Backfill UPDATE should succeed");

        sqlx::query("DELETE FROM activity_cache WHERE data_type = 'previous_github_stats'")
            .execute(&pool)
            .await
            .expect("KV cleanup should succeed");

        let (prs_merged, issues_closed): (i32, i32) = sqlx::query_as(
            r#"SELECT total_prs_merged, total_issues_closed
               FROM github_stats_snapshots
               WHERE user_id = 1 AND snapshot_date = '2025-11-30'"#,
        )
        .fetch_one(&pool)
        .await
        .expect("Snapshot should be queryable");

        assert_eq!(
            prs_merged, 12,
            "total_prs_merged must be backfilled from the legacy KV"
        );
        assert_eq!(
            issues_closed, 8,
            "total_issues_closed must be backfilled from the legacy KV"
        );

        let kv_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM activity_cache WHERE data_type = 'previous_github_stats'",
        )
        .fetch_one(&pool)
        .await
        .expect("Should count remaining KV rows");
        assert_eq!(kv_count, 0, "Legacy KV must be deleted");
    }

    /// Issue #189 / migration v12: when a user has older snapshots that
    /// were inserted *after* the most recent one (e.g. a gap-fill
    /// backfill writes a `2025-11-01` row long after `2025-11-30` was
    /// already saved), the backfill must still target the row with the
    /// latest `snapshot_date`. Picking `MAX(id)` would land on the
    /// backdated row and leave the actual latest row at the default 0
    /// — letting the next sync re-award lifetime XP.
    #[tokio::test]
    async fn test_migration_v12_targets_latest_snapshot_by_date_not_id() {
        let pool = create_test_pool().await;
        run_migrations(&pool)
            .await
            .expect("Migrations should run cleanly");

        sqlx::query(
            "INSERT INTO users (github_id, username, access_token_encrypted) VALUES (?, ?, ?)",
        )
        .bind(54321_i64)
        .bind("backdater")
        .bind("encrypted")
        .execute(&pool)
        .await
        .expect("Should insert user");

        // Insert "today" first (lower id, latest date), then a backdated
        // older row (higher id, earlier date). MAX(id) would pick the
        // backdated row; MAX(snapshot_date) correctly picks today.
        sqlx::query(
            r#"INSERT INTO github_stats_snapshots
               (user_id, total_commits, total_prs, total_prs_merged, total_reviews,
                total_issues, total_issues_closed, total_stars_received,
                total_contributions, snapshot_date)
               VALUES (1, 100, 20, 0, 30, 15, 0, 50, 200, '2025-11-30')"#,
        )
        .execute(&pool)
        .await
        .expect("Should insert today snapshot");

        sqlx::query(
            r#"INSERT INTO github_stats_snapshots
               (user_id, total_commits, total_prs, total_prs_merged, total_reviews,
                total_issues, total_issues_closed, total_stars_received,
                total_contributions, snapshot_date)
               VALUES (1, 50, 10, 0, 15, 8, 0, 25, 100, '2025-11-01')"#,
        )
        .execute(&pool)
        .await
        .expect("Should insert backdated snapshot");

        sqlx::query(
            r#"INSERT INTO activity_cache
               (user_id, data_type, data_json, fetched_at, expires_at)
               VALUES (1, 'previous_github_stats',
                       '{"totalPrsMerged":12,"totalIssuesClosed":8}',
                       '2025-11-30T00:00:00Z', '2125-11-30T00:00:00Z')"#,
        )
        .execute(&pool)
        .await
        .expect("Should insert legacy KV");

        sqlx::query(
            r#"
            UPDATE github_stats_snapshots
            SET total_prs_merged = COALESCE(
                    (SELECT CAST(json_extract(ac.data_json, '$.totalPrsMerged') AS INTEGER)
                     FROM activity_cache ac
                     WHERE ac.user_id = github_stats_snapshots.user_id
                       AND ac.data_type = 'previous_github_stats'),
                    total_prs_merged
                ),
                total_issues_closed = COALESCE(
                    (SELECT CAST(json_extract(ac.data_json, '$.totalIssuesClosed') AS INTEGER)
                     FROM activity_cache ac
                     WHERE ac.user_id = github_stats_snapshots.user_id
                       AND ac.data_type = 'previous_github_stats'),
                    total_issues_closed
                )
            WHERE snapshot_date = (
                SELECT MAX(s2.snapshot_date)
                FROM github_stats_snapshots s2
                WHERE s2.user_id = github_stats_snapshots.user_id
            );
            "#,
        )
        .execute(&pool)
        .await
        .expect("Backfill UPDATE should succeed");

        let (today_pm, today_ic): (i32, i32) = sqlx::query_as(
            "SELECT total_prs_merged, total_issues_closed FROM github_stats_snapshots WHERE user_id = 1 AND snapshot_date = '2025-11-30'"
        )
        .fetch_one(&pool)
        .await
        .expect("Should query today snapshot");

        let (back_pm, back_ic): (i32, i32) = sqlx::query_as(
            "SELECT total_prs_merged, total_issues_closed FROM github_stats_snapshots WHERE user_id = 1 AND snapshot_date = '2025-11-01'"
        )
        .fetch_one(&pool)
        .await
        .expect("Should query backdated snapshot");

        assert_eq!(today_pm, 12, "Latest-by-date row must be backfilled");
        assert_eq!(today_ic, 8, "Latest-by-date row must be backfilled");
        assert_eq!(back_pm, 0, "Backdated row must remain untouched");
        assert_eq!(back_ic, 0, "Backdated row must remain untouched");
    }

    /// Issue #189 / migration v12: when a user has a `previous_github_stats`
    /// KV but no snapshot row at all (possible if a prior snapshot save
    /// failed and the error was swallowed), the migration must seed a
    /// fallback snapshot from the KV's JSON before deleting the KV —
    /// otherwise that user loses their only XP baseline and the first
    /// post-migration sync re-awards lifetime XP.
    #[tokio::test]
    async fn test_migration_v12_seeds_snapshot_when_only_kv_exists() {
        let pool = create_test_pool().await;
        run_migrations(&pool)
            .await
            .expect("Migrations should run cleanly");

        sqlx::query(
            "INSERT INTO users (github_id, username, access_token_encrypted) VALUES (?, ?, ?)",
        )
        .bind(99999_i64)
        .bind("kvonly")
        .bind("encrypted")
        .execute(&pool)
        .await
        .expect("Should insert user");

        sqlx::query(
            r#"INSERT INTO activity_cache
               (user_id, data_type, data_json, fetched_at, expires_at)
               VALUES (1, 'previous_github_stats',
                       '{"totalCommits":50,"totalPrs":7,"totalPrsMerged":4,"totalReviews":11,"totalIssues":3,"totalIssuesClosed":2,"totalStarsReceived":18,"totalContributions":80}',
                       '2025-11-29T12:00:00Z', '2125-11-29T12:00:00Z')"#,
        )
        .execute(&pool)
        .await
        .expect("Should insert legacy KV");

        // Re-run the v12 seed step (idempotent: NOT EXISTS guards a re-insert).
        sqlx::query(
            r#"
            INSERT INTO github_stats_snapshots (
                user_id, total_commits, total_prs, total_prs_merged, total_reviews,
                total_issues, total_issues_closed, total_stars_received,
                total_contributions, snapshot_date
            )
            SELECT
                ac.user_id,
                COALESCE(CAST(json_extract(ac.data_json, '$.totalCommits')        AS INTEGER), 0),
                COALESCE(CAST(json_extract(ac.data_json, '$.totalPrs')            AS INTEGER), 0),
                COALESCE(CAST(json_extract(ac.data_json, '$.totalPrsMerged')      AS INTEGER), 0),
                COALESCE(CAST(json_extract(ac.data_json, '$.totalReviews')        AS INTEGER), 0),
                COALESCE(CAST(json_extract(ac.data_json, '$.totalIssues')         AS INTEGER), 0),
                COALESCE(CAST(json_extract(ac.data_json, '$.totalIssuesClosed')   AS INTEGER), 0),
                COALESCE(CAST(json_extract(ac.data_json, '$.totalStarsReceived')  AS INTEGER), 0),
                COALESCE(CAST(json_extract(ac.data_json, '$.totalContributions')  AS INTEGER), 0),
                DATE(ac.fetched_at)
            FROM activity_cache ac
            WHERE ac.data_type = 'previous_github_stats'
              AND NOT EXISTS (
                  SELECT 1 FROM github_stats_snapshots s WHERE s.user_id = ac.user_id
              );
            "#,
        )
        .execute(&pool)
        .await
        .expect("Seed INSERT should succeed");

        let row: (i32, i32, i32, i32, i32, i32, i32, i32, String) = sqlx::query_as(
            r#"SELECT total_commits, total_prs, total_prs_merged, total_reviews,
                      total_issues, total_issues_closed, total_stars_received,
                      total_contributions, snapshot_date
               FROM github_stats_snapshots WHERE user_id = 1"#,
        )
        .fetch_one(&pool)
        .await
        .expect("Seeded snapshot should exist");

        assert_eq!(row.0, 50);
        assert_eq!(row.1, 7);
        assert_eq!(row.2, 4);
        assert_eq!(row.3, 11);
        assert_eq!(row.4, 3);
        assert_eq!(row.5, 2);
        assert_eq!(row.6, 18);
        assert_eq!(row.7, 80);
        assert_eq!(
            row.8, "2025-11-29",
            "snapshot_date must come from DATE(fetched_at)"
        );
    }

    #[tokio::test]
    async fn test_migrations_are_idempotent() {
        let pool = create_test_pool().await;

        // Run migrations twice
        run_migrations(&pool)
            .await
            .expect("First run should succeed");
        run_migrations(&pool)
            .await
            .expect("Second run should also succeed");

        // Check migration count
        let count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM _migrations")
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
