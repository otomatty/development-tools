//! Projects Page Loading Components
//!
//! Loading skeleton and loading state components for the projects page.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   └─ src/components/pages/projects/mod.rs
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

use leptos::prelude::*;

/// Projects page loading skeleton
#[component]
pub fn ProjectsSkeleton() -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {(0..6).map(|_| view! {
                <div class="bg-dt-card border border-slate-700/50 rounded-lg p-4 animate-pulse">
                    <div class="flex items-center gap-3 mb-3">
                        <div class="w-10 h-10 bg-slate-700/50 rounded-lg"></div>
                        <div class="flex-1 space-y-2">
                            <div class="h-4 bg-slate-700/50 rounded w-32"></div>
                            <div class="h-3 bg-slate-700/50 rounded w-24"></div>
                        </div>
                    </div>
                    <div class="h-3 bg-slate-700/50 rounded w-full mb-2"></div>
                    <div class="h-3 bg-slate-700/50 rounded w-3/4 mb-4"></div>
                    <div class="flex items-center justify-between pt-3 border-t border-slate-700/50">
                        <div class="h-4 bg-slate-700/50 rounded w-16"></div>
                        <div class="h-4 bg-slate-700/50 rounded w-20"></div>
                    </div>
                </div>
            }).collect_view()}
        </div>
    }
}

/// Loading spinner component
#[component]
pub fn LoadingSpinner() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center py-12">
            <div class="animate-spin w-8 h-8 border-2 border-gm-accent-cyan border-t-transparent rounded-full"/>
        </div>
    }
}
