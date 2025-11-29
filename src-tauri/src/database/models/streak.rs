//! Streak calculation utilities

use serde::{Deserialize, Serialize};

/// Streak milestone definition
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

/// Daily bonus XP for maintaining streak
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

/// Days until next milestone
pub fn days_to_next_milestone(current_streak: i32) -> Option<i32> {
    get_next_milestone(current_streak).map(|m| m.days - current_streak)
}

/// Check if streak is about to break (no activity today)
pub fn is_streak_at_risk(last_activity: Option<chrono::NaiveDate>) -> bool {
    match last_activity {
        None => false, // No streak to break
        Some(date) => {
            let today = chrono::Utc::now().date_naive();
            // If last activity was before yesterday, streak is at risk
            date < today
        }
    }
}

/// Streak-related constants and utilities module (for backward compatibility)
pub mod streak {
    pub use super::{
        calculate_streak_bonus, days_to_next_milestone, get_next_milestone, is_streak_at_risk,
        StreakBonusResult, StreakMilestone, DAILY_STREAK_BONUS, STREAK_MILESTONES,
    };
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
    fn test_next_milestone() {
        assert!(get_next_milestone(0).is_some());
        assert_eq!(get_next_milestone(0).unwrap().days, 7);
        assert_eq!(get_next_milestone(7).unwrap().days, 14);
        assert_eq!(get_next_milestone(365), None);
    }

    #[test]
    fn test_days_to_next_milestone() {
        assert_eq!(days_to_next_milestone(0), Some(7));
        assert_eq!(days_to_next_milestone(5), Some(2));
        assert_eq!(days_to_next_milestone(7), Some(7)); // Next is 14
        assert_eq!(days_to_next_milestone(365), None);
    }
}
