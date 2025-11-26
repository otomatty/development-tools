//! Home page components
//!
//! This module contains all components for the gamification home page.

pub mod login_card;
pub mod profile_card;
pub mod stats_display;
pub mod contribution_graph;
pub mod badge_grid;

pub use login_card::LoginCard;
pub use profile_card::ProfileCard;
pub use stats_display::StatsDisplay;
pub use contribution_graph::ContributionGraph;
pub use badge_grid::BadgeGrid;

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::tauri_api;
use crate::types::{AuthState, Badge, BadgeDefinition, GitHubStats, LevelInfo, UserStats};

/// Home page component
#[component]
pub fn HomePage() -> impl IntoView {
    // State
    let (auth_state, set_auth_state) = signal(AuthState::default());
    let (loading, set_loading) = signal(true);
    let (github_stats, set_github_stats) = signal(Option::<GitHubStats>::None);
    let (level_info, set_level_info) = signal(Option::<LevelInfo>::None);
    let (user_stats, set_user_stats) = signal(Option::<UserStats>::None);
    let (badges, set_badges) = signal(Vec::<Badge>::new());
    let (badge_definitions, set_badge_definitions) = signal(Vec::<BadgeDefinition>::new());
    let (error, set_error) = signal(Option::<String>::None);

    // Load initial data
    spawn_local(async move {
        // Get auth state
        match tauri_api::get_auth_state().await {
            Ok(state) => {
                set_auth_state.set(state.clone());
                
                // If logged in, load additional data
                if state.is_logged_in {
                    load_user_data(
                        set_github_stats,
                        set_level_info,
                        set_user_stats,
                        set_badges,
                        set_error,
                    ).await;
                }
            }
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to get auth state: {}", e).into());
                set_error.set(Some(e));
            }
        }
        
        // Load badge definitions (doesn't require auth)
        if let Ok(defs) = tauri_api::get_badge_definitions().await {
            set_badge_definitions.set(defs);
        }
        
        set_loading.set(false);
    });

    // Handle login
    let on_login = move |_| {
        spawn_local(async move {
            set_loading.set(true);
            match tauri_api::start_oauth_login().await {
                Ok(auth_url) => {
                    // Open browser for OAuth
                    web_sys::console::log_1(&format!("Opening auth URL: {}", auth_url).into());
                    // In a real app, this would open the system browser
                    // For now, we'll just log it
                    if let Some(window) = web_sys::window() {
                        let _ = window.open_with_url(&auth_url);
                    }
                }
                Err(e) => {
                    set_error.set(Some(format!("Login failed: {}", e)));
                }
            }
            set_loading.set(false);
        });
    };

    // Handle logout
    let on_logout = move |_| {
        spawn_local(async move {
            set_loading.set(true);
            if let Err(e) = tauri_api::logout().await {
                set_error.set(Some(format!("Logout failed: {}", e)));
            } else {
                set_auth_state.set(AuthState::default());
                set_github_stats.set(None);
                set_level_info.set(None);
                set_user_stats.set(None);
                set_badges.set(Vec::new());
            }
            set_loading.set(false);
        });
    };

    // Handle sync
    let on_sync = move |_| {
        spawn_local(async move {
            set_loading.set(true);
            load_user_data(
                set_github_stats,
                set_level_info,
                set_user_stats,
                set_badges,
                set_error,
            ).await;
            set_loading.set(false);
        });
    };

    view! {
        <div class="flex-1 overflow-y-auto bg-gradient-to-br from-gm-bg-primary via-gm-bg-secondary to-gm-bg-primary min-h-full">
            <div class="max-w-6xl mx-auto p-6 space-y-6">
                // Header
                <div class="flex items-center justify-between">
                    <h1 class="text-3xl font-gaming font-bold text-gm-accent-cyan">
                        "Dashboard"
                    </h1>
                    
                    <Show when=move || auth_state.get().is_logged_in>
                        <button
                            class="px-4 py-2 bg-gm-bg-card border border-gm-accent-cyan/30 rounded-lg text-gm-accent-cyan hover:bg-gm-accent-cyan/10 transition-all duration-200 flex items-center gap-2"
                            on:click=on_sync
                            disabled=move || loading.get()
                        >
                            <span class=move || if loading.get() { "animate-spin" } else { "" }>"↻"</span>
                            "Sync"
                        </button>
                    </Show>
                </div>

                // Error display
                <Show when=move || error.get().is_some()>
                    <div class="p-4 bg-gm-error/20 border border-gm-error/50 rounded-lg text-gm-error">
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>

                // Loading state
                <Show when=move || loading.get()>
                    <div class="flex items-center justify-center py-20">
                        <div class="animate-spin w-12 h-12 border-4 border-gm-accent-cyan border-t-transparent rounded-full"/>
                    </div>
                </Show>

                // Content
                <Show when=move || !loading.get()>
                    <Show
                        when=move || auth_state.get().is_logged_in
                        fallback=move || view! {
                            <LoginCard on_login=on_login />
                        }
                    >
                        // Profile Card
                        <ProfileCard
                            auth_state=auth_state
                            level_info=level_info
                            user_stats=user_stats
                            on_logout=on_logout
                        />

                        // Stats Grid
                        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                            // Stats Display
                            <StatsDisplay
                                github_stats=github_stats
                                user_stats=user_stats
                            />

                            // Badges
                            <BadgeGrid
                                badges=badges
                                definitions=badge_definitions
                            />
                        </div>

                        // Contribution Graph
                        <ContributionGraph
                            github_stats=github_stats
                        />
                    </Show>
                </Show>
            </div>
        </div>
    }
}

/// Load user data from API
async fn load_user_data(
    set_github_stats: WriteSignal<Option<GitHubStats>>,
    set_level_info: WriteSignal<Option<LevelInfo>>,
    set_user_stats: WriteSignal<Option<UserStats>>,
    set_badges: WriteSignal<Vec<Badge>>,
    set_error: WriteSignal<Option<String>>,
) {
    // Load GitHub stats
    match tauri_api::get_github_stats().await {
        Ok(stats) => set_github_stats.set(Some(stats)),
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

