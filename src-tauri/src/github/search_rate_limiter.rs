//! Sliding-window rate limit tracker for GitHub Search API.
//!
//! GitHub does not expose the Search API rate limit via response headers, so we
//! approximate it in-process by recording call timestamps and evicting entries
//! older than the rolling window (default: 60 seconds, 30 requests/min for
//! authenticated users).
//!
//! `GitHubClient` is constructed per Tauri command invocation, so the limiter
//! lives as a process-global behind `OnceLock`.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

/// Authenticated user limit per minute. Unauthenticated callers actually have
/// 10/min — out of scope for this issue; override via `with_limit` if needed.
pub const DEFAULT_SEARCH_LIMIT: u32 = 30;
pub const DEFAULT_WINDOW_SECS: i64 = 60;

pub trait Clock: Send + Sync {
    fn now_secs(&self) -> i64;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now_secs(&self) -> i64 {
        chrono::Utc::now().timestamp()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AcquireOutcome {
    Granted,
    /// Slot unavailable. Carries both the suggested sleep `duration` and the
    /// epoch-seconds `reset` (= oldest in-window timestamp + window) so
    /// callers can return a `RateLimited(reset)` error without re-locking.
    WaitFor {
        duration: Duration,
        reset: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchSnapshot {
    pub limit: i32,
    pub remaining: i32,
    pub reset: i64,
    pub used: i32,
}

pub struct SearchRateLimiter {
    limit: u32,
    window_secs: i64,
    timestamps: Mutex<VecDeque<i64>>,
    clock: Arc<dyn Clock>,
}

impl Default for SearchRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchRateLimiter {
    pub fn new() -> Self {
        Self {
            limit: DEFAULT_SEARCH_LIMIT,
            window_secs: DEFAULT_WINDOW_SECS,
            timestamps: Mutex::new(VecDeque::with_capacity(DEFAULT_SEARCH_LIMIT as usize)),
            clock: Arc::new(SystemClock),
        }
    }

    #[allow(dead_code)] // exposed for tests and future opt-in overrides
    pub fn with_clock(clock: Arc<dyn Clock>) -> Self {
        Self {
            limit: DEFAULT_SEARCH_LIMIT,
            window_secs: DEFAULT_WINDOW_SECS,
            timestamps: Mutex::new(VecDeque::with_capacity(DEFAULT_SEARCH_LIMIT as usize)),
            clock,
        }
    }

    #[allow(dead_code)] // exposed for tests and future opt-in overrides
    pub fn with_limit(mut self, limit: u32) -> Self {
        // 1..=i32::MAX: 0 would panic on `front().expect` in `try_acquire`
        // when the deque is empty (impossible-to-fill state), and any value
        // above `i32::MAX` would corrupt the `u32 -> i32` cast in `snapshot()`.
        self.limit = limit.clamp(1, i32::MAX as u32);
        self
    }

    /// Drop timestamps older than the window. Also treats a backward clock
    /// jump (`now < oldest`) as a signal to evict everything — safer than
    /// leaving stale entries that would never age out.
    fn evict_expired(&self, deque: &mut VecDeque<i64>, now: i64) {
        let cutoff = now - self.window_secs;
        while let Some(&front) = deque.front() {
            if front <= cutoff || front > now {
                deque.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn try_acquire(&self) -> AcquireOutcome {
        let now = self.clock.now_secs();
        let mut deque = self
            .timestamps
            .lock()
            .expect("SearchRateLimiter mutex poisoned");
        self.evict_expired(&mut deque, now);

        if (deque.len() as u32) < self.limit {
            deque.push_back(now);
            AcquireOutcome::Granted
        } else {
            let oldest = *deque.front().expect("deque is full so front must exist");
            let reset = oldest + self.window_secs;
            let wait_secs = (reset - now).max(1);
            AcquireOutcome::WaitFor {
                duration: Duration::from_secs(wait_secs as u64),
                reset,
            }
        }
    }

    pub fn snapshot(&self) -> SearchSnapshot {
        let now = self.clock.now_secs();
        let mut deque = self
            .timestamps
            .lock()
            .expect("SearchRateLimiter mutex poisoned");
        self.evict_expired(&mut deque, now);

        let used = deque.len() as i32;
        let limit = self.limit as i32;
        let remaining = (limit - used).max(0);
        let reset = deque
            .front()
            .map(|oldest| oldest + self.window_secs)
            .unwrap_or(now + self.window_secs);

        SearchSnapshot {
            limit,
            remaining,
            reset,
            used,
        }
    }
}

/// Process-global limiter. `GitHubClient` is constructed per command, so the
/// limiter cannot live on the client struct.
pub fn global() -> &'static SearchRateLimiter {
    static G: OnceLock<SearchRateLimiter> = OnceLock::new();
    G.get_or_init(SearchRateLimiter::new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI64, Ordering};

    struct FakeClock(AtomicI64);

    impl FakeClock {
        fn new(start: i64) -> Arc<Self> {
            Arc::new(Self(AtomicI64::new(start)))
        }
        fn set(&self, t: i64) {
            self.0.store(t, Ordering::SeqCst);
        }
    }

    impl Clock for FakeClock {
        fn now_secs(&self) -> i64 {
            self.0.load(Ordering::SeqCst)
        }
    }

    fn limiter_with(clock: Arc<FakeClock>) -> SearchRateLimiter {
        SearchRateLimiter::with_clock(clock as Arc<dyn Clock>)
    }

    #[test]
    fn acquire_under_limit_returns_granted() {
        let clock = FakeClock::new(0);
        let limiter = limiter_with(clock);
        for _ in 0..29 {
            assert_eq!(limiter.try_acquire(), AcquireOutcome::Granted);
        }
        let snap = limiter.snapshot();
        assert_eq!(snap.used, 29);
        assert_eq!(snap.remaining, 1);
    }

    #[test]
    fn acquire_at_limit_returns_wait() {
        let clock = FakeClock::new(0);
        let limiter = limiter_with(clock);
        for _ in 0..30 {
            assert_eq!(limiter.try_acquire(), AcquireOutcome::Granted);
        }
        assert_eq!(
            limiter.try_acquire(),
            AcquireOutcome::WaitFor {
                duration: Duration::from_secs(60),
                reset: 60,
            }
        );
    }

    #[test]
    fn oldest_ages_out_after_window() {
        let clock = FakeClock::new(0);
        let limiter = limiter_with(clock.clone());
        for _ in 0..30 {
            limiter.try_acquire();
        }
        clock.set(61);
        assert_eq!(limiter.try_acquire(), AcquireOutcome::Granted);
        let snap = limiter.snapshot();
        // All 30 original entries aged out (cutoff = 61 - 60 = 1, so ts=0 is evicted)
        // Only the new entry at t=61 remains.
        assert_eq!(snap.used, 1);
        assert_eq!(snap.remaining, 29);
    }

    #[test]
    fn partial_aging() {
        let clock = FakeClock::new(0);
        let limiter = limiter_with(clock.clone());
        for _ in 0..10 {
            limiter.try_acquire();
        }
        clock.set(30);
        for _ in 0..10 {
            limiter.try_acquire();
        }
        clock.set(61);
        // cutoff = 1: ts=0 entries evicted, ts=30 entries kept.
        let snap = limiter.snapshot();
        assert_eq!(snap.used, 10);
        assert_eq!(snap.remaining, 20);
        // reset = oldest (30) + window (60) = 90
        assert_eq!(snap.reset, 90);
    }

    #[test]
    fn snapshot_when_empty() {
        let clock = FakeClock::new(100);
        let limiter = limiter_with(clock);
        let snap = limiter.snapshot();
        assert_eq!(snap.limit, 30);
        assert_eq!(snap.remaining, 30);
        assert_eq!(snap.used, 0);
        assert_eq!(snap.reset, 160);
    }

    #[test]
    fn snapshot_reset_uses_oldest_plus_window() {
        let clock = FakeClock::new(0);
        let limiter = limiter_with(clock.clone());
        limiter.try_acquire();
        clock.set(5);
        limiter.try_acquire();
        let snap = limiter.snapshot();
        assert_eq!(snap.reset, 0 + 60);
        assert_eq!(snap.used, 2);
    }

    #[test]
    fn wait_duration_non_negative() {
        let clock = FakeClock::new(0);
        let limiter = limiter_with(clock.clone());
        for _ in 0..30 {
            limiter.try_acquire();
        }
        clock.set(59);
        match limiter.try_acquire() {
            AcquireOutcome::WaitFor { duration, reset } => {
                assert!(duration.as_secs() >= 1);
                assert_eq!(reset, 60);
            }
            AcquireOutcome::Granted => panic!("should be at limit"),
        }
    }

    #[test]
    fn clock_jump_backward_safe() {
        let clock = FakeClock::new(1000);
        let limiter = limiter_with(clock.clone());
        for _ in 0..30 {
            limiter.try_acquire();
        }
        // Clock jumps backward past all recorded timestamps.
        clock.set(100);
        // Evicts everything (front > now branch), so we can acquire again.
        assert_eq!(limiter.try_acquire(), AcquireOutcome::Granted);
        let snap = limiter.snapshot();
        assert_eq!(snap.used, 1);
    }

    #[test]
    fn with_limit_overrides_default() {
        let clock = FakeClock::new(0);
        let limiter = SearchRateLimiter::with_clock(clock as Arc<dyn Clock>).with_limit(2);
        assert_eq!(limiter.try_acquire(), AcquireOutcome::Granted);
        assert_eq!(limiter.try_acquire(), AcquireOutcome::Granted);
        assert!(matches!(
            limiter.try_acquire(),
            AcquireOutcome::WaitFor { .. }
        ));
    }

    #[test]
    fn with_limit_zero_clamped_to_one() {
        let clock = FakeClock::new(0);
        let limiter = SearchRateLimiter::with_clock(clock as Arc<dyn Clock>).with_limit(0);
        // Clamped to 1 — first acquire succeeds, second is full.
        assert_eq!(limiter.try_acquire(), AcquireOutcome::Granted);
        assert!(matches!(
            limiter.try_acquire(),
            AcquireOutcome::WaitFor { .. }
        ));
    }

    #[test]
    fn with_limit_above_i32_max_clamped() {
        let clock = FakeClock::new(0);
        let limiter = SearchRateLimiter::with_clock(clock as Arc<dyn Clock>).with_limit(u32::MAX);
        // `snapshot()` must not corrupt; `limit` fits in i32.
        let snap = limiter.snapshot();
        assert_eq!(snap.limit, i32::MAX);
    }
}
