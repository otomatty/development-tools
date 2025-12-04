//! Dashboard Content Component
//!
//! The main dashboard content for logged-in users on the home page.
//! Displays profile card, stats, challenges, badges, and contribution graph.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this component):
//!   └─ src/components/pages/home_page.rs
//! Dependencies (Feature components used):
//!   ├─ features/gamification/profile_card.rs - ProfileCard
//!   ├─ features/gamification/stats_display.rs - StatsDisplay
//!   ├─ features/gamification/challenge_card.rs - ChallengeCard
//!   ├─ features/gamification/badge_grid.rs - BadgeGrid
//!   └─ features/gamification/contribution_graph.rs - ContributionGraph
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

use leptos::prelude::*;

use crate::components::features::gamification::{
    BadgeGrid, ChallengeCard, ContributionGraph, ProfileCard, StatsDisplay,
};
use crate::types::{AppPage, AuthState, GitHubStats, LevelInfo, StatsDiffResult, UserStats};

/// Dashboard content component for logged-in users
#[component]
pub fn DashboardContent(
    auth_state: ReadSignal<AuthState>,
    level_info: ReadSignal<Option<LevelInfo>>,
    user_stats: ReadSignal<Option<UserStats>>,
    github_stats: ReadSignal<Option<GitHubStats>>,
    stats_diff: ReadSignal<Option<StatsDiffResult>>,
    on_logout: Callback<leptos::ev::MouseEvent>,
    set_current_page: WriteSignal<AppPage>,
) -> impl IntoView {
    view! {
        // Profile Card
        <ProfileCard
            auth_state=auth_state
            level_info=level_info
            user_stats=user_stats
            on_logout=move |e| on_logout.run(e)
            on_settings=move |_| set_current_page.set(AppPage::Settings)
        />

        // Quick Actions
        <div class="flex justify-end">
            <button
                class="flex items-center gap-2 px-4 py-2 text-sm text-dt-text-sub hover:text-gm-accent-cyan transition-colors"
                on:click=move |_| set_current_page.set(AppPage::XpHistory)
            >
                <span>"📜"</span>
                <span>"XP獲得履歴を見る"</span>
                <span class="text-xs">"→"</span>
            </button>
        </div>

        // Stats and Challenges Grid
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
            // Stats Display
            <StatsDisplay
                github_stats=github_stats
                user_stats=user_stats
                stats_diff=stats_diff
            />

            // Challenges
            <ChallengeCard />
        </div>

        // Badges Section
        <BadgeGrid />

        // Contribution Graph
        <ContributionGraph
            github_stats=github_stats
        />
    }
}
