//! GitHub Notifications commands for Tauri
//!
//! Surface mentions, review requests, and issue / PR comments to the frontend.
//! Uses ETag-based conditional fetching (persisted on `sync_metadata`) so
//! polling costs zero rate-limit budget when nothing has changed. The
//! returned items are also persisted to `activity_cache` so that 304
//! responses (and cold starts) can still render the last-known list
//! without forcing a fresh fetch.
//!
//! Related Issue: GitHub Issue #186 - GitHub Notifications 連携
//! Related Audit: 監査レポート §1 ギャップ表 / §8 G-05.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Emitter, State};

use super::auth::AppState;
use crate::auth::map_github_result;
use crate::database::cache_durations;
use crate::database::cache_types;
use crate::github::client::GitHubError;
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// True when the list came from the local cache rather than a fresh
    /// GitHub fetch (e.g. GitHub responded 304).
    pub from_cache: bool,
    /// `x-poll-interval` (seconds) hint from GitHub. The scheduler honours
    /// this; the UI just surfaces it for diagnostics.
    pub poll_interval_seconds: Option<u64>,
}

/// Event emitted when the scheduler observes new notification activity.
///
/// Includes the freshly-fetched items so the UI can update directly
/// without a second round-trip — a re-fetch would race the just-persisted
/// ETag and come back as 304.
///
/// `user_id` is the local DB id of the user the scheduler captured before
/// it awaited GitHub. The frontend cross-checks this against the
/// currently logged-in user and discards mismatches: if an account
/// switch happens while `run_notifications_sync` is mid-flight, the
/// emitted event carries the *previous* user's data, and applying it
/// blindly to user B's UI would leak unread counts / repo titles
/// across accounts.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationsUpdatedEvent {
    pub user_id: i64,
    pub unread_count: i32,
    pub new_count: i32,
    pub items: Vec<NotificationItem>,
}

