//! Badge-related models and utilities

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

// Re-export badge module types at the module level for backward compatibility
pub use badge::{
    BadgeCondition, BadgeDefinition, BadgeEvalContext, BadgeEvalResult, BadgeProgress,
    BadgeWithProgress, calculate_progress, evaluate_badges, evaluate_condition,
    get_all_badge_definitions, get_badges_with_progress, get_near_completion_badges,
};
