//! Database models
//!
//! This module defines the data structures that map to database tables.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

// ============================================
// User Settings
// ============================================

/// Notification method options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NotificationMethod {
    AppOnly,
    OsOnly,
    #[default]
    Both,
    None,
}

impl NotificationMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationMethod::AppOnly => "app_only",
            NotificationMethod::OsOnly => "os_only",
            NotificationMethod::Both => "both",
            NotificationMethod::None => "none",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "app_only" => NotificationMethod::AppOnly,
            "os_only" => NotificationMethod::OsOnly,
            "both" => NotificationMethod::Both,
            "none" => NotificationMethod::None,
            _ => NotificationMethod::Both, // default
        }
    }
}

/// User settings model - stores user preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSettings {
    pub id: i64,
    pub user_id: i64,
    
    // Notification settings
    pub notification_method: NotificationMethod,
    pub notify_xp_gain: bool,
    pub notify_level_up: bool,
    pub notify_badge_earned: bool,
    pub notify_streak_update: bool,
    pub notify_streak_milestone: bool,
    
    // Sync settings
    pub sync_interval_minutes: i32,
    pub background_sync: bool,
    pub sync_on_startup: bool,
    
    // Appearance settings
    pub animations_enabled: bool,
    
    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            id: 0,
            user_id: 0,
            notification_method: NotificationMethod::Both,
            notify_xp_gain: true,
            notify_level_up: true,
            notify_badge_earned: true,
            notify_streak_update: true,
            notify_streak_milestone: true,
            sync_interval_minutes: 60,
            background_sync: true,
            sync_on_startup: true,
            animations_enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

/// Settings defaults as constants
pub mod settings_defaults {
    use super::NotificationMethod;

    pub const NOTIFICATION_METHOD: NotificationMethod = NotificationMethod::Both;
    pub const NOTIFY_XP_GAIN: bool = true;
    pub const NOTIFY_LEVEL_UP: bool = true;
    pub const NOTIFY_BADGE_EARNED: bool = true;
    pub const NOTIFY_STREAK_UPDATE: bool = true;
    pub const NOTIFY_STREAK_MILESTONE: bool = true;
    pub const SYNC_INTERVAL_MINUTES: i32 = 60;
    pub const BACKGROUND_SYNC: bool = true;
    pub const SYNC_ON_STARTUP: bool = true;
    pub const ANIMATIONS_ENABLED: bool = true;

    /// Available sync interval options (minutes, label)
    /// This is the single source of truth - frontend should fetch this via command
    pub const SYNC_INTERVALS: &[(i32, &str)] = &[
        (5, "5分"),
        (15, "15分"),
        (30, "30分"),
        (60, "1時間"),
        (180, "3時間"),
        (0, "手動のみ"),
    ];
}

/// Database info for display in settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseInfo {
    pub path: String,
    pub size_bytes: u64,
    pub cache_size_bytes: u64,
}

/// Result of clearing cache
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearCacheResult {
    pub cleared_entries: i32,
    pub freed_bytes: u64,
}

/// Data export structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportData {
    pub exported_at: String,
    pub version: String,
    pub user: ExportUser,
    pub stats: UserStats,
    pub badges: Vec<Badge>,
    pub xp_history: Vec<XpHistoryEntry>,
}

/// User info for export
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportUser {
    pub github_id: i64,
    pub username: String,
}

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

        // Additional test cases for Issue #7
        #[test]
        fn test_level_table_values() {
            // Verify level table from PRD
            // Level 1: 0 XP
            assert_eq!(xp_for_level(1), 0);
            // Level 5: 800 XP
            assert_eq!(xp_for_level(5), 800);
            // Level 10: 4,050 XP
            assert_eq!(xp_for_level(10), 4050);
            // Level 25: 28,800 XP
            assert_eq!(xp_for_level(25), 28800);
            // Level 50: 120,050 XP
            assert_eq!(xp_for_level(50), 120050);
        }

        #[test]
        fn test_level_boundaries() {
            // Test edge cases at level boundaries
            assert_eq!(level_from_xp(49), 1);
            assert_eq!(level_from_xp(50), 2);
            assert_eq!(level_from_xp(51), 2);
            
            assert_eq!(level_from_xp(199), 2);
            assert_eq!(level_from_xp(200), 3);
            assert_eq!(level_from_xp(201), 3);
        }

        #[test]
        fn test_xp_to_next_level_edge_cases() {
            // At level 1, need 50 XP to reach level 2
            assert_eq!(xp_to_next_level(0), 50);
            assert_eq!(xp_to_next_level(25), 25);
            assert_eq!(xp_to_next_level(49), 1);
            
            // At level 2, need 150 more XP to reach level 3 (200 - 50 = 150)
            assert_eq!(xp_to_next_level(50), 150);
            
            // At max level, should return 0
            assert_eq!(xp_to_next_level(999999), 0);
        }

        #[test]
        fn test_progress_percentage() {
            // Progress should be accurate within level range
            // Level 1: 0-49 XP
            assert_eq!(progress_to_next_level(0), 0.0);
            assert_eq!(progress_to_next_level(25), 50.0);
            
            // Level 2: 50-199 XP (150 XP range)
            // At 50 XP: start of level 2, progress = 0%
            assert_eq!(progress_to_next_level(50), 0.0);
            // At 125 XP: middle of level 2, progress = 50%
            assert_eq!(progress_to_next_level(125), 50.0);
            
            // At max level, should return 100%
            assert_eq!(progress_to_next_level(999999), 100.0);
        }

        #[test]
        fn test_level_up_detection() {
            // Utility function to detect level up
            fn would_level_up(current_xp: u32, xp_gain: u32) -> bool {
                let current_level = level_from_xp(current_xp);
                let new_level = level_from_xp(current_xp + xp_gain);
                new_level > current_level
            }

            // 49 XP + 1 XP = 50 XP -> Level 2
            assert!(would_level_up(49, 1));
            // 49 XP + 0 XP = 49 XP -> Still Level 1
            assert!(!would_level_up(49, 0));
            // 0 XP + 50 XP = 50 XP -> Level 2
            assert!(would_level_up(0, 50));
            // 0 XP + 200 XP = 200 XP -> Level 3 (skip level 2)
            assert!(would_level_up(0, 200));
        }
    }
}

/// XP calculation utilities
pub mod xp {
    use super::XpActionType;

