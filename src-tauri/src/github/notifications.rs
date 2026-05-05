//! GitHub Notifications API client
//!
//! Wraps the REST `GET /notifications` and related mark-as-read endpoints.
//! Uses ETag-based conditional requests so polling repeatedly costs zero
//! rate-limit budget when nothing has changed (GitHub returns 304 with no
//! body).
//!
//! Related Issue: GitHub Issue #186 - GitHub Notifications 連携
//! Related Audit: 監査レポート §1 ギャップ表 / §8 G-05.

use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, IF_NONE_MATCH, USER_AGENT};
use serde::{Deserialize, Serialize};

use super::client::{GitHubError, GitHubResult};

const GITHUB_API_URL: &str = "https://api.github.com";
const USER_AGENT_VALUE: &str = "development-tools/1.0";

/// One row of the `GET /notifications` response.
///
/// Only the fields the UI consumes are deserialised; the rest of the payload
/// (e.g. `subscription_url`) is ignored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubNotification {
    /// Stable thread ID — used for `mark thread as read`.
    pub id: String,
    pub unread: bool,
    /// Reason GitHub surfaced this notification (mention, review_requested,
    /// assign, comment, etc.).
    pub reason: String,
    pub updated_at: DateTime<Utc>,
    pub last_read_at: Option<DateTime<Utc>>,
    pub subject: NotificationSubject,
    pub repository: NotificationRepository,
}

/// Inner `subject` envelope: title + the API URL of the underlying issue / PR.
///
/// `url` looks like `https://api.github.com/repos/{owner}/{repo}/issues/{n}`
/// or `.../pulls/{n}`. We translate it to a browser URL on the frontend so
/// clicking a notification opens the actual page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSubject {
    pub title: String,
    /// `Issue` / `PullRequest` / `Commit` / `Discussion` / etc.
    #[serde(rename = "type")]
    pub kind: String,
    pub url: Option<String>,
    pub latest_comment_url: Option<String>,
}

/// Subset of the repository payload we need for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRepository {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub private: bool,
}

/// Result of a conditional fetch.
///
/// `NotModified` is the happy path when the ETag is still fresh — the caller
/// keeps showing whatever is already cached and skips DB writes.
#[derive(Debug, Clone)]
pub enum NotificationsResponse {
    Modified {
        notifications: Vec<GitHubNotification>,
        /// New ETag to persist for the next conditional request.
        etag: Option<String>,
        /// `x-poll-interval` header (in seconds). GitHub asks clients to
        /// honour this — it's their adaptive throttle for the notifications
        /// endpoint.
        poll_interval_seconds: Option<u64>,
    },
    NotModified {
        poll_interval_seconds: Option<u64>,
    },
}

/// GitHub Notifications client.
///
/// Mirrors `IssuesClient` in shape — separate from `GitHubClient` because the
/// `If-None-Match` plumbing and 304 handling don't fit the existing
/// success-or-error JSON-only `get` helper.
pub struct NotificationsClient {
    client: reqwest::Client,
    access_token: String,
}

