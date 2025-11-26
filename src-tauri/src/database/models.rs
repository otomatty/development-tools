//! Database models
//!
//! This module defines the data structures that map to database tables.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// User model - represents a GitHub user
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i64,
    pub github_id: i64,
    pub username: String,
    pub avatar_url: Option<String>,
    #[serde(skip_serializing)]
    pub access_token_encrypted: String,
    #[serde(skip_serializing)]
    pub refresh_token_encrypted: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User statistics model - gamification data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStats {
    pub id: i64,
    pub user_id: i64,
    pub total_xp: i32,
    pub current_level: i32,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub last_activity_date: Option<NaiveDate>,
    pub total_commits: i32,
    pub total_prs: i32,
    pub total_reviews: i32,
    pub total_issues: i32,
    pub updated_at: DateTime<Utc>,
}

impl Default for UserStats {
    fn default() -> Self {
        Self {
            id: 0,
            user_id: 0,
            total_xp: 0,
            current_level: 1,
            current_streak: 0,
            longest_streak: 0,
            last_activity_date: None,
            total_commits: 0,
            total_prs: 0,
            total_reviews: 0,
            total_issues: 0,
            updated_at: Utc::now(),
        }
    }
}

/// Badge rarity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BadgeRarity {
    Bronze,
    Silver,
    Gold,
    Platinum,
}

/// Badge type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BadgeType {
    Milestone,
    Streak,
    Collaboration,
    Quality,
    Challenge,
}

impl std::fmt::Display for BadgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BadgeType::Milestone => write!(f, "milestone"),
            BadgeType::Streak => write!(f, "streak"),
            BadgeType::Collaboration => write!(f, "collaboration"),
            BadgeType::Quality => write!(f, "quality"),
            BadgeType::Challenge => write!(f, "challenge"),
        }
    }
}

/// Badge model - earned achievements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    pub id: i64,
    pub user_id: i64,
    pub badge_type: String,
    pub badge_id: String,
    pub earned_at: DateTime<Utc>,
}

/// Challenge type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChallengeType {
    Daily,
    Weekly,
}

impl std::fmt::Display for ChallengeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChallengeType::Daily => write!(f, "daily"),
            ChallengeType::Weekly => write!(f, "weekly"),
        }
    }
}

/// Challenge status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChallengeStatus {
    Active,
    Completed,
    Failed,
}

impl std::fmt::Display for ChallengeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChallengeStatus::Active => write!(f, "active"),
            ChallengeStatus::Completed => write!(f, "completed"),
            ChallengeStatus::Failed => write!(f, "failed"),
        }
    }
}

/// Target metric types for challenges
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TargetMetric {
    Commits,
    PullRequests,
    Reviews,
    Issues,
}

impl std::fmt::Display for TargetMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetMetric::Commits => write!(f, "commits"),
            TargetMetric::PullRequests => write!(f, "prs"),
            TargetMetric::Reviews => write!(f, "reviews"),
            TargetMetric::Issues => write!(f, "issues"),
        }
    }
}

/// Challenge model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Challenge {
    pub id: i64,
    pub user_id: i64,
    pub challenge_type: String,
    pub target_metric: String,
    pub target_value: i32,
    pub current_value: i32,
    pub reward_xp: i32,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub status: String,
    pub completed_at: Option<DateTime<Utc>>,
}

/// XP action types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum XpActionType {
    Commit,
    PullRequestCreated,
    PullRequestMerged,
    IssueCreated,
    IssueClosed,
    Review,
    StarReceived,
    StreakBonus,
    ChallengeCompleted,
}

impl XpActionType {
    /// Get the XP value for this action type
    pub fn xp_value(&self) -> i32 {
        match self {
            XpActionType::Commit => 10,
            XpActionType::PullRequestCreated => 30,
            XpActionType::PullRequestMerged => 50,
            XpActionType::IssueCreated => 15,
            XpActionType::IssueClosed => 40,
            XpActionType::Review => 25,
            XpActionType::StarReceived => 5,
            XpActionType::StreakBonus => 20,
            XpActionType::ChallengeCompleted => 0, // Dynamic based on challenge
        }
    }
}

