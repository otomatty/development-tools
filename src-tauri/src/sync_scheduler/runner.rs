//! Sync Scheduler runner.
//!
//! Spawns a long-running task that drives the GitHub stats sync based on the
//! user's settings. The decision logic itself is pure (see [`super::actions`]);
//! this file is the side-effect layer that fetches inputs, executes the sync,
//! and persists progress.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use tauri::{AppHandle, Manager};
use tokio::sync::{Notify, RwLock};

use crate::commands::auth::AppState;
use crate::commands::github::run_github_sync;
use crate::database::models::code_stats::SyncMetadata;
use crate::database::models::UserSettings;
use crate::github::client::GitHubError;

use super::actions::{decide_action, next_sync_at};
use super::state::{
    skip_reasons, SchedulerAction, SchedulerInputs, SchedulerStatus, GITHUB_STATS_SYNC_TYPE,
};

/// Handle returned by [`start_scheduler`] and stored in Tauri's managed state.
///
/// Used by the settings command to wake the loop after a config change, and
/// by `get_scheduler_status` to surface live status to the UI.
#[derive(Clone)]
pub struct SyncSchedulerHandle {
    notify: Arc<Notify>,
    status: Arc<RwLock<SchedulerStatus>>,
}

impl SyncSchedulerHandle {
    /// Wake the scheduler loop so it re-reads settings immediately.
    pub fn notify_config_changed(&self) {
        self.notify.notify_one();
    }

    /// Snapshot of the current scheduler status.
    pub async fn status(&self) -> SchedulerStatus {
        self.status.read().await.clone()
    }
}

/// Spawn the scheduler loop. Idempotent at the call-site: `lib.rs` should call
/// this exactly once during `setup`.
pub fn start_scheduler(app: AppHandle) -> SyncSchedulerHandle {
    let notify = Arc::new(Notify::new());
    let status = Arc::new(RwLock::new(SchedulerStatus::default()));

    let handle = SyncSchedulerHandle {
        notify: notify.clone(),
        status: status.clone(),
    };

    tauri::async_runtime::spawn(async move {
        // TODO: [INFRA] logクレートに置換（ログ基盤整備時に一括対応）
        eprintln!("Scheduler: starting sync scheduler loop");
        run_loop(app, notify, status).await;
        eprintln!("Scheduler: sync scheduler loop exited");
    });

    handle
}

async fn run_loop(app: AppHandle, notify: Arc<Notify>, status: Arc<RwLock<SchedulerStatus>>) {
    let mut is_first_run = true;

    loop {
        let state = app.state::<AppState>();

        // Wait for login if there is no current user.
        let user = match state.token_manager.get_current_user().await {
            Ok(Some(u)) => u,
            Ok(None) => {
                set_status_logged_out(&status).await;
                wait_for_change_or_timeout(&notify, 60).await;
                continue;
            }
            Err(e) => {
                eprintln!("Scheduler: failed to read current user: {}", e);
                wait_for_change_or_timeout(&notify, 60).await;
                continue;
            }
        };

        let settings = match state.db.get_or_create_user_settings(user.id).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Scheduler: failed to load user settings: {}", e);
                wait_for_change_or_timeout(&notify, 60).await;
                continue;
            }
        };

        let metadata = state
            .db
            .get_sync_metadata(user.id, GITHUB_STATS_SYNC_TYPE)
            .await
            .ok()
            .flatten();

        let now = Utc::now();
        let inputs = build_inputs(&settings, metadata.as_ref(), is_first_run, now);
        let projected_next = next_sync_at(&inputs);

        write_status(&status, &settings, metadata.as_ref(), projected_next, true).await;

        let action = decide_action(&inputs);
        match action {
            SchedulerAction::RunSync => {
                is_first_run = false;
                eprintln!("Scheduler: running scheduled sync for user {}", user.id);

                match run_github_sync(&app, state.inner()).await {
                    Ok(_) => {
                        // Post-sync metadata is persisted inside `run_github_sync`
                        // so manual and scheduled flows stay in sync. Nothing to
                        // do here besides loop back; the next iteration sees the
                        // fresh `last_sync_at` and computes a Sleep action.
                    }
                    Err(err_msg) => {
                        eprintln!("Scheduler: scheduled sync failed: {}", err_msg);
                        let now = Utc::now();
                        let mut sleep_secs = MIN_FAILURE_SLEEP_SECONDS;

                        if let Some(reset_at) = parse_rate_limit_reset(&err_msg) {
                            // Persist the reset time so subsequent decisions
                            // return RateLimited until it passes.
                            let _ = state
                                .db
                                .record_sync_rate_limit(user.id, GITHUB_STATS_SYNC_TYPE, reset_at)
                                .await;
                            let _ = state
                                .db
                                .record_sync_skipped(
                                    user.id,
                                    GITHUB_STATS_SYNC_TYPE,
                                    skip_reasons::RATE_LIMITED,
                                    now,
                                )
                                .await;
                            update_status_skipped(&status, skip_reasons::RATE_LIMITED, now).await;
                            // Sleep until the reset, but never less than the
                            // failure floor. clamp_failure_sleep handles
                            // already-passed resets.
                            sleep_secs = seconds_until(reset_at, now);
                        } else if classify_rate_limited(&err_msg) {
                            // Rate-limited but the reset timestamp couldn't be
                            // parsed — record the reason and back off by the
                            // failure floor so we don't hammer the API.
                            let _ = state
                                .db
                                .record_sync_skipped(
                                    user.id,
                                    GITHUB_STATS_SYNC_TYPE,
                                    skip_reasons::RATE_LIMITED,
                                    now,
                                )
                                .await;
                            update_status_skipped(&status, skip_reasons::RATE_LIMITED, now).await;
                        }

                        // Always sleep at least MIN_FAILURE_SLEEP_SECONDS after a
                        // failure so transient errors don't trigger a tight
                        // retry loop.
                        wait_for_change_or_timeout(&notify, sleep_secs).await;
                    }
                }
            }
            SchedulerAction::Sleep { seconds } => {
                is_first_run = false;
                wait_for_change_or_timeout(&notify, seconds).await;
            }
            SchedulerAction::Idle { reason } => {
                is_first_run = false;
                let now = Utc::now();
                let _ = state
                    .db
                    .record_sync_skipped(user.id, GITHUB_STATS_SYNC_TYPE, reason, now)
                    .await;
                // Surface the just-written skip reason on the in-memory
                // SchedulerStatus so the UI sees it without waiting for the
                // next loop iteration.
                update_status_skipped(&status, reason, now).await;
                // Wake on settings changes immediately, but also re-poll on a
                // bounded interval as a safety net for non-settings transitions
                // (logout/login, an account switch, etc.) that don't currently
                // emit a notify.
                wait_for_change_or_timeout(&notify, IDLE_POLL_SECONDS).await;
            }
            SchedulerAction::RateLimited { reason, seconds } => {
                is_first_run = false;
                let now = Utc::now();
                let _ = state
                    .db
                    .record_sync_skipped(user.id, GITHUB_STATS_SYNC_TYPE, reason, now)
                    .await;
                update_status_skipped(&status, reason, now).await;
                wait_for_change_or_timeout(&notify, seconds).await;
            }
        }
    }
}