impl NotificationsClient {
    pub fn new(access_token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            access_token,
        }
    }

    fn build_headers(&self, etag: Option<&str>) -> GitHubResult<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.access_token))
                .map_err(|e| GitHubError::ApiError(format!("Invalid token format: {}", e)))?,
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_static("2022-11-28"),
        );
        if let Some(etag_value) = etag {
            // GitHub returns weak ETags for the notifications endpoint
            // (e.g. `W/"abcdef"`); we forward whatever was persisted
            // verbatim. Invalid bytes degrade gracefully — drop the header
            // and refetch unconditionally on the next call.
            if let Ok(header_value) = HeaderValue::from_str(etag_value) {
                headers.insert(IF_NONE_MATCH, header_value);
            }
        }
        Ok(headers)
    }

    /// Fetch notifications, optionally conditional on a previously stored
    /// ETag. Returns `NotModified` on 304 so the caller can short-circuit.
    ///
    /// `all = false` (the GitHub default) means only unread notifications.
    /// We hardcode this — the UI surfaces unread items only — but expose it
    /// as a parameter for future flexibility.
    pub async fn list_notifications(
        &self,
        etag: Option<&str>,
        all: bool,
    ) -> GitHubResult<NotificationsResponse> {
        let url = format!(
            "{}/notifications?all={}&per_page=50",
            GITHUB_API_URL,
            if all { "true" } else { "false" }
        );

        let headers = self.build_headers(etag)?;
        let response = self.client.get(&url).headers(headers).send().await?;

        let status = response.status();
        let new_etag = response
            .headers()
            .get(reqwest::header::ETAG)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let poll_interval_seconds = response
            .headers()
            .get("x-poll-interval")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        if status == reqwest::StatusCode::NOT_MODIFIED {
            return Ok(NotificationsResponse::NotModified {
                poll_interval_seconds,
            });
        }

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(GitHubError::Unauthorized);
        }

        if status == reqwest::StatusCode::FORBIDDEN {
            // Mirror IssuesClient's 0-remaining → RateLimited mapping so the
            // scheduler treats notifications throttling identically to the
            // stats sync.
            let remaining_zero = response
                .headers()
                .get("x-ratelimit-remaining")
                .and_then(|v| v.to_str().ok())
                .map(|v| v == "0")
                .unwrap_or(false);
            if remaining_zero {
                let reset = response
                    .headers()
                    .get("x-ratelimit-reset")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<i64>().ok())
                    .unwrap_or(0);
                return Err(GitHubError::RateLimited(reset));
            }
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(GitHubError::ApiError(format!(
                "Status {}: {}",
                status, error_text
            )));
        }

        let notifications: Vec<GitHubNotification> = response.json().await?;
        Ok(NotificationsResponse::Modified {
            notifications,
            etag: new_etag,
            poll_interval_seconds,
        })
    }

    /// Mark a single notification thread as read.
    ///
    /// `PATCH /notifications/threads/{thread_id}` returns 205 (Reset Content)
    /// on success per the GitHub docs.
    pub async fn mark_thread_as_read(&self, thread_id: &str) -> GitHubResult<()> {
        let url = format!("{}/notifications/threads/{}", GITHUB_API_URL, thread_id);
        let headers = self.build_headers(None)?;
        let response = self.client.patch(&url).headers(headers).send().await?;

        match response.status() {
            status if status.is_success() => Ok(()),
            reqwest::StatusCode::RESET_CONTENT => Ok(()),
            reqwest::StatusCode::NOT_MODIFIED => Ok(()),
            reqwest::StatusCode::UNAUTHORIZED => Err(GitHubError::Unauthorized),
            reqwest::StatusCode::NOT_FOUND => Err(GitHubError::NotFound(thread_id.to_string())),
            status => {
                let error_text = response.text().await.unwrap_or_default();
                Err(GitHubError::ApiError(format!(
                    "Status {}: {}",
                    status, error_text
                )))
            }
        }
    }
}

/// Translate a notification subject's API URL to a browser URL.
///
/// The notifications endpoint surfaces API URLs like
/// `https://api.github.com/repos/octo/test/issues/42` or
/// `.../pulls/42`. Clicking through should open the GitHub web UI, so we
/// rewrite `api.github.com/repos/...` → `github.com/...` and translate
/// `pulls` → `pull` (plural is API-only).
///
/// Falls back to the repository's `html_url` when the subject URL is
/// missing (some `Discussion` and `Commit` rows have no API URL).
pub fn build_html_url(notification: &GitHubNotification) -> String {
    if let Some(api_url) = &notification.subject.url {
        if let Some(rest) = api_url.strip_prefix("https://api.github.com/repos/") {
            // rest = "{owner}/{repo}/{issues|pulls}/{n}" or
            // "{owner}/{repo}/{commits}/{sha}".
            let translated = rest.replacen("/pulls/", "/pull/", 1);
            return format!("https://github.com/{}", translated);
        }
        return api_url.clone();
    }
    notification.repository.html_url.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_notification(api_url: Option<&str>) -> GitHubNotification {
        GitHubNotification {
            id: "123".to_string(),
            unread: true,
            reason: "mention".to_string(),
            updated_at: Utc::now(),
            last_read_at: None,
            subject: NotificationSubject {
                title: "Test".to_string(),
                kind: "Issue".to_string(),
                url: api_url.map(|s| s.to_string()),
                latest_comment_url: None,
            },
            repository: NotificationRepository {
                id: 1,
                name: "repo".to_string(),
                full_name: "octo/repo".to_string(),
                html_url: "https://github.com/octo/repo".to_string(),
                private: false,
            },
        }
    }

    #[test]
    fn build_html_url_translates_issue_api_url() {
        let n = make_notification(Some(
            "https://api.github.com/repos/octo/repo/issues/42",
        ));
        assert_eq!(build_html_url(&n), "https://github.com/octo/repo/issues/42");
    }

    #[test]
    fn build_html_url_translates_pull_request_api_url() {
        // The notifications API uses the plural `pulls` segment, but the web
        // UI is at `/pull/{n}`. Forgetting to translate causes 404s.
        let n = make_notification(Some(
            "https://api.github.com/repos/octo/repo/pulls/7",
        ));
        assert_eq!(build_html_url(&n), "https://github.com/octo/repo/pull/7");
    }

    #[test]
    fn build_html_url_falls_back_to_repo_when_subject_url_missing() {
        let n = make_notification(None);
        assert_eq!(build_html_url(&n), "https://github.com/octo/repo");
    }

    #[test]
    fn build_html_url_passes_through_unexpected_host() {
        // Defensive: a GHES instance would surface a different host. We
        // forward the URL untouched rather than mangle it.
        let url = "https://ghe.example.com/api/v3/repos/o/r/issues/1";
        let n = make_notification(Some(url));
        assert_eq!(build_html_url(&n), url);
    }
}
