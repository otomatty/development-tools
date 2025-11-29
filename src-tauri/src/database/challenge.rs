//! Challenge generation and progress tracking logic
//!
//! This module handles automatic challenge generation based on user activity
//! and progress updates during GitHub sync.

use chrono::{DateTime, Datelike, Duration, Utc, Weekday};
use serde::{Deserialize, Serialize};

use super::models::{Challenge, UserStats};

/// Current GitHub stats used for challenge progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeStats {
    pub commits: i32,
    pub prs: i32,
    pub reviews: i32,
    pub issues: i32,
}

impl ChallengeStats {
    /// Create new challenge stats from GitHub data
    pub fn new(commits: i32, prs: i32, reviews: i32, issues: i32) -> Self {
        Self {
            commits,
            prs,
            reviews,
            issues,
        }
    }

    /// Get value for a specific metric
    pub fn get_metric(&self, metric: &str) -> i32 {
        match metric {
            "commits" => self.commits,
            "prs" => self.prs,
            "reviews" => self.reviews,
            "issues" => self.issues,
            _ => 0,
        }
    }
}

/// Configuration for challenge generation
pub struct ChallengeGeneratorConfig {
    /// Multiplier for weekly target (based on past 4-week average)
    pub weekly_target_multiplier: f32,
    /// Multiplier for daily target
    pub daily_target_multiplier: f32,
    /// Minimum target values
    pub min_commits: i32,
    pub min_prs: i32,
    pub min_reviews: i32,
    pub min_issues: i32,
}

impl Default for ChallengeGeneratorConfig {
    fn default() -> Self {
        Self {
            weekly_target_multiplier: 1.1,
            daily_target_multiplier: 1.0,
            min_commits: 1,
            min_prs: 1,
            min_reviews: 1,
            min_issues: 1,
        }
    }
}

/// Recommended challenge targets based on user activity
#[derive(Debug, Clone)]
pub struct RecommendedTargets {
    pub daily_commits: i32,
    pub weekly_commits: i32,
    pub daily_prs: i32,
    pub weekly_prs: i32,
    pub daily_reviews: i32,
    pub weekly_reviews: i32,
    pub daily_issues: i32,
    pub weekly_issues: i32,
}

/// Challenge template for auto-generation
#[derive(Debug, Clone)]
pub struct ChallengeTemplate {
    pub challenge_type: String,
    pub target_metric: String,
    pub target_value: i32,
    pub reward_xp: i32,
}

/// Historical stats for calculating recommended targets
#[derive(Debug, Clone, Default)]
pub struct HistoricalStats {
    /// Total commits in last 4 weeks
    pub commits_4w: i32,
    /// Total PRs in last 4 weeks
    pub prs_4w: i32,
    /// Total reviews in last 4 weeks
    pub reviews_4w: i32,
    /// Total issues in last 4 weeks
    pub issues_4w: i32,
    /// Days with activity in last 4 weeks
    pub active_days_4w: i32,
}

impl HistoricalStats {
    /// Calculate average daily activity
    pub fn avg_daily(&self) -> (f32, f32, f32, f32) {
        let days = self.active_days_4w.max(1) as f32;
        (
            self.commits_4w as f32 / days,
            self.prs_4w as f32 / days,
            self.reviews_4w as f32 / days,
            self.issues_4w as f32 / days,
        )
    }

    /// Calculate average weekly activity
    pub fn avg_weekly(&self) -> (f32, f32, f32, f32) {
        let weeks = 4.0;
        (
            self.commits_4w as f32 / weeks,
            self.prs_4w as f32 / weeks,
            self.reviews_4w as f32 / weeks,
            self.issues_4w as f32 / weeks,
        )
    }
}

/// Calculate recommended challenge targets based on historical stats
pub fn calculate_recommended_targets(
    historical: &HistoricalStats,
    config: &ChallengeGeneratorConfig,
) -> RecommendedTargets {
    let (daily_commits, daily_prs, daily_reviews, daily_issues) = historical.avg_daily();
    let (weekly_commits, weekly_prs, weekly_reviews, weekly_issues) = historical.avg_weekly();

    RecommendedTargets {
        daily_commits: ((daily_commits * config.daily_target_multiplier).ceil() as i32)
            .max(config.min_commits),
        weekly_commits: ((weekly_commits * config.weekly_target_multiplier).ceil() as i32)
            .max(config.min_commits),
        daily_prs: ((daily_prs * config.daily_target_multiplier).ceil() as i32)
            .max(config.min_prs),
        weekly_prs: ((weekly_prs * config.weekly_target_multiplier).ceil() as i32)
            .max(config.min_prs),
        daily_reviews: ((daily_reviews * config.daily_target_multiplier).ceil() as i32)
            .max(config.min_reviews),
        weekly_reviews: ((weekly_reviews * config.weekly_target_multiplier).ceil() as i32)
            .max(config.min_reviews),
        daily_issues: ((daily_issues * config.daily_target_multiplier).ceil() as i32)
            .max(config.min_issues),
        weekly_issues: ((weekly_issues * config.weekly_target_multiplier).ceil() as i32)
            .max(config.min_issues),
    }
}

