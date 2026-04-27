//! Sync Scheduler decision logic.
//!
//! Pure functions that evaluate whether a sync is due. Side effects (DB I/O,
//! GitHub calls, sleeping) live in [`super::runner`].

use chrono::{DateTime, Duration, Utc};

use super::state::{
    skip_reasons, SchedulerAction, SchedulerInputs, MAX_SLEEP_SECONDS, MIN_SLEEP_SECONDS,
};

/// Critical-rate-limit threshold. Below this many remaining REST calls the
/// scheduler will skip and sleep until the reset.
const RATE_LIMIT_FLOOR: i32 = 50;

/// Decide what the scheduler should do at `inputs.now`.
///
/// Order of evaluation, from highest to lowest precedence:
/// 1. `background_sync = false` → Idle
/// 2. `sync_interval_minutes <= 0` (and not first-run startup) → Idle
/// 3. Rate limit critical and reset is in the future → RateLimited
/// 4. First run with `sync_on_startup = true` → RunSync
/// 5. Last sync older than the interval → RunSync
/// 6. Otherwise → Sleep until the next due time
pub fn decide_action(inputs: &SchedulerInputs) -> SchedulerAction {
    if !inputs.background_sync {
        // Even with sync_on_startup=true we honor background_sync=false on
        // subsequent loop iterations; the startup sync runs *once* before the
        // scheduler enters this evaluation if the user wants it.
        if !(inputs.is_first_run && inputs.sync_on_startup) {
            return SchedulerAction::Idle {
                reason: skip_reasons::BACKGROUND_DISABLED,
            };
        }
    }

    if inputs.is_first_run && inputs.sync_on_startup {
        return SchedulerAction::RunSync;
    }

    if inputs.sync_interval_minutes <= 0 {
        return SchedulerAction::Idle {
            reason: skip_reasons::MANUAL_ONLY,
        };
    }

    if let Some(rate_action) = check_rate_limit(inputs) {
        return rate_action;
    }

    let interval = Duration::minutes(inputs.sync_interval_minutes as i64);
    match inputs.last_sync_at {
        None => SchedulerAction::RunSync,
        Some(last) => {
            let elapsed = inputs.now.signed_duration_since(last);
            if elapsed >= interval {
                SchedulerAction::RunSync
            } else {
                let remaining = interval - elapsed;
                SchedulerAction::Sleep {
                    seconds: clamp_sleep(remaining),
                }
            }
        }
    }
}

/// Project the next time a sync would run, given the same inputs.
///
/// Returns `None` when scheduling is disabled (manual-only or
/// background_sync=false).
pub fn next_sync_at(inputs: &SchedulerInputs) -> Option<DateTime<Utc>> {
    if !inputs.background_sync {
        return None;
    }
    if inputs.sync_interval_minutes <= 0 {
        return None;
    }
    let interval = Duration::minutes(inputs.sync_interval_minutes as i64);
    match inputs.last_sync_at {
        None => Some(inputs.now),
        Some(last) => Some(last + interval),
    }
}

fn check_rate_limit(inputs: &SchedulerInputs) -> Option<SchedulerAction> {
    let remaining = inputs.rate_limit_remaining?;
    if remaining > RATE_LIMIT_FLOOR {
        return None;
    }
    let reset = inputs.rate_limit_reset_at?;
    if reset <= inputs.now {
        return None;
    }
    let until_reset = reset - inputs.now;
    Some(SchedulerAction::RateLimited {
        reason: skip_reasons::RATE_LIMITED,
        seconds: clamp_sleep(until_reset),
    })
}

