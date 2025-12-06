//! Dashboard Content Component
//!
//! The main dashboard content for logged-in users on the home page.
//! Displays profile card, stats, challenges, badges, and contribution graph.
//!
//! Performance Optimizations:
//! - Phase 3: Skeleton UI improvements (Issue #126)
//!   Each section loads independently with its own skeleton, showing content
//!   as data arrives instead of waiting for the entire dashboard to load.
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
//!   ├─ Issue #117: Home page gamification
//!   └─ Phase 3 Performance: https://github.com/otomatty/development-tools/issues/126

use leptos::prelude::*;

use crate::components::features::gamification::{
    BadgeGrid, ChallengeCard, ContributionGraph, ProfileCard, StatsDisplay,
};
use crate::components::home::skeleton::{
    BadgeGridSkeleton, ChallengeCardSkeleton, ContributionGraphSkeleton, ProfileCardSkeleton,
    StatsDisplaySkeleton,
};
use crate::types::{AppPage, AuthState, GitHubStats, LevelInfo, StatsDiffResult, UserStats};

/// Dashboard content component for logged-in users
///
/// Phase 3: Each section is wrapped in its own Suspense boundary, allowing
/// sections to load independently and display content progressively as data arrives.
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
        // Phase 3: Skeleton UI - Section 1: Profile Card
        // Independent loading with its own skeleton
        <Suspense fallback=move || view! { <ProfileCardSkeleton /> }>
            <ProfileCard
                auth_state=auth_state
                level_info=level_info
                user_stats=user_stats
                on_logout=move |e| on_logout.run(e)
                on_settings=move |_| set_current_page.set(AppPage::Settings)
            />
        </Suspense>

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
            // Phase 3: Skeleton UI - Section 2: Stats Display
            // Independent loading with its own skeleton
            <Suspense fallback=move || view! { <StatsDisplaySkeleton /> }>
                <StatsDisplay
                    github_stats=github_stats
                    user_stats=user_stats
                    stats_diff=stats_diff
                />
            </Suspense>

            // Phase 3: Skeleton UI - Section 3: Challenges
            // Independent loading with its own skeleton
            <Suspense fallback=move || view! { <ChallengeCardSkeleton /> }>
                <ChallengeCard />
            </Suspense>
        </div>

        // Phase 3: Skeleton UI - Section 4: Badges
        // Independent loading with its own skeleton
        <Suspense fallback=move || view! { <BadgeGridSkeleton /> }>
            <BadgeGrid />
        </Suspense>

        // Phase 3: Skeleton UI - Section 5: Contribution Graph
        // Independent loading with its own skeleton
        <Suspense fallback=move || view! { <ContributionGraphSkeleton /> }>
            <ContributionGraph
                github_stats=github_stats
            />
        </Suspense>
    }
}
