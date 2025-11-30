//! Challenge card component
//!
//! Displays active challenges with progress bars and completion status.
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this component):
//!   └─ src/components/home/mod.rs
//!
//! Dependencies (Files this module imports):
//!   └─ src/components/network_status.rs (use_is_online)
//!
//! Related Documentation:
//!   ├─ Spec: (TODO: create challenge_card.spec.md)
//!   └─ Issue: GitHub Issue #10 (Phase 5: Offline Support)

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::network_status::use_is_online;
use crate::tauri_api;
use crate::types::ChallengeInfo;

/// Challenge card component - displays a list of active challenges
#[component]
pub fn ChallengeCard() -> impl IntoView {
    // Network status
    let is_online = use_is_online();
    
    // Signals for challenges
    let (challenges, set_challenges) = signal::<Vec<ChallengeInfo>>(Vec::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<String>>(None);

    // Load challenges on mount
    Effect::new(move |_| {
        spawn_local(async move {
            set_loading.set(true);
            match tauri_api::get_active_challenges().await {
                Ok(c) => {
                    set_challenges.set(c);
                    set_error.set(None);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to load challenges: {}", e).into());
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        });
    });

    // Reload challenges function - only works when online
    let reload_challenges = move || {
        if !is_online.get_untracked() {
            return;
        }
        spawn_local(async move {
            match tauri_api::get_active_challenges().await {
                Ok(c) => {
                    set_challenges.set(c);
                    set_error.set(None);
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
        });
    };

    view! {
        <div class="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-gold/20">
            <div class="flex items-center justify-between mb-4">
                <h3 class="text-xl font-gaming font-bold text-gm-accent-gold">
                    "🎯 Challenges"
                </h3>
                <div class="relative group">
                    <button
                        class=move || {
                            let online = is_online.get();
                            format!(
                                "p-2 rounded-lg transition-all duration-200 {}",
                                if online {
                                    "bg-gm-bg-secondary/50 hover:bg-gm-bg-secondary text-gm-text-secondary hover:text-gm-text-primary"
                                } else {
                                    "bg-gm-bg-secondary/30 text-gm-text-muted cursor-not-allowed"
                                }
                            )
                        }
                        on:click=move |_| reload_challenges()
                        disabled=move || !is_online.get()
                        title=move || {
                            if is_online.get() {
                                "チャレンジを更新"
                            } else {
                                "オフラインのため更新できません"
                            }
                        }
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                        </svg>
                    </button>
                    
                    // Offline tooltip
                    <Show when=move || !is_online.get()>
                        <div class="absolute -bottom-10 right-0 px-3 py-1.5 bg-gm-bg-dark/95 text-gm-warning text-xs rounded-lg border border-gm-warning/30 whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity duration-200 z-10">
                            "⚠️ オフライン"
                        </div>
                    </Show>
                </div>
            </div>

            // Loading state
            <Show when=move || loading.get()>
                <div class="space-y-3">
                    <ChallengeSkeleton />
                    <ChallengeSkeleton />
                </div>
            </Show>

            // Error state
            <Show when=move || error.get().is_some()>
                <div class="p-3 bg-gm-error/20 border border-gm-error/50 rounded-lg text-gm-error text-sm">
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            // Challenges list
            <Show when=move || !loading.get() && error.get().is_none()>
                <Show
                    when=move || !challenges.get().is_empty()
                    fallback=|| view! {
                        <div class="text-center py-8 text-gm-text-secondary">
                            <div class="text-4xl mb-2">"🎮"</div>
                            <p class="text-sm">"アクティブなチャレンジはありません"</p>
                            <p class="text-xs mt-1 text-gm-text-muted">
                                "GitHub同期時に自動生成されます"
                            </p>
                        </div>
                    }
                >
                    <div class="space-y-3">
                        <For
                            each=move || challenges.get()
                            key=|c| c.id
                            children=move |challenge| {
                                view! {
                                    <ChallengeItem challenge=challenge />
                                }
                            }
                        />
                    </div>
                </Show>
            </Show>
        </div>
    }
}

/// Single challenge item component
#[component]
fn ChallengeItem(challenge: ChallengeInfo) -> impl IntoView {
    let progress = challenge.progress_percent.min(100.0);
    let is_completed = challenge.is_completed;
    let is_expired = challenge.is_expired;

    // Determine colors based on status
    let (bg_color, border_color, progress_color) = if is_completed {
        ("bg-gm-success/10", "border-gm-success/30", "bg-gradient-to-r from-gm-success to-gm-accent-cyan")
    } else if is_expired {
        ("bg-gm-error/10", "border-gm-error/30", "bg-gm-error/50")
    } else if progress >= 75.0 {
        ("bg-gm-accent-gold/10", "border-gm-accent-gold/30", "bg-gradient-to-r from-gm-accent-gold to-gm-accent-pink")
    } else {
        ("bg-gm-bg-secondary/50", "border-gm-accent-purple/20", "bg-gradient-to-r from-gm-accent-purple to-gm-accent-cyan")
    };

    // Challenge type badge color
    let type_badge_class = if challenge.challenge_type == "daily" {
        "bg-gm-accent-cyan/20 text-gm-accent-cyan border-gm-accent-cyan/30"
    } else {
        "bg-gm-accent-purple/20 text-gm-accent-purple border-gm-accent-purple/30"
    };

    view! {
        <div class=format!(
            "p-4 rounded-xl border {} {} transition-all duration-300 hover:scale-[1.02]",
            bg_color, border_color
        )>
            // Header row
            <div class="flex items-center justify-between mb-3">
                <div class="flex items-center gap-2">
                    // Challenge type badge
                    <span class=format!(
                        "px-2 py-0.5 text-xs font-medium rounded-full border {}",
                        type_badge_class
                    )>
                        {challenge.challenge_type_label()}
                    </span>
                    // Metric icon and name
                    <span class="text-gm-text-secondary text-sm">
                        {challenge.target_metric_icon()}
                        " "
                        {challenge.target_metric_label()}
                    </span>
                </div>
                // Status/Time remaining
                <span class="text-xs text-gm-text-muted">
                    {if is_completed {
                        "✅ 達成!".to_string()
                    } else if is_expired {
                        "⏰ 期限切れ".to_string()
                    } else {
                        challenge.remaining_time_label()
                    }}
                </span>
            </div>

            // Progress section
            <div class="space-y-2">
                // Progress text
                <div class="flex items-baseline justify-between">
                    <span class="text-2xl font-bold text-gm-text-primary">
                        {challenge.current_value}
                        <span class="text-sm text-gm-text-secondary font-normal">
                            " / "{challenge.target_value}
                        </span>
                    </span>
                    <span class="text-sm font-medium text-gm-accent-gold">
                        "+"{challenge.reward_xp}" XP"
                    </span>
                </div>

                // Progress bar
                <div class="relative h-3 bg-gm-bg-tertiary rounded-full overflow-hidden">
                    <div
                        class=format!(
                            "absolute inset-y-0 left-0 {} rounded-full transition-all duration-500",
                            progress_color
                        )
                        style=format!("width: {}%", progress)
                    >
                        // Animated shine effect for active challenges
                        {if !is_completed && !is_expired {
                            Some(view! {
                                <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent animate-shimmer" />
                            })
                        } else {
                            None
                        }}
                    </div>
                </div>

                // Progress percentage
                <div class="text-right">
                    <span class="text-xs text-gm-text-muted">
                        {format!("{:.0}%", progress)}
                    </span>
                </div>
            </div>
        </div>
    }
}

/// Skeleton loader for challenges
#[component]
fn ChallengeSkeleton() -> impl IntoView {
    view! {
        <div class="p-4 rounded-xl border border-gm-accent-purple/10 bg-gm-bg-secondary/30 animate-pulse">
            <div class="flex items-center justify-between mb-3">
                <div class="flex items-center gap-2">
                    <div class="h-5 w-16 bg-gm-bg-tertiary rounded-full" />
                    <div class="h-4 w-20 bg-gm-bg-tertiary rounded" />
                </div>
                <div class="h-4 w-16 bg-gm-bg-tertiary rounded" />
            </div>
            <div class="space-y-2">
                <div class="flex justify-between">
                    <div class="h-8 w-24 bg-gm-bg-tertiary rounded" />
                    <div class="h-5 w-16 bg-gm-bg-tertiary rounded" />
                </div>
                <div class="h-3 bg-gm-bg-tertiary rounded-full" />
            </div>
        </div>
    }
}