fn clamp_sleep(d: Duration) -> u64 {
    let secs = d.num_seconds().max(0) as u64;
    secs.clamp(MIN_SLEEP_SECONDS, MAX_SLEEP_SECONDS)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_inputs() -> SchedulerInputs {
        SchedulerInputs {
            sync_on_startup: false,
            sync_interval_minutes: 60,
            background_sync: true,
            last_sync_at: None,
            rate_limit_remaining: None,
            rate_limit_reset_at: None,
            is_first_run: false,
            now: Utc::now(),
        }
    }

    /// TC-001: `sync_on_startup=true` runs immediately on first iteration.
    #[test]
    fn first_run_with_startup_sync_runs() {
        let inputs = SchedulerInputs {
            is_first_run: true,
            sync_on_startup: true,
            ..base_inputs()
        };
        assert_eq!(decide_action(&inputs), SchedulerAction::RunSync);
    }

    /// TC-002: First run without startup sync waits for the interval.
    #[test]
    fn first_run_without_startup_sync_runs_when_no_history() {
        let inputs = SchedulerInputs {
            is_first_run: true,
            sync_on_startup: false,
            ..base_inputs()
        };
        // No history => RunSync (catch-up) when background_sync is on.
        assert_eq!(decide_action(&inputs), SchedulerAction::RunSync);
    }

    /// TC-003: `background_sync=false` halts the scheduler unless it's the
    /// first run with startup sync.
    #[test]
    fn background_sync_off_idles() {
        let inputs = SchedulerInputs {
            background_sync: false,
            is_first_run: false,
            ..base_inputs()
        };
        assert!(matches!(
            decide_action(&inputs),
            SchedulerAction::Idle { .. }
        ));
    }

    /// TC-004: `background_sync=false` still allows startup sync once.
    #[test]
    fn background_sync_off_allows_one_startup_sync() {
        let inputs = SchedulerInputs {
            background_sync: false,
            is_first_run: true,
            sync_on_startup: true,
            ..base_inputs()
        };
        assert_eq!(decide_action(&inputs), SchedulerAction::RunSync);
    }

    /// TC-005: `sync_interval_minutes=0` (manual only) idles.
    #[test]
    fn manual_only_idles() {
        let inputs = SchedulerInputs {
            sync_interval_minutes: 0,
            ..base_inputs()
        };
        assert!(matches!(
            decide_action(&inputs),
            SchedulerAction::Idle { .. }
        ));
    }

    /// TC-006: When the interval has elapsed since last sync, run.
    #[test]
    fn runs_when_interval_elapsed() {
        let now = Utc::now();
        let inputs = SchedulerInputs {
            sync_interval_minutes: 5,
            last_sync_at: Some(now - Duration::minutes(10)),
            now,
            ..base_inputs()
        };
        assert_eq!(decide_action(&inputs), SchedulerAction::RunSync);
    }

    /// TC-007: When the interval has not elapsed, sleep for the remainder.
    #[test]
    fn sleeps_when_interval_not_elapsed() {
        let now = Utc::now();
        let inputs = SchedulerInputs {
            sync_interval_minutes: 60,
            last_sync_at: Some(now - Duration::minutes(10)),
            now,
            ..base_inputs()
        };
        match decide_action(&inputs) {
            SchedulerAction::Sleep { seconds } => {
                // ~50 minutes remaining, but capped at MAX_SLEEP_SECONDS.
                assert!(seconds <= MAX_SLEEP_SECONDS);
                assert!(seconds >= MIN_SLEEP_SECONDS);
            }
            other => panic!("expected Sleep, got {other:?}"),
        }
    }

    /// TC-008: Rate limit critical with future reset → RateLimited.
    #[test]
    fn rate_limit_skips_until_reset() {
        let now = Utc::now();
        let inputs = SchedulerInputs {
            sync_interval_minutes: 5,
            last_sync_at: Some(now - Duration::minutes(10)),
            rate_limit_remaining: Some(10),
            rate_limit_reset_at: Some(now + Duration::minutes(2)),
            now,
            ..base_inputs()
        };
        match decide_action(&inputs) {
            SchedulerAction::RateLimited { seconds, .. } => {
                assert!(seconds >= MIN_SLEEP_SECONDS);
            }
            other => panic!("expected RateLimited, got {other:?}"),
        }
    }

    /// TC-009: Rate limit not critical → normal eligibility wins.
    #[test]
    fn rate_limit_with_plenty_remaining_runs() {
        let now = Utc::now();
        let inputs = SchedulerInputs {
            sync_interval_minutes: 5,
            last_sync_at: Some(now - Duration::minutes(10)),
            rate_limit_remaining: Some(4500),
            rate_limit_reset_at: Some(now + Duration::minutes(50)),
            now,
            ..base_inputs()
        };
        assert_eq!(decide_action(&inputs), SchedulerAction::RunSync);
    }

    /// TC-010: Rate limit critical but reset is in the past → not skipped.
    #[test]
    fn rate_limit_with_past_reset_runs() {
        let now = Utc::now();
        let inputs = SchedulerInputs {
            sync_interval_minutes: 5,
            last_sync_at: Some(now - Duration::minutes(10)),
            rate_limit_remaining: Some(10),
            rate_limit_reset_at: Some(now - Duration::minutes(1)),
            now,
            ..base_inputs()
        };
        assert_eq!(decide_action(&inputs), SchedulerAction::RunSync);
    }

    /// TC-011: `next_sync_at` returns last+interval.
    #[test]
    fn next_sync_at_projects_correctly() {
        let now = Utc::now();
        let inputs = SchedulerInputs {
            sync_interval_minutes: 30,
            last_sync_at: Some(now - Duration::minutes(10)),
            now,
            ..base_inputs()
        };
        let next = next_sync_at(&inputs).unwrap();
        let diff = next - now;
        // last+30min = now-10min + 30min = now+20min
        assert!(diff.num_seconds() >= 19 * 60);
        assert!(diff.num_seconds() <= 21 * 60);
    }

    /// TC-012: `next_sync_at` is None when scheduling is disabled.
    #[test]
    fn next_sync_at_none_when_disabled() {
        let disabled = SchedulerInputs {
            sync_interval_minutes: 0,
            ..base_inputs()
        };
        assert!(next_sync_at(&disabled).is_none());

        let bg_off = SchedulerInputs {
            background_sync: false,
            ..base_inputs()
        };
        assert!(next_sync_at(&bg_off).is_none());
    }
}