    /// Calculate total XP from activity counts difference
    pub fn calculate_xp_from_diff(
        commits_diff: i32,
        prs_created_diff: i32,
        prs_merged_diff: i32,
        issues_created_diff: i32,
        issues_closed_diff: i32,
        reviews_diff: i32,
        stars_diff: i32,
    ) -> i32 {
        let mut total_xp = 0;
        
        if commits_diff > 0 {
            total_xp += commits_diff * XpActionType::Commit.xp_value();
        }
        if prs_created_diff > 0 {
            total_xp += prs_created_diff * XpActionType::PullRequestCreated.xp_value();
        }
        if prs_merged_diff > 0 {
            total_xp += prs_merged_diff * XpActionType::PullRequestMerged.xp_value();
        }
        if issues_created_diff > 0 {
            total_xp += issues_created_diff * XpActionType::IssueCreated.xp_value();
        }
        if issues_closed_diff > 0 {
            total_xp += issues_closed_diff * XpActionType::IssueClosed.xp_value();
        }
        if reviews_diff > 0 {
            total_xp += reviews_diff * XpActionType::Review.xp_value();
        }
        if stars_diff > 0 {
            total_xp += stars_diff * XpActionType::StarReceived.xp_value();
        }
        
        total_xp
    }

    /// Detailed XP breakdown from activity counts difference
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct XpBreakdown {
        pub commits_xp: i32,
        pub prs_created_xp: i32,
        pub prs_merged_xp: i32,
        pub issues_created_xp: i32,
        pub issues_closed_xp: i32,
        pub reviews_xp: i32,
        pub stars_xp: i32,
        pub total_xp: i32,
    }

    impl XpBreakdown {
        pub fn calculate(
            commits_diff: i32,
            prs_created_diff: i32,
            prs_merged_diff: i32,
            issues_created_diff: i32,
            issues_closed_diff: i32,
            reviews_diff: i32,
            stars_diff: i32,
        ) -> Self {
            let commits_xp = commits_diff.max(0) * XpActionType::Commit.xp_value();
            let prs_created_xp = prs_created_diff.max(0) * XpActionType::PullRequestCreated.xp_value();
            let prs_merged_xp = prs_merged_diff.max(0) * XpActionType::PullRequestMerged.xp_value();
            let issues_created_xp = issues_created_diff.max(0) * XpActionType::IssueCreated.xp_value();
            let issues_closed_xp = issues_closed_diff.max(0) * XpActionType::IssueClosed.xp_value();
            let reviews_xp = reviews_diff.max(0) * XpActionType::Review.xp_value();
            let stars_xp = stars_diff.max(0) * XpActionType::StarReceived.xp_value();
            
            let total_xp = commits_xp + prs_created_xp + prs_merged_xp + 
                          issues_created_xp + issues_closed_xp + reviews_xp + stars_xp;
            
            Self {
                commits_xp,
                prs_created_xp,
                prs_merged_xp,
                issues_created_xp,
                issues_closed_xp,
                reviews_xp,
                stars_xp,
                total_xp,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_xp_action_values() {
            // Verify XP values from PRD
            assert_eq!(XpActionType::Commit.xp_value(), 10);
            assert_eq!(XpActionType::PullRequestCreated.xp_value(), 30);
            assert_eq!(XpActionType::PullRequestMerged.xp_value(), 50);
            assert_eq!(XpActionType::IssueCreated.xp_value(), 15);
            assert_eq!(XpActionType::IssueClosed.xp_value(), 40);
            assert_eq!(XpActionType::Review.xp_value(), 25);
            assert_eq!(XpActionType::StarReceived.xp_value(), 5);
            assert_eq!(XpActionType::StreakBonus.xp_value(), 20);
        }

        #[test]
        fn test_calculate_xp_from_diff() {
            // 5 commits = 50 XP
            assert_eq!(calculate_xp_from_diff(5, 0, 0, 0, 0, 0, 0), 50);
            
            // 1 PR created = 30 XP
            assert_eq!(calculate_xp_from_diff(0, 1, 0, 0, 0, 0, 0), 30);
            
            // 1 PR merged = 50 XP
            assert_eq!(calculate_xp_from_diff(0, 0, 1, 0, 0, 0, 0), 50);
            
            // Combined: 10 commits + 2 PRs created + 1 PR merged = 100 + 60 + 50 = 210 XP
            assert_eq!(calculate_xp_from_diff(10, 2, 1, 0, 0, 0, 0), 210);
        }

        #[test]
        fn test_calculate_xp_ignores_negative_diff() {
            // Negative diff should be ignored (counts decreased)
            assert_eq!(calculate_xp_from_diff(-5, 0, 0, 0, 0, 0, 0), 0);
            assert_eq!(calculate_xp_from_diff(-5, 2, 0, 0, 0, 0, 0), 60);
        }

        #[test]
        fn test_xp_breakdown() {
            let breakdown = XpBreakdown::calculate(10, 2, 1, 3, 2, 5, 10);
            
            assert_eq!(breakdown.commits_xp, 100);        // 10 * 10
            assert_eq!(breakdown.prs_created_xp, 60);     // 2 * 30
            assert_eq!(breakdown.prs_merged_xp, 50);      // 1 * 50
            assert_eq!(breakdown.issues_created_xp, 45);  // 3 * 15
            assert_eq!(breakdown.issues_closed_xp, 80);   // 2 * 40
            assert_eq!(breakdown.reviews_xp, 125);        // 5 * 25
            assert_eq!(breakdown.stars_xp, 50);           // 10 * 5
            assert_eq!(breakdown.total_xp, 510);
        }
    }
}

/// Badge evaluation utilities
pub mod badge {
    use serde::{Deserialize, Serialize};

    /// Badge definition with condition
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BadgeDefinition {
        pub id: String,
        pub name: String,
        pub description: String,
        pub badge_type: String,
        pub rarity: String,
        pub icon: String,
        pub condition: BadgeCondition,
    }

    /// Badge condition types
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "type")]
    pub enum BadgeCondition {
        /// Commits milestone
        Commits { threshold: i32 },
        /// Streak milestone (uses longest_streak)
        Streak { days: i32 },
        /// Weekly streak - consecutive weeks with at least one commit
        WeeklyStreak { weeks: i32 },
        /// Monthly streak - consecutive months with at least one commit  
        MonthlyStreak { months: i32 },
        /// Reviews milestone
        Reviews { threshold: i32 },
        /// PRs merged milestone
        PrsMerged { threshold: i32 },
        /// Issues closed milestone
        IssuesClosed { threshold: i32 },
        /// PR merge rate (requires min_prs)
        PrMergeRate { min_rate: f32, min_prs: i32 },
        /// Languages used
        Languages { count: i32 },
        /// Level reached
        Level { threshold: i32 },
        /// Stars received on repositories
        StarsReceived { threshold: i32 },
    }

