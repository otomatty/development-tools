//! Gamification commands for Tauri
//!
//! These commands handle the gamification features: XP, levels, badges, etc.

use chrono::{DateTime, Duration, Utc};
use tauri::{command, AppHandle, State};

use super::auth::AppState;
use crate::auth::map_github_result;
use crate::database::{badge, level, xp::XpBreakdown, Badge, UserStats, XpHistoryEntry};
use crate::github::GitHubClient;

/// Level info for frontend
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelInfo {
    pub current_level: i32,
    pub total_xp: i32,
    pub xp_for_current_level: i32,
    pub xp_for_next_level: i32,
    pub xp_to_next_level: i32,
    pub progress_percent: f32,
}

/// Get level info for current user
#[command]
pub async fn get_level_info(state: State<'_, AppState>) -> Result<Option<LevelInfo>, String> {
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(u) = user {
        let stats = state
            .db
            .get_user_stats(u.id)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(s) = stats {
            let total_xp = s.total_xp;
            let current_level = level::level_from_xp(total_xp);

            Ok(Some(LevelInfo {
                current_level,
                total_xp,
                xp_for_current_level: level::xp_for_level(current_level),
                xp_for_next_level: level::xp_for_level(current_level + 1),
                xp_to_next_level: level::xp_to_next_level(total_xp),
                progress_percent: level::progress_to_next_level(total_xp),
            }))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

/// Add XP to current user (for testing/admin purposes)
#[command]
pub async fn add_xp(
    state: State<'_, AppState>,
    amount: i32,
    action_type: String,
    description: Option<String>,
) -> Result<UserStats, String> {
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Not logged in")?;

    // Record XP gain (no breakdown for manual XP additions)
    state
        .db
        .record_xp_gain(
            user.id,
            &action_type,
            amount,
            description.as_deref(),
            None,
            None,
        )
        .await
        .map_err(|e| e.to_string())?;

    // Update user stats
    state
        .db
        .add_xp(user.id, amount)
        .await
        .map_err(|e| e.to_string())
}

/// Get user's badges
#[command]
pub async fn get_badges(state: State<'_, AppState>) -> Result<Vec<Badge>, String> {
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Not logged in")?;

    state
        .db
        .get_user_badges(user.id)
        .await
        .map_err(|e| e.to_string())
}

/// Award a badge to current user
#[command]
pub async fn award_badge(
    state: State<'_, AppState>,
    badge_type: String,
    badge_id: String,
) -> Result<bool, String> {
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Not logged in")?;

    // Check if already has badge
    let has_badge = state
        .db
        .has_badge(user.id, &badge_id)
        .await
        .map_err(|e| e.to_string())?;

    if has_badge {
        return Ok(false);
    }

    state
        .db
        .award_badge(user.id, &badge_type, &badge_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(true)
}

/// Get recent XP history
#[command]
pub async fn get_xp_history(
    state: State<'_, AppState>,
    limit: Option<i32>,
) -> Result<Vec<XpHistoryEntry>, String> {
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Not logged in")?;

    state
        .db
        .get_recent_xp_history(user.id, limit.unwrap_or(10))
        .await
        .map_err(|e| e.to_string())
}

/// Badge definition for frontend
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgeDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub badge_type: String,
    pub rarity: String,
    pub icon: String,
}

/// Get all available badge definitions
#[command]
pub fn get_badge_definitions() -> Vec<BadgeDefinition> {
    badge::get_all_badge_definitions()
        .into_iter()
        .map(|def| BadgeDefinition {
            id: def.id,
            name: def.name,
            description: def.description,
            badge_type: def.badge_type,
            rarity: def.rarity,
            icon: def.icon,
        })
        .collect()
}

// ============================================================================
// Past-year XP recalculation (Issue #194 / Audit §6.2 / §8 G-13)
//
// `contributionCalendar` is GraphQL-capped to ~1 year, so this is the
// maximum window we can ever recalculate. The user-facing rate-limit
// guard is intentionally short (1 hour): the GraphQL call itself is
// cheap, but recalculation churns `xp_history` and the comparison UI is
// not useful to re-run rapidly. Concurrent invocations are serialised
// through `AppState::sync_lock` so a manual recalc can't race a
// scheduler-driven `run_github_sync`.
// ============================================================================

/// Maximum span for `contributionCalendar` — GitHub rejects `from` values
/// older than this. Mirrors the cap documented in
/// `XpBreakdown::calculate` / Issue #194.
const RECALC_MAX_WINDOW_DAYS: i64 = 365;

/// Minimum interval between recalculations per user. Short enough that
/// fixing a mistake is easy, long enough that the button can't be
/// drum-rolled into mass `xp_history` rows.
const RECALC_RATE_LIMIT_MINUTES: i64 = 60;

/// XP breakdown surface for the recalculation result.
///
/// Mirrors `XpBreakdownResult` from `commands::github` so the frontend
/// can render the same breakdown widget without depending on
/// gamification-internal types. Defined locally to avoid a cross-command
/// import cycle.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecalculatedXpBreakdown {
    pub commits_xp: i32,
    pub prs_created_xp: i32,
    pub prs_merged_xp: i32,
    pub issues_created_xp: i32,
    pub issues_closed_xp: i32,
    pub reviews_xp: i32,
    pub stars_xp: i32,
    pub streak_bonus_xp: i32,
    pub total_xp: i32,
}

impl From<&XpBreakdown> for RecalculatedXpBreakdown {
    fn from(b: &XpBreakdown) -> Self {
        Self {
            commits_xp: b.commits_xp,
            prs_created_xp: b.prs_created_xp,
            prs_merged_xp: b.prs_merged_xp,
            issues_created_xp: b.issues_created_xp,
            issues_closed_xp: b.issues_closed_xp,
            reviews_xp: b.reviews_xp,
            stars_xp: b.stars_xp,
            streak_bonus_xp: b.streak_bonus_xp,
            total_xp: b.total_xp,
        }
    }
}

/// Result returned to the frontend after a successful recalculation.
///
/// Contains both the freshly computed total and the existing live total
/// for the same window so the UI can render the "before / after" diff
/// required by Issue #194's DoD ("計算前後の値を比較表示できる"). The
/// recalculated row is persisted in `xp_history` with `source =
/// 'recalculated'` (NOT folded into `user_stats.total_xp`) — its row id
/// is returned in `recalculation_history_id` so callers can drill into
/// it from the XP history page.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecalculationResult {
    /// Inclusive lower bound of the window (RFC3339).
    pub since: String,
    /// Exclusive upper bound of the window — the moment the recalc ran
    /// (RFC3339).
    pub until: String,
    /// Total XP the recalculation produced for the window.
    pub recalculated_total_xp: i32,
    /// Per-category breakdown driving `recalculated_total_xp`.
    pub recalculated_breakdown: RecalculatedXpBreakdown,
    /// Sum of `xp_history.xp_amount` rows with `source = 'live'` that
    /// were created within `[since, until]`. Used to render the diff.
    pub previous_live_total_xp_in_window: i32,
    /// `recalculated_total_xp - previous_live_total_xp_in_window`. Can
    /// be negative when live XP was higher (e.g. legacy XP rules
    /// granted more than the current ruleset).
    pub xp_diff: i32,
    /// `xp_history.id` of the inserted `source = 'recalculated'` row.
    pub recalculation_history_id: i64,
    /// Number of days the recalculation window spanned, for UI labels.
    pub window_days: i64,
    /// Raw GitHub category totals over the window (commits / PRs /
    /// issues / reviews). Exposed so the UI can explain *why* the
    /// breakdown looks the way it does without a second API call.
    pub contributions: RecalcContributionTotals,
    /// Categories the recalculation cannot fill from
    /// `contributionsCollection` alone (PR merges, issues closed,
    /// stars). Surfaced so the UI can disclose the gap rather than
    /// silently treating them as zero.
    pub uncovered_categories: Vec<&'static str>,
}

/// GitHub-side category totals consumed by `XpBreakdown::calculate`.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecalcContributionTotals {
    pub commits: i32,
    pub pull_requests: i32,
    pub issues: i32,
    pub reviews: i32,
    pub current_streak: i32,
}

