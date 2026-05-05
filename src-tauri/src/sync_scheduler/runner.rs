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

use crate::auth::{classify_unauthorized, handle_unauthorized, reasons};
use crate::commands::auth::AppState;
use crate::commands::github::run_github_sync;
use crate::commands::notifications::{
    run_notifications_sync, NotificationsSyncOutcome, GITHUB_NOTIFICATIONS_SYNC_TYPE,
};
use crate::database::models::code_stats::SyncMetadata;
use crate::database::models::UserSettings;
use crate::github::client::GitHubError;

use super::actions::{decide_action, next_sync_at};
use super::state::{
    skip_reasons, SchedulerAction, SchedulerInputs, SchedulerStatus, GITHUB_STATS_SYNC_TYPE,
    MAX_SLEEP_SECONDS,
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
    // Volatile fallback for the rate-limit reset time when persisting it to
    // `sync_metadata` fails. Without this, a transient DB error would cause
    // the next iteration to lose the rate-limit context and risk hitting the
    // GitHub API again before the reset.
    let mut rate_limit_reset_fallback: Option<DateTime<Utc>> = None;
    // In-memory floor for the next notifications poll, scoped to the
    // user that produced the hint. Tracks GitHub's `x-poll-interval`
    // header (and rate-limit resets) so we don't poll notifications
    // more aggressively than GitHub asks. Keyed by user.id so an
    // account switch doesn't carry the previous user's backoff onto
    // the new session — their `sync_metadata` row holds any persistent
    // rate-limit state.
    let mut notifications_next_allowed: Option<(i64, DateTime<Utc>)> = None;

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

        // Treat a DB error here as a hard failure rather than silently
        // falling back to None — without this guard a transient SQLite issue
        // would zero out `last_sync_at` and the next decision would
        // immediately RunSync again until recovery.
        let metadata = match state
            .db
            .get_sync_metadata(user.id, GITHUB_STATS_SYNC_TYPE)
            .await
        {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Scheduler: failed to read sync_metadata: {}", e);
                wait_for_change_or_timeout(&notify, MIN_FAILURE_SLEEP_SECONDS).await;
                continue;
            }
        };

        let now = Utc::now();
        // Drop the fallback once its reset has passed.
        if let Some(reset) = rate_limit_reset_fallback {
            if reset <= now {
                rate_limit_reset_fallback = None;
            }
        }
        let inputs = build_inputs(
            &settings,
            metadata.as_ref(),
            rate_limit_reset_fallback,
            is_first_run,
            now,
        );
        let projected_next = next_sync_at(&inputs);

        write_status(&status, &settings, metadata.as_ref(), projected_next, true).await;

        // Poll GitHub Notifications on its own cadence (Issue #186).
        // Independent of the stats `SchedulerAction` so a stats Sleep /
        // Idle / RateLimited doesn't freeze the inbox. The endpoint is
        // conditional via ETag (304 = zero rate budget) and the call is
        // self-throttled via `notifications_due_to_poll`, which honours
        // both GitHub's `x-poll-interval` hint and any persisted
        // rate-limit reset.
        //
        // Honour `background_sync = false` here too: the contract is
        // "background API activity halts when the user opts out", and
        // notifications are pure background activity (the user can still
        // refresh manually via the bell button which calls
        // `get_notifications` directly).
        // Only the in-memory floor for the current user applies; an
        // account switch invalidates the previous user's backoff window
        // (their persisted `sync_metadata.rate_limit_reset_at` is per-user
        // and remains the source of truth across sessions).
        let next_allowed_for_user = notifications_next_allowed
            .filter(|(uid, _)| *uid == user.id)
            .map(|(_, at)| at);
        if settings.background_sync
            && notifications_due_to_poll(state.inner(), user.id, next_allowed_for_user, now).await
        {
            match run_notifications_sync(&app, state.inner()).await {
                Ok(NotificationsSyncOutcome::Ok {
                    poll_interval_seconds,
                }) => {
                    notifications_next_allowed = poll_interval_seconds
                        .map(|secs| (user.id, Utc::now() + chrono::Duration::seconds(secs as i64)));
                }
                Ok(NotificationsSyncOutcome::RateLimited { reset_at }) => {
                    eprintln!(
                        "Scheduler: notifications API rate-limited until {}",
                        reset_at.to_rfc3339()
                    );
                    notifications_next_allowed = Some((user.id, reset_at));
                }
                Err(e) => {
                    eprintln!("Scheduler: notifications sync failed: {}", e);
                }
            }
        }

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
                        // A successful sync invalidates any in-memory rate-limit
                        // fallback we may have been carrying.
                        rate_limit_reset_fallback = None;
                    }
                    Err(err_msg) => {
                        eprintln!("Scheduler: scheduled sync failed: {}", err_msg);
                        let now = Utc::now();
                        let mut sleep_secs = MIN_FAILURE_SLEEP_SECONDS;

                        // 401 detection: a revoked / invalidated token would
                        // otherwise loop forever in MIN_FAILURE_SLEEP_SECONDS
                        // increments because the failure isn't classified as
                        // rate-limit. `run_github_sync` may have already
                        // emitted `auth-expired` from inside the user-facing
                        // command path, but the scheduler runs independently
                        // (no `app` was wired through `map_github_result` for
                        // every internal call), so re-trigger here. The
                        // emitter is idempotent on the frontend side.
                        if classify_unauthorized(&err_msg) {
                            handle_unauthorized(
                                &app,
                                state.inner(),
                                reasons::SCHEDULER_UNAUTHORIZED,
                            )
                            .await;
                            // Skip straight to the logged-out wait branch on
                            // the next iteration — the user is now signed out.
                            wait_for_change_or_timeout(&notify, MIN_FAILURE_SLEEP_SECONDS).await;
                            continue;
                        }

                        if let Some(reset_at) = parse_rate_limit_reset(&err_msg) {
                            // Persist the reset time so subsequent decisions
                            // return RateLimited until it passes. These writes
                            // are correctness-critical: silently dropping them
                            // would let the next iteration hit the API
                            // immediately and burn through whatever budget the
                            // reset is supposed to wait out.
                            match state
                                .db
                                .record_sync_rate_limit(user.id, GITHUB_STATS_SYNC_TYPE, reset_at)
                                .await
                            {
                                Ok(()) => {
                                    // DB has the canonical value now; the
                                    // in-memory fallback is no longer needed.
                                    rate_limit_reset_fallback = None;
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Scheduler: record_sync_rate_limit failed: {} \
                                         (using in-memory fallback)",
                                        e
                                    );
                                    rate_limit_reset_fallback = Some(reset_at);
                                }
                            }
                            log_db_err(
                                "record_sync_skipped (rate_limited)",
                                state
                                    .db
                                    .record_sync_skipped(
                                        user.id,
                                        GITHUB_STATS_SYNC_TYPE,
                                        skip_reasons::RATE_LIMITED,
                                        now,
                                    )
                                    .await,
                            );
                            update_status_skipped(&status, skip_reasons::RATE_LIMITED, now).await;
                            // Sleep until the reset, but never less than the
                            // failure floor. seconds_until handles already-
                            // passed resets via the MIN clamp.
                            sleep_secs = seconds_until(reset_at, now);
                            // Patch the cached next_sync_at so the UI doesn't
                            // keep showing the pre-failure projection (which
                            // is `last + interval`, often already past).
                            set_next_sync_at(
                                &status,
                                now + chrono::Duration::seconds(sleep_secs as i64),
                            )
                            .await;
                        } else if classify_rate_limited(&err_msg) {
                            // Rate-limited but the reset timestamp couldn't be
                            // parsed — record the reason and back off by the
                            // failure floor so we don't hammer the API.
                            log_db_err(
                                "record_sync_skipped (rate_limited, no reset)",
                                state
                                    .db
                                    .record_sync_skipped(
                                        user.id,
                                        GITHUB_STATS_SYNC_TYPE,
                                        skip_reasons::RATE_LIMITED,
                                        now,
                                    )
                                    .await,
                            );
                            update_status_skipped(&status, skip_reasons::RATE_LIMITED, now).await;
                            set_next_sync_at(
                                &status,
                                now + chrono::Duration::seconds(sleep_secs as i64),
                            )
                            .await;
                        }

                        // Always sleep at least MIN_FAILURE_SLEEP_SECONDS after a
                        // failure so transient errors don't trigger a tight
                        // retry loop. Capped by `cap_sleep_for_notifications`
                        // so a long stats backoff (rate-limit reset can be
                        // 30 min away) doesn't hold up an otherwise-healthy
                        // notifications stream.
                        let capped = cap_sleep_for_notifications(
                            sleep_secs,
                            notifications_next_allowed.as_ref(),
                            user.id,
                            Utc::now(),
                        );
                        wait_for_change_or_timeout(&notify, capped).await;
                    }
                }
            }
            SchedulerAction::Sleep { seconds } => {
                // First-run users with `sync_on_startup=false` and no history
                // would otherwise hit the `last_sync_at=None → RunSync` branch
                // on the *next* iteration (because `is_first_run` is flipped
                // off here) and auto-sync after only MAX_SLEEP_SECONDS instead
                // of the configured interval. Persist a synthetic baseline so
                // the elapsed-time check works correctly from now on.
                //
                // Guard against re-persisting on every Sleep: if a baseline
                // already exists, leave it alone — overwriting it would reset
                // the interval countdown on every iteration and the first
                // auto-sync would never fire.
                let needs_baseline = metadata
                    .as_ref()
                    .and_then(|m| m.last_sync_at_parsed())
                    .is_none()
                    && metadata
                        .as_ref()
                        .and_then(|m| m.scheduler_baseline_at_parsed())
                        .is_none()
                    && !settings.sync_on_startup;
                if needs_baseline {
                    persist_startup_baseline(&state.db, user.id).await;
                }
                is_first_run = false;
                let capped = cap_sleep_for_notifications(
                    seconds,
                    notifications_next_allowed.as_ref(),
                    user.id,
                    Utc::now(),
                );
                wait_for_change_or_timeout(&notify, capped).await;
            }
            SchedulerAction::Idle { reason } => {
                is_first_run = false;
                let now = Utc::now();
                log_db_err(
                    "record_sync_skipped (idle)",
                    state
                        .db
                        .record_sync_skipped(user.id, GITHUB_STATS_SYNC_TYPE, reason, now)
                        .await,
                );
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
                log_db_err(
                    "record_sync_skipped (rate_limited)",
                    state
                        .db
                        .record_sync_skipped(user.id, GITHUB_STATS_SYNC_TYPE, reason, now)
                        .await,
                );
                update_status_skipped(&status, reason, now).await;
                // Cap the rate-limit backoff so the notifications stream
                // doesn't get starved during a stats-side rate limit.
                let capped = cap_sleep_for_notifications(
                    seconds,
                    notifications_next_allowed.as_ref(),
                    user.id,
                    Utc::now(),
                );
                wait_for_change_or_timeout(&notify, capped).await;
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

/// Override the in-memory `next_sync_at` after a rate-limit failure.
///
/// The pre-action `write_status` call already populated `next_sync_at` from
/// `next_sync_at(&inputs)`, but that prediction was computed before the
/// failure was known. During a rate-limit backoff the runner actually wakes
/// at `reset_at`, so we patch the cached status here so `get_scheduler_status`
/// stops reporting the stale (often already-past) projection.
async fn set_next_sync_at(status: &RwLock<SchedulerStatus>, next: DateTime<Utc>) {
    let mut s = status.write().await;
    s.next_sync_at = Some(next.to_rfc3339());
}

fn build_inputs(
    settings: &UserSettings,
    metadata: Option<&SyncMetadata>,
    rate_limit_reset_fallback: Option<DateTime<Utc>>,
    is_first_run: bool,
    now: DateTime<Utc>,
) -> SchedulerInputs {
    // Prefer the *later* future reset between DB and the in-memory
    // fallback. A stale (already-past) DB reset must not shadow a future
    // fallback — otherwise the scheduler would resume hitting GitHub before
    // the real reset window after `record_sync_rate_limit` failed to
    // persist.
    let db_reset = metadata.and_then(|m| m.rate_limit_reset_at_parsed());
    let db_remaining = metadata.and_then(|m| m.rate_limit_remaining);
    let db_future = db_reset.filter(|d| *d > now);
    let fb_future = rate_limit_reset_fallback.filter(|f| *f > now);
    let (rate_limit_remaining, rate_limit_reset_at) = match (db_future, fb_future) {
        // Both future: pick the later (more conservative) reset. Whichever
        // wins, we use the matching `remaining`. If we picked the fallback,
        // we just got rate-limited so remaining is effectively 0.
        (Some(d), Some(f)) if d >= f => (db_remaining, Some(d)),
        (Some(_), Some(f)) => (Some(0), Some(f)),
        (Some(d), None) => (db_remaining, Some(d)),
        (None, Some(f)) => (Some(0), Some(f)),
        // Neither is a *future* reset: pass DB's raw values through. A stale
        // DB reset will be filtered later by `active_rate_limit_until`'s
        // `reset > now` check, so leaving it in place is harmless.
        (None, None) => (db_remaining, db_reset),
    };

    // Effective baseline for the interval calculation: prefer the real
    // last_sync_at, fall back to the synthesized scheduler_baseline_at when
    // the user opted out of startup sync. Keeping these as separate columns
    // means the UI never sees the synthetic timestamp as "last sync".
    let last_sync_at = metadata
        .and_then(|m| m.last_sync_at_parsed())
        .or_else(|| metadata.and_then(|m| m.scheduler_baseline_at_parsed()));

    SchedulerInputs {
        sync_on_startup: settings.sync_on_startup,
        sync_interval_minutes: settings.sync_interval_minutes,
        background_sync: settings.background_sync,
        last_sync_at,
        rate_limit_remaining,
        rate_limit_reset_at,
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
    // Full reset so a previous user's last_sync_at / interval_minutes /
    // skip-history doesn't leak across logouts or account switches. The only
    // logout-specific override is the skip reason so the UI can explain why
    // the scheduler is parked.
    *s = SchedulerStatus {
        running: true,
        last_skipped_reason: Some(skip_reasons::NOT_LOGGED_IN.to_string()),
        ..SchedulerStatus::default()
    };
}

/// Cap a stats-decided sleep so the notifications poll cadence isn't
/// starved when stats has a long backoff (rate-limit failure, etc.).
/// Without this, `MAX_FAILURE_SLEEP_SECONDS` (30 min) on the stats side
/// would freeze the inbox for that long even when the notifications
/// endpoint has plenty of budget. Returns the smaller of the intended
/// stats sleep and the time until notifications are next allowed to
/// poll (defaulting to `MAX_SLEEP_SECONDS` when there's no in-memory
/// hint, since that's already the loop's natural cap on responsiveness).
fn cap_sleep_for_notifications(
    intended: u64,
    notifications_next_allowed: Option<&(i64, DateTime<Utc>)>,
    user_id: i64,
    now: DateTime<Utc>,
) -> u64 {
    let notif_wake = match notifications_next_allowed {
        Some(&(uid, at)) if uid == user_id => (at - now).num_seconds().max(0) as u64,
        _ => MAX_SLEEP_SECONDS,
    };
    intended.min(notif_wake)
}

async fn wait_for_change_or_timeout(notify: &Notify, seconds: u64) {
    use std::time::Duration;
    tokio::select! {
        _ = notify.notified() => {}
        _ = tokio::time::sleep(Duration::from_secs(seconds)) => {}
    }
}

/// Log a `DbResult<()>` if it's an error, with `op` as the calling site.
///
/// Replaces a fleet of `let _ = state.db.foo().await` calls so transient DB
/// failures inside the scheduler are at least visible in the logs. We don't
/// propagate the error further because each call site has already decided
/// what to do regardless (sleep / continue) — the goal is solely
/// observability.
// TODO: [INFRA] logクレートに置換（ログ基盤整備時に一括対応）
fn log_db_err<T>(op: &str, result: crate::database::DbResult<T>) {
    if let Err(e) = result {
        eprintln!("Scheduler: {} failed: {}", op, e);
    }
}

/// Persist a synthetic interval baseline so the `sync_on_startup=false`
/// opt-out survives across iterations.
///
/// Without this, the second iteration would see `last_sync_at = None` again
/// (because no real sync has happened yet) and `decide_action` would fall
/// through to its catch-up branch — auto-syncing the user shortly after
/// startup, contradicting their explicit "don't sync on startup" choice.
///
/// Writes to `sync_metadata.scheduler_baseline_at` rather than `last_sync_at`
/// so the UI's "最終自動同期" doesn't misreport a phantom sync that never
/// happened. `build_inputs` falls back to this column when `last_sync_at`
/// is None.
async fn persist_startup_baseline(db: &crate::database::Database, user_id: i64) {
    log_db_err(
        "persist_startup_baseline",
        db.record_scheduler_baseline(user_id, GITHUB_STATS_SYNC_TYPE, Utc::now())
            .await,
    );
}

/// Check whether the notifications endpoint is currently rate-limited per
/// the persisted `sync_metadata` row.
///
/// `record_sync_rate_limit` writes the reset timestamp inside
/// `run_notifications_sync` when GitHub returns a 0-remaining 403; we
/// honour it here so the next tick doesn't immediately re-poll and burn
/// through the limit again. A read failure errs on "not throttled" — it's
/// safer to re-poll once than to silently freeze the inbox.
async fn notifications_throttled(state: &AppState, user_id: i64) -> bool {
    let metadata = match state
        .db
        .get_sync_metadata(user_id, GITHUB_NOTIFICATIONS_SYNC_TYPE)
        .await
    {
        Ok(m) => m,
        Err(e) => {
            eprintln!(
                "Scheduler: failed to read notifications sync_metadata: {}",
                e
            );
            return false;
        }
    };

    metadata
        .and_then(|m| m.rate_limit_reset_at_parsed())
        .is_some_and(|reset| reset > Utc::now())
}

/// Decide whether to fire a notifications poll on this scheduler tick.
///
/// Composes two throttles:
/// 1. Persisted rate-limit reset (`sync_metadata.rate_limit_reset_at`,
///    written by `run_notifications_sync` when GitHub returns 403 with
///    0 remaining) — survives across app restarts.
/// 2. In-memory `next_allowed_at` derived from GitHub's
///    `x-poll-interval` header — softer hint, not persisted because
///    GitHub re-emits it on every response.
///
/// Either being in the future blocks the poll. The persisted-rate-limit
/// branch logs a single line per skip; the soft poll-interval window
/// fires every short tick and is intentionally silent to avoid spam.
async fn notifications_due_to_poll(
    state: &AppState,
    user_id: i64,
    next_allowed_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> bool {
    if let Some(allowed) = next_allowed_at {
        if allowed > now {
            return false;
        }
    }
    if notifications_throttled(state, user_id).await {
        eprintln!("Scheduler: skipping notifications poll while rate-limited");
        return false;
    }
    true
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

    #[test]
    fn build_inputs_uses_db_when_db_reset_is_later() {
        // When DB has a future reset later than the fallback, the DB value
        // wins (more conservative wait).
        let now = Utc::now();
        let reset_db = now + chrono::Duration::minutes(60);
        let fallback = Some(now + chrono::Duration::minutes(10));
        let mut settings = UserSettings::default();
        settings.sync_interval_minutes = 60;
        settings.background_sync = true;
        settings.sync_on_startup = false;

        let metadata = SyncMetadata {
            id: 1,
            user_id: 1,
            sync_type: GITHUB_STATS_SYNC_TYPE.to_string(),
            last_sync_at: None,
            last_sync_cursor: None,
            etag: None,
            rate_limit_remaining: Some(123),
            rate_limit_reset_at: Some(reset_db.to_rfc3339()),
            scheduler_baseline_at: None,
            last_skipped_at: None,
            last_skipped_reason: None,
        };

        let inputs = build_inputs(&settings, Some(&metadata), fallback, false, now);
        // DB's `remaining` should pass through when DB wins.
        assert_eq!(inputs.rate_limit_remaining, Some(123));
        assert_eq!(
            inputs.rate_limit_reset_at.unwrap().timestamp(),
            reset_db.timestamp()
        );
    }

    #[test]
    fn build_inputs_uses_fallback_when_db_lacks_reset() {
        let now = Utc::now();
        let fallback_reset = now + chrono::Duration::minutes(10);
        let settings = UserSettings::default();

        let inputs = build_inputs(&settings, None, Some(fallback_reset), false, now);
        assert_eq!(inputs.rate_limit_remaining, Some(0));
        assert_eq!(
            inputs.rate_limit_reset_at.unwrap().timestamp(),
            fallback_reset.timestamp()
        );
    }

    #[test]
    fn build_inputs_uses_scheduler_baseline_when_no_real_sync() {
        let now = Utc::now();
        let baseline = now - chrono::Duration::minutes(20);
        let metadata = SyncMetadata {
            id: 1,
            user_id: 1,
            sync_type: GITHUB_STATS_SYNC_TYPE.to_string(),
            last_sync_at: None,
            last_sync_cursor: None,
            etag: None,
            rate_limit_remaining: None,
            rate_limit_reset_at: None,
            scheduler_baseline_at: Some(baseline.to_rfc3339()),
            last_skipped_at: None,
            last_skipped_reason: None,
        };
        let settings = UserSettings::default();

        let inputs = build_inputs(&settings, Some(&metadata), None, false, now);
        // last_sync_at falls back to the synthetic baseline so decide_action
        // sees a real elapsed time.
        assert_eq!(
            inputs.last_sync_at.unwrap().timestamp(),
            baseline.timestamp()
        );
    }

    #[test]
    fn build_inputs_prefers_real_last_sync_over_baseline() {
        let now = Utc::now();
        let real_last = now - chrono::Duration::minutes(5);
        let stale_baseline = now - chrono::Duration::days(3);
        let metadata = SyncMetadata {
            id: 1,
            user_id: 1,
            sync_type: GITHUB_STATS_SYNC_TYPE.to_string(),
            last_sync_at: Some(real_last.to_rfc3339()),
            last_sync_cursor: None,
            etag: None,
            rate_limit_remaining: None,
            rate_limit_reset_at: None,
            scheduler_baseline_at: Some(stale_baseline.to_rfc3339()),
            last_skipped_at: None,
            last_skipped_reason: None,
        };
        let settings = UserSettings::default();

        let inputs = build_inputs(&settings, Some(&metadata), None, false, now);
        // Real last_sync_at wins over the synthetic baseline.
        assert_eq!(
            inputs.last_sync_at.unwrap().timestamp(),
            real_last.timestamp()
        );
    }

    #[test]
    fn build_inputs_uses_fallback_when_db_reset_is_stale() {
        // Regression: DB reset in the past must not shadow a future fallback.
        // Without this, after a failed record_sync_rate_limit the loop would
        // resume hitting GitHub before the real reset.
        let now = Utc::now();
        let stale_db_reset = now - chrono::Duration::minutes(10);
        let fresh_fallback = now + chrono::Duration::minutes(10);
        let metadata = SyncMetadata {
            id: 1,
            user_id: 1,
            sync_type: GITHUB_STATS_SYNC_TYPE.to_string(),
            last_sync_at: Some((now - chrono::Duration::minutes(1)).to_rfc3339()),
            last_sync_cursor: None,
            etag: None,
            rate_limit_remaining: Some(2000),
            rate_limit_reset_at: Some(stale_db_reset.to_rfc3339()),
            scheduler_baseline_at: None,
            last_skipped_at: None,
            last_skipped_reason: None,
        };
        let settings = UserSettings::default();

        let inputs = build_inputs(&settings, Some(&metadata), Some(fresh_fallback), false, now);
        assert_eq!(inputs.rate_limit_remaining, Some(0));
        assert_eq!(
            inputs.rate_limit_reset_at.unwrap().timestamp(),
            fresh_fallback.timestamp()
        );
    }

    #[test]
    fn build_inputs_picks_later_when_both_db_and_fallback_future() {
        // When both DB and fallback are in the future, pick the later one
        // (more conservative wait).
        let now = Utc::now();
        let earlier = now + chrono::Duration::minutes(5);
        let later = now + chrono::Duration::minutes(15);
        let metadata = SyncMetadata {
            id: 1,
            user_id: 1,
            sync_type: GITHUB_STATS_SYNC_TYPE.to_string(),
            last_sync_at: None,
            last_sync_cursor: None,
            etag: None,
            rate_limit_remaining: Some(0),
            rate_limit_reset_at: Some(earlier.to_rfc3339()),
            scheduler_baseline_at: None,
            last_skipped_at: None,
            last_skipped_reason: None,
        };
        let settings = UserSettings::default();

        let inputs = build_inputs(&settings, Some(&metadata), Some(later), false, now);
        assert_eq!(
            inputs.rate_limit_reset_at.unwrap().timestamp(),
            later.timestamp()
        );
    }

    #[test]
    fn build_inputs_ignores_expired_fallback() {
        let now = Utc::now();
        let expired_fallback = now - chrono::Duration::minutes(1);
        let settings = UserSettings::default();

        let inputs = build_inputs(&settings, None, Some(expired_fallback), false, now);
        assert_eq!(inputs.rate_limit_remaining, None);
        assert_eq!(inputs.rate_limit_reset_at, None);
    }

    #[tokio::test]
    async fn set_next_sync_at_overrides_cached_status() {
        let status = RwLock::new(SchedulerStatus {
            next_sync_at: Some("2026-04-01T00:00:00Z".to_string()),
            ..SchedulerStatus::default()
        });

        let new_next = Utc::now() + chrono::Duration::minutes(20);
        set_next_sync_at(&status, new_next).await;

        let s = status.read().await;
        assert_eq!(s.next_sync_at, Some(new_next.to_rfc3339()));
    }

    #[tokio::test]
    async fn set_status_logged_out_clears_prior_user_state() {
        // Pre-populate the status with a previous user's data.
        let status = RwLock::new(SchedulerStatus {
            running: true,
            background_sync_enabled: true,
            interval_minutes: 60,
            sync_on_startup: true,
            last_sync_at: Some("2026-04-01T00:00:00Z".to_string()),
            next_sync_at: Some("2026-04-01T01:00:00Z".to_string()),
            last_skipped_at: Some("2026-04-01T00:30:00Z".to_string()),
            last_skipped_reason: Some(skip_reasons::RATE_LIMITED.to_string()),
        });

        set_status_logged_out(&status).await;

        let s = status.read().await;
        assert!(s.running);
        assert_eq!(
            s.last_skipped_reason.as_deref(),
            Some(skip_reasons::NOT_LOGGED_IN)
        );
        // Everything else from the prior session must be cleared.
        assert!(!s.background_sync_enabled);
        assert_eq!(s.interval_minutes, 0);
        assert!(!s.sync_on_startup);
        assert!(s.last_sync_at.is_none());
        assert!(s.next_sync_at.is_none());
        assert!(s.last_skipped_at.is_none());
    }
}
