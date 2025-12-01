//! XP-related models
//!
//! Note: XPやカウント系の値は意味的にはu32（符号なし整数）が適切ですが、
//! SQLiteのINTEGER型が符号あり整数であり、sqlxがi32としてマッピングするため、
//! DB層との整合性を保つためにi32で統一しています。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// XP source types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum XpSource {
    Commit,
    PullRequest,
    Review,
    Issue,
    StreakBonus,
    ChallengeComplete,
    BadgeEarned,
    DailyLogin,
}

impl std::fmt::Display for XpSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XpSource::Commit => write!(f, "commit"),
            XpSource::PullRequest => write!(f, "pull_request"),
            XpSource::Review => write!(f, "review"),
            XpSource::Issue => write!(f, "issue"),
            XpSource::StreakBonus => write!(f, "streak_bonus"),
            XpSource::ChallengeComplete => write!(f, "challenge_complete"),
            XpSource::BadgeEarned => write!(f, "badge_earned"),
            XpSource::DailyLogin => write!(f, "daily_login"),
        }
    }
}

impl From<String> for XpSource {
    fn from(s: String) -> Self {
        match s.as_str() {
            "commit" => XpSource::Commit,
            "pull_request" => XpSource::PullRequest,
            "review" => XpSource::Review,
            "issue" => XpSource::Issue,
            "streak_bonus" => XpSource::StreakBonus,
            "challenge_complete" => XpSource::ChallengeComplete,
            "badge_earned" => XpSource::BadgeEarned,
            "daily_login" => XpSource::DailyLogin,
            _ => XpSource::Commit,
        }
    }
}

/// XP history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XpHistoryEntry {
    pub id: i64,
    pub user_id: i64,
    pub action_type: String,
    pub xp_amount: i32,
    pub description: Option<String>,
    pub github_event_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// XP action types for database
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XpActionType {
    Commit,
    PullRequest,
    PullRequestMerged,
    Review,
    Issue,
    IssueClosed,
    StreakBonus,
    Star,
}

impl XpActionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            XpActionType::Commit => "commit",
            XpActionType::PullRequest => "pull_request",
            XpActionType::PullRequestMerged => "pull_request_merged",
            XpActionType::Review => "review",
            XpActionType::Issue => "issue",
            XpActionType::IssueClosed => "issue_closed",
            XpActionType::StreakBonus => "streak_bonus",
            XpActionType::Star => "star",
        }
    }
}

impl std::fmt::Display for XpActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// XP for a commit
pub const COMMIT_XP: i32 = 10;
/// XP for a pull request
pub const PR_XP: i32 = 25;
/// XP for a code review
pub const REVIEW_XP: i32 = 15;
/// XP for an issue
pub const ISSUE_XP: i32 = 10;
/// XP for daily login
pub const DAILY_LOGIN_XP: i32 = 5;
/// XP bonus multiplier for streak (percentage)
pub const STREAK_BONUS_PERCENT: i32 = 10;
/// Maximum streak bonus multiplier
pub const MAX_STREAK_BONUS_PERCENT: i32 = 100;

/// Calculate XP with streak bonus
pub fn with_streak_bonus(base_xp: i32, streak: i32) -> i32 {
    let bonus_percent = (streak * STREAK_BONUS_PERCENT).min(MAX_STREAK_BONUS_PERCENT);
    base_xp + (base_xp * bonus_percent / 100)
}

/// Calculate total XP from activities
pub fn calculate_activity_xp(commits: i32, prs: i32, reviews: i32, issues: i32) -> i32 {
    commits * COMMIT_XP + prs * PR_XP + reviews * REVIEW_XP + issues * ISSUE_XP
}

/// XP breakdown for sync result
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct XpBreakdown {
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

impl XpBreakdown {
    pub fn calculate(
        commits: i32,
        prs_created: i32,
        prs_merged: i32,
        issues_created: i32,
        issues_closed: i32,
        reviews: i32,
        stars: i32,
        streak: i32,
    ) -> Self {
        let commits_xp = commits * 10;
        let prs_created_xp = prs_created * 25;
        let prs_merged_xp = prs_merged * 50;
        let issues_created_xp = issues_created * 5;
        let issues_closed_xp = issues_closed * 10;
        let reviews_xp = reviews * 15;
        let stars_xp = stars * 5;

        let base_total = commits_xp
            + prs_created_xp
            + prs_merged_xp
            + issues_created_xp
            + issues_closed_xp
            + reviews_xp
            + stars_xp;

        let streak_bonus_xp = if streak > 0 {
            (base_total * streak.min(10)) / 100
        } else {
            0
        };

        let total_xp = base_total + streak_bonus_xp;

        Self {
            commits_xp,
            prs_created_xp,
            prs_merged_xp,
            issues_created_xp,
            issues_closed_xp,
            reviews_xp,
            stars_xp,
            streak_bonus_xp,
            total_xp,
        }
    }
}

/// XP values module (for backward compatibility)
pub mod xp {
    pub use super::{
        calculate_activity_xp, with_streak_bonus, XpActionType, XpBreakdown, COMMIT_XP,
        DAILY_LOGIN_XP, ISSUE_XP, MAX_STREAK_BONUS_PERCENT, PR_XP, REVIEW_XP, STREAK_BONUS_PERCENT,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streak_bonus() {
        // No streak
        assert_eq!(with_streak_bonus(100, 0), 100);
        // 5 day streak = 50% bonus
        assert_eq!(with_streak_bonus(100, 5), 150);
        // 10 day streak = 100% bonus (max)
        assert_eq!(with_streak_bonus(100, 10), 200);
        // 15 day streak = still 100% bonus (capped)
        assert_eq!(with_streak_bonus(100, 15), 200);
    }

    #[test]
    fn test_activity_xp() {
        let xp = calculate_activity_xp(10, 2, 5, 3);
        // 10 commits * 10 + 2 PRs * 25 + 5 reviews * 15 + 3 issues * 10
        // = 100 + 50 + 75 + 30 = 255
        assert_eq!(xp, 255);
    }
}
