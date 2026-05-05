//! GitHub Notifications commands for Tauri
//!
//! Surface mentions, review requests, and issue / PR comments to the frontend.
//! Uses ETag-based conditional fetching (persisted on `sync_metadata`) so
//! polling costs zero rate-limit budget when nothing has changed.
//!
//! Related Issue: GitHub Issue #186 - GitHub Notifications 連携
//! Related Audit: 監査レポート §1 ギャップ表 / §8 G-05.

use chrono::{DateTime, Utc};
use serde::Serialize;
use tauri::{command, AppHandle, Emitter, State};

use super::auth::AppState;
use crate::auth::map_github_result;
use crate::github::{
    build_notification_html_url, GitHubNotification, NotificationsClient, NotificationsResponse,
};

/// Sync type key used to namespace notifications metadata in `sync_metadata`.
/// Distinct from `github_stats` so the two ETag streams don't collide.
pub const GITHUB_NOTIFICATIONS_SYNC_TYPE: &str = "github_notifications";

/// One notification surfaced to the frontend.
///
/// Flat shape — each row pre-computes the browser URL so the UI doesn't need
/// to mirror `build_html_url`'s API → web translation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationItem {
    pub id: String,
    pub unread: bool,
    /// `mention` / `review_requested` / `assign` / `comment` / etc. Surfaced
    /// verbatim so the UI can decide how to label / filter each type.
    pub reason: String,
    pub title: String,
    /// `Issue` / `PullRequest` / `Commit` / `Discussion`.
    pub subject_type: String,
    pub repo_full_name: String,
    pub repo_url: String,
    pub html_url: String,
    /// ISO8601.
    pub updated_at: String,
    pub last_read_at: Option<String>,
}

impl From<&GitHubNotification> for NotificationItem {
    fn from(n: &GitHubNotification) -> Self {
        Self {
            id: n.id.clone(),
            unread: n.unread,
            reason: n.reason.clone(),
            title: n.subject.title.clone(),
            subject_type: n.subject.kind.clone(),
            repo_full_name: n.repository.full_name.clone(),
            repo_url: n.repository.html_url.clone(),
            html_url: build_notification_html_url(n),
            updated_at: n.updated_at.to_rfc3339(),
            last_read_at: n.last_read_at.as_ref().map(|t| t.to_rfc3339()),
        }
    }
}

/// Aggregated payload returned by `get_notifications`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationsPayload {
    pub items: Vec<NotificationItem>,
    pub unread_count: i32,
    /// True when GitHub returned 304 — the items list is empty and the
    /// caller should keep showing whatever it already had.
    pub from_cache: bool,
    /// `x-poll-interval` (seconds) hint from GitHub. The scheduler honours
    /// this; the UI just surfaces it for diagnostics.
    pub poll_interval_seconds: Option<u64>,
}

/// Event emitted when new unread notifications are observed during a sync.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationsUpdatedEvent {
    pub unread_count: i32,
    pub new_count: i32,
}

