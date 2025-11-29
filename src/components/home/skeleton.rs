//! Home page skeleton components
//!
//! Skeleton loading states for the dashboard/home page.

use leptos::prelude::*;
use crate::components::skeleton::{
    Skeleton, SkeletonAvatar, SkeletonCard, SkeletonStat, SkeletonBadge, SkeletonGraph
};

/// Full home page skeleton - shows while initial data is loading
#[component]
pub fn HomeSkeleton() -> impl IntoView {
    view! {
        <div class="space-y-6">
            // Profile Card Skeleton
            <ProfileCardSkeleton />
            
            // Stats Grid
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                // Stats Display Skeleton
                <StatsDisplaySkeleton />
                
                // Badge Grid Skeleton
                <BadgeGridSkeleton />
            </div>
            
            // Contribution Graph Skeleton
            <ContributionGraphSkeleton />
        </div>
    }
}

/// Profile card skeleton
#[component]
pub fn ProfileCardSkeleton() -> impl IntoView {
    view! {
        <SkeletonCard class="border-gm-accent-cyan/20">
            <div class="flex items-start justify-between">
                // User info section
                <div class="flex items-center gap-6">
                    // Avatar with level badge placeholder
                    <div class="relative">
                        <SkeletonAvatar size="5rem" />
                        // Level badge placeholder
                        <div class="absolute -bottom-2 -right-2">
                            <Skeleton
                                width="3rem"
                                height="1.5rem"
                                rounded="rounded-lg"
                            />
                        </div>
                    </div>
                    
                    // Username and XP
                    <div class="space-y-3">
                        // Username
                        <Skeleton
                            width="8rem"
                            height="1.75rem"
                            rounded="rounded"
                        />
                        
                        // XP Progress
                        <div class="space-y-2">
                            <div class="flex items-center justify-between">
                                <Skeleton
                                    width="4rem"
                                    height="0.875rem"
                                    rounded="rounded"
                                />
                                <Skeleton
                                    width="6rem"
                                    height="0.875rem"
                                    rounded="rounded"
                                />
                            </div>
                            // Progress bar
                            <Skeleton
                                width="16rem"
                                height="0.75rem"
                                rounded="rounded-full"
                            />
                        </div>
                    </div>
                </div>
                
                // Stats quick view
                <div class="flex items-center gap-6">
                    // Streak placeholder
                    <div class="text-center space-y-1">
                        <div class="flex items-center gap-2">
                            <Skeleton
                                width="2rem"
                                height="2rem"
                                rounded="rounded"
                            />
                            <Skeleton
                                width="2rem"
                                height="2rem"
                                rounded="rounded"
                            />
                        </div>
                        <Skeleton
                            width="4rem"
                            height="0.75rem"
                            rounded="rounded"
                        />
                    </div>
                    
                    // Commits placeholder
                    <div class="text-center space-y-1">
                        <div class="flex items-center gap-2">
                            <Skeleton
                                width="2rem"
                                height="2rem"
                                rounded="rounded"
                            />
                            <Skeleton
                                width="2rem"
                                height="2rem"
                                rounded="rounded"
                            />
                        </div>
                        <Skeleton
                            width="4rem"
                            height="0.75rem"
                            rounded="rounded"
                        />
                    </div>
                    
                    // Action buttons placeholder
                    <div class="flex gap-2">
                        <Skeleton
                            width="2rem"
                            height="2rem"
                            rounded="rounded"
                        />
                        <Skeleton
                            width="2rem"
                            height="2rem"
                            rounded="rounded"
                        />
                    </div>
                </div>
            </div>
        </SkeletonCard>
    }
}

/// Stats display skeleton
#[component]
pub fn StatsDisplaySkeleton() -> impl IntoView {
    view! {
        <div class="space-y-6">
            // Streak Section Skeleton
            <StreakSectionSkeleton />
            
            // Statistics Card
            <SkeletonCard class="border-gm-accent-purple/20">
                // Title
                <div class="flex items-center gap-2 mb-4">
                    <Skeleton
                        width="1.5rem"
                        height="1.5rem"
                        rounded="rounded"
                    />
                    <Skeleton
                        width="6rem"
                        height="1.25rem"
                        rounded="rounded"
                    />
                </div>
                
                // Stats Grid
                <div class="grid grid-cols-2 gap-4">
                    <SkeletonStat />
                    <SkeletonStat />
                    <SkeletonStat />
                    <SkeletonStat />
                    <SkeletonStat />
                    <SkeletonStat />
                </div>
            </SkeletonCard>
        </div>
    }
}