/// Minimum back-off between sync failures. Prevents a tight retry loop when
/// the GitHub API returns errors without parseable rate-limit metadata.
const MIN_FAILURE_SLEEP_SECONDS: u64 = 60;

/// Cap on how long the runner will sleep after a rate-limit failure even if
/// the reset is far in the future. Bounded so a config change can break out.
const MAX_FAILURE_SLEEP_SECONDS: u64 = 30 * 60;

/// Maximum time the loop stays parked in the `Idle` state before re-polling.
///
/// The notify channel covers settings updates, but other transitions
/// (logout/login, account switch, manual sync from the UI) currently don't
/// emit a notify. This bounded wait makes the loop self-healing — it'll
/// observe such state changes within at most this many seconds.
const IDLE_POLL_SECONDS: u64 = 5 * 60;

fn seconds_until(target: DateTime<Utc>, now: DateTime<Utc>) -> u64 {
    let diff = target.signed_duration_since(now).num_seconds();
    let secs = if diff < 0 { 0 } else { diff as u64 };
    secs.clamp(MIN_FAILURE_SLEEP_SECONDS, MAX_FAILURE_SLEEP_SECONDS)
}

/// Parse the `Resets at <unix_ts>` suffix produced by
/// [`GitHubError::RateLimited`]'s Display impl.
///
/// The error string travels through `.map_err(|e| e.to_string())` before it
/// reaches the runner, so we extract the timestamp by string match rather than
/// by destructuring the typed error.
fn parse_rate_limit_reset(err_msg: &str) -> Option<DateTime<Utc>> {
    const NEEDLE: &str = "Resets at ";
    let idx = err_msg.find(NEEDLE)?;
    let tail = &err_msg[idx + NEEDLE.len()..];
    // Take leading digits (and optional minus) only.
    let end = tail
        .find(|c: char| !c.is_ascii_digit() && c != '-')
        .unwrap_or(tail.len());
    let ts: i64 = tail[..end].parse().ok()?;
    DateTime::from_timestamp(ts, 0)
}

/// Update only the skip-related fields of the in-memory status so the UI sees
/// the new skip without waiting for the next loop iteration.
async fn update_status_skipped(
    status: &RwLock<SchedulerStatus>,
    reason: &str,
    when: DateTime<Utc>,
) {
    let mut s = status.write().await;
    s.last_skipped_reason = Some(reason.to_string());
    s.last_skipped_at = Some(when.to_rfc3339());
}