/// Fetch the authenticated user's notifications.
///
/// Pulls the previous ETag from `sync_metadata` to issue a conditional
/// request. On 304 returns `from_cache = true` with an empty list — the
/// frontend should ignore the empty payload and keep its current state.
#[command]
pub async fn get_notifications(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<NotificationsPayload, String> {
    let token = state
        .token_manager
        .get_access_token()
        .await
        .map_err(|e| e.to_string())?;

    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Not logged in")?;

    let metadata = state
        .db
        .get_sync_metadata(user.id, GITHUB_NOTIFICATIONS_SYNC_TYPE)
        .await
        .map_err(|e| e.to_string())?;
    let prior_etag = metadata.as_ref().and_then(|m| m.etag.clone());

    let client = NotificationsClient::new(token);
    let response = map_github_result(
        &app,
        state.inner(),
        client.list_notifications(prior_etag.as_deref(), false).await,
    )
    .await?;

    match response {
        NotificationsResponse::NotModified {
            poll_interval_seconds,
        } => {
            // Refresh `last_sync_at` so the scheduler knows we polled
            // successfully even when nothing changed. The ETag we already
            // hold remains valid.
            persist_sync_success(state.inner(), user.id, None).await;
            Ok(NotificationsPayload {
                items: Vec::new(),
                unread_count: 0,
                from_cache: true,
                poll_interval_seconds,
            })
        }
        NotificationsResponse::Modified {
            notifications,
            etag,
            poll_interval_seconds,
        } => {
            let items: Vec<NotificationItem> =
                notifications.iter().map(NotificationItem::from).collect();
            let unread_count = items.iter().filter(|i| i.unread).count() as i32;

            // Persist the new ETag and the latest seen `updated_at` so the
            // next poll can issue a conditional request and detect
            // genuinely-new items. A failure here is logged but doesn't
            // fail the command — we already have the data.
            let new_cursor = notifications
                .iter()
                .map(|n| n.updated_at)
                .max()
                .map(|t| t.to_rfc3339());
            persist_sync_success_with_cursor(
                state.inner(),
                user.id,
                etag.as_deref(),
                new_cursor.as_deref(),
            )
            .await;

            Ok(NotificationsPayload {
                items,
                unread_count,
                from_cache: false,
                poll_interval_seconds,
            })
        }
    }
}

/// Mark a single GitHub notification thread as read.
///
/// `thread_id` is the `id` field from a notification row.
#[command]
pub async fn mark_notification_read(
    app: AppHandle,
    state: State<'_, AppState>,
    thread_id: String,
) -> Result<(), String> {
    let token = state
        .token_manager
        .get_access_token()
        .await
        .map_err(|e| e.to_string())?;

    let client = NotificationsClient::new(token);
    map_github_result(&app, state.inner(), client.mark_thread_as_read(&thread_id).await).await
}

/// Refresh notifications and emit a `notifications-updated` event when the
/// unread count changes. Used by the background scheduler.
///
/// Returns the unread count so the scheduler can record it; errors are
/// returned as strings to match the rest of the command surface.
pub async fn run_notifications_sync(
    app: &AppHandle,
    state: &AppState,
) -> Result<i32, String> {
    let token = state
        .token_manager
        .get_access_token()
        .await
        .map_err(|e| e.to_string())?;

    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Not logged in")?;

    let metadata = state
        .db
        .get_sync_metadata(user.id, GITHUB_NOTIFICATIONS_SYNC_TYPE)
        .await
        .map_err(|e| e.to_string())?;
    let prior_etag = metadata.as_ref().and_then(|m| m.etag.clone());

    let client = NotificationsClient::new(token);
    let response = map_github_result(
        app,
        state,
        client.list_notifications(prior_etag.as_deref(), false).await,
    )
    .await?;

    match response {
        NotificationsResponse::NotModified { .. } => {
            persist_sync_success(state, user.id, None).await;
            // No-op event: the unread count didn't change. We deliberately
            // skip emitting `notifications-updated` to avoid waking the UI
            // for nothing.
            Ok(0)
        }
        NotificationsResponse::Modified {
            notifications,
            etag,
            ..
        } => {
            let unread_count = notifications.iter().filter(|n| n.unread).count() as i32;

            // Find the cutoff used for "new since last poll": stored in
            // `sync_metadata.last_sync_cursor` as an RFC3339 timestamp. The
            // first run has no cutoff and we suppress toasts to avoid a
            // backlog dump.
            let last_seen_at = metadata
                .as_ref()
                .and_then(|m| m.last_sync_cursor.as_ref())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc));

            let new_count = match last_seen_at {
                Some(cutoff) => notifications
                    .iter()
                    .filter(|n| n.unread && n.updated_at > cutoff)
                    .count() as i32,
                None => 0,
            };

            // Surface the actually-new unread mentions / review requests as
            // OS notifications. Suppressed on first poll (no cutoff yet) to
            // avoid spamming a backlog of pre-existing items.
            if last_seen_at.is_some() {
                maybe_send_os_notifications(app, state, user.id, &notifications, last_seen_at)
                    .await;
            }

            let new_cursor = notifications
                .iter()
                .map(|n| n.updated_at)
                .max()
                .map(|t| t.to_rfc3339());
            persist_sync_success_with_cursor(state, user.id, etag.as_deref(), new_cursor.as_deref())
                .await;

            // Emit the event regardless of whether the count is zero — the UI
            // consumes this as a "go re-fetch the list" signal.
            let event = NotificationsUpdatedEvent {
                unread_count,
                new_count,
            };
            let _ = app.emit("notifications-updated", &event);

            Ok(unread_count)
        }
    }
}