    /// User stats for badge evaluation
    #[derive(Debug, Clone, Default)]
    pub struct BadgeEvalContext {
        pub total_commits: i32,
        pub current_streak: i32,
        pub longest_streak: i32,
        pub weekly_streak: i32,
        pub monthly_streak: i32,
        pub total_reviews: i32,
        pub total_prs: i32,
        pub total_prs_merged: i32,
        pub total_issues_closed: i32,
        pub languages_count: i32,
        pub current_level: i32,
        pub total_stars_received: i32,
    }

    /// All badge definitions
    pub fn get_all_badge_definitions() -> Vec<BadgeDefinition> {
        vec![
            // Milestone badges
            BadgeDefinition {
                id: "first_blood".to_string(),
                name: "First Blood".to_string(),
                description: "Make your first commit".to_string(),
                badge_type: "milestone".to_string(),
                rarity: "bronze".to_string(),
                icon: "🎯".to_string(),
                condition: BadgeCondition::Commits { threshold: 1 },
            },
            BadgeDefinition {
                id: "century".to_string(),
                name: "Century".to_string(),
                description: "Reach 100 commits".to_string(),
                badge_type: "milestone".to_string(),
                rarity: "silver".to_string(),
                icon: "💯".to_string(),
                condition: BadgeCondition::Commits { threshold: 100 },
            },
            BadgeDefinition {
                id: "thousand_cuts".to_string(),
                name: "Thousand Cuts".to_string(),
                description: "Reach 1,000 commits".to_string(),
                badge_type: "milestone".to_string(),
                rarity: "gold".to_string(),
                icon: "⚔️".to_string(),
                condition: BadgeCondition::Commits { threshold: 1000 },
            },
            BadgeDefinition {
                id: "legendary".to_string(),
                name: "Legendary".to_string(),
                description: "Reach 10,000 commits".to_string(),
                badge_type: "milestone".to_string(),
                rarity: "platinum".to_string(),
                icon: "🏆".to_string(),
                condition: BadgeCondition::Commits { threshold: 10000 },
            },
            // Streak badges
            BadgeDefinition {
                id: "on_fire".to_string(),
                name: "On Fire".to_string(),
                description: "7 day commit streak".to_string(),
                badge_type: "streak".to_string(),
                rarity: "bronze".to_string(),
                icon: "🔥".to_string(),
                condition: BadgeCondition::Streak { days: 7 },
            },
            BadgeDefinition {
                id: "unstoppable".to_string(),
                name: "Unstoppable".to_string(),
                description: "30 day commit streak".to_string(),
                badge_type: "streak".to_string(),
                rarity: "silver".to_string(),
                icon: "💪".to_string(),
                condition: BadgeCondition::Streak { days: 30 },
            },
            BadgeDefinition {
                id: "immortal".to_string(),
                name: "Immortal".to_string(),
                description: "365 day commit streak".to_string(),
                badge_type: "streak".to_string(),
                rarity: "platinum".to_string(),
                icon: "👑".to_string(),
                condition: BadgeCondition::Streak { days: 365 },
            },
            // Collaboration badges
            BadgeDefinition {
                id: "team_player".to_string(),
                name: "Team Player".to_string(),
                description: "Complete your first review".to_string(),
                badge_type: "collaboration".to_string(),
                rarity: "bronze".to_string(),
                icon: "🤝".to_string(),
                condition: BadgeCondition::Reviews { threshold: 1 },
            },
            BadgeDefinition {
                id: "mentor".to_string(),
                name: "Mentor".to_string(),
                description: "Complete 50 reviews".to_string(),
                badge_type: "collaboration".to_string(),
                rarity: "silver".to_string(),
                icon: "🎓".to_string(),
                condition: BadgeCondition::Reviews { threshold: 50 },
            },
            BadgeDefinition {
                id: "guardian".to_string(),
                name: "Guardian".to_string(),
                description: "Merge 100 PRs".to_string(),
                badge_type: "collaboration".to_string(),
                rarity: "gold".to_string(),
                icon: "🛡️".to_string(),
                condition: BadgeCondition::PrsMerged { threshold: 100 },
            },
            // Quality badges
            BadgeDefinition {
                id: "clean_coder".to_string(),
                name: "Clean Coder".to_string(),
                description: "90%+ PR merge rate (10+ PRs)".to_string(),
                badge_type: "quality".to_string(),
                rarity: "gold".to_string(),
                icon: "✨".to_string(),
                condition: BadgeCondition::PrMergeRate {
                    min_rate: 0.9,
                    min_prs: 10,
                },
            },
            BadgeDefinition {
                id: "bug_hunter".to_string(),
                name: "Bug Hunter".to_string(),
                description: "Close 50 issues".to_string(),
                badge_type: "quality".to_string(),
                rarity: "silver".to_string(),
                icon: "🐛".to_string(),
                condition: BadgeCondition::IssuesClosed { threshold: 50 },
            },
            BadgeDefinition {
                id: "polyglot".to_string(),
                name: "Polyglot".to_string(),
                description: "Use 5+ programming languages".to_string(),
                badge_type: "quality".to_string(),
                rarity: "silver".to_string(),
                icon: "🌍".to_string(),
                condition: BadgeCondition::Languages { count: 5 },
            },
            // Language badges (expanded)
            BadgeDefinition {
                id: "polyglot_3".to_string(),
                name: "Trilingual".to_string(),
                description: "Use 3+ programming languages".to_string(),
                badge_type: "language".to_string(),
                rarity: "bronze".to_string(),
                icon: "🗣️".to_string(),
                condition: BadgeCondition::Languages { count: 3 },
            },
            BadgeDefinition {
                id: "polyglot_10".to_string(),
                name: "Language Master".to_string(),
                description: "Use 10+ programming languages".to_string(),
                badge_type: "language".to_string(),
                rarity: "gold".to_string(),
                icon: "📚".to_string(),
                condition: BadgeCondition::Languages { count: 10 },
            },
            // Level badges
            BadgeDefinition {
                id: "level_5".to_string(),
                name: "Rising Star".to_string(),
                description: "Reach level 5".to_string(),
                badge_type: "level".to_string(),
                rarity: "bronze".to_string(),
                icon: "⭐".to_string(),
                condition: BadgeCondition::Level { threshold: 5 },
            },
            BadgeDefinition {
                id: "level_10".to_string(),
                name: "Skilled Developer".to_string(),
                description: "Reach level 10".to_string(),
                badge_type: "level".to_string(),
                rarity: "silver".to_string(),
                icon: "🌟".to_string(),
                condition: BadgeCondition::Level { threshold: 10 },
            },
            BadgeDefinition {
                id: "level_25".to_string(),
                name: "Expert".to_string(),
                description: "Reach level 25".to_string(),
                badge_type: "level".to_string(),
                rarity: "silver".to_string(),
                icon: "💫".to_string(),
                condition: BadgeCondition::Level { threshold: 25 },
            },
            BadgeDefinition {
                id: "level_50".to_string(),
                name: "Master".to_string(),
                description: "Reach level 50".to_string(),
                badge_type: "level".to_string(),
                rarity: "gold".to_string(),
                icon: "🏅".to_string(),
                condition: BadgeCondition::Level { threshold: 50 },
            },
            BadgeDefinition {
                id: "level_100".to_string(),
                name: "Grandmaster".to_string(),
                description: "Reach level 100".to_string(),
                badge_type: "level".to_string(),
                rarity: "platinum".to_string(),
                icon: "👑".to_string(),
                condition: BadgeCondition::Level { threshold: 100 },
            },
            // Star badges
            BadgeDefinition {
                id: "star_1".to_string(),
                name: "First Star".to_string(),
                description: "Receive your first star".to_string(),
                badge_type: "stars".to_string(),
                rarity: "bronze".to_string(),
                icon: "✨".to_string(),
                condition: BadgeCondition::StarsReceived { threshold: 1 },
            },
            BadgeDefinition {
                id: "star_10".to_string(),
                name: "Rising Repository".to_string(),
                description: "Receive 10 stars".to_string(),
                badge_type: "stars".to_string(),
                rarity: "bronze".to_string(),
                icon: "🌠".to_string(),
                condition: BadgeCondition::StarsReceived { threshold: 10 },
            },
            BadgeDefinition {
                id: "star_50".to_string(),
                name: "Popular Project".to_string(),
                description: "Receive 50 stars".to_string(),
                badge_type: "stars".to_string(),
                rarity: "silver".to_string(),
                icon: "⭐".to_string(),
                condition: BadgeCondition::StarsReceived { threshold: 50 },
            },
            BadgeDefinition {
                id: "star_100".to_string(),
                name: "Star Magnet".to_string(),
                description: "Receive 100 stars".to_string(),
                badge_type: "stars".to_string(),
                rarity: "gold".to_string(),
                icon: "🎖️".to_string(),
                condition: BadgeCondition::StarsReceived { threshold: 100 },
            },
            BadgeDefinition {
                id: "star_1000".to_string(),
                name: "Open Source Hero".to_string(),
                description: "Receive 1000 stars".to_string(),
                badge_type: "stars".to_string(),
                rarity: "platinum".to_string(),
                icon: "🌌".to_string(),
                condition: BadgeCondition::StarsReceived { threshold: 1000 },
            },
            // Weekly streak (consistency) badges
            BadgeDefinition {
                id: "weekly_3".to_string(),
                name: "Consistent Coder".to_string(),
                description: "Contribute for 3 consecutive weeks".to_string(),
                badge_type: "consistency".to_string(),
                rarity: "bronze".to_string(),
                icon: "📅".to_string(),
                condition: BadgeCondition::WeeklyStreak { weeks: 3 },
            },
            BadgeDefinition {
                id: "weekly_12".to_string(),
                name: "Quarter Champion".to_string(),
                description: "Contribute for 12 consecutive weeks".to_string(),
                badge_type: "consistency".to_string(),
                rarity: "silver".to_string(),
                icon: "🗓️".to_string(),
                condition: BadgeCondition::WeeklyStreak { weeks: 12 },
            },
            // Monthly streak (consistency) badges
            BadgeDefinition {
                id: "monthly_6".to_string(),
                name: "Half Year Hero".to_string(),
                description: "Contribute for 6 consecutive months".to_string(),
                badge_type: "consistency".to_string(),
                rarity: "gold".to_string(),
                icon: "📆".to_string(),
                condition: BadgeCondition::MonthlyStreak { months: 6 },
            },
            BadgeDefinition {
                id: "monthly_12".to_string(),
                name: "Year Round Developer".to_string(),
                description: "Contribute for 12 consecutive months".to_string(),
                badge_type: "consistency".to_string(),
                rarity: "platinum".to_string(),
                icon: "🎖️".to_string(),
                condition: BadgeCondition::MonthlyStreak { months: 12 },
            },
        ]
    }

