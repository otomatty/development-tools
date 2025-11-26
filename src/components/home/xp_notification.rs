//! XP notification component
//!
//! Displays XP gain notifications with animation.

use leptos::prelude::*;

use crate::types::XpGainedEvent;

/// XP notification component
#[component]
pub fn XpNotification<F>(
    event: ReadSignal<Option<XpGainedEvent>>,
    on_close: F,
) -> impl IntoView 
where
    F: Fn() + 'static + Clone + Send + Sync,
{
    view! {
        <Show when=move || event.get().is_some()>
            {
                let on_close = on_close.clone();
                move || {
                    let e = event.get().unwrap();
                    let on_close_inner = on_close.clone();
                    
                    view! {
                        <div class="fixed top-4 right-4 z-50 animate-slide-in">
                            <div class="p-4 bg-gm-bg-card/95 backdrop-blur-sm rounded-xl border border-gm-accent-cyan/30 shadow-neon-cyan min-w-80">
                                // Header
                                <div class="flex items-center justify-between mb-3">
                                    <div class="flex items-center gap-2">
                                        <span class="text-2xl animate-bounce">"✨"</span>
                                        <span class="text-gm-accent-cyan font-gaming font-bold">
                                            "XP Gained!"
                                        </span>
                                    </div>
                                    <button
                                        class="text-dt-text-sub hover:text-white transition-colors"
                                        on:click=move |_| on_close_inner()
                                    >
                                        "✕"
                                    </button>
                                </div>
                                
                                // XP amount
                                <div class="text-center mb-3">
                                    <span class="text-4xl font-gaming-mono font-bold text-gm-success animate-pulse">
                                        "+" {e.xp_gained} " XP"
                                    </span>
                                </div>
                                
                                // Breakdown (if non-zero)
                                {
                                    let breakdown = e.xp_breakdown.clone();
                                    let show_breakdown = breakdown.total_xp > 0;
                                    view! {
                                        <Show when=move || show_breakdown>
                                            <div class="space-y-1 text-sm text-dt-text-sub border-t border-slate-700/50 pt-3">
                                                {
                                                    let bd = breakdown.clone();
                                                    let show_commits = bd.commits_xp > 0;
                                                    view! {
                                                        <Show when=move || show_commits>
                                                            <div class="flex justify-between">
                                                                <span>"📝 Commits"</span>
                                                                <span class="text-gm-accent-cyan">"+" {bd.commits_xp}</span>
                                                            </div>
                                                        </Show>
                                                    }
                                                }
                                                {
                                                    let bd = breakdown.clone();
                                                    let show_prs_created = bd.prs_created_xp > 0;
                                                    view! {
                                                        <Show when=move || show_prs_created>
                                                            <div class="flex justify-between">
                                                                <span>"🔀 PRs Created"</span>
                                                                <span class="text-gm-accent-cyan">"+" {bd.prs_created_xp}</span>
                                                            </div>
                                                        </Show>
                                                    }
                                                }
                                                {
                                                    let bd = breakdown.clone();
                                                    let show_prs_merged = bd.prs_merged_xp > 0;
                                                    view! {
                                                        <Show when=move || show_prs_merged>
                                                            <div class="flex justify-between">
                                                                <span>"✅ PRs Merged"</span>
                                                                <span class="text-gm-accent-cyan">"+" {bd.prs_merged_xp}</span>
                                                            </div>
                                                        </Show>
                                                    }
                                                }
                                                {
                                                    let bd = breakdown.clone();
                                                    let show_issues_created = bd.issues_created_xp > 0;
                                                    view! {
                                                        <Show when=move || show_issues_created>
                                                            <div class="flex justify-between">
                                                                <span>"📋 Issues Created"</span>
                                                                <span class="text-gm-accent-cyan">"+" {bd.issues_created_xp}</span>
                                                            </div>
                                                        </Show>
                                                    }
                                                }
                                                {
                                                    let bd = breakdown.clone();
                                                    let show_issues_closed = bd.issues_closed_xp > 0;
                                                    view! {
                                                        <Show when=move || show_issues_closed>
                                                            <div class="flex justify-between">
                                                                <span>"🎯 Issues Closed"</span>
                                                                <span class="text-gm-accent-cyan">"+" {bd.issues_closed_xp}</span>
                                                            </div>
                                                        </Show>
                                                    }
                                                }
                                                {
                                                    let bd = breakdown.clone();
                                                    let show_reviews = bd.reviews_xp > 0;
                                                    view! {
                                                        <Show when=move || show_reviews>
                                                            <div class="flex justify-between">
                                                                <span>"👁️ Reviews"</span>
                                                                <span class="text-gm-accent-cyan">"+" {bd.reviews_xp}</span>
                                                            </div>
                                                        </Show>
                                                    }
                                                }
                                                {
                                                    let bd = breakdown.clone();
                                                    let show_stars = bd.stars_xp > 0;
                                                    view! {
                                                        <Show when=move || show_stars>
                                                            <div class="flex justify-between">
                                                                <span>"⭐ Stars"</span>
                                                                <span class="text-gm-accent-cyan">"+" {bd.stars_xp}</span>
                                                            </div>
                                                        </Show>
                                                    }
                                                }
                                                {
                                                    let bd = breakdown.clone();
                                                    let show_streak = bd.streak_bonus_xp > 0;
                                                    view! {
                                                        <Show when=move || show_streak>
                                                            <div class="flex justify-between">
                                                                <span>"🔥 Streak Bonus"</span>
                                                                <span class="text-gm-warning">"+" {bd.streak_bonus_xp}</span>
                                                            </div>
                                                        </Show>
                                                    }
                                                }
                                            </div>
                                        </Show>
                                    }
                                }
                            </div>
                        </div>
                    }
                }
            }
        </Show>
    }
}