/// Fetch the authenticated user's notifications.
///
/// Pulls the previous ETag from `sync_metadata` to issue a conditional
/// request. On 304 (or transient network failure) returns the last-known
/// list from `activity_cache` with `from_cache = true`.
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
    let prior_cursor = metadata.as_ref().and_then(|m| m.last_sync_cursor.clone());

    let client = NotificationsClient::new(token);
    let raw_result = client
        .list_notifications(prior_etag.as_deref(), false)
        .await;
    // Snapshot the unauthorized variant *before* `map_github_result`
    // consumes the result. Matching on the typed error here is more
    // robust than string-matching the Display output, which would silently
    // start treating 401s as cacheable transient failures the moment the
    // error message is reworded or localised.
    let unauthorized = matches!(&raw_result, Err(GitHubError::Unauthorized));

    // For transient failures (network blip, GitHub 5xx, even rate limit)
    // we'd rather serve the user's last-known list than return a hard
    // error and clear the dropdown. Auth-expired still propagates so the
    // session-expired banner can fire and the user can re-login. When
    // the cache is empty (first run, post-cleanup, etc.) we *also*
    // propagate — pretending the user has 0 notifications when really
    // we just couldn't reach GitHub would silently mask outages.
    let response = match map_github_result(&app, state.inner(), raw_result).await {
        Ok(r) => r,
        Err(err_msg) => {
            if unauthorized {
                return Err(err_msg);
            }
            let cached = load_cached_items(state.inner(), user.id).await;
            if cached.is_empty() {
                return Err(err_msg);
            }
            eprintln!(
                "get_notifications: transient failure, serving cache: {}",
                err_msg
            );
            let unread_count = cached.iter().filter(|i| i.unread).count() as i32;
            return Ok(NotificationsPayload {
                items: cached,
                unread_count,
                from_cache: true,
                poll_interval_seconds: None,
            });
        }
    };

    let response = match response {
        NotificationsResponse::NotModified {
            poll_interval_seconds,
        } => {
            // Return the last-known list from the cache so cold starts
            // (where the in-memory store is empty but the persisted ETag
            // is fresh) still show notifications.
            let cached = load_cached_items(state.inner(), user.id).await;
            if !cached.is_empty() {
                // Happy 304 path: preserve existing ETag and cursor (the
                // server hasn't given us new ones) and return the cached
                // list.
                persist_sync_success_with_cursor(
                    state.inner(),
                    user.id,
                    prior_etag.as_deref(),
                    prior_cursor.as_deref(),
                )
                .await;
                let unread_count = cached.iter().filter(|i| i.unread).count() as i32;
                return Ok(NotificationsPayload {
                    items: cached,
                    unread_count,
                    from_cache: true,
                    poll_interval_seconds,
                });
            }
            // Recovery path: 304 with no cached items means the persisted
            // ETag still matches GitHub but the local cache row was wiped
            // (TTL'd by `clear_expired_cache` on startup). Without a
            // re-fetch the user would see an empty inbox until the next
            // GitHub-side change. Drop the conditional request and try
            // once more — the second call cannot 304 because we send no
            // `If-None-Match`, so it always returns Modified.
            eprintln!("get_notifications: 304 with empty cache, refetching unconditionally");
            let raw_retry = client.list_notifications(None, false).await;
            map_github_result(&app, state.inner(), raw_retry).await?
        }
        modified @ NotificationsResponse::Modified { .. } => modified,
    };

    match response {
        // The recovery path can't produce a 304 (no If-None-Match was
        // sent), so collapsing back to NotModified here is unreachable.
        // Treat it defensively: preserve metadata and return whatever
        // cache holds.
        NotificationsResponse::NotModified {
            poll_interval_seconds,
        } => {
            persist_sync_success_with_cursor(
                state.inner(),
                user.id,
                prior_etag.as_deref(),
                prior_cursor.as_deref(),
            )
            .await;
            let cached = load_cached_items(state.inner(), user.id).await;
            let unread_count = cached.iter().filter(|i| i.unread).count() as i32;
            Ok(NotificationsPayload {
                items: cached,
                unread_count,
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

            // Persist the new ETag + the latest seen `updated_at` so the
            // next poll can issue a conditional request and detect
            // genuinely-new items. The cursor is monotonic — see
            // `merge_cursor` for why.
            let new_cursor = merge_cursor(prior_cursor.as_deref(), &notifications);
            persist_sync_success_with_cursor(
                state.inner(),
                user.id,
                etag.as_deref(),
                new_cursor.as_deref(),
            )
            .await;

            // Mirror the items into `activity_cache` so the next 304 (or a
            // cold start before the first scheduler tick) can still serve
            // a populated list.
            save_items_cache(state.inner(), user.id, &items).await;

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
    map_github_result(
        &app,
        state.inner(),
        client.mark_thread_as_read(&thread_id).await,
    )
    .await
}

/// Outcome of [`run_notifications_sync`] — surfaced to the scheduler so it
/// can apply rate-limit backoff to the notifications stream independently
/// of the stats sync.
pub enum NotificationsSyncOutcome {
    /// Sync completed (304 or fresh data); the scheduler can poll again on
    /// its normal cadence (or honour the `poll_interval_seconds` hint when
    /// GitHub asks for a longer wait).
    Ok {
        /// `x-poll-interval` (seconds) header from GitHub. The scheduler
        /// uses this as a soft floor on the next notifications poll —
        /// GitHub asks clients to honour it as their adaptive throttle.
        poll_interval_seconds: Option<u64>,
    },
    /// GitHub responded with 0 remaining + a reset timestamp. The scheduler
    /// should hold off on the next notifications poll until then.
    RateLimited { reset_at: DateTime<Utc> },
}

/// Refresh notifications and emit a `notifications-updated` event when new
/// activity is observed. Used by the background scheduler.
///
/// Returns a [`NotificationsSyncOutcome`] so the scheduler can record a
/// rate-limit reset and back off accordingly. Other errors (network /
/// auth) are returned as strings to match the rest of the command surface.
pub async fn run_notifications_sync(
    app: &AppHandle,
    state: &AppState,
) -> Result<NotificationsSyncOutcome, String> {
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
    let prior_cursor = metadata.as_ref().and_then(|m| m.last_sync_cursor.clone());

    let client = NotificationsClient::new(token);
    let raw_result = client
        .list_notifications(prior_etag.as_deref(), false)
        .await;

    // Intercept rate-limit errors so we can persist the reset and back off
    // on subsequent ticks. Other errors fall through to the standard
    // `map_github_result` treatment (which handles auth-expired etc.).
    if let Err(GitHubError::RateLimited(reset_ts)) = &raw_result {
        let reset_at = DateTime::from_timestamp(*reset_ts, 0).unwrap_or_else(Utc::now);
        if let Err(e) = state
            .db
            .record_sync_rate_limit(user.id, GITHUB_NOTIFICATIONS_SYNC_TYPE, reset_at)
            .await
        {
            eprintln!("Failed to record notifications rate-limit: {}", e);
        }
        return Ok(NotificationsSyncOutcome::RateLimited { reset_at });
    }

    let response = map_github_result(app, state, raw_result).await?;

    match response {
        NotificationsResponse::NotModified {
            poll_interval_seconds,
        } => {
            // 304: explicitly preserve the existing ETag and cursor (the
            // server hasn't given us new ones).
            persist_sync_success_with_cursor(
                state,
                user.id,
                prior_etag.as_deref(),
                prior_cursor.as_deref(),
            )
            .await;
            // No-op event: the unread count didn't change. Skip emitting
            // `notifications-updated` to avoid waking the UI for nothing.
            Ok(NotificationsSyncOutcome::Ok {
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

            // Cutoff used for "new since last poll": stored in
            // `sync_metadata.last_sync_cursor` as an RFC3339 timestamp.
            // The first run has no cutoff and we suppress toasts to avoid
            // a backlog dump.
            let last_seen_at = prior_cursor
                .as_ref()
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

            let new_cursor = merge_cursor(prior_cursor.as_deref(), &notifications);
            persist_sync_success_with_cursor(
                state,
                user.id,
                etag.as_deref(),
                new_cursor.as_deref(),
            )
            .await;
            save_items_cache(state, user.id, &items).await;

            // Emit the items inline — the frontend would otherwise have
            // to re-fetch and immediately hit the just-saved ETag, getting
            // 304 and stale UI back. `user_id` is captured before the
            // GitHub round-trip so the frontend can discard the event if
            // an account switch happened while we were awaiting the API.
            let event = NotificationsUpdatedEvent {
                user_id: user.id,
                unread_count,
                new_count,
                items,
            };
            let _ = app.emit("notifications-updated", &event);

            Ok(NotificationsSyncOutcome::Ok {
                poll_interval_seconds,
            })
        }
    }
}

/// Compute the cursor to persist after a successful fetch.
///
/// Returns the *later* of the prior cursor and the highest `updated_at`
/// observed in this response. Without the monotonic guard, marking the
/// newest unread thread as read makes the next unread-only response have
/// a lower max `updated_at` than the prior cursor — persisting that
/// would let already-seen threads with timestamps between the two
/// values re-appear as "new" on the following poll, inflating
/// `new_count` and re-firing OS toasts.
fn merge_cursor(prior: Option<&str>, notifications: &[GitHubNotification]) -> Option<String> {
    let observed = notifications.iter().map(|n| n.updated_at).max();
    let prior_dt = prior
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));
    match (observed, prior_dt) {
        (Some(o), Some(p)) => Some(o.max(p).to_rfc3339()),
        (Some(o), None) => Some(o.to_rfc3339()),
        (None, p) => p.map(|dt| dt.to_rfc3339()),
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
            eprintln!(
                "Failed to load user settings for notifications toast: {}",
                e
            );
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
/// Always writes `last_sync_at`. ETag and cursor are passed through to
/// `update_sync_metadata`, which uses `COALESCE(?, col)` so `None` keeps
/// the existing column value — call sites should pass the prior values
/// explicitly on 304 to make the intent obvious in code review.
///
/// Best-effort writes — failures are logged but never fail the user-facing
/// command. Mirrors `commands::github::persist_sync_success` for the stats
/// sync.
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

/// Save the notifications list to `activity_cache` so 304 responses (and
/// cold starts before the scheduler ticks) can still serve a populated UI.
async fn save_items_cache(state: &AppState, user_id: i64, items: &[NotificationItem]) {
    let json = match serde_json::to_string(items) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Failed to serialize notifications cache: {}", e);
            return;
        }
    };
    let expires_at = Utc::now() + chrono::Duration::minutes(cache_durations::GITHUB_NOTIFICATIONS);
    if let Err(e) = state
        .db
        .save_cache(
            user_id,
            cache_types::GITHUB_NOTIFICATIONS,
            &json,
            expires_at,
        )
        .await
    {
        eprintln!("Failed to persist notifications cache: {}", e);
    }
}

/// Load the cached notifications list. Falls back to an empty Vec on miss
/// or parse failure — callers treat this as "no known items".
///
/// We use `get_any_cache` (rather than `get_cache`, which filters expired)
/// because the cache TTL is just a stale-render bound: a slightly-stale
/// list is still better than an empty UI when GitHub responds 304.
async fn load_cached_items(state: &AppState, user_id: i64) -> Vec<NotificationItem> {
    match state
        .db
        .get_any_cache(user_id, cache_types::GITHUB_NOTIFICATIONS)
        .await
    {
        Ok(Some((json, _cached_at, _expires_at))) => {
            serde_json::from_str(&json).unwrap_or_default()
        }
        Ok(None) => Vec::new(),
        Err(e) => {
            eprintln!("Failed to load notifications cache: {}", e);
            Vec::new()
        }
    }
}
