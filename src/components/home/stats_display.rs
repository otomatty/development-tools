//! Stats display component
//!
//! Shows detailed statistics in a grid layout.

use leptos::prelude::*;

use crate::types::{GitHubStats, UserStats};

/// Stats display component
#[component]
pub fn StatsDisplay(
    github_stats: ReadSignal<Option<GitHubStats>>,
    user_stats: ReadSignal<Option<UserStats>>,
) -> impl IntoView {
    view! {
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

                // Contributions
                <StatCard
                    icon="📈"
                    label="Total Contributions"
                    value=Signal::derive(move || {
                        github_stats.get()
                            .map(|s| s.total_contributions.to_string())
                            .unwrap_or_else(|| "-".to_string())
                    })
                    color="cyan"
                />

                // Current Streak
                <StatCard
                    icon="🔥"
                    label="Current Streak"
                    value=Signal::derive(move || {
                        user_stats.get()
                            .map(|s| format!("{} days", s.current_streak))
                            .unwrap_or_else(|| "-".to_string())
                    })
                    color="orange"
                />

                // Longest Streak
                <StatCard
                    icon="🏆"
                    label="Longest Streak"
                    value=Signal::derive(move || {
                        user_stats.get()
                            .map(|s| format!("{} days", s.longest_streak))
                            .unwrap_or_else(|| "-".to_string())
                    })
                    color="gold"
                />
            </div>
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