/// Recalculate XP for the past `contributionCalendar` window
/// (≤ 1 year). Inserts an audit row in `xp_history` with
/// `source = 'recalculated'` and returns the comparison against the
/// existing live XP earned in the same window.
///
/// `since` is an optional RFC3339 timestamp; when omitted the full year
/// window is used. Out-of-range values (in the future, or older than
/// 1 year) are rejected with a descriptive error.
#[command]
pub async fn recalculate_xp_history(
    app: AppHandle,
    state: State<'_, AppState>,
    since: Option<String>,
) -> Result<RecalculationResult, String> {
    let user = state
        .token_manager
        .get_current_user()
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Not logged in")?;

    let now = Utc::now();
    let max_lookback = now - Duration::days(RECALC_MAX_WINDOW_DAYS);

    let parsed_since = match since.as_deref() {
        Some(s) => DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| format!("Invalid `since` timestamp ({}): {}", s, e))?,
        None => max_lookback,
    };

    if parsed_since > now {
        return Err("`since` cannot be in the future".to_string());
    }
    if parsed_since < max_lookback {
        return Err(format!(
            "`since` must be within the last {} days (contributionCalendar limit)",
            RECALC_MAX_WINDOW_DAYS
        ));
    }

    // Hold the sync lock so a manual recalc can't race a scheduler-driven
    // `run_github_sync` *and* so two concurrent recalc calls can't both
    // pass the rate-limit guard before either has written its row. The
    // rate-limit check therefore runs INSIDE the locked section — moving
    // it before `lock()` would re-introduce the TOCTOU window flagged in
    // PR #217 review.
    let _guard = state.sync_lock.lock().await;

    // Re-take the clock after acquiring the lock; `lock()` itself can
    // wait for the scheduler-driven sync to finish, so the upper bound
    // of the window must reflect that wait.
    let recalc_at = Utc::now();

    // Re-derive the 365-day floor against `recalc_at` and clamp
    // `parsed_since` forward if the lock wait pushed the original lower
    // bound past the contributionCalendar cap. Without this, a long
    // lock wait (e.g. behind a scheduler-driven sync) could send a
    // `[from, to]` span > 365 days to GitHub and trigger an intermittent
    // GraphQL rejection for a request that was valid at submit time.
    let effective_max_lookback = recalc_at - Duration::days(RECALC_MAX_WINDOW_DAYS);
    let parsed_since = if parsed_since < effective_max_lookback {
        effective_max_lookback
    } else {
        parsed_since
    };

    if let Some(last) = state
        .db
        .get_last_recalculation_at(user.id)
        .await
        .map_err(|e| e.to_string())?
    {
        let next_allowed = last + Duration::minutes(RECALC_RATE_LIMIT_MINUTES);
        if next_allowed > recalc_at {
            let wait_minutes = (next_allowed - recalc_at).num_minutes().max(1);
            return Err(format!(
                "XP 再計算は{}分に1回までです。次回は約{}分後 ({}) に実行できます。",
                RECALC_RATE_LIMIT_MINUTES,
                wait_minutes,
                next_allowed.to_rfc3339()
            ));
        }
    }

    let token = state
        .token_manager
        .get_access_token()
        .await
        .map_err(|e| e.to_string())?;

    let client = GitHubClient::new(token);
    let contributions = map_github_result(
        &app,
        state.inner(),
        client
            .get_contribution_calendar_window(&user.username, Some(parsed_since), Some(recalc_at))
            .await,
    )
    .await?;

    // Use the live current_streak so the recalculation's streak bonus
    // matches what `run_github_sync` would award today. Streak history
    // is not part of `contributionsCollection`, so we deliberately read
    // it from local state rather than re-deriving it.
    let current_streak = state
        .db
        .get_user_stats(user.id)
        .await
        .map_err(|e| e.to_string())?
        .map(|s| s.current_streak)
        .unwrap_or(0);

    // `contributionsCollection` only exposes counts for the four
    // categories below — PR merges, issues closed, and stars require
    // separate (Search API) requests that we deliberately avoid here
    // to keep the recalculation cheap and rate-limit-safe. The UI
    // surfaces the gap via `uncovered_categories`.
    let commits = clamp_to_u64(contributions.total_commit_contributions);
    let prs = clamp_to_u64(contributions.total_pull_request_contributions);
    let issues = clamp_to_u64(contributions.total_issue_contributions);
    let reviews = clamp_to_u64(contributions.total_pull_request_review_contributions);

    let breakdown = XpBreakdown::calculate(
        commits,
        prs,
        /* prs_merged */ 0,
        issues,
        /* issues_closed */ 0,
        reviews,
        /* stars */ 0,
        current_streak,
    );

    let window_days = (recalc_at - parsed_since).num_days().max(0);
    let description = format!(
        "過去{}日分のXP再計算 (commits={}, prs={}, issues={}, reviews={}, streak={})",
        window_days,
        contributions.total_commit_contributions,
        contributions.total_pull_request_contributions,
        contributions.total_issue_contributions,
        contributions.total_pull_request_review_contributions,
        current_streak,
    );

    let recalc_id = state
        .db
        .record_xp_recalculation(
            user.id,
            breakdown.total_xp,
            Some(&description),
            Some(&breakdown),
        )
        .await
        .map_err(|e| e.to_string())?;

    // Bound the live-XP comparison query to the exact `[since, recalc_at]`
    // window the result reports, so the "before" total never drifts even
    // if `run_github_sync` was queued behind our sync_lock and recorded a
    // row between `recalc_at` and this query — that row would belong to
    // the *next* window, not this one.
    let previous_live_total = state
        .db
        .get_xp_total_in_range(
            user.id,
            parsed_since,
            recalc_at,
            crate::database::repository::XP_HISTORY_SOURCE_LIVE,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(RecalculationResult {
        since: parsed_since.to_rfc3339(),
        until: recalc_at.to_rfc3339(),
        recalculated_total_xp: breakdown.total_xp,
        recalculated_breakdown: RecalculatedXpBreakdown::from(&breakdown),
        previous_live_total_xp_in_window: previous_live_total,
        xp_diff: breakdown.total_xp - previous_live_total,
        recalculation_history_id: recalc_id,
        window_days,
        contributions: RecalcContributionTotals {
            commits: contributions.total_commit_contributions,
            pull_requests: contributions.total_pull_request_contributions,
            issues: contributions.total_issue_contributions,
            reviews: contributions.total_pull_request_review_contributions,
            current_streak,
        },
        uncovered_categories: vec!["prs_merged", "issues_closed", "stars"],
    })
}

/// Clamp a possibly-negative `i32` count to `u64` so it can feed
/// `XpBreakdown::calculate`. Mirrors the helper in `commands::github`
/// but duplicated here to avoid pulling the entire github command
/// module into the gamification surface.
fn clamp_to_u64(value: i32) -> u64 {
    if value < 0 {
        0
    } else {
        value as u64
    }
}