/// Generate default weekly challenges for a new user or when no historical data
pub fn generate_default_weekly_challenges() -> Vec<ChallengeTemplate> {
    vec![
        ChallengeTemplate {
            challenge_type: "weekly".to_string(),
            target_metric: "commits".to_string(),
            target_value: 5,
            reward_xp: 50,
        },
        ChallengeTemplate {
            challenge_type: "weekly".to_string(),
            target_metric: "prs".to_string(),
            target_value: 2,
            reward_xp: 80,
        },
        ChallengeTemplate {
            challenge_type: "weekly".to_string(),
            target_metric: "reviews".to_string(),
            target_value: 3,
            reward_xp: 60,
        },
    ]
}

/// Generate default daily challenges
pub fn generate_default_daily_challenges() -> Vec<ChallengeTemplate> {
    vec![ChallengeTemplate {
        challenge_type: "daily".to_string(),
        target_metric: "commits".to_string(),
        target_value: 1,
        reward_xp: 10,
    }]
}

/// Generate weekly challenges based on recommended targets
pub fn generate_weekly_challenges(targets: &RecommendedTargets) -> Vec<ChallengeTemplate> {
    vec![
        ChallengeTemplate {
            challenge_type: "weekly".to_string(),
            target_metric: "commits".to_string(),
            target_value: targets.weekly_commits,
            reward_xp: calculate_reward_xp("commits", targets.weekly_commits),
        },
        ChallengeTemplate {
            challenge_type: "weekly".to_string(),
            target_metric: "prs".to_string(),
            target_value: targets.weekly_prs,
            reward_xp: calculate_reward_xp("prs", targets.weekly_prs),
        },
        ChallengeTemplate {
            challenge_type: "weekly".to_string(),
            target_metric: "reviews".to_string(),
            target_value: targets.weekly_reviews,
            reward_xp: calculate_reward_xp("reviews", targets.weekly_reviews),
        },
    ]
}

/// Generate daily challenges based on recommended targets
pub fn generate_daily_challenges(targets: &RecommendedTargets) -> Vec<ChallengeTemplate> {
    vec![ChallengeTemplate {
        challenge_type: "daily".to_string(),
        target_metric: "commits".to_string(),
        target_value: targets.daily_commits,
        reward_xp: calculate_reward_xp("commits", targets.daily_commits),
    }]
}

/// Calculate reward XP based on target metric and value
pub fn calculate_reward_xp(target_metric: &str, target_value: i32) -> i32 {
    let base_xp = match target_metric {
        "commits" => 10,
        "prs" => 40,
        "reviews" => 20,
        "issues" => 25,
        _ => 10,
    };
    base_xp * target_value
}

/// Calculate challenge start and end dates based on type
pub fn calculate_challenge_period(
    challenge_type: &str,
    now: DateTime<Utc>,
) -> (DateTime<Utc>, DateTime<Utc>) {
    match challenge_type {
        "daily" => {
            // Daily challenge: from now to end of today (midnight UTC)
            let today = now.date_naive();
            let tomorrow = today + Duration::days(1);
            let end = tomorrow.and_hms_opt(0, 0, 0).unwrap().and_utc();
            (now, end)
        }
        "weekly" => {
            // Weekly challenge: from now to end of Sunday (midnight UTC)
            let today = now.date_naive();
            let days_until_sunday = (7 - today.weekday().num_days_from_monday()) % 7;
            // If today is Sunday, set end to next Sunday
            let days_to_add = if days_until_sunday == 0 { 7 } else { days_until_sunday };
            let sunday = today + Duration::days(days_to_add as i64);
            let next_monday = sunday + Duration::days(1);
            let end = next_monday.and_hms_opt(0, 0, 0).unwrap().and_utc();
            (now, end)
        }
        _ => (now, now + Duration::days(7)), // Default to 7 days
    }
}

/// Check if it's time to generate new daily challenges (start of day)
pub fn should_generate_daily_challenges(
    last_daily_challenge_date: Option<chrono::NaiveDate>,
    now: DateTime<Utc>,
) -> bool {
    match last_daily_challenge_date {
        Some(date) => now.date_naive() > date,
        None => true, // No challenges yet, generate them
    }
}

/// Check if it's time to generate new weekly challenges (Monday)
pub fn should_generate_weekly_challenges(
    last_weekly_challenge_date: Option<chrono::NaiveDate>,
    now: DateTime<Utc>,
) -> bool {
    let today = now.date_naive();
    let is_monday = today.weekday() == Weekday::Mon;

    match last_weekly_challenge_date {
        Some(date) => {
            // Generate if it's a new week (date is before this week's Monday)
            let days_since_monday = today.weekday().num_days_from_monday();
            let this_monday = today - Duration::days(days_since_monday as i64);
            date < this_monday
        }
        None => true, // No challenges yet, generate them
    }
}