    /// Evaluate if a badge condition is met
    pub fn evaluate_condition(condition: &BadgeCondition, context: &BadgeEvalContext) -> bool {
        match condition {
            BadgeCondition::Commits { threshold } => context.total_commits >= *threshold,
            BadgeCondition::Streak { days } => {
                context.current_streak >= *days || context.longest_streak >= *days
            }
            BadgeCondition::WeeklyStreak { weeks } => context.weekly_streak >= *weeks,
            BadgeCondition::MonthlyStreak { months } => context.monthly_streak >= *months,
            BadgeCondition::Reviews { threshold } => context.total_reviews >= *threshold,
            BadgeCondition::PrsMerged { threshold } => context.total_prs_merged >= *threshold,
            BadgeCondition::IssuesClosed { threshold } => context.total_issues_closed >= *threshold,
            BadgeCondition::PrMergeRate { min_rate, min_prs } => {
                if context.total_prs < *min_prs {
                    return false;
                }
                if context.total_prs == 0 {
                    return false;
                }
                let rate = context.total_prs_merged as f32 / context.total_prs as f32;
                rate >= *min_rate
            }
            BadgeCondition::Languages { count } => context.languages_count >= *count,
            BadgeCondition::Level { threshold } => context.current_level >= *threshold,
            BadgeCondition::StarsReceived { threshold } => context.total_stars_received >= *threshold,
        }
    }

    /// Result of badge evaluation
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BadgeEvalResult {
        pub badge_id: String,
        pub badge_type: String,
        pub newly_earned: bool,
    }

    /// Badge progress information
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct BadgeProgress {
        pub badge_id: String,
        pub current_value: i32,
        pub target_value: i32,
        pub progress_percent: f32, // 0.0 - 100.0
    }

