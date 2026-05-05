//! GitHub activity timeline commands for Tauri
//!
//! Surfaces the authenticated user's recent GitHub events
//! (`GET /users/{username}/events`) as a normalised feed for the home
//! activity timeline. The endpoint already caps the response at the
//! last 90 days / 300 events; we just paginate, normalise, and cache.
//!
//! Related Issue: GitHub Issue #187 - GitHub連携 アクティビティ・タイムライン UI 配線

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{command, AppHandle, State};

use super::auth::AppState;
use super::github::CachedResponse;
use crate::auth::{handle_unauthorized, reasons};
use crate::database::{cache_durations, cache_types};
use crate::github::client::GitHubError;
use crate::github::types::ActivityEvent;
use crate::github::GitHubClient;

/// Default page size matches GitHub's documented maximum for the events
/// endpoint. The 90-day / 300-event cap is enforced by GitHub itself, so
/// asking for 100 keeps round-trip count to a minimum without going over.
const DEFAULT_PER_PAGE: i32 = 100;

/// Normalised activity feed row. Pre-extracts the fields the React UI
/// renders so the frontend does not have to walk GitHub's polymorphic
/// `payload` for every event type.
///
/// Anything the UI does not need (full commit list, full review body, etc.)
/// is intentionally dropped to keep the cached payload small.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityFeedItem {
    /// GitHub event id — stable, used as React key.
    pub id: String,
    /// `PushEvent` / `PullRequestEvent` / `IssuesEvent` / etc. Forwarded
    /// verbatim so the UI can decide on icon + copy per type.
    pub event_type: String,
    /// ISO8601.
    pub created_at: String,
    /// `owner/repo`.
    pub repo_name: String,
    /// `https://github.com/{owner}/{repo}` — pre-built so the UI does not
    /// have to translate the `api.github.com` URL the API returns.
    pub repo_url: String,
    /// `opened` / `closed` / `started` / `created` / etc. when the event
    /// type carries an `action`.
    pub action: Option<String>,
    /// PR / issue / release title, or for Create/Delete events the ref name.
    pub title: Option<String>,
    /// Browser URL of the artefact involved (PR, issue, release, comment).
    /// Falls back to `repo_url` on the frontend when absent.
    pub target_url: Option<String>,
    /// PR or issue number, when applicable.
    pub number: Option<i32>,
    /// Branch / tag name for `PushEvent`, `CreateEvent`, `DeleteEvent`.
    pub ref_name: Option<String>,
    /// `branch` / `tag` / `repository` for `CreateEvent` / `DeleteEvent`.
    pub ref_type: Option<String>,
    /// Number of commits in a `PushEvent`.
    pub commits_count: Option<i32>,
}

/// Aggregated payload returned by `get_activity_feed_with_cache`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ActivityFeed {
    pub items: Vec<ActivityFeedItem>,
}

/// Network-eligible error variants (worth a cache fallback).
fn is_network_or_rate_limit_error(error: &GitHubError) -> bool {
    matches!(
        error,
        GitHubError::HttpRequest(_) | GitHubError::RateLimited(_)
    )
}

