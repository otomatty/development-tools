//! Base skeleton component
//!
//! Provides the fundamental skeleton building block with shimmer animation.

use leptos::prelude::*;
use crate::components::use_animation_context_or_default;

/// Base skeleton component with shimmer effect
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
