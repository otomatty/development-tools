//! Animation context module
//!
//! Provides a context for controlling animation state across the app.

#![allow(dead_code)]

use leptos::prelude::*;

/// Animation state context
/// This holds the global animation enabled state
#[derive(Clone, Copy)]
pub struct AnimationContext {
    /// Whether animations are enabled
    pub enabled: Signal<bool>,
    /// Set the animation enabled state
    pub set_enabled: WriteSignal<bool>,
}

impl AnimationContext {
    /// Create a new animation context
    pub fn new(enabled: bool) -> Self {
        let (get_enabled, set_enabled) = signal(enabled);
        Self {
            enabled: get_enabled.into(),
            set_enabled,
        }
    }

    /// Check if animations are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.get()
    }

    /// Get animation class based on enabled state
    /// Returns the animation class if enabled, empty string otherwise
    pub fn get_animation_class(&self, animation_class: &str) -> String {
        if self.enabled.get() {
            animation_class.to_string()
        } else {
            String::new()
        }
    }

    /// Get conditional class based on enabled state
    /// Returns the first class if enabled, second class otherwise
    pub fn conditional_class(&self, enabled_class: &str, disabled_class: &str) -> String {
        if self.enabled.get() {
            enabled_class.to_string()
        } else {
            disabled_class.to_string()
        }
    }
}

impl Default for AnimationContext {
    fn default() -> Self {
        Self::new(true) // Animations enabled by default
    }
}

/// Provide animation context to children
#[component]
pub fn AnimationProvider(
    /// Initial enabled state
    #[prop(default = true)]
    initial_enabled: bool,
    children: Children,
) -> impl IntoView {
    let context = AnimationContext::new(initial_enabled);
    provide_context(context);

    children()
}

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

/// Helper to conditionally apply animation class
pub fn animation_class(enabled: bool, class: &str) -> String {
    if enabled {
        class.to_string()
    } else {
        String::new()
    }
}

/// Helper to get body class for global animation control
pub fn get_body_animation_class(enabled: bool) -> &'static str {
    if enabled {
        ""
    } else {
        "no-animation"
    }
}

