//! Mock Server Page Loading Components
//!
//! Loading skeleton and loading state components for the mock server page.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   └─ src/components/pages/mock_server/mod.rs
//! Imports (shared modules):
//!   └─ crate::components::pages::shared_loading::{LoadingSpinner, ListSkeleton}
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

pub use crate::components::pages::shared_loading::LoadingSpinner;
use leptos::prelude::*;

/// Mock server page loading skeleton
#[component]
pub fn MockServerSkeleton() -> impl IntoView {
    view! {
        <div class="space-y-6 animate-pulse">
            // Server status skeleton
            <div class="bg-dt-card border border-slate-700/50 rounded-lg p-4">
                <div class="flex items-center gap-4">
                    <div class="w-12 h-12 bg-slate-700/50 rounded-lg"></div>
                    <div class="flex-1 space-y-2">
                        <div class="h-4 bg-slate-700/50 rounded w-24"></div>
                        <div class="h-3 bg-slate-700/50 rounded w-48"></div>
                    </div>
                </div>
            </div>

            // Directory mappings skeleton
            <div class="bg-dt-card border border-slate-700/50 rounded-lg p-4">
                <div class="h-5 bg-slate-700/50 rounded w-32 mb-4"></div>
                <div class="space-y-3">
                    {(0..3).map(|_| view! {
                        <div class="flex items-center gap-4 p-3 bg-slate-800/50 rounded-lg">
                            <div class="w-8 h-8 bg-slate-700/50 rounded"></div>
                            <div class="flex-1 space-y-2">
                                <div class="h-4 bg-slate-700/50 rounded w-24"></div>
                                <div class="h-3 bg-slate-700/50 rounded w-48"></div>
                            </div>
                        </div>
                    }).collect_view()}
                </div>
            </div>

            // CORS settings skeleton
            <div class="bg-dt-card border border-slate-700/50 rounded-lg p-4">
                <div class="h-5 bg-slate-700/50 rounded w-28 mb-4"></div>
                <div class="flex items-center gap-4">
                    <div class="w-12 h-6 bg-slate-700/50 rounded-full"></div>
                    <div class="h-4 bg-slate-700/50 rounded w-32"></div>
                </div>
            </div>
        </div>
    }
}