/// Map a raw `ActivityEvent` (the API shape) into the flat row the UI
/// renders. Pure / stateless — exposed for unit tests.
///
/// The events endpoint sets `repo.url` to the api.github.com URL, so we
/// always synthesise the browser URL from `owner/repo` instead.
pub fn normalize_event(event: &ActivityEvent) -> ActivityFeedItem {
    let payload = &event.payload;
    let repo_url = format!("https://github.com/{}", event.repo.name);

    let action = payload
        .get("action")
        .and_then(Value::as_str)
        .map(str::to_string);

    let mut title: Option<String> = None;
    let mut target_url: Option<String> = None;
    let mut number: Option<i32> = None;
    let mut ref_name: Option<String> = None;
    let mut ref_type: Option<String> = None;
    let mut commits_count: Option<i32> = None;

    match event.event_type.as_str() {
        "PushEvent" => {
            ref_name = payload
                .get("ref")
                .and_then(Value::as_str)
                .map(str::to_string);
            commits_count = payload
                .get("size")
                .and_then(Value::as_i64)
                .map(|v| v as i32);
            target_url = Some(repo_url.clone());
        }
        "PullRequestEvent" | "PullRequestReviewEvent" | "PullRequestReviewCommentEvent" => {
            if let Some(pr) = payload.get("pull_request") {
                title = pr.get("title").and_then(Value::as_str).map(str::to_string);
                target_url = pr
                    .get("html_url")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                number = pr.get("number").and_then(Value::as_i64).map(|v| v as i32);
            }
            // Fall back to the top-level `number` when available
            // (PullRequestEvent has it).
            if number.is_none() {
                number = payload
                    .get("number")
                    .and_then(Value::as_i64)
                    .map(|v| v as i32);
            }
        }
        "IssuesEvent" | "IssueCommentEvent" => {
            if let Some(issue) = payload.get("issue") {
                title = issue
                    .get("title")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                target_url = issue
                    .get("html_url")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                number = issue
                    .get("number")
                    .and_then(Value::as_i64)
                    .map(|v| v as i32);
            }
            // For `IssueCommentEvent`, prefer the comment's html_url so the
            // click drops the user at the comment anchor.
            if event.event_type == "IssueCommentEvent" {
                if let Some(comment_url) = payload
                    .get("comment")
                    .and_then(|c| c.get("html_url"))
                    .and_then(Value::as_str)
                {
                    target_url = Some(comment_url.to_string());
                }
            }
        }
        "ReleaseEvent" => {
            if let Some(release) = payload.get("release") {
                title = release
                    .get("name")
                    .and_then(Value::as_str)
                    .filter(|s| !s.is_empty())
                    .or_else(|| release.get("tag_name").and_then(Value::as_str))
                    .map(str::to_string);
                target_url = release
                    .get("html_url")
                    .and_then(Value::as_str)
                    .map(str::to_string);
            }
        }
        "CreateEvent" | "DeleteEvent" => {
            ref_name = payload
                .get("ref")
                .and_then(Value::as_str)
                .map(str::to_string);
            ref_type = payload
                .get("ref_type")
                .and_then(Value::as_str)
                .map(str::to_string);
            target_url = Some(repo_url.clone());
        }
        "ForkEvent" => {
            if let Some(forkee) = payload.get("forkee") {
                title = forkee
                    .get("full_name")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                target_url = forkee
                    .get("html_url")
                    .and_then(Value::as_str)
                    .map(str::to_string);
            }
        }
        "WatchEvent" | "PublicEvent" | "MemberEvent" => {
            target_url = Some(repo_url.clone());
        }
        _ => {
            // Unknown event types still render — the UI shows a generic
            // "activity in {repo}" line. Set a safe target.
            target_url = Some(repo_url.clone());
        }
    }

    ActivityFeedItem {
        id: event.id.clone(),
        event_type: event.event_type.clone(),
        created_at: event.created_at.to_rfc3339(),
        repo_name: event.repo.name.clone(),
        repo_url,
        action,
        title,
        target_url,
        number,
        ref_name,
        ref_type,
        commits_count,
    }
}

