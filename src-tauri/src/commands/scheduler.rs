//! Sync scheduler commands.
//!
//! Surface the live status of the background sync scheduler to the UI.

use crate::sync_scheduler::{SchedulerStatus, SyncSchedulerHandle};

/// Get the current scheduler status (next sync time, last sync, skip reason).
///
/// Used by `SyncSettings` to show the user when the next automatic sync will
/// run and why a previous sync was skipped (e.g. rate limited).
#[tauri::command]
pub async fn get_scheduler_status(
    scheduler: tauri::State<'_, SyncSchedulerHandle>,
) -> Result<SchedulerStatus, String> {
    Ok(scheduler.status().await)
}
