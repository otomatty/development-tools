//! Level calculation utilities

/// Base XP required for level 2
const BASE_XP: i32 = 100;
/// XP growth factor per level
const GROWTH_FACTOR: f64 = 1.15;

/// Calculate total XP required to reach a specific level
pub fn xp_for_level(level: i32) -> i32 {
    if level <= 1 {
        return 0;
    }
    let mut total_xp = 0;
    for l in 2..=level {
        let xp_for_this_level = (BASE_XP as f64 * GROWTH_FACTOR.powi(l - 2)).round() as i32;
        total_xp += xp_for_this_level;
    }
    total_xp
}

/// Calculate current level from total XP
pub fn level_from_xp(total_xp: impl Into<i32>) -> i32 {
    let total_xp = total_xp.into();
    let mut level = 1;
    let mut accumulated_xp = 0;
    loop {
        let xp_for_next = (BASE_XP as f64 * GROWTH_FACTOR.powi(level - 1)).round() as i32;
        if accumulated_xp + xp_for_next > total_xp {
            break;
        }
        accumulated_xp += xp_for_next;
        level += 1;
    }
    level
}

/// Calculate XP progress within current level
pub fn xp_progress_in_level(total_xp: i32) -> (i32, i32) {
    let current_level = level_from_xp(total_xp);
    let xp_at_current_level = xp_for_level(current_level);
    let xp_at_next_level = xp_for_level(current_level + 1);
    let current_progress = total_xp - xp_at_current_level;
    let level_requirement = xp_at_next_level - xp_at_current_level;
    (current_progress, level_requirement)
}

/// Calculate XP remaining to reach next level
pub fn xp_to_next_level(total_xp: i32) -> i32 {
    let current_level = level_from_xp(total_xp);
    let xp_at_next_level = xp_for_level(current_level + 1);
    xp_at_next_level - total_xp
}

/// Get level title/rank name based on level
pub fn level_title(level: i32) -> &'static str {
    match level {
        1..=4 => "Novice",
        5..=9 => "Apprentice",
        10..=14 => "Developer",
        15..=24 => "Senior Developer",
        25..=39 => "Expert",
        40..=59 => "Master",
        60..=79 => "Grandmaster",
        80..=99 => "Legend",
        _ => "Mythic",
    }
}

/// Level system module (for backward compatibility)
pub mod level {
    pub use super::{
        level_from_xp, level_title, progress_to_next_level, xp_for_level, xp_progress_in_level,
        xp_to_next_level,
    };
}

/// Calculate progress to next level as percentage
pub fn progress_to_next_level(total_xp: i32) -> f32 {
    let (current, required) = xp_progress_in_level(total_xp);
    if required == 0 {
        return 100.0;
    }
    (current as f32 / required as f32) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xp_for_level_1() {
        assert_eq!(xp_for_level(1), 0);
    }

    #[test]
    fn test_xp_for_level_2() {
        assert_eq!(xp_for_level(2), 100);
    }

    #[test]
    fn test_xp_for_level_3() {
        // Level 2: 100, Level 3: 100 * 1.15 = 115
        assert_eq!(xp_for_level(3), 215);
    }

    #[test]
    fn test_level_from_xp_0() {
        assert_eq!(level_from_xp(0), 1);
    }

    #[test]
    fn test_level_from_xp_99() {
        assert_eq!(level_from_xp(99), 1);
    }

    #[test]
    fn test_level_from_xp_100() {
        assert_eq!(level_from_xp(100), 2);
    }

    #[test]
    fn test_level_from_xp_214() {
        assert_eq!(level_from_xp(214), 2);
    }

    #[test]
    fn test_level_from_xp_215() {
        assert_eq!(level_from_xp(215), 3);
    }

    #[test]
    fn test_xp_progress_at_start() {
        let (current, required) = xp_progress_in_level(0);
        assert_eq!(current, 0);
        assert_eq!(required, 100); // XP needed for level 2
    }

    #[test]
    fn test_xp_progress_mid_level() {
        let (current, required) = xp_progress_in_level(50);
        assert_eq!(current, 50);
        assert_eq!(required, 100);
    }

    #[test]
    fn test_xp_progress_level_2() {
        let (current, required) = xp_progress_in_level(150);
        // At level 2, need 115 for level 3
        // Current progress: 150 - 100 = 50
        assert_eq!(current, 50);
        assert_eq!(required, 115);
    }

    #[test]
    fn test_level_title() {
        assert_eq!(level_title(1), "Novice");
        assert_eq!(level_title(5), "Apprentice");
        assert_eq!(level_title(10), "Developer");
        assert_eq!(level_title(15), "Senior Developer");
        assert_eq!(level_title(25), "Expert");
        assert_eq!(level_title(40), "Master");
        assert_eq!(level_title(60), "Grandmaster");
        assert_eq!(level_title(80), "Legend");
        assert_eq!(level_title(100), "Mythic");
    }
}