    /// Badge definition with progress information
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct BadgeWithProgress {
        pub id: String,
        pub name: String,
        pub description: String,
        pub badge_type: String,
        pub rarity: String,
        pub icon: String,
        pub earned: bool,
        pub earned_at: Option<String>,
        pub progress: Option<BadgeProgress>,
    }

    /// Calculate progress for a badge condition
    pub fn calculate_progress(badge_id: &str, condition: &BadgeCondition, context: &BadgeEvalContext) -> BadgeProgress {
        match condition {
            BadgeCondition::Commits { threshold } => BadgeProgress {
                badge_id: badge_id.to_string(),
                current_value: context.total_commits,
                target_value: *threshold,
                progress_percent: calculate_percent(context.total_commits, *threshold),
            },
            BadgeCondition::Streak { days } => {
                let max_streak = context.current_streak.max(context.longest_streak);
                BadgeProgress {
                    badge_id: badge_id.to_string(),
                    current_value: max_streak,
                    target_value: *days,
                    progress_percent: calculate_percent(max_streak, *days),
                }
            }
            BadgeCondition::Reviews { threshold } => BadgeProgress {
                badge_id: badge_id.to_string(),
                current_value: context.total_reviews,
                target_value: *threshold,
                progress_percent: calculate_percent(context.total_reviews, *threshold),
            },
            BadgeCondition::PrsMerged { threshold } => BadgeProgress {
                badge_id: badge_id.to_string(),
                current_value: context.total_prs_merged,
                target_value: *threshold,
                progress_percent: calculate_percent(context.total_prs_merged, *threshold),
            },
            BadgeCondition::IssuesClosed { threshold } => BadgeProgress {
                badge_id: badge_id.to_string(),
                current_value: context.total_issues_closed,
                target_value: *threshold,
                progress_percent: calculate_percent(context.total_issues_closed, *threshold),
            },
            BadgeCondition::PrMergeRate { min_rate, min_prs } => {
                // For merge rate, we show progress towards min_prs if not reached
                // Otherwise, show progress towards target rate
                if context.total_prs < *min_prs {
                    BadgeProgress {
                        badge_id: badge_id.to_string(),
                        current_value: context.total_prs,
                        target_value: *min_prs,
                        progress_percent: calculate_percent(context.total_prs, *min_prs),
                    }
                } else {
                    let rate = if context.total_prs > 0 {
                        context.total_prs_merged as f32 / context.total_prs as f32
                    } else {
                        0.0
                    };
                    let rate_percent = (rate * 100.0).round() as i32;
                    let target_percent = (*min_rate * 100.0).round() as i32;
                    BadgeProgress {
                        badge_id: badge_id.to_string(),
                        current_value: rate_percent,
                        target_value: target_percent,
                        progress_percent: (rate / *min_rate * 100.0).min(100.0),
                    }
                }
            }
            BadgeCondition::Languages { count } => BadgeProgress {
                badge_id: badge_id.to_string(),
                current_value: context.languages_count,
                target_value: *count,
                progress_percent: calculate_percent(context.languages_count, *count),
            },
            BadgeCondition::Level { threshold } => BadgeProgress {
                badge_id: badge_id.to_string(),
                current_value: context.current_level,
                target_value: *threshold,
                progress_percent: calculate_percent(context.current_level, *threshold),
            },
            BadgeCondition::StarsReceived { threshold } => BadgeProgress {
                badge_id: badge_id.to_string(),
                current_value: context.total_stars_received,
                target_value: *threshold,
                progress_percent: calculate_percent(context.total_stars_received, *threshold),
            },
            BadgeCondition::WeeklyStreak { weeks } => BadgeProgress {
                badge_id: badge_id.to_string(),
                current_value: context.weekly_streak,
                target_value: *weeks,
                progress_percent: calculate_percent(context.weekly_streak, *weeks),
            },
            BadgeCondition::MonthlyStreak { months } => BadgeProgress {
                badge_id: badge_id.to_string(),
                current_value: context.monthly_streak,
                target_value: *months,
                progress_percent: calculate_percent(context.monthly_streak, *months),
            },
        }
    }

    /// Helper to calculate percentage (capped at 100%)
    fn calculate_percent(current: i32, target: i32) -> f32 {
        if target == 0 {
            return 100.0;
        }
        ((current as f32 / target as f32) * 100.0).min(100.0)
    }

    /// Get all badges with progress information
    pub fn get_badges_with_progress(
        context: &BadgeEvalContext,
        earned_badges: &[(String, Option<String>)], // (badge_id, earned_at)
    ) -> Vec<BadgeWithProgress> {
        let definitions = get_all_badge_definitions();
        let mut results = Vec::new();

        for def in definitions {
            let earned_info = earned_badges.iter().find(|(id, _)| id == &def.id);
            let is_earned = earned_info.is_some();
            let earned_at = earned_info.and_then(|(_, at)| at.clone());

            let progress = if !is_earned {
                Some(calculate_progress(&def.id, &def.condition, context))
            } else {
                None
            };

            results.push(BadgeWithProgress {
                id: def.id,
                name: def.name,
                description: def.description,
                badge_type: def.badge_type,
                rarity: def.rarity,
                icon: def.icon,
                earned: is_earned,
                earned_at,
                progress,
            });
        }

        results
    }