impl std::fmt::Display for XpActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XpActionType::Commit => write!(f, "commit"),
            XpActionType::PullRequestCreated => write!(f, "pr_created"),
            XpActionType::PullRequestMerged => write!(f, "pr_merged"),
            XpActionType::IssueCreated => write!(f, "issue_created"),
            XpActionType::IssueClosed => write!(f, "issue_closed"),
            XpActionType::Review => write!(f, "review"),
            XpActionType::StarReceived => write!(f, "star_received"),
            XpActionType::StreakBonus => write!(f, "streak_bonus"),
            XpActionType::ChallengeCompleted => write!(f, "challenge_completed"),
        }
    }
}

/// XP history entry model
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

/// Activity cache entry model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityCache {
    pub id: i64,
    pub user_id: i64,
    pub data_type: String,
    pub data_json: String,
    pub fetched_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Cache data types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheDataType {
    ContributionGraph,
    UserProfile,
    Repositories,
    RecentActivity,
}

impl std::fmt::Display for CacheDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheDataType::ContributionGraph => write!(f, "contribution_graph"),
            CacheDataType::UserProfile => write!(f, "user_profile"),
            CacheDataType::Repositories => write!(f, "repositories"),
            CacheDataType::RecentActivity => write!(f, "recent_activity"),
        }
    }
}

/// Level calculation utilities
pub mod level {
    /// Calculate XP required for a specific level
    ///
    /// Formula: XP = 50 * (level - 1)^2
    pub fn xp_for_level(level: u32) -> u32 {
        if level <= 1 {
            return 0;
        }
        50 * (level - 1).pow(2)
    }

    /// Calculate level from total XP
    pub fn level_from_xp(total_xp: u32) -> u32 {
        if total_xp == 0 {
            return 1;
        }
        let level = ((total_xp as f64 / 50.0).sqrt() + 1.0).floor() as u32;
        level.min(100) // Max level is 100
    }

    /// Calculate XP needed for next level
    pub fn xp_to_next_level(current_xp: u32) -> u32 {
        let current_level = level_from_xp(current_xp);
        if current_level >= 100 {
            return 0;
        }
        let next_level_xp = xp_for_level(current_level + 1);
        next_level_xp.saturating_sub(current_xp)
    }

    /// Calculate progress percentage to next level
    pub fn progress_to_next_level(current_xp: u32) -> f32 {
        let current_level = level_from_xp(current_xp);
        if current_level >= 100 {
            return 100.0;
        }

        let current_level_xp = xp_for_level(current_level);
        let next_level_xp = xp_for_level(current_level + 1);
        let level_range = next_level_xp - current_level_xp;

        if level_range == 0 {
            return 100.0;
        }

        let progress = current_xp - current_level_xp;
        (progress as f32 / level_range as f32) * 100.0
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_xp_for_level() {
            assert_eq!(xp_for_level(1), 0);
            assert_eq!(xp_for_level(2), 50);
            assert_eq!(xp_for_level(3), 200);
            assert_eq!(xp_for_level(10), 4050);
        }

        #[test]
        fn test_level_from_xp() {
            assert_eq!(level_from_xp(0), 1);
            assert_eq!(level_from_xp(49), 1);
            assert_eq!(level_from_xp(50), 2);
            assert_eq!(level_from_xp(199), 2);
            assert_eq!(level_from_xp(200), 3);
            assert_eq!(level_from_xp(4050), 10);
        }

        #[test]
        fn test_max_level() {
            assert_eq!(level_from_xp(999999), 100);
        }

        #[test]
        fn test_xp_to_next_level() {
            assert_eq!(xp_to_next_level(0), 50);
            assert_eq!(xp_to_next_level(50), 150);
        }

        #[test]
        fn test_progress_to_next_level() {
            assert_eq!(progress_to_next_level(0), 0.0);
            assert_eq!(progress_to_next_level(25), 50.0);
            assert_eq!(progress_to_next_level(50), 0.0); // At level 2, progress resets
        }
    }
}