/// Level up modal component
#[component]
pub fn LevelUpModal<F>(
    event: ReadSignal<Option<XpGainedEvent>>,
    on_close: F,
) -> impl IntoView 
where
    F: Fn() + 'static + Clone + Send + Sync,
{
    view! {
        <Show when=move || event.get().map(|e| e.level_up).unwrap_or(false)>
            {
                let on_close = on_close.clone();
                move || {
                    let e = event.get().unwrap();
                    let on_close_overlay = on_close.clone();
                    let on_close_button = on_close.clone();
                    
                    view! {
                        // Overlay
                        <div 
                            class="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center animate-fade-in"
                            on:click=move |_| on_close_overlay()
                        >
                            // Modal content
                            <div 
                                class="relative p-8 bg-gm-bg-card rounded-2xl border-2 border-gm-accent-purple shadow-neon-purple max-w-md w-full mx-4 animate-scale-in"
                                on:click=|ev| ev.stop_propagation()
                            >
                                // Particles effect (CSS only)
                                <div class="absolute inset-0 overflow-hidden rounded-2xl pointer-events-none">
                                    <div class="particle particle-1"/>
                                    <div class="particle particle-2"/>
                                    <div class="particle particle-3"/>
                                    <div class="particle particle-4"/>
                                    <div class="particle particle-5"/>
                                </div>
                                
                                // Content
                                <div class="relative text-center space-y-6">
                                    // Trophy icon with glow
                                    <div class="text-8xl animate-bounce-slow">
                                        "🏆"
                                    </div>
                                    
                                    // Title
                                    <h2 class="text-3xl font-gaming font-bold bg-gradient-to-r from-gm-accent-cyan via-gm-accent-purple to-gm-accent-pink bg-clip-text text-transparent animate-pulse">
                                        "LEVEL UP!"
                                    </h2>
                                    
                                    // Level display
                                    <div class="flex items-center justify-center gap-4">
                                        <span class="text-4xl font-gaming-mono text-dt-text-sub">
                                            "Lv." {e.old_level}
                                        </span>
                                        <span class="text-2xl text-gm-accent-cyan animate-pulse">"→"</span>
                                        <span class="text-5xl font-gaming-mono font-bold text-gm-accent-cyan">
                                            "Lv." {e.new_level}
                                        </span>
                                    </div>
                                    
                                    // Total XP
                                    <div class="text-lg text-dt-text-sub">
                                        "Total XP: "
                                        <span class="text-gm-accent-cyan font-gaming-mono font-bold">
                                            {e.total_xp}
                                        </span>
                                    </div>
                                    
                                    // Close button
                                    <button
                                        class="px-8 py-3 bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple rounded-lg text-white font-gaming font-bold hover:shadow-neon-cyan transition-all duration-200"
                                        on:click=move |_| on_close_button()
                                    >
                                        "Awesome!"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }
                }
            }
        </Show>
    }
}

