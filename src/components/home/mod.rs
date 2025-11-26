//! Home page components
//!
//! This module contains all components for the gamification home page.

pub mod login_card;
pub mod profile_card;
pub mod stats_display;
pub mod contribution_graph;
pub mod badge_grid;
pub mod xp_notification;

pub use login_card::LoginCard;
pub use profile_card::ProfileCard;
pub use stats_display::StatsDisplay;
pub use contribution_graph::ContributionGraph;
pub use badge_grid::BadgeGrid;
pub use xp_notification::{LevelUpModal, XpNotification};

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

use crate::tauri_api;
use crate::types::{AuthState, Badge, BadgeDefinition, GitHubStats, LevelInfo, UserStats, XpGainedEvent};

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
    
    // XP notification state
    let (xp_event, set_xp_event) = signal(Option::<XpGainedEvent>::None);
    let (level_up_event, set_level_up_event) = signal(Option::<XpGainedEvent>::None);
    
    // Auto-sync state
    let (auto_sync_enabled, set_auto_sync_enabled) = signal(true);
    let (last_sync_time, set_last_sync_time) = signal(Option::<String>::None);

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

    // Auto-sync interval (15 minutes = 900,000 ms)
    const AUTO_SYNC_INTERVAL_MS: i32 = 15 * 60 * 1000;

    // Setup auto-sync timer
    {
        let auth_state = auth_state.clone();
        let auto_sync_enabled = auto_sync_enabled.clone();
        
        spawn_local(async move {
            // Wait for initial load
            loop {
                // Check every 15 minutes
                if let Some(window) = web_sys::window() {
                    let auth = auth_state.get();
                    let enabled = auto_sync_enabled.get();
                    
                    if auth.is_logged_in && enabled {
                        web_sys::console::log_1(&"Auto-sync: Syncing GitHub stats...".into());
                        
                        match tauri_api::sync_github_stats().await {
                            Ok(sync_result) => {
                                set_user_stats.set(Some(sync_result.user_stats.clone()));
                                
                                // Update last sync time
                                let now = js_sys::Date::new_0();
                                let time_str = format!(
                                    "{:02}:{:02}",
                                    now.get_hours(),
                                    now.get_minutes()
                                );
                                set_last_sync_time.set(Some(time_str));
                                
                                // Show notification if XP gained
                                if sync_result.xp_gained > 0 {
                                    let event = XpGainedEvent {
                                        xp_gained: sync_result.xp_gained,
                                        total_xp: sync_result.user_stats.total_xp as u32,
                                        old_level: sync_result.old_level,
                                        new_level: sync_result.new_level,
                                        level_up: sync_result.level_up,
                                        xp_breakdown: sync_result.xp_breakdown,
                                        streak_bonus: sync_result.streak_bonus,
                                    };
                                    
                                    if sync_result.level_up {
                                        set_level_up_event.set(Some(event.clone()));
                                    } else {
                                        set_xp_event.set(Some(event));
                                        
                                        // Auto-hide after 5 seconds
                                        let closure = wasm_bindgen::closure::Closure::once(move || {
                                            set_xp_event.set(None);
                                        });
                                        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                                            closure.as_ref().unchecked_ref(),
                                            5000,
                                        );
                                        closure.forget();
                                    }
                                }
                                
                                // Reload level info
                                if let Ok(info) = tauri_api::get_level_info().await {
                                    set_level_info.set(info);
                                }
                                
                                web_sys::console::log_1(&format!("Auto-sync: Completed, XP gained: {}", sync_result.xp_gained).into());
                            }
                            Err(e) => {
                                web_sys::console::error_1(&format!("Auto-sync failed: {}", e).into());
                            }
                        }
                    }
                    
                    // Wait for next interval using a promise-based sleep
                    let promise = js_sys::Promise::new(&mut |resolve, _| {
                        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                            &resolve,
                            AUTO_SYNC_INTERVAL_MS,
                        );
                    });
                    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                } else {
                    break;
                }
            }
        });
    }

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
            
            // Use sync_github_stats which returns XP info
            match tauri_api::sync_github_stats().await {
                Ok(sync_result) => {
                    // Update user stats from sync result
                    set_user_stats.set(Some(sync_result.user_stats.clone()));
                    
                    // Show XP notification if XP was gained
                    if sync_result.xp_gained > 0 {
                        let event = XpGainedEvent {
                            xp_gained: sync_result.xp_gained,
                            total_xp: sync_result.user_stats.total_xp as u32,
                            old_level: sync_result.old_level,
                            new_level: sync_result.new_level,
                            level_up: sync_result.level_up,
                            xp_breakdown: sync_result.xp_breakdown,
                            streak_bonus: sync_result.streak_bonus,
                        };
                        
                        if sync_result.level_up {
                            set_level_up_event.set(Some(event.clone()));
                        } else {
                            set_xp_event.set(Some(event));
                            
                            // Auto-hide XP notification after 5 seconds using web_sys
                            if let Some(window) = web_sys::window() {
                                let closure = wasm_bindgen::closure::Closure::once(move || {
                                    set_xp_event.set(None);
                                });
                                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                                    closure.as_ref().unchecked_ref(),
                                    5000,
                                );
                                closure.forget(); // Prevent closure from being dropped
                            }
                        }
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to sync: {}", e).into());
                    set_error.set(Some(format!("Sync failed: {}", e)));
                }
            }
            
            // Load other data
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
    
    // Callbacks for closing notifications
    let on_close_xp = move || set_xp_event.set(None);
    let on_close_level_up = move || set_level_up_event.set(None);

    view! {
        <div class="flex-1 overflow-y-auto bg-gradient-to-br from-gm-bg-primary via-gm-bg-secondary to-gm-bg-primary min-h-full">
            // XP Notification
            <XpNotification event=xp_event on_close=on_close_xp />
            
            // Level Up Modal
            <LevelUpModal event=level_up_event on_close=on_close_level_up />
            
            <div class="max-w-6xl mx-auto p-6 space-y-6">
                // Header
                <div class="flex items-center justify-between">
                    <Show when=move || auth_state.get().is_logged_in>
                        <div class="flex items-center gap-4">
                            // Auto-sync toggle
                            <div class="flex items-center gap-2 text-sm">
                                <span class="text-dt-text-sub">"Auto-sync"</span>
                                <button
                                    class=move || format!(
                                        "relative w-12 h-6 rounded-full transition-colors duration-200 {}",
                                        if auto_sync_enabled.get() { "bg-gm-accent-cyan" } else { "bg-slate-600" }
                                    )
                                    on:click=move |_| set_auto_sync_enabled.set(!auto_sync_enabled.get())
                                >
                                    <span
                                        class=move || format!(
                                            "absolute top-1 w-4 h-4 rounded-full bg-white transition-transform duration-200 {}",
                                            if auto_sync_enabled.get() { "translate-x-7" } else { "translate-x-1" }
                                        )
                                    />
                                </button>
                            </div>
                            
                            // Last sync time
                            <Show when=move || last_sync_time.get().is_some()>
                                <span class="text-xs text-dt-text-sub">
                                    "Last: " {move || last_sync_time.get().unwrap_or_default()}
                                </span>
                            </Show>
                            
                            // Manual sync button
                            <button
                                class="px-4 py-2 bg-gm-bg-card border border-gm-accent-cyan/30 rounded-lg text-gm-accent-cyan hover:bg-gm-accent-cyan/10 transition-all duration-200 flex items-center gap-2"
                                on:click=on_sync
                                disabled=move || loading.get()
                            >
                                <span class=move || if loading.get() { "animate-spin" } else { "" }>"↻"</span>
                                "Sync"
                            </button>
                        </div>
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

