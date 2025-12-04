//! Project Dashboard Loading Components
//!
//! Loading skeleton and loading state components for the project dashboard page.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   └─ src/components/pages/project_dashboard/mod.rs
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

use leptos::prelude::*;

/// Project dashboard loading skeleton
#[component]
pub fn ProjectDashboardSkeleton() -> impl IntoView {
    view! {
        <div class="flex-1 flex flex-col overflow-hidden animate-pulse">
            // Header skeleton
            <div class="p-4 border-b border-slate-700/50 bg-dt-card/50">
                <div class="flex items-center gap-4">
                    <div class="w-10 h-10 bg-slate-700/50 rounded-lg"></div>
                    <div class="space-y-2">
                        <div class="h-5 bg-slate-700/50 rounded w-32"></div>
                        <div class="h-3 bg-slate-700/50 rounded w-24"></div>
                    </div>
                </div>
            </div>

            // Kanban skeleton
            <div class="flex-1 p-4 overflow-x-auto">
                <div class="flex gap-4 h-full">
                    {(0..5).map(|_| view! {
                        <div class="flex-shrink-0 w-72 bg-slate-800/50 rounded-lg p-4">
                            <div class="h-5 bg-slate-700/50 rounded w-24 mb-4"></div>
                            <div class="space-y-3">
                                {(0..3).map(|_| view! {
                                    <div class="bg-slate-700/30 rounded-lg p-3 space-y-2">
                                        <div class="h-4 bg-slate-700/50 rounded w-full"></div>
                                        <div class="h-3 bg-slate-700/50 rounded w-3/4"></div>
                                        <div class="flex gap-2">
                                            <div class="h-5 bg-slate-700/50 rounded w-12"></div>
                                            <div class="h-5 bg-slate-700/50 rounded w-16"></div>
                                        </div>
                                    </div>
                                }).collect_view()}
                            </div>
                        </div>
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}

/// Loading spinner component
#[component]
pub fn LoadingSpinner() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center h-full">
            <div class="animate-spin w-8 h-8 border-2 border-gm-accent-cyan border-t-transparent rounded-full"/>
        </div>
    }
}

/// Not linked state component
#[component]
pub fn NotLinkedState(on_link: Callback<()>) -> impl IntoView {
    use crate::components::icons::Icon;

    view! {
        <div class="flex flex-col items-center justify-center h-full text-center px-4">
            <Icon name="github".to_string() class="w-20 h-20 text-slate-600 mb-4".to_string() />
            <h2 class="text-xl font-semibold text-dt-text mb-2">"Link a GitHub Repository"</h2>
            <p class="text-dt-text-sub mb-6 max-w-md">
                "Connect a GitHub repository to start managing issues with your kanban board."
            </p>
            <button
                class="flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple text-white rounded-lg hover:opacity-90 transition-opacity"
                on:click=move |_| on_link.run(())
            >
                <Icon name="link".to_string() class="w-5 h-5".to_string() />
                <span>"Link Repository"</span>
            </button>
        </div>
    }
}
