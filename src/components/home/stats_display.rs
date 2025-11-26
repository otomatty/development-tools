//! Stats display component
//!
//! Shows detailed statistics in a grid layout.

use leptos::prelude::*;

use crate::types::{GitHubStats, UserStats};

/// Streak milestone thresholds
const STREAK_MILESTONES: &[(i32, &str)] = &[
    (7, "On Fire 🔥"),
    (14, "Two Weeks 💪"),
    (30, "Month Strong 🌟"),
    (100, "Century 💯"),
    (365, "Legendary 👑"),
];

/// Get next milestone for a streak
fn get_next_milestone(current_streak: i32) -> Option<(i32, &'static str)> {
    STREAK_MILESTONES
        .iter()
        .find(|(days, _)| *days > current_streak)
        .copied()
}

/// Stats display component
#[component]
pub fn StatsDisplay(
    github_stats: ReadSignal<Option<GitHubStats>>,
    user_stats: ReadSignal<Option<UserStats>>,
) -> impl IntoView {
    view! {
        <div class="space-y-6">
            // Streak Section - Prominent display
            <StreakSection user_stats=user_stats />

            // Stats Grid
            <div class="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-purple/20">
                <h3 class="text-xl font-gaming font-bold text-gm-accent-purple mb-4">
                    "📊 Statistics"
                </h3>

                <div class="grid grid-cols-2 gap-4">
                    // Commits
                    <StatCard
                        icon="📝"
                        label="Total Commits"
                        value=Signal::derive(move || {
                            github_stats.get()
                                .map(|s| s.total_commits.to_string())
                                .unwrap_or_else(|| "-".to_string())
                        })
                        color="cyan"
                    />

                    // Pull Requests
                    <StatCard
                        icon="🔀"
                        label="Pull Requests"
                        value=Signal::derive(move || {
                            github_stats.get()
                                .map(|s| s.total_prs.to_string())
                                .unwrap_or_else(|| "-".to_string())
                        })
                        color="purple"
                    />

                    // Reviews
                    <StatCard
                        icon="👁️"
                        label="Code Reviews"
                        value=Signal::derive(move || {
                            github_stats.get()
                                .map(|s| s.total_reviews.to_string())
                                .unwrap_or_else(|| "-".to_string())
                        })
                        color="pink"
                    />

                    // Issues
                    <StatCard
                        icon="🎯"
                        label="Issues"
                        value=Signal::derive(move || {
                            github_stats.get()
                                .map(|s| s.total_issues.to_string())
                                .unwrap_or_else(|| "-".to_string())
                        })
                        color="green"
                    />

                    // Stars
                    <StatCard
                        icon="⭐"
                        label="Stars Received"
                        value=Signal::derive(move || {
                            github_stats.get()
                                .map(|s| s.total_stars_received.to_string())
                                .unwrap_or_else(|| "-".to_string())
                        })
                        color="gold"
                    />

                    // Languages
                    <StatCard
                        icon="🌍"
                        label="Languages"
                        value=Signal::derive(move || {
                            github_stats.get()
                                .map(|s| s.languages_count.to_string())
                                .unwrap_or_else(|| "-".to_string())
                        })
                        color="cyan"
                    />
                </div>
            </div>
        </div>
    }
}

/// Streak section component with prominent display
#[component]
fn StreakSection(user_stats: ReadSignal<Option<UserStats>>) -> impl IntoView {
    let current_streak = move || user_stats.get().map(|s| s.current_streak).unwrap_or(0);
    let longest_streak = move || user_stats.get().map(|s| s.longest_streak).unwrap_or(0);
    
    // Flame intensity based on streak
    let flame_class = move || {
        let streak = current_streak();
        if streak >= 30 {
            "animate-pulse text-orange-500" // Strong flame
        } else if streak >= 7 {
            "animate-bounce text-orange-400" // Moderate flame
        } else if streak > 0 {
            "text-orange-300" // Small flame
        } else {
            "text-slate-500 opacity-50" // No streak
        }
    };

    let next_milestone_info = move || {
        let streak = current_streak();
        get_next_milestone(streak).map(|(days, name)| {
            let days_left = days - streak;
            (days_left, name)
        })
    };

    view! {
        <div class="p-6 bg-gradient-to-br from-orange-900/30 via-gm-bg-card/80 to-red-900/20 backdrop-blur-sm rounded-2xl border border-orange-500/30 shadow-lg shadow-orange-500/10">
            <div class="flex items-center justify-between">
                // Left side: Current streak with flame
                <div class="flex items-center gap-4">
                    <div class=move || format!("text-5xl {}", flame_class())>
                        "🔥"
                    </div>
                    <div>
                        <div class="text-4xl font-gaming-mono font-bold text-orange-400">
                            {move || current_streak()}<span class="text-lg text-orange-300/70">" days"</span>
                        </div>
                        <div class="text-sm text-dt-text-sub">"Current Streak"</div>
                    </div>
                </div>

                // Right side: Best streak and next milestone
                <div class="text-right space-y-2">
                    <div class="flex items-center gap-2 justify-end">
                        <span class="text-badge-gold">"🏆"</span>
                        <span class="text-lg font-gaming-mono text-badge-gold">{move || longest_streak()}</span>
                        <span class="text-xs text-dt-text-sub">"Best"</span>
                    </div>
                    
                    // Next milestone
                    <Show when=move || next_milestone_info().is_some()>
                        {move || {
                            if let Some((days_left, name)) = next_milestone_info() {
                                view! {
                                    <div class="text-xs text-dt-text-sub">
                                        <span class="text-gm-accent-cyan">{days_left}</span>
                                        " days to "
                                        <span class="text-orange-300">{name}</span>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <div /> }.into_any()
                            }
                        }}
                    </Show>
                </div>
            </div>

            // Progress to next milestone
            <Show when=move || next_milestone_info().is_some()>
                {move || {
                    if let Some((days_left, _)) = next_milestone_info() {
                        let streak = current_streak();
                        let target = streak + days_left;
                        let progress = if target > 0 {
                            (streak as f32 / target as f32) * 100.0
                        } else {
                            0.0
                        };
                        
                        view! {
                            <div class="mt-4">
                                <div class="h-2 bg-slate-700/50 rounded-full overflow-hidden">
                                    <div 
                                        class="h-full bg-gradient-to-r from-orange-500 to-red-500 rounded-full transition-all duration-500"
                                        style=format!("width: {}%", progress)
                                    />
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! { <div /> }.into_any()
                    }
                }}
            </Show>
        </div>
    }
}

/// Individual stat card
#[component]
fn StatCard(
    icon: &'static str,
    label: &'static str,
    #[prop(into)] value: Signal<String>,
    color: &'static str,
) -> impl IntoView {
    let color_class = match color {
        "cyan" => "text-gm-accent-cyan",
        "purple" => "text-gm-accent-purple",
        "pink" => "text-gm-accent-pink",
        "green" => "text-gm-success",
        "orange" => "text-gm-warning",
        "gold" => "text-badge-gold",
        _ => "text-dt-text",
    };

    view! {
        <div class="p-4 bg-gm-bg-secondary/50 rounded-xl border border-slate-700/30 hover:border-gm-accent-cyan/30 transition-all duration-200">
            <div class="flex items-center gap-3">
                <span class="text-2xl">{icon}</span>
                <div>
                    <div class=format!("text-xl font-gaming-mono font-bold {}", color_class)>
                        {move || value.get()}
                    </div>
                    <div class="text-xs text-dt-text-sub">{label}</div>
                </div>
            </div>
        </div>
    }
}