/// Context for updating challenge progress
#[derive(Debug, Clone)]
pub struct ChallengeProgressContext {
    pub total_commits: i32,
    pub total_prs: i32,
    pub total_reviews: i32,
    pub total_issues: i32,
}

/// Calculate the current value for a challenge based on GitHub stats diff
pub fn calculate_progress_for_metric(
    metric: &str,
    prev_stats: &ChallengeProgressContext,
    current_stats: &ChallengeProgressContext,
    challenge_start_stats: Option<&ChallengeProgressContext>,
) -> i32 {
    // If we have start stats, calculate progress since challenge started
    // Otherwise, calculate diff from previous sync
    let base_stats = challenge_start_stats.unwrap_or(prev_stats);

    match metric {
        "commits" => (current_stats.total_commits - base_stats.total_commits).max(0),
        "prs" => (current_stats.total_prs - base_stats.total_prs).max(0),
        "reviews" => (current_stats.total_reviews - base_stats.total_reviews).max(0),
        "issues" => (current_stats.total_issues - base_stats.total_issues).max(0),
        _ => 0,
    }
}

/// Result of challenge progress update
#[derive(Debug, Clone)]
pub struct ChallengeUpdateResult {
    pub challenge_id: i64,
    pub old_value: i32,
    pub new_value: i32,
    pub target_value: i32,
    pub just_completed: bool,
    pub reward_xp: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_calculate_recommended_targets() {
        let historical = HistoricalStats {
            commits_4w: 40,
            prs_4w: 8,
            reviews_4w: 12,
            issues_4w: 4,
            active_days_4w: 20,
        };

        let config = ChallengeGeneratorConfig::default();
        let targets = calculate_recommended_targets(&historical, &config);

        // Weekly: avg 10 commits/week * 1.1 = 11
        assert_eq!(targets.weekly_commits, 11);
        // Daily: avg 2 commits/day * 1.0 = 2
        assert_eq!(targets.daily_commits, 2);
    }

    #[test]
    fn test_calculate_reward_xp() {
        assert_eq!(calculate_reward_xp("commits", 5), 50);
        assert_eq!(calculate_reward_xp("prs", 2), 80);
        assert_eq!(calculate_reward_xp("reviews", 3), 60);
        assert_eq!(calculate_reward_xp("issues", 4), 100);
    }

    #[test]
    fn test_generate_default_weekly_challenges() {
        let challenges = generate_default_weekly_challenges();
        assert_eq!(challenges.len(), 3);
        assert!(challenges.iter().any(|c| c.target_metric == "commits"));
        assert!(challenges.iter().any(|c| c.target_metric == "prs"));
        assert!(challenges.iter().any(|c| c.target_metric == "reviews"));
    }

    #[test]
    fn test_calculate_challenge_period_daily() {
        let now = Utc::now();
        let (start, end) = calculate_challenge_period("daily", now);

        assert_eq!(start, now);
        assert!(end > now);
        // End should be at midnight
        assert_eq!(end.time().hour(), 0);
        assert_eq!(end.time().minute(), 0);
    }

    #[test]
    fn test_calculate_challenge_period_weekly() {
        let now = Utc::now();
        let (start, end) = calculate_challenge_period("weekly", now);

        assert_eq!(start, now);
        assert!(end > now);
        assert!(end <= now + Duration::days(8)); // Max 7 days + 1 day buffer
    }

    #[test]
    fn test_calculate_progress_for_metric() {
        let prev = ChallengeProgressContext {
            total_commits: 100,
            total_prs: 10,
            total_reviews: 20,
            total_issues: 5,
        };
        let current = ChallengeProgressContext {
            total_commits: 105,
            total_prs: 12,
            total_reviews: 23,
            total_issues: 6,
        };

        assert_eq!(calculate_progress_for_metric("commits", &prev, &current, None), 5);
        assert_eq!(calculate_progress_for_metric("prs", &prev, &current, None), 2);
        assert_eq!(calculate_progress_for_metric("reviews", &prev, &current, None), 3);
        assert_eq!(calculate_progress_for_metric("issues", &prev, &current, None), 1);
    }

    #[test]
    fn test_should_generate_daily_challenges() {
        let now = Utc::now();
        let today = now.date_naive();
        let yesterday = today - Duration::days(1);

        // No previous challenge - should generate
        assert!(should_generate_daily_challenges(None, now));

        // Last challenge was today - should not generate
        assert!(!should_generate_daily_challenges(Some(today), now));

        // Last challenge was yesterday - should generate
        assert!(should_generate_daily_challenges(Some(yesterday), now));
    }
}