/// Streak section skeleton
#[component]
fn StreakSectionSkeleton() -> impl IntoView {
    view! {
        <div class="p-6 bg-gradient-to-br from-orange-900/30 via-gm-bg-card/80 to-red-900/20 backdrop-blur-sm rounded-2xl border border-orange-500/30">
            <div class="flex items-center justify-between">
                // Left side: streak display
                <div class="flex items-center gap-4">
                    // Flame icon
                    <Skeleton
                        width="3rem"
                        height="3rem"
                        rounded="rounded"
                    />
                    <div class="space-y-1">
                        // Days count
                        <Skeleton
                            width="6rem"
                            height="2.5rem"
                            rounded="rounded"
                        />
                        // Label
                        <Skeleton
                            width="5rem"
                            height="0.875rem"
                            rounded="rounded"
                        />
                    </div>
                </div>
                
                // Right side: best streak and milestone
                <div class="text-right space-y-2">
                    <Skeleton
                        width="5rem"
                        height="1.25rem"
                        rounded="rounded"
                    />
                    <Skeleton
                        width="7rem"
                        height="0.75rem"
                        rounded="rounded"
                    />
                </div>
            </div>
            
            // Progress bar
            <div class="mt-4">
                <Skeleton
                    width="100%"
                    height="0.5rem"
                    rounded="rounded-full"
                />
            </div>
        </div>
    }
}

/// Badge grid skeleton
#[component]
pub fn BadgeGridSkeleton() -> impl IntoView {
    view! {
        <SkeletonCard class="border-badge-gold/20">
            // Title
            <div class="flex items-center gap-2 mb-4">
                <Skeleton
                    width="1.5rem"
                    height="1.5rem"
                    rounded="rounded"
                />
                <Skeleton
                    width="4rem"
                    height="1.25rem"
                    rounded="rounded"
                />
            </div>
            
            // Badge count
            <div class="mb-4">
                <Skeleton
                    width="6rem"
                    height="0.875rem"
                    rounded="rounded"
                />
            </div>
            
            // Badge grid
            <div class="grid grid-cols-4 gap-3">
                {(0..12).map(|_| {
                    view! {
                        <SkeletonBadge />
                    }
                }).collect_view()}
            </div>
        </SkeletonCard>
    }
}

/// Contribution graph skeleton
#[component]
pub fn ContributionGraphSkeleton() -> impl IntoView {
    view! {
        <SkeletonCard class="border-gm-success/20">
            // Header
            <div class="flex items-center justify-between mb-4">
                // Title
                <div class="flex items-center gap-2">
                    <Skeleton
                        width="1.5rem"
                        height="1.5rem"
                        rounded="rounded"
                    />
                    <Skeleton
                        width="10rem"
                        height="1.25rem"
                        rounded="rounded"
                    />
                </div>
                
                // Contribution count
                <Skeleton
                    width="10rem"
                    height="0.875rem"
                    rounded="rounded"
                />
            </div>
            
            // Graph
            <div class="overflow-x-auto">
                <SkeletonGraph weeks=40 days=7 />
            </div>
            
            // Legend
            <div class="flex items-center justify-end gap-2 mt-4">
                <Skeleton
                    width="2rem"
                    height="0.75rem"
                    rounded="rounded"
                />
                {(0..5).map(|_| {
                    view! {
                        <Skeleton
                            width="0.75rem"
                            height="0.75rem"
                            rounded="rounded-sm"
                        />
                    }
                }).collect_view()}
                <Skeleton
                    width="2rem"
                    height="0.75rem"
                    rounded="rounded"
                />
            </div>
        </SkeletonCard>
    }
}
