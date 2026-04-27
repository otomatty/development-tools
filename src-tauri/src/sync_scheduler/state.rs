//! Sync Scheduler state types.
//!
//! Pure data types shared between the decision logic in [`super::actions`] and
//! the side-effect runner in [`super::runner`]. No I/O lives here.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Sync type key persisted in `sync_metadata.sync_type`.
///
/// The scheduler currently drives only the GitHub stats sync, but the type is
/// kept as a `&str` constant so future sync types can be added without
/// touching the runner's plumbing.
pub const GITHUB_STATS_SYNC_TYPE: &str = "github_stats";

/// Reason strings persisted to `sync_metadata.last_skipped_reason`.
pub mod skip_reasons {
    /// `background_sync` is OFF and the user is not actively syncing.
    pub const BACKGROUND_DISABLED: &str = "background_sync_disabled";
    /// `sync_interval_minutes` is 0 (manual only).
    pub const MANUAL_ONLY: &str = "manual_only";
    /// API rate limit was hit.
    pub const RATE_LIMITED: &str = "rate_limited";
    /// User is not logged in (no GitHub token available).
    pub const NOT_LOGGED_IN: &str = "not_logged_in";
}

/// User-visible scheduler status surfaced to the UI.
///
/// Mirrors the camelCase shape consumed by the SyncSettings React component.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulerStatus {
    /// Whether the scheduler loop is currently running (i.e. spawned).
    pub running: bool,
    /// Whether the user has enabled background sync.
    pub background_sync_enabled: bool,
    /// Currently configured interval in minutes (0 = manual only).
    pub interval_minutes: i32,
    /// Whether sync_on_startup is configured.
    pub sync_on_startup: bool,
    /// Last successful sync time (RFC3339).
    pub last_sync_at: Option<String>,
    /// Predicted next sync time (RFC3339), if any.
    pub next_sync_at: Option<String>,
    /// Last time a sync was skipped (RFC3339).
    pub last_skipped_at: Option<String>,
    /// Reason a sync was skipped (one of [`skip_reasons`]).
    pub last_skipped_reason: Option<String>,
}

/// Snapshot of the inputs needed to make a scheduling decision.
///
/// Built by the runner each iteration from `user_settings`, `sync_metadata`,
/// and the current clock. Keeping this pure makes the decision logic
/// trivially unit-testable.
#[derive(Debug, Clone)]
pub struct SchedulerInputs {
    pub sync_on_startup: bool,
    pub sync_interval_minutes: i32,
    pub background_sync: bool,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub rate_limit_remaining: Option<i32>,
    pub rate_limit_reset_at: Option<DateTime<Utc>>,
    /// True iff this is the first decision since the scheduler started.
    pub is_first_run: bool,
    pub now: DateTime<Utc>,
}

/// What the scheduler should do next.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchedulerAction {
    /// Run the sync immediately.
    RunSync,
    /// Sleep for `seconds`, then re-evaluate.
    Sleep { seconds: u64 },
    /// Background sync is disabled — wait until config changes.
    Idle { reason: &'static str },
    /// Rate-limited; sleep for `seconds` (computed from the reset timestamp
    /// when known, or a back-off floor when not).
    RateLimited { reason: &'static str, seconds: u64 },
}

/// Hard floor on how long the loop sleeps between checks.
///
/// Even if the user picks a 5-minute interval the loop must wake up at least
/// this often so that `update_settings` can re-arm the next sync without
/// waiting for the previous timer to expire (the runner uses a notify channel
/// to wake earlier, but this is the safety net).
pub const MIN_SLEEP_SECONDS: u64 = 30;

/// Cap on a single sleep slice. Keeping it bounded means a config change is
/// observed within at most this many seconds even if the notify is missed.
pub const MAX_SLEEP_SECONDS: u64 = 5 * 60;