/// Fetch the authenticated user's recent GitHub events with a short
/// SQLite cache fallback.
///
/// Behaviour mirrors `get_my_pr_progress_with_cache`:
/// - Success → cache + return `from_cache: false`.
/// - Network / rate-limit error → fall back to last cached payload (any age)
///   if available, else surface the error.
/// - 401 → trigger the central auth-expired flow; do not fall back to cache.
///
/// REST budget: 5000 req/hr. With a 5-minute TTL and one user, even
/// pathological refresh loops stay nowhere near the limit.
#[command]
pub async fn get_activity_feed_with_cache(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CachedResponse<ActivityFeed>, String> {
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

    let client = GitHubClient::new(token);

    match client
        .get_user_events(&user.username, DEFAULT_PER_PAGE, 1)
        .await
    {
        Ok(events) => {
            let items: Vec<ActivityFeedItem> = events.iter().map(normalize_event).collect();
            let payload = ActivityFeed { items };

            let payload_json = serde_json::to_string(&payload)
                .map_err(|e| format!("Failed to serialize activity feed: {}", e))?;

            let now = Utc::now();
            let expires_at = now + chrono::Duration::minutes(cache_durations::ACTIVITY_FEED);

            // Best-effort cache write — a failure here shouldn't block the
            // response. Mirrors the other `*_with_cache` commands.
            let _ = state
                .db
                .save_cache(
                    user.id,
                    cache_types::ACTIVITY_FEED,
                    &payload_json,
                    expires_at,
                )
                .await;

            Ok(CachedResponse {
                data: payload,
                from_cache: false,
                cached_at: Some(now.to_rfc3339()),
                expires_at: Some(expires_at.to_rfc3339()),
            })
        }
        Err(GitHubError::Unauthorized) => {
            handle_unauthorized(&app, state.inner(), reasons::GITHUB_UNAUTHORIZED).await;
            Err(GitHubError::Unauthorized.to_string())
        }
        Err(api_error) => {
            if !is_network_or_rate_limit_error(&api_error) {
                return Err(format!("GitHub API error: {}", api_error));
            }

            eprintln!(
                "Activity feed fetch failed, attempting cache fallback: {}",
                api_error
            );

            let cache_result = state
                .db
                .get_any_cache(user.id, cache_types::ACTIVITY_FEED)
                .await
                .map_err(|e| format!("Failed to read activity_feed cache: {}", e))?;

            match cache_result {
                Some((data_json, cached_at, expires_at)) => {
                    let payload: ActivityFeed = serde_json::from_str(&data_json)
                        .map_err(|e| format!("Failed to parse cached data: {}", e))?;
                    Ok(CachedResponse {
                        data: payload,
                        from_cache: true,
                        cached_at: Some(cached_at),
                        expires_at: Some(expires_at),
                    })
                }
                None => Err(format!(
                    "GitHub APIにアクセスできず、キャッシュもありません: {}",
                    api_error
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::types::EventRepo;
    use chrono::TimeZone;
    use serde_json::json;

    fn make_event(event_type: &str, payload: Value) -> ActivityEvent {
        ActivityEvent {
            id: "12345".into(),
            event_type: event_type.into(),
            created_at: Utc.with_ymd_and_hms(2026, 4, 1, 12, 0, 0).unwrap(),
            repo: EventRepo {
                id: 1,
                name: "octo/test".into(),
            },
            payload,
        }
    }

    #[test]
    fn normalize_push_event_extracts_ref_and_size() {
        let event = make_event(
            "PushEvent",
            json!({
                "ref": "refs/heads/main",
                "size": 3,
                "commits": [],
            }),
        );
        let item = normalize_event(&event);

        assert_eq!(item.event_type, "PushEvent");
        assert_eq!(item.repo_name, "octo/test");
        assert_eq!(item.repo_url, "https://github.com/octo/test");
        assert_eq!(item.ref_name.as_deref(), Some("refs/heads/main"));
        assert_eq!(item.commits_count, Some(3));
        assert_eq!(
            item.target_url.as_deref(),
            Some("https://github.com/octo/test")
        );
        assert!(item.title.is_none());
    }

    #[test]
    fn normalize_pull_request_event_extracts_pr_title_and_url() {
        let event = make_event(
            "PullRequestEvent",
            json!({
                "action": "opened",
                "number": 42,
                "pull_request": {
                    "title": "Fix bug",
                    "html_url": "https://github.com/octo/test/pull/42",
                    "number": 42,
                },
            }),
        );
        let item = normalize_event(&event);

        assert_eq!(item.action.as_deref(), Some("opened"));
        assert_eq!(item.title.as_deref(), Some("Fix bug"));
        assert_eq!(
            item.target_url.as_deref(),
            Some("https://github.com/octo/test/pull/42")
        );
        assert_eq!(item.number, Some(42));
    }

    #[test]
    fn normalize_issues_event_extracts_issue_fields() {
        let event = make_event(
            "IssuesEvent",
            json!({
                "action": "closed",
                "issue": {
                    "title": "Crash on launch",
                    "html_url": "https://github.com/octo/test/issues/7",
                    "number": 7,
                },
            }),
        );
        let item = normalize_event(&event);

        assert_eq!(item.action.as_deref(), Some("closed"));
        assert_eq!(item.title.as_deref(), Some("Crash on launch"));
        assert_eq!(
            item.target_url.as_deref(),
            Some("https://github.com/octo/test/issues/7")
        );
        assert_eq!(item.number, Some(7));
    }

    #[test]
    fn normalize_issue_comment_event_prefers_comment_url() {
        let event = make_event(
            "IssueCommentEvent",
            json!({
                "action": "created",
                "issue": {
                    "title": "Feature request",
                    "html_url": "https://github.com/octo/test/issues/3",
                    "number": 3,
                },
                "comment": {
                    "html_url": "https://github.com/octo/test/issues/3#issuecomment-99",
                },
            }),
        );
        let item = normalize_event(&event);

        // Title still comes from the issue, but the URL drops at the comment.
        assert_eq!(item.title.as_deref(), Some("Feature request"));
        assert_eq!(
            item.target_url.as_deref(),
            Some("https://github.com/octo/test/issues/3#issuecomment-99")
        );
    }

    #[test]
    fn normalize_release_event_falls_back_to_tag_when_name_missing() {
        let event = make_event(
            "ReleaseEvent",
            json!({
                "action": "published",
                "release": {
                    "name": "",
                    "tag_name": "v1.2.3",
                    "html_url": "https://github.com/octo/test/releases/tag/v1.2.3",
                },
            }),
        );
        let item = normalize_event(&event);

        assert_eq!(item.title.as_deref(), Some("v1.2.3"));
        assert_eq!(
            item.target_url.as_deref(),
            Some("https://github.com/octo/test/releases/tag/v1.2.3")
        );
    }

    #[test]
    fn normalize_create_event_extracts_ref_type_and_name() {
        let event = make_event(
            "CreateEvent",
            json!({
                "ref": "feature/x",
                "ref_type": "branch",
            }),
        );
        let item = normalize_event(&event);

        assert_eq!(item.ref_name.as_deref(), Some("feature/x"));
        assert_eq!(item.ref_type.as_deref(), Some("branch"));
        assert_eq!(
            item.target_url.as_deref(),
            Some("https://github.com/octo/test")
        );
    }

    #[test]
    fn normalize_unknown_event_type_still_yields_a_row_with_repo_target() {
        let event = make_event("FutureEventType", json!({}));
        let item = normalize_event(&event);

        assert_eq!(item.event_type, "FutureEventType");
        assert_eq!(
            item.target_url.as_deref(),
            Some("https://github.com/octo/test")
        );
    }
}