fn build_inputs(
    settings: &UserSettings,
    metadata: Option<&SyncMetadata>,
    is_first_run: bool,
    now: DateTime<Utc>,
) -> SchedulerInputs {
    SchedulerInputs {
        sync_on_startup: settings.sync_on_startup,
        sync_interval_minutes: settings.sync_interval_minutes,
        background_sync: settings.background_sync,
        last_sync_at: metadata.and_then(|m| m.last_sync_at_parsed()),
        rate_limit_remaining: metadata.and_then(|m| m.rate_limit_remaining),
        rate_limit_reset_at: metadata.and_then(|m| m.rate_limit_reset_at_parsed()),
        is_first_run,
        now,
    }
}

async fn write_status(
    status: &RwLock<SchedulerStatus>,
    settings: &UserSettings,
    metadata: Option<&SyncMetadata>,
    next_sync: Option<DateTime<Utc>>,
    running: bool,
) {
    let mut s = status.write().await;
    s.running = running;
    s.background_sync_enabled = settings.background_sync;
    s.interval_minutes = settings.sync_interval_minutes;
    s.sync_on_startup = settings.sync_on_startup;
    s.last_sync_at = metadata.and_then(|m| m.last_sync_at.clone());
    s.next_sync_at = next_sync.map(|t| t.to_rfc3339());
    s.last_skipped_at = metadata.and_then(|m| m.last_skipped_at.clone());
    s.last_skipped_reason = metadata.and_then(|m| m.last_skipped_reason.clone());
}

async fn set_status_logged_out(status: &RwLock<SchedulerStatus>) {
    let mut s = status.write().await;
    s.running = true;
    s.last_skipped_reason = Some(skip_reasons::NOT_LOGGED_IN.to_string());
    s.next_sync_at = None;
}

async fn wait_for_change_or_timeout(notify: &Notify, seconds: u64) {
    use std::time::Duration;
    tokio::select! {
        _ = notify.notified() => {}
        _ = tokio::time::sleep(Duration::from_secs(seconds)) => {}
    }
}

/// Classify a `run_github_sync` error string as rate-limited.
///
/// `run_github_sync` stringifies its underlying error so we have to match on
/// the formatted message. [`GitHubError::RateLimited`] is `Display`'d as
/// `"Rate limit exceeded. Resets at <ts>"`.
fn classify_rate_limited(err_msg: &str) -> bool {
    // Belt and suspenders: also accept "rate limit" (lowercase, any wording).
    let lower = err_msg.to_lowercase();
    lower.contains("rate limit")
        || lower.contains(&format!("{}", GitHubError::RateLimited(0)).to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_rate_limited_recognizes_github_error() {
        let msg = format!("{}", GitHubError::RateLimited(1_700_000_000));
        assert!(classify_rate_limited(&msg));
    }

    #[test]
    fn classify_rate_limited_recognizes_lowercased() {
        assert!(classify_rate_limited("rate limit reached"));
        assert!(classify_rate_limited("Rate Limit hit at endpoint X"));
    }

    #[test]
    fn classify_rate_limited_rejects_unrelated() {
        assert!(!classify_rate_limited("unauthorized"));
        assert!(!classify_rate_limited("network error"));
    }

    #[test]
    fn parse_rate_limit_reset_extracts_timestamp() {
        let msg = format!("{}", GitHubError::RateLimited(1_700_000_000));
        let parsed = parse_rate_limit_reset(&msg).expect("should parse timestamp");
        assert_eq!(parsed.timestamp(), 1_700_000_000);
    }

    #[test]
    fn parse_rate_limit_reset_handles_trailing_text() {
        let msg = "Rate limit exceeded. Resets at 1700000000 (some context)";
        let parsed = parse_rate_limit_reset(msg).expect("should parse timestamp");
        assert_eq!(parsed.timestamp(), 1_700_000_000);
    }

    #[test]
    fn parse_rate_limit_reset_returns_none_for_unrelated() {
        assert!(parse_rate_limit_reset("network error").is_none());
        assert!(parse_rate_limit_reset("Rate limit").is_none());
    }

    #[test]
    fn seconds_until_clamps_past_target_to_min() {
        let now = Utc::now();
        let past = now - chrono::Duration::seconds(120);
        assert_eq!(seconds_until(past, now), MIN_FAILURE_SLEEP_SECONDS);
    }

    #[test]
    fn seconds_until_clamps_far_future_to_max() {
        let now = Utc::now();
        let far = now + chrono::Duration::hours(2);
        assert_eq!(seconds_until(far, now), MAX_FAILURE_SLEEP_SECONDS);
    }

    #[test]
    fn seconds_until_passes_through_in_range() {
        let now = Utc::now();
        let future = now + chrono::Duration::seconds(300);
        assert_eq!(seconds_until(future, now), 300);
    }
}
