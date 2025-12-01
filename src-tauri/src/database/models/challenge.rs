//! Challenge-related models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Challenge type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

impl From<String> for ChallengeType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "weekly" => ChallengeType::Weekly,
            _ => ChallengeType::Daily,
        }
    }
}

/// Challenge requirement types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChallengeRequirement {
    Commits { count: i32 },
    Reviews { count: i32 },
    PullRequests { count: i32 },
    Issues { count: i32 },
    Streak { days: i32 },
}

/// Challenge model - active challenges
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
    pub status: String, // "active", "completed", "failed"
    pub completed_at: Option<DateTime<Utc>>,
}

impl Challenge {
    /// Check if challenge is completed
    pub fn is_completed(&self) -> bool {
        self.status == "completed"
    }

    /// Check if challenge is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.end_date && self.status == "active"
    }

    /// Check if challenge is active (not completed and not expired)
    pub fn is_active(&self) -> bool {
        self.status == "active" && !self.is_expired()
    }

    /// Get progress percentage (0-100)
    pub fn progress_percent(&self) -> f32 {
        if self.target_value == 0 {
            return 100.0;
        }
        ((self.current_value as f32 / self.target_value as f32) * 100.0).min(100.0)
    }
}

/// Template for generating challenges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeTemplate {
    pub id: String,
    pub title: String,
    pub description: String,
    pub challenge_type: ChallengeType,
    pub requirement: ChallengeRequirement,
    pub base_xp_reward: i32,
}

/// Challenge pool for generating new challenges
pub fn get_challenge_templates() -> Vec<ChallengeTemplate> {
    vec![
        // Daily challenges
        ChallengeTemplate {
            id: "daily_commits_3".to_string(),
            title: "Daily Grind".to_string(),
            description: "Make 3 commits today".to_string(),
            challenge_type: ChallengeType::Daily,
            requirement: ChallengeRequirement::Commits { count: 3 },
            base_xp_reward: 50,
        },
        ChallengeTemplate {
            id: "daily_commits_5".to_string(),
            title: "Commit Marathon".to_string(),
            description: "Make 5 commits today".to_string(),
            challenge_type: ChallengeType::Daily,
            requirement: ChallengeRequirement::Commits { count: 5 },
            base_xp_reward: 80,
        },
        ChallengeTemplate {
            id: "daily_commits_10".to_string(),
            title: "Commit Spree".to_string(),
            description: "Make 10 commits today".to_string(),
            challenge_type: ChallengeType::Daily,
            requirement: ChallengeRequirement::Commits { count: 10 },
            base_xp_reward: 150,
        },
        ChallengeTemplate {
            id: "daily_review_1".to_string(),
            title: "Code Reviewer".to_string(),
            description: "Complete 1 code review today".to_string(),
            challenge_type: ChallengeType::Daily,
            requirement: ChallengeRequirement::Reviews { count: 1 },
            base_xp_reward: 40,
        },
        ChallengeTemplate {
            id: "daily_pr_1".to_string(),
            title: "Pull Request Hero".to_string(),
            description: "Open a pull request today".to_string(),
            challenge_type: ChallengeType::Daily,
            requirement: ChallengeRequirement::PullRequests { count: 1 },
            base_xp_reward: 60,
        },
        ChallengeTemplate {
            id: "daily_issue_1".to_string(),
            title: "Issue Tracker".to_string(),
            description: "Create or close an issue today".to_string(),
            challenge_type: ChallengeType::Daily,
            requirement: ChallengeRequirement::Issues { count: 1 },
            base_xp_reward: 30,
        },
        // Weekly challenges
        ChallengeTemplate {
            id: "weekly_commits_10".to_string(),
            title: "Weekly Warrior".to_string(),
            description: "Make 10 commits this week".to_string(),
            challenge_type: ChallengeType::Weekly,
            requirement: ChallengeRequirement::Commits { count: 10 },
            base_xp_reward: 100,
        },
        ChallengeTemplate {
            id: "weekly_commits_25".to_string(),
            title: "Commit Champion".to_string(),
            description: "Make 25 commits this week".to_string(),
            challenge_type: ChallengeType::Weekly,
            requirement: ChallengeRequirement::Commits { count: 25 },
            base_xp_reward: 200,
        },
        ChallengeTemplate {
            id: "weekly_commits_50".to_string(),
            title: "Commit Legend".to_string(),
            description: "Make 50 commits this week".to_string(),
            challenge_type: ChallengeType::Weekly,
            requirement: ChallengeRequirement::Commits { count: 50 },
            base_xp_reward: 400,
        },
        ChallengeTemplate {
            id: "weekly_reviews_3".to_string(),
            title: "Review Routine".to_string(),
            description: "Complete 3 code reviews this week".to_string(),
            challenge_type: ChallengeType::Weekly,
            requirement: ChallengeRequirement::Reviews { count: 3 },
            base_xp_reward: 120,
        },
        ChallengeTemplate {
            id: "weekly_prs_3".to_string(),
            title: "PR Machine".to_string(),
            description: "Open 3 pull requests this week".to_string(),
            challenge_type: ChallengeType::Weekly,
            requirement: ChallengeRequirement::PullRequests { count: 3 },
            base_xp_reward: 150,
        },
        ChallengeTemplate {
            id: "weekly_streak_7".to_string(),
            title: "Streak Keeper".to_string(),
            description: "Maintain a 7-day streak this week".to_string(),
            challenge_type: ChallengeType::Weekly,
            requirement: ChallengeRequirement::Streak { days: 7 },
            base_xp_reward: 250,
        },
    ]
}