    /// Get badges that are close to being earned (progress >= threshold%)
    pub fn get_near_completion_badges(
        context: &BadgeEvalContext,
        earned_badge_ids: &[String],
        threshold_percent: f32,
    ) -> Vec<BadgeWithProgress> {
        let definitions = get_all_badge_definitions();
        let mut results = Vec::new();

        for def in definitions {
            if earned_badge_ids.contains(&def.id) {
                continue;
            }

            let progress = calculate_progress(&def.id, &def.condition, context);

            if progress.progress_percent >= threshold_percent && progress.progress_percent < 100.0 {
                results.push(BadgeWithProgress {
                    id: def.id,
                    name: def.name,
                    description: def.description,
                    badge_type: def.badge_type,
                    rarity: def.rarity,
                    icon: def.icon,
                    earned: false,
                    earned_at: None,
                    progress: Some(progress),
                });
            }
        }

        // Sort by progress descending (closest to completion first)
        results.sort_by(|a, b| {
            let a_progress = a.progress.as_ref().map(|p| p.progress_percent).unwrap_or(0.0);
            let b_progress = b.progress.as_ref().map(|p| p.progress_percent).unwrap_or(0.0);
            b_progress.partial_cmp(&a_progress).unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Evaluate all badges and return which ones should be awarded
    pub fn evaluate_badges(
        context: &BadgeEvalContext,
        already_earned: &[String],
    ) -> Vec<BadgeEvalResult> {
        let definitions = get_all_badge_definitions();
        let mut results = Vec::new();

        for def in definitions {
            let is_earned = already_earned.iter().any(|id| id == &def.id);
            let condition_met = evaluate_condition(&def.condition, context);

            if condition_met && !is_earned {
                results.push(BadgeEvalResult {
                    badge_id: def.id,
                    badge_type: def.badge_type,
                    newly_earned: true,
                });
            }
        }

        results
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_first_blood_badge() {
            let context = BadgeEvalContext {
                total_commits: 1,
                ..Default::default()
            };
            let results = evaluate_badges(&context, &[]);
            assert!(results.iter().any(|r| r.badge_id == "first_blood"));
        }

        #[test]
        fn test_century_badge() {
            let context = BadgeEvalContext {
                total_commits: 100,
                ..Default::default()
            };
            let results = evaluate_badges(&context, &["first_blood".to_string()]);
            assert!(results.iter().any(|r| r.badge_id == "century"));
        }

        #[test]
        fn test_streak_badge_on_fire() {
            let context = BadgeEvalContext {
                current_streak: 7,
                ..Default::default()
            };
            let results = evaluate_badges(&context, &[]);
            assert!(results.iter().any(|r| r.badge_id == "on_fire"));
        }

        #[test]
        fn test_streak_badge_with_longest_streak() {
            let context = BadgeEvalContext {
                current_streak: 3, // Current is broken
                longest_streak: 7, // But longest qualifies
                ..Default::default()
            };
            let results = evaluate_badges(&context, &[]);
            assert!(results.iter().any(|r| r.badge_id == "on_fire"));
        }

        #[test]
        fn test_team_player_badge() {
            let context = BadgeEvalContext {
                total_reviews: 1,
                ..Default::default()
            };
            let results = evaluate_badges(&context, &[]);
            assert!(results.iter().any(|r| r.badge_id == "team_player"));
        }

        #[test]
        fn test_clean_coder_badge() {
            let context = BadgeEvalContext {
                total_prs: 15,
                total_prs_merged: 14, // 93.3% merge rate
                ..Default::default()
            };
            let results = evaluate_badges(&context, &[]);
            assert!(results.iter().any(|r| r.badge_id == "clean_coder"));
        }

        #[test]
        fn test_clean_coder_badge_not_enough_prs() {
            let context = BadgeEvalContext {
                total_prs: 5, // Less than 10 required
                total_prs_merged: 5,
                ..Default::default()
            };
            let results = evaluate_badges(&context, &[]);
            assert!(!results.iter().any(|r| r.badge_id == "clean_coder"));
        }

        #[test]
        fn test_polyglot_badge() {
            let context = BadgeEvalContext {
                languages_count: 5,
                ..Default::default()
            };
            let results = evaluate_badges(&context, &[]);
            assert!(results.iter().any(|r| r.badge_id == "polyglot"));
        }

        #[test]
        fn test_already_earned_badge_not_returned() {
            let context = BadgeEvalContext {
                total_commits: 100,
                ..Default::default()
            };
            let results = evaluate_badges(&context, &["first_blood".to_string(), "century".to_string()]);
            assert!(!results.iter().any(|r| r.badge_id == "first_blood"));
            assert!(!results.iter().any(|r| r.badge_id == "century"));
        }

        #[test]
        fn test_guardian_badge() {
            let context = BadgeEvalContext {
                total_prs_merged: 100,
                ..Default::default()
            };
            let results = evaluate_badges(&context, &[]);
            assert!(results.iter().any(|r| r.badge_id == "guardian"));
        }

        #[test]
        fn test_bug_hunter_badge() {
            let context = BadgeEvalContext {
                total_issues_closed: 50,
                ..Default::default()
            };
            let results = evaluate_badges(&context, &[]);
            assert!(results.iter().any(|r| r.badge_id == "bug_hunter"));
        }

        // ====================================
        // Phase 1: New Badge Condition Tests
        // ====================================

        #[test]
        fn test_level_badge_level_5() {
            let context = BadgeEvalContext {
                current_level: 5,
                ..Default::default()
            };
            let results = evaluate_badges(&context, &[]);
            assert!(results.iter().any(|r| r.badge_id == "level_5"));
        }

        #[test]
        fn test_level_badge_level_10() {
            let context = BadgeEvalContext {
                current_level: 10,
                ..Default::default()
            };
            let results = evaluate_badges(&context, &[]);
            assert!(results.iter().any(|r| r.badge_id == "level_10"));
            assert!(results.iter().any(|r| r.badge_id == "level_5")); // Also qualifies for level 5
        }

        #[test]
        fn test_level_badge_not_reached() {
            let context = BadgeEvalContext {
                current_level: 4,
                ..Default::default()
            };
            let results = evaluate_badges(&context, &[]);
            assert!(!results.iter().any(|r| r.badge_id == "level_5"));
        }

        #[test]
        fn test_stars_badge_first_star() {
            let context = BadgeEvalContext {
                total_stars_received: 1,
                ..Default::default()
            };
            let results = evaluate_badges(&context, &[]);
            assert!(results.iter().any(|r| r.badge_id == "star_1"));
        }

        #[test]
        fn test_stars_badge_100_stars() {
            let context = BadgeEvalContext {
                total_stars_received: 100,
                ..Default::default()
            };
            let results = evaluate_badges(&context, &[]);
            assert!(results.iter().any(|r| r.badge_id == "star_100"));
            assert!(results.iter().any(|r| r.badge_id == "star_50"));
            assert!(results.iter().any(|r| r.badge_id == "star_10"));
            assert!(results.iter().any(|r| r.badge_id == "star_1"));
        }

        #[test]
        fn test_language_badge_trilingual() {
            let context = BadgeEvalContext {
                languages_count: 3,
                ..Default::default()
            };
            let results = evaluate_badges(&context, &[]);
            assert!(results.iter().any(|r| r.badge_id == "polyglot_3"));
        }

        #[test]
        fn test_language_badge_master() {
            let context = BadgeEvalContext {
                languages_count: 10,
                ..Default::default()
            };
            let results = evaluate_badges(&context, &[]);
            assert!(results.iter().any(|r| r.badge_id == "polyglot_10"));
            assert!(results.iter().any(|r| r.badge_id == "polyglot")); // 5 languages
            assert!(results.iter().any(|r| r.badge_id == "polyglot_3")); // 3 languages
        }

        // ====================================
        // Phase 3: Progress Calculation Tests
        // ====================================

        #[test]
        fn test_progress_commits_50_percent() {
            let context = BadgeEvalContext {
                total_commits: 50,
                ..Default::default()
            };
            let progress = calculate_progress(
                "test_badge",
                &BadgeCondition::Commits { threshold: 100 },
                &context,
            );
            assert_eq!(progress.badge_id, "test_badge");
            assert_eq!(progress.current_value, 50);
            assert_eq!(progress.target_value, 100);
            assert!((progress.progress_percent - 50.0).abs() < 0.01);
        }

        #[test]
        fn test_progress_caps_at_100_percent() {
            let context = BadgeEvalContext {
                total_commits: 150,
                ..Default::default()
            };
            let progress = calculate_progress(
                "test_badge",
                &BadgeCondition::Commits { threshold: 100 },
                &context,
            );
            assert_eq!(progress.progress_percent, 100.0);
        }

        #[test]
        fn test_progress_level_badge() {
            let context = BadgeEvalContext {
                current_level: 12,
                ..Default::default()
            };
            let progress = calculate_progress(
                "level_25",
                &BadgeCondition::Level { threshold: 25 },
                &context,
            );
            assert_eq!(progress.current_value, 12);
            assert_eq!(progress.target_value, 25);
            assert!((progress.progress_percent - 48.0).abs() < 0.01);
        }

        #[test]
        fn test_progress_streak_uses_max() {
            let context = BadgeEvalContext {
                current_streak: 5,
                longest_streak: 10,
                ..Default::default()
            };
            let progress = calculate_progress(
                "streak_30",
                &BadgeCondition::Streak { days: 30 },
                &context,
            );
            assert_eq!(progress.current_value, 10); // Uses longest_streak
            assert_eq!(progress.target_value, 30);
        }

        #[test]
        fn test_get_near_completion_badges() {
            let context = BadgeEvalContext {
                total_commits: 80, // 80% towards century (100)
                current_level: 4,  // 80% towards level_5 (5)
                ..Default::default()
            };
            let earned_badge_ids = vec!["first_blood".to_string()];
            let near_badges = get_near_completion_badges(&context, &earned_badge_ids, 50.0);
            
            // Should include badges with >= 50% progress
            assert!(near_badges.iter().any(|b| b.id == "century"));
            assert!(near_badges.iter().any(|b| b.id == "level_5"));
            
            // Should not include earned badges
            assert!(!near_badges.iter().any(|b| b.id == "first_blood"));
        }

        #[test]
        fn test_get_badges_with_progress() {
            let context = BadgeEvalContext {
                total_commits: 50,
                ..Default::default()
            };
            let earned_badges = vec![("first_blood".to_string(), Some("2025-01-01".to_string()))];
            let badges = get_badges_with_progress(&context, &earned_badges);
            
            // first_blood should be earned with no progress
            let first_blood = badges.iter().find(|b| b.id == "first_blood").unwrap();
            assert!(first_blood.earned);
            assert!(first_blood.earned_at.is_some());
            assert!(first_blood.progress.is_none());
            
            // century should have progress
            let century = badges.iter().find(|b| b.id == "century").unwrap();
            assert!(!century.earned);
            assert!(century.progress.is_some());
            let prog = century.progress.as_ref().unwrap();
            assert_eq!(prog.current_value, 50);
            assert_eq!(prog.target_value, 100);
        }

        // ====================================
        // Phase 2-B: Weekly/Monthly Streak Badge Tests
        // ====================================

        #[test]
        fn test_weekly_streak_badge_consistent_coder() {
            let context = BadgeEvalContext {
                weekly_streak: 3,
                ..Default::default()
            };
            let results = evaluate_badges(&context, &[]);
            assert!(results.iter().any(|r| r.badge_id == "weekly_3"));
        }

        #[test]
        fn test_weekly_streak_badge_quarter_champion() {
            let context = BadgeEvalContext {
                weekly_streak: 12,
                ..Default::default()
            };
            let results = evaluate_badges(&context, &[]);
            assert!(results.iter().any(|r| r.badge_id == "weekly_12"));
            assert!(results.iter().any(|r| r.badge_id == "weekly_3")); // Also qualifies for 3 weeks
        }

        #[test]
        fn test_monthly_streak_badge_half_year() {
            let context = BadgeEvalContext {
                monthly_streak: 6,
                ..Default::default()
            };
            let results = evaluate_badges(&context, &[]);
            assert!(results.iter().any(|r| r.badge_id == "monthly_6"));
        }

        #[test]
        fn test_monthly_streak_badge_year_round() {
            let context = BadgeEvalContext {
                monthly_streak: 12,
                ..Default::default()
            };
            let results = evaluate_badges(&context, &[]);
            assert!(results.iter().any(|r| r.badge_id == "monthly_12"));
            assert!(results.iter().any(|r| r.badge_id == "monthly_6")); // Also qualifies for 6 months
        }

        #[test]
        fn test_weekly_streak_not_reached() {
            let context = BadgeEvalContext {
                weekly_streak: 2,
                ..Default::default()
            };
            let results = evaluate_badges(&context, &[]);
            assert!(!results.iter().any(|r| r.badge_id == "weekly_3"));
        }

        #[test]
        fn test_progress_weekly_streak() {
            let context = BadgeEvalContext {
                weekly_streak: 6,
                ..Default::default()
            };
            let progress = calculate_progress(
                "weekly_12",
                &BadgeCondition::WeeklyStreak { weeks: 12 },
                &context,
            );
            assert_eq!(progress.current_value, 6);
            assert_eq!(progress.target_value, 12);
            assert!((progress.progress_percent - 50.0).abs() < 0.01);
        }

        #[test]
        fn test_progress_monthly_streak() {
            let context = BadgeEvalContext {
                monthly_streak: 3,
                ..Default::default()
            };
            let progress = calculate_progress(
                "monthly_6",
                &BadgeCondition::MonthlyStreak { months: 6 },
                &context,
            );
            assert_eq!(progress.current_value, 3);
            assert_eq!(progress.target_value, 6);
            assert!((progress.progress_percent - 50.0).abs() < 0.01);
        }
    }
}

/// Streak bonus calculation utilities
pub mod streak {
    use serde::{Deserialize, Serialize};

    /// Streak milestone thresholds and their XP bonuses
    /// Based on PRD specification
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct StreakMilestone {
        pub days: i32,
        pub xp_bonus: i32,
    }

    /// All streak milestones (ordered by days)
    pub const STREAK_MILESTONES: &[StreakMilestone] = &[
        StreakMilestone { days: 7, xp_bonus: 50 },
        StreakMilestone { days: 14, xp_bonus: 100 },
        StreakMilestone { days: 30, xp_bonus: 200 },
        StreakMilestone { days: 100, xp_bonus: 500 },
        StreakMilestone { days: 365, xp_bonus: 1000 },
    ];

    /// Daily streak bonus (for each day with activity)
    pub const DAILY_STREAK_BONUS: i32 = 20;

    /// Result of streak bonus calculation
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    pub struct StreakBonusResult {
        /// Daily bonus XP (if streak continues)
        pub daily_bonus: i32,
        /// Milestone bonus XP (if a milestone is reached)
        pub milestone_bonus: i32,
        /// Total bonus XP
        pub total_bonus: i32,
        /// Milestone reached (if any)
        pub milestone_reached: Option<i32>,
        /// Current streak days
        pub current_streak: i32,
    }

    /// Calculate streak bonus when streak is updated
    ///
    /// # Arguments
    /// * `old_streak` - Previous streak count
    /// * `new_streak` - New streak count after activity
    ///
    /// # Returns
    /// StreakBonusResult with breakdown of bonuses
    pub fn calculate_streak_bonus(old_streak: i32, new_streak: i32) -> StreakBonusResult {
        let mut result = StreakBonusResult {
            current_streak: new_streak,
            ..Default::default()
        };

        // If streak didn't increase, no bonus
        if new_streak <= old_streak {
            return result;
        }

        // Daily bonus for continuing streak
        result.daily_bonus = DAILY_STREAK_BONUS;

        // Check for milestone bonuses
        for milestone in STREAK_MILESTONES {
            // Award milestone bonus if we just crossed this threshold
            if old_streak < milestone.days && new_streak >= milestone.days {
                result.milestone_bonus = milestone.xp_bonus;
                result.milestone_reached = Some(milestone.days);
                // Only award the highest milestone reached
                break;
            }
        }

        result.total_bonus = result.daily_bonus + result.milestone_bonus;
        result
    }

    /// Get the next milestone for a given streak
    pub fn get_next_milestone(current_streak: i32) -> Option<StreakMilestone> {
        STREAK_MILESTONES
            .iter()
            .find(|m| m.days > current_streak)
            .copied()
    }

    /// Get days until next milestone
    pub fn days_to_next_milestone(current_streak: i32) -> Option<i32> {
        get_next_milestone(current_streak).map(|m| m.days - current_streak)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_streak_bonus_no_increase() {
            // Streak didn't increase (same day activity)
            let result = calculate_streak_bonus(5, 5);
            assert_eq!(result.total_bonus, 0);
            assert_eq!(result.daily_bonus, 0);
            assert_eq!(result.milestone_bonus, 0);
        }

        #[test]
        fn test_streak_bonus_daily() {
            // Simple streak increase without milestone
            let result = calculate_streak_bonus(3, 4);
            assert_eq!(result.daily_bonus, 20);
            assert_eq!(result.milestone_bonus, 0);
            assert_eq!(result.total_bonus, 20);
            assert_eq!(result.milestone_reached, None);
        }

        #[test]
        fn test_streak_bonus_7_day_milestone() {
            // Reaching 7 day milestone
            let result = calculate_streak_bonus(6, 7);
            assert_eq!(result.daily_bonus, 20);
            assert_eq!(result.milestone_bonus, 50);
            assert_eq!(result.total_bonus, 70);
            assert_eq!(result.milestone_reached, Some(7));
        }

        #[test]
        fn test_streak_bonus_14_day_milestone() {
            // Reaching 14 day milestone
            let result = calculate_streak_bonus(13, 14);
            assert_eq!(result.daily_bonus, 20);
            assert_eq!(result.milestone_bonus, 100);
            assert_eq!(result.total_bonus, 120);
            assert_eq!(result.milestone_reached, Some(14));
        }

        #[test]
        fn test_streak_bonus_30_day_milestone() {
            // Reaching 30 day milestone
            let result = calculate_streak_bonus(29, 30);
            assert_eq!(result.daily_bonus, 20);
            assert_eq!(result.milestone_bonus, 200);
            assert_eq!(result.total_bonus, 220);
            assert_eq!(result.milestone_reached, Some(30));
        }

        #[test]
        fn test_streak_bonus_100_day_milestone() {
            // Reaching 100 day milestone
            let result = calculate_streak_bonus(99, 100);
            assert_eq!(result.daily_bonus, 20);
            assert_eq!(result.milestone_bonus, 500);
            assert_eq!(result.total_bonus, 520);
            assert_eq!(result.milestone_reached, Some(100));
        }

        #[test]
        fn test_streak_bonus_365_day_milestone() {
            // Reaching 365 day milestone
            let result = calculate_streak_bonus(364, 365);
            assert_eq!(result.daily_bonus, 20);
            assert_eq!(result.milestone_bonus, 1000);
            assert_eq!(result.total_bonus, 1020);
            assert_eq!(result.milestone_reached, Some(365));
        }

        #[test]
        fn test_streak_bonus_after_milestones() {
            // After all milestones, only daily bonus
            let result = calculate_streak_bonus(400, 401);
            assert_eq!(result.daily_bonus, 20);
            assert_eq!(result.milestone_bonus, 0);
            assert_eq!(result.total_bonus, 20);
            assert_eq!(result.milestone_reached, None);
        }

        #[test]
        fn test_get_next_milestone() {
            assert_eq!(get_next_milestone(0), Some(StreakMilestone { days: 7, xp_bonus: 50 }));
            assert_eq!(get_next_milestone(7), Some(StreakMilestone { days: 14, xp_bonus: 100 }));
            assert_eq!(get_next_milestone(100), Some(StreakMilestone { days: 365, xp_bonus: 1000 }));
            assert_eq!(get_next_milestone(365), None);
        }

        #[test]
        fn test_days_to_next_milestone() {
            assert_eq!(days_to_next_milestone(0), Some(7));
            assert_eq!(days_to_next_milestone(5), Some(2));
            assert_eq!(days_to_next_milestone(10), Some(4));
            assert_eq!(days_to_next_milestone(365), None);
        }
    }
}

