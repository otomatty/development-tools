//! Animation hooks
//!
//! アニメーションコンテキストを使用するためのカスタムフック。
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   - src/hooks/mod.rs
//!   - src/components/ (various components)
//! Dependencies:
//!   - src/contexts/animation_context.rs

use leptos::prelude::*;

use crate::contexts::AnimationContext;

/// Hook to use animation context
/// Returns None if context is not available
pub fn use_animation_context() -> Option<AnimationContext> {
    use_context::<AnimationContext>()
}

/// Hook to use animation context with default
/// Returns default context if not available
pub fn use_animation_context_or_default() -> AnimationContext {
    use_context::<AnimationContext>().unwrap_or_default()
}

/// Hook to check if animations are enabled
/// Returns a closure that can be called to get the current animation state
pub fn use_is_animated() -> impl Fn() -> bool {
    let ctx = use_animation_context_or_default();
    move || ctx.enabled.get()
}
