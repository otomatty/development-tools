//! Home Data Loader
//!
//! Functions for loading user data from the API for the home page.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   └─ src/components/pages/home_page.rs
//! Dependencies:
//!   └─ src/tauri_api.rs
//! Related Documentation:
//!   ├─ Issue: https://github.com/otomatty/development-tools/issues/117
//!   └─ Phase 1 Performance: https://github.com/otomatty/development-tools/issues/124

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::tauri_api;
use crate::types::{Badge, GitHubStats, LevelInfo, UserStats};

/// Load user data from API with parallel execution
///
/// This function loads all user data in parallel using spawn_local().
/// Each API call is executed concurrently using leptos::task::spawn_local.
/// This significantly improves the initial page load time compared to sequential execution.
///
/// Performance improvement:
/// - Before: ~300ms (sequential: 100ms + 50ms + 50ms + 100ms)
/// - After: ~100ms (parallel: max of 100ms)
pub async fn load_user_data(
    set_github_stats: WriteSignal<Option<GitHubStats>>,
    set_level_info: WriteSignal<Option<LevelInfo>>,
    set_user_stats: WriteSignal<Option<UserStats>>,
    set_badges: WriteSignal<Vec<Badge>>,
    set_error: WriteSignal<Option<String>>,
    set_data_from_cache: WriteSignal<bool>,
    set_cache_timestamp: WriteSignal<Option<String>>,
) {
    // Load GitHub stats with cache fallback (Task 1/4)
    {
        let set_github_stats = set_github_stats.clone();
        let set_data_from_cache = set_data_from_cache.clone();
        let set_cache_timestamp = set_cache_timestamp.clone();
        let set_error = set_error.clone();

        spawn_local(async move {
            match tauri_api::get_github_stats_with_cache().await {
                Ok(response) => {
                    set_github_stats.set(Some(response.data));
                    if response.from_cache {
                        set_data_from_cache.set(true);
                        set_cache_timestamp.set(response.cached_at);
                        web_sys::console::log_1(
                            &"GitHub stats loaded from cache (offline mode)".into(),
                        );
                    } else {
                        // Fresh data - clear cache indicator
                        set_data_from_cache.set(false);
                        set_cache_timestamp.set(None);
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to get GitHub stats: {}", e).into());
                    set_error.set(Some(format!("Failed to load GitHub stats: {}", e)));
                }
            }
        });
    }

    // Load level info (Task 2/4)
    {
        let set_level_info = set_level_info.clone();
        spawn_local(async move {
            match tauri_api::get_level_info().await {
                Ok(info) => set_level_info.set(info),
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to get level info: {}", e).into());
                }
            }
        });
    }

    // Load user stats (Task 3/4)
    {
        let set_user_stats = set_user_stats.clone();
        spawn_local(async move {
            match tauri_api::get_user_stats().await {
                Ok(stats) => set_user_stats.set(stats),
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to get user stats: {}", e).into());
                }
            }
        });
    }

    // Load badges (Task 4/4)
    {
        let set_badges = set_badges.clone();
        spawn_local(async move {
            match tauri_api::get_badges().await {
                Ok(b) => set_badges.set(b),
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to get badges: {}", e).into());
                }
            }
        });
    }
}
