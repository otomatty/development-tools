//! Base skeleton component
//!
//! Provides the fundamental skeleton building block with pulse animation.
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this component):
//!   └─ src/components/ui/skeleton/mod.rs
//!
//! Related Documentation:
//!   └─ GitHub Issue: #114

use crate::components::use_animation_context_or_default;
use leptos::prelude::*;

/// Base skeleton component with pulse effect
///
/// # Props
/// - `width`: CSS width value (e.g., "100%", "200px", "20rem")
/// - `height`: CSS height value (e.g., "1rem", "40px")
/// - `rounded`: Border radius class (e.g., "rounded", "rounded-full", "rounded-xl")
/// - `class`: Additional CSS classes
#[component]
pub fn Skeleton(
    /// Width of the skeleton (CSS value)
    #[prop(default = "100%")]
    width: &'static str,
    /// Height of the skeleton (CSS value)
    #[prop(default = "1rem")]
    height: &'static str,
    /// Border radius class
    #[prop(default = "rounded")]
    rounded: &'static str,
    /// Additional CSS classes
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let animation_ctx = use_animation_context_or_default();

    let animation_class = move || {
        if animation_ctx.enabled.get() {
            "animate-pulse"
        } else {
            ""
        }
    };

    view! {
        <div
            class=move || format!(
                "bg-slate-700/50 {} {} {}",
                rounded,
                animation_class(),
                class
            )
            style=format!("width: {}; height: {};", width, height)
        />
    }
}
