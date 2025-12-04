//! Settings Page Loading Components
//!
//! Loading skeleton and loading state components for the settings page.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   └─ src/components/pages/settings/mod.rs
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

use leptos::prelude::*;

/// Settings page loading skeleton
#[component]
pub fn SettingsSkeleton() -> impl IntoView {
    view! {
        <div class="space-y-4 animate-pulse">
            {(0..6).map(|_| view! {
                <div class="bg-gm-bg-card/80 rounded-2xl border border-gm-accent-cyan/20 p-4">
                    <div class="flex items-center justify-between">
                        <div class="flex items-center gap-3">
                            <div class="w-5 h-5 bg-slate-700/50 rounded"></div>
                            <div class="h-5 bg-slate-700/50 rounded w-32"></div>
                        </div>
                        <div class="w-5 h-5 bg-slate-700/50 rounded"></div>
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
