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
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

use leptos::prelude::*;

use crate::tauri_api;
use crate::types::{Badge, GitHubStats, LevelInfo, UserStats};

/// Load user data from API
pub async fn load_user_data(
    set_github_stats: WriteSignal<Option<GitHubStats>>,
    set_level_info: WriteSignal<Option<LevelInfo>>,
    set_user_stats: WriteSignal<Option<UserStats>>,
    set_badges: WriteSignal<Vec<Badge>>,
    set_error: WriteSignal<Option<String>>,
    set_data_from_cache: WriteSignal<bool>,
    set_cache_timestamp: WriteSignal<Option<String>>,
) {
    // Load GitHub stats with cache fallback
    match tauri_api::get_github_stats_with_cache().await {
        Ok(response) => {
            set_github_stats.set(Some(response.data));
            if response.from_cache {
                set_data_from_cache.set(true);
                set_cache_timestamp.set(response.cached_at);
                web_sys::console::log_1(&"GitHub stats loaded from cache (offline mode)".into());
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

    // Load level info
    if let Ok(info) = tauri_api::get_level_info().await {
        set_level_info.set(info);
    }

    // Load user stats
    if let Ok(stats) = tauri_api::get_user_stats().await {
        set_user_stats.set(stats);
    }

    // Load badges
    if let Ok(b) = tauri_api::get_badges().await {
        set_badges.set(b);
    }
}
