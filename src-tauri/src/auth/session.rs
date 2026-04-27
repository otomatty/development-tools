//! Session lifecycle helpers
//!
//! Centralizes the response to GitHub `401 Unauthorized` errors so every
//! command — both frontend-invoked and scheduler-driven — converges on the
//! same behavior:
//!
//! 1. Clear the stored access token (`TokenManager::logout`) so subsequent
//!    calls don't keep retrying with a known-bad credential.
//! 2. Emit `auth-expired` so the frontend can surface a re-login banner /
//!    modal and stop firing API requests until the user re-authenticates.
//!
//! See Issue #181.

use tauri::{AppHandle, Emitter};

use crate::commands::auth::AppState;
use crate::github::client::GitHubError;

/// Tauri event name emitted whenever the backend detects that the current
/// GitHub token is no longer valid (revoked, expired, or otherwise rejected).
pub const AUTH_EXPIRED_EVENT: &str = "auth-expired";

/// Reason codes carried in [`AuthExpiredEvent::reason`].
///
/// Kept as `&'static str` rather than an enum so additional sources (sync
/// scheduler, startup validation, individual commands) can attach their own
/// context without forcing a breaking change in the frontend payload.
pub mod reasons {
    /// A GitHub REST/GraphQL call returned `401 Unauthorized`.
    pub const GITHUB_UNAUTHORIZED: &str = "github_unauthorized";
    /// The startup `validate_token` probe failed.
    pub const STARTUP_VALIDATION_FAILED: &str = "startup_validation_failed";
    /// The user explicitly invoked `validate_token` and it returned false.
    pub const MANUAL_VALIDATION_FAILED: &str = "manual_validation_failed";
    /// The background sync scheduler observed a 401 from `run_github_sync`.
    pub const SCHEDULER_UNAUTHORIZED: &str = "scheduler_unauthorized";
}

/// Payload for the `auth-expired` event consumed by the frontend.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuthExpiredEvent {
    /// Machine-readable reason code (see [`reasons`]).
    pub reason: String,
    /// Human-readable Japanese message safe to show in the UI verbatim.
    pub message: String,
}

impl AuthExpiredEvent {
    /// Build the default Japanese message paired with a reason code.
    pub fn new(reason: &str) -> Self {
        Self {
            reason: reason.to_string(),
            message:
                "GitHub の認証が切れました。再度ログインして連携をやり直してください。"
                    .to_string(),
        }
    }
}

/// Clear the stored credential and notify the frontend that the session is
/// no longer valid.
///
/// Idempotent: callers that hit multiple 401s in a row (e.g. parallel API
/// fan-out inside a single sync) can invoke this repeatedly without ill
/// effect — `TokenManager::logout` is a no-op once the user has been cleared,
/// and the frontend listener tolerates duplicate events.
pub async fn handle_unauthorized(app: &AppHandle, state: &AppState, reason: &str) {
    if let Err(e) = state.token_manager.logout().await {
        // Token cleanup is best-effort: we still want to emit the event so
        // the UI surfaces the re-login prompt even if the DB write failed.
        // TODO: [INFRA] logクレートに置換（ログ基盤整備時に一括対応）
        eprintln!(
            "Auth: failed to clear stored tokens during 401 handling ({}): {}",
            reason, e
        );
    }

    let payload = AuthExpiredEvent::new(reason);
    if let Err(e) = app.emit(AUTH_EXPIRED_EVENT, &payload) {
        // TODO: [INFRA] logクレートに置換（ログ基盤整備時に一括対応）
        eprintln!(
            "Auth: failed to emit '{}' event ({}): {}",
            AUTH_EXPIRED_EVENT, reason, e
        );
    }
}

/// Bridge a typed [`GitHubError`] result into the `Result<T, String>` that
/// Tauri commands return, while transparently triggering the auth-expired
/// flow whenever the GitHub API responded with `401 Unauthorized`.
///
/// Use this at every call-site that previously did `.map_err(|e| e.to_string())`
/// on a `GitHubResult<T>`. Non-auth errors are stringified unchanged so the
/// existing scheduler / UI error-classification logic keeps working.
pub async fn map_github_result<T>(
    app: &AppHandle,
    state: &AppState,
    result: Result<T, GitHubError>,
) -> Result<T, String> {
    match result {
        Ok(v) => Ok(v),
        Err(GitHubError::Unauthorized) => {
            handle_unauthorized(app, state, reasons::GITHUB_UNAUTHORIZED).await;
            Err(GitHubError::Unauthorized.to_string())
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Detect whether a stringified error originated from a GitHub 401 response.
///
/// `run_github_sync` (and other helpers used by the scheduler) flatten typed
/// errors into `String`, so the scheduler has to match on the formatted
/// message. We anchor on the canonical [`GitHubError::Unauthorized`] Display
/// output ("Authentication failed") and also accept the explicit
/// "Unauthorized" tag emitted by [`map_github_result`].
pub fn classify_unauthorized(err_msg: &str) -> bool {
    let lower = err_msg.to_lowercase();
    lower.contains("authentication failed") || lower.contains("unauthorized")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_expired_event_carries_reason() {
        let event = AuthExpiredEvent::new(reasons::GITHUB_UNAUTHORIZED);
        assert_eq!(event.reason, "github_unauthorized");
        assert!(event.message.contains("再"));
    }

    #[test]
    fn auth_expired_event_serializes_camel_case() {
        let event = AuthExpiredEvent::new(reasons::SCHEDULER_UNAUTHORIZED);
        let json = serde_json::to_string(&event).expect("serialize");
        // camelCase rename means the field stays "reason" (already lowercase)
        // and "message" stays "message"; the assertion checks the payload
        // shape the frontend listener expects.
        assert!(json.contains("\"reason\":\"scheduler_unauthorized\""));
        assert!(json.contains("\"message\":"));
    }

    #[test]
    fn classify_unauthorized_recognizes_github_error_display() {
        let msg = format!("{}", GitHubError::Unauthorized);
        assert!(classify_unauthorized(&msg));
    }

    #[test]
    fn classify_unauthorized_recognizes_explicit_tag() {
        assert!(classify_unauthorized(
            "Unauthorized: GitHub token is invalid or revoked"
        ));
        assert!(classify_unauthorized("UNAUTHORIZED"));
    }

    #[test]
    fn classify_unauthorized_rejects_unrelated() {
        assert!(!classify_unauthorized("rate limit exceeded. resets at 123"));
        assert!(!classify_unauthorized("network error"));
        assert!(!classify_unauthorized("Resource not found: /user"));
    }

    #[test]
    fn auth_expired_event_constants_are_distinct() {
        // Defensive: prevent accidental shadowing if reasons are renamed.
        let names = [
            reasons::GITHUB_UNAUTHORIZED,
            reasons::STARTUP_VALIDATION_FAILED,
            reasons::MANUAL_VALIDATION_FAILED,
            reasons::SCHEDULER_UNAUTHORIZED,
        ];
        let mut seen = std::collections::HashSet::new();
        for n in names {
            assert!(seen.insert(n), "duplicate reason code: {}", n);
        }
    }
}
