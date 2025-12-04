//! Home Page Loading Components
//!
//! Loading skeleton and loading state components for the home page.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   └─ src/components/pages/home/mod.rs
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

use leptos::prelude::*;

/// Home page loading skeleton
/// Re-exports the HomeSkeleton from home module
pub use crate::components::home::HomeSkeleton;

/// Simple loading spinner component
#[component]
pub fn LoadingSpinner() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center py-12">
            <div class="animate-spin w-8 h-8 border-2 border-gm-accent-cyan border-t-transparent rounded-full"/>
        </div>
    }
}
