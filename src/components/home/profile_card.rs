//! Profile card component
//!
//! Displays user profile, level, and XP progress.

use leptos::ev;
use leptos::prelude::*;

use crate::types::{AuthState, LevelInfo, UserStats};

/// Profile card component
#[component]
pub fn ProfileCard<F, G>(
    auth_state: ReadSignal<AuthState>,
    level_info: ReadSignal<Option<LevelInfo>>,
    user_stats: ReadSignal<Option<UserStats>>,
    on_logout: F,
    on_settings: G,
) -> impl IntoView
where
    F: Fn(ev::MouseEvent) + 'static + Clone + Send,
    G: Fn(ev::MouseEvent) + 'static + Clone + Send,
{
    view! {
        <div class="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-cyan/20 shadow-lg">
            <div class="flex items-start justify-between">
                // User info section
                <div class="flex items-center gap-6">
                    // Avatar
                    <div class="relative">
                        {move || {
                            let state = auth_state.get();
                            if let Some(user) = state.user {
                                let avatar = user.avatar_url.unwrap_or_default();
                                view! {
                                    <img
                                        src=avatar
                                        alt="Avatar"
                                        class="w-20 h-20 rounded-xl border-2 border-gm-accent-cyan shadow-neon-cyan"
                                    />
                                }.into_any()
                            } else {
                                view! {
                                    <div class="w-20 h-20 rounded-xl bg-gm-bg-secondary border-2 border-gm-accent-cyan flex items-center justify-center">
                                        <span class="text-3xl">"👤"</span>
                                    </div>
                                }.into_any()
                            }
                        }}
                        
                        // Level badge
                        {move || {
                            level_info.get().map(|info| view! {
                                <div class="absolute -bottom-2 -right-2 px-2 py-1 bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple rounded-lg text-white font-gaming text-sm font-bold shadow-neon-cyan">
                                    "Lv." {info.current_level}
                                </div>
                            })
                        }}
                    </div>

                    // Username and XP
                    <div class="space-y-2">
                        <h2 class="text-2xl font-gaming font-bold text-white">
                            {move || {
                                auth_state.get().user.map(|u| u.username).unwrap_or_else(|| "User".to_string())
                            }}
                        </h2>

                        // XP Progress Bar
                        {move || {
                            level_info.get().map(|info| view! {
                                <div class="space-y-1">
                                    <div class="flex items-center justify-between text-sm">
                                        <span class="text-gm-accent-cyan font-gaming-mono">
                                            {info.total_xp} " XP"
                                        </span>
                                        <span class="text-dt-text-sub">
                                            {info.xp_to_next_level} " to next level"
                                        </span>
                                    </div>
                                    <div class="w-64 h-3 bg-gm-bg-secondary rounded-full overflow-hidden">
                                        <div
                                            class="h-full bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple rounded-full transition-all duration-500"
                                            style=move || format!("width: {}%", info.progress_percent)
                                        />
                                    </div>
                                </div>
                            })
                        }}
                    </div>
                </div>

                // Stats quick view
                <div class="flex items-center gap-6">
                    // Streak
                    {move || {
                        user_stats.get().map(|stats| view! {
                            <div class="text-center">
                                <div class="flex items-center gap-2 text-gm-warning">
                                    <span class="text-2xl">"🔥"</span>
                                    <span class="text-3xl font-gaming-mono font-bold">{stats.current_streak}</span>
                                </div>
                                <div class="text-xs text-dt-text-sub">"Day Streak"</div>
                            </div>
                        })
                    }}

                    // Total Commits
                    {move || {
                        user_stats.get().map(|stats| view! {
                            <div class="text-center">
                                <div class="flex items-center gap-2 text-gm-success">
                                    <span class="text-2xl">"⭐"</span>
                                    <span class="text-3xl font-gaming-mono font-bold">{stats.total_commits}</span>
                                </div>
                                <div class="text-xs text-dt-text-sub">"Commits"</div>
                            </div>
                        })
                    }}

                    // Settings button
                    <button
                        class="p-2 text-dt-text-sub hover:text-gm-accent-cyan transition-colors"
                        title="Settings"
                        on:click=on_settings
                    >
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
                        </svg>
                    </button>

                    // Logout button
                    <button
                        class="p-2 text-dt-text-sub hover:text-gm-error transition-colors"
                        title="Logout"
                        on:click=on_logout
                    >
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"/>
                        </svg>
                    </button>
                </div>
            </div>
        </div>
    }
}