/// Send OS-native notifications for items strictly newer than `cutoff`.
///
/// Limits to a small batch so a burst of activity doesn't dump dozens of
/// toasts. The user's notification settings still gate whether anything is
/// sent at the OS level — see `utils::notifications::send_notification`.
async fn maybe_send_os_notifications(
    app: &AppHandle,
    state: &AppState,
    user_id: i64,
    notifications: &[GitHubNotification],
    cutoff: Option<DateTime<Utc>>,
) {
    const MAX_TOASTS: usize = 3;

    let settings = match state.db.get_or_create_user_settings(user_id).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load user settings for notifications toast: {}", e);
            return;
        }
    };

    // Highest-signal reasons surfaced first; comment-only updates are noisier
    // so we let the in-app badge handle them.
    let priority_reasons = ["mention", "review_requested", "assign", "team_mention"];
    let toasts: Vec<&GitHubNotification> = notifications
        .iter()
        .filter(|n| {
            n.unread
                && priority_reasons.contains(&n.reason.as_str())
                && cutoff.is_none_or(|c| n.updated_at > c)
        })
        .take(MAX_TOASTS)
        .collect();

    for n in toasts {
        let title = match n.reason.as_str() {
            "mention" | "team_mention" => "GitHub: メンション",
            "review_requested" => "GitHub: レビュー依頼",
            "assign" => "GitHub: アサイン",
            _ => "GitHub 通知",
        };
        let body = format!("{} ({})", n.subject.title, n.repository.full_name);
        if let Err(e) = crate::utils::notifications::send_notification(app, &settings, title, &body)
        {
            eprintln!("Failed to send GitHub notification toast: {}", e);
        }
    }
}

/// Persist a successful notifications sync to `sync_metadata`.
///
/// Best-effort writes — failures are logged but never fail the user-facing
/// command. Mirrors `commands::github::persist_sync_success` for the stats
/// sync.
async fn persist_sync_success(state: &AppState, user_id: i64, etag: Option<&str>) {
    persist_sync_success_with_cursor(state, user_id, etag, None).await;
}

/// Variant of [`persist_sync_success`] that also writes the
/// `last_sync_cursor` (used to track the most recent notification
/// `updated_at` so the next poll knows what's "new").
async fn persist_sync_success_with_cursor(
    state: &AppState,
    user_id: i64,
    etag: Option<&str>,
    cursor: Option<&str>,
) {
    if let Err(e) = state
        .db
        .get_or_create_sync_metadata(user_id, GITHUB_NOTIFICATIONS_SYNC_TYPE)
        .await
    {
        eprintln!("Failed to ensure notifications sync_metadata row: {}", e);
        return;
    }

    let now = chrono::Utc::now().to_rfc3339();
    if let Err(e) = state
        .db
        .update_sync_metadata(
            user_id,
            GITHUB_NOTIFICATIONS_SYNC_TYPE,
            Some(now),
            cursor,
            etag,
            None,
            None,
        )
        .await
    {
        eprintln!(
            "Failed to update notifications sync_metadata after sync: {}",
            e
        );
    }

    if let Err(e) = state
        .db
        .clear_sync_skipped(user_id, GITHUB_NOTIFICATIONS_SYNC_TYPE)
        .await
    {
        eprintln!("Failed to clear notifications sync_skipped: {}", e);
    }
    if let Err(e) = state
        .db
        .clear_sync_rate_limit(user_id, GITHUB_NOTIFICATIONS_SYNC_TYPE)
        .await
    {
        eprintln!("Failed to clear notifications sync_rate_limit: {}", e);
    }
}
