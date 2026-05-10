//! Small numeric helpers shared across commands.
//!
//! Currently just `clamp_to_u64`, used wherever a possibly-negative
//! cumulative GitHub counter has to feed `XpBreakdown::calculate`'s
//! `u64`-typed inputs. Centralising it avoids the duplicate definitions
//! `commands::github` and `commands::gamification` previously kept in
//! sync by hand (PR #217 review).

/// Clamp a possibly-negative `i32` count to `u64` (negative → 0).
///
/// Used as the boundary between GitHub's `i32`-typed cumulative counters
/// and `XpBreakdown::calculate`'s `u64`-typed inputs. A regression in
/// any cumulative metric (e.g. losing a star) clamps to 0 instead of
/// wrapping into a giant positive value.
#[inline]
pub fn clamp_to_u64(value: i32) -> u64 {
    value.max(0) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_negative_to_zero() {
        assert_eq!(clamp_to_u64(-1), 0);
        assert_eq!(clamp_to_u64(i32::MIN), 0);
    }

    #[test]
    fn passes_non_negative_through() {
        assert_eq!(clamp_to_u64(0), 0);
        assert_eq!(clamp_to_u64(42), 42);
        assert_eq!(clamp_to_u64(i32::MAX), i32::MAX as u64);
    }
}
