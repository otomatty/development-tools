/*!
 * Sync Scheduler Concept
 *
 * Drives the background GitHub sync (startup / periodic / background) based on
 * `user_settings`. See `sync_scheduler.spec.md` for the full specification.
 *
 * DEPENDENCY MAP:
 *
 * Parents (Files that import this Concept):
 *   ├─ src-tauri/src/lib.rs                       (start_scheduler in setup)
 *   └─ src-tauri/src/commands/settings.rs         (notify on update)
 *   └─ src-tauri/src/commands/scheduler.rs        (status command)
 *
 * Related Documentation:
 *   ├─ Spec: ./sync_scheduler.spec.md
 *   ├─ Tests: ./actions.rs (#[cfg(test)] mod tests), ./runner.rs (#[cfg(test)] mod tests)
 *   ├─ Issue: https://github.com/otomatty/development-tools/issues/180
 *   ├─ Audit: docs/02_research/2026_04/20260425_github_integration_audit.md (§4.1, §9.4, §8 G-07)
 *   └─ Cache spec: src-tauri/src/commands/cache_fallback.spec.md
 */

pub mod actions;
pub mod runner;
pub mod state;

pub use actions::{decide_action, next_sync_at};
pub use runner::{start_scheduler, SyncSchedulerHandle};
pub use state::{
    skip_reasons, SchedulerAction, SchedulerInputs, SchedulerStatus, GITHUB_STATS_SYNC_TYPE,
};
