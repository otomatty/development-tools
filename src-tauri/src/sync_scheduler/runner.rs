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
                        let now = Utc::now();
                        if let Err(e) = state
                            .db
                            .update_sync_metadata(
                                user.id,
                                GITHUB_STATS_SYNC_TYPE,
                                Some(now.to_rfc3339()),
                                None,
                                None,
                                None,
                                None,
                            )
                            .await
                        {
                            eprintln!("Scheduler: failed to update sync_metadata: {}", e);
                        }
                        let _ = state
                            .db
                            .clear_sync_skipped(user.id, GITHUB_STATS_SYNC_TYPE)
                            .await;
                    }
                    Err(err_msg) => {
                        eprintln!("Scheduler: scheduled sync failed: {}", err_msg);
                        if classify_rate_limited(&err_msg) {
                            let _ = state
                                .db
                                .record_sync_skipped(
                                    user.id,
                                    GITHUB_STATS_SYNC_TYPE,
                                    skip_reasons::RATE_LIMITED,
                                    Utc::now(),
                                )
                                .await;
                        }
                    }
                }
                // Loop back; the next iteration computes a fresh sleep.
            }
            SchedulerAction::Sleep { seconds } => {
                is_first_run = false;
                wait_for_change_or_timeout(&notify, seconds).await;
            }
            SchedulerAction::Idle { reason } => {
                is_first_run = false;
                let _ = state
                    .db
                    .record_sync_skipped(
                        user.id,
                        GITHUB_STATS_SYNC_TYPE,
                        reason,
                        Utc::now(),
                    )
                    .await;
                // Update status so the UI reflects the idle state.
                write_status(&status, &settings, metadata.as_ref(), None, true).await;
                // Wait until the user toggles a setting.
                notify.notified().await;
            }
            SchedulerAction::RateLimited { reason, seconds } => {
                is_first_run = false;
                let _ = state
                    .db
                    .record_sync_skipped(
                        user.id,
                        GITHUB_STATS_SYNC_TYPE,
                        reason,
                        Utc::now(),
                    )
                    .await;
                wait_for_change_or_timeout(&notify, seconds).await;
            }
        }
    }
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
}
