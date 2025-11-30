//! Animated emoji component
//!
//! Displays animated emojis based on Google Noto Animated Emoji concept.
//! Uses CSS animations as a first phase implementation.
//! Future: Lottie integration for richer animations.

use leptos::prelude::*;

use crate::components::use_animation_context_or_default;

/// Supported animated emoji types
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EmojiType {
    /// 🔥 Fire - for streaks
    Fire,
    /// 🏆 Trophy - for achievements, best records
    Trophy,
    /// ⭐ Star - for ratings, star counts
    Star,
    /// 🎯 Target - for goals, badges
    Target,
    /// 💪 Muscle - for streak milestones
    Muscle,
    /// 👑 Crown - for highest level badges
    Crown,
    /// 🎉 Party - for level ups, badge unlocks
    Party,
    /// ✨ Sparkles - for quality badges, XP notifications
    Sparkles,
    /// 🚀 Rocket - for growth, progress
    Rocket,
}

impl EmojiType {
    /// Get the unicode emoji character
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Fire => "🔥",
            Self::Trophy => "🏆",
            Self::Star => "⭐",
            Self::Target => "🎯",
            Self::Muscle => "💪",
            Self::Crown => "👑",
            Self::Party => "🎉",
            Self::Sparkles => "✨",
            Self::Rocket => "🚀",
        }
    }

    /// Get the CSS animation class for this emoji
    pub fn animation_class(&self) -> &'static str {
        match self {
            Self::Fire => "animate-emoji-flame",
            Self::Trophy => "animate-emoji-shine",
            Self::Star => "animate-emoji-twinkle",
            Self::Target => "animate-emoji-pulse-scale",
            Self::Muscle => "animate-emoji-flex",
            Self::Crown => "animate-emoji-float",
            Self::Party => "animate-emoji-bounce",
            Self::Sparkles => "animate-emoji-sparkle",
            Self::Rocket => "animate-emoji-launch",
        }
    }

    /// Get the base color class for this emoji (used when not animating)
    pub fn color_class(&self) -> &'static str {
        match self {
            Self::Fire => "text-orange-400",
            Self::Trophy => "text-yellow-400",
            Self::Star => "text-yellow-300",
            Self::Target => "text-red-400",
            Self::Muscle => "text-amber-400",
            Self::Crown => "text-yellow-500",
            Self::Party => "text-pink-400",
            Self::Sparkles => "text-cyan-300",
            Self::Rocket => "text-blue-400",
        }
    }
}

/// Animation intensity levels
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum AnimationIntensity {
    /// No animation
    None,
    /// Subtle animation
    Subtle,
    /// Normal animation
    #[default]
    Normal,
    /// Strong/intense animation
    Strong,
}

impl AnimationIntensity {
    /// Get the CSS class modifier for this intensity
    pub fn class_modifier(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::Subtle => "animation-subtle",
            Self::Normal => "",
            Self::Strong => "animation-strong",
        }
    }
}

/// Animated emoji component
///
/// Displays an emoji with optional CSS animation based on the animation context.
/// Supports hover-only animation mode for better UX.
///
/// # Example
/// ```rust
/// view! {
///     <AnimatedEmoji
///         emoji=EmojiType::Fire
///         size="text-4xl"
///         hover_only=true
///     />
/// }
/// ```
#[component]
pub fn AnimatedEmoji(
    /// The type of emoji to display
    emoji: EmojiType,
    /// CSS size class (e.g., "text-2xl", "text-4xl")
    #[prop(default = "text-2xl")]
    size: &'static str,
    /// Only animate when hovered
    #[prop(default = false)]
    hover_only: bool,
    /// Animation intensity
    #[prop(default = AnimationIntensity::Normal)]
    intensity: AnimationIntensity,
    /// Additional CSS classes
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let animation_ctx = use_animation_context_or_default();
    let (is_hovered, set_is_hovered) = signal(false);

    let computed_class = move || {
        let enabled = animation_ctx.is_enabled();
        let hovered = is_hovered.get();
        let should_animate = enabled && intensity != AnimationIntensity::None && (!hover_only || hovered);

        let mut classes = vec![size.to_string()];
        
        if should_animate {
            classes.push(emoji.animation_class().to_string());
            if !intensity.class_modifier().is_empty() {
                classes.push(intensity.class_modifier().to_string());
            }
        }

        // Add transition for smooth animation start/stop
        classes.push("transition-transform duration-200".to_string());

        // Add custom classes
        if !class.is_empty() {
            classes.push(class.to_string());
        }

        classes.join(" ")
    };

    view! {
        <span
            class=computed_class
            on:mouseenter=move |_| set_is_hovered.set(true)
            on:mouseleave=move |_| set_is_hovered.set(false)
            role="img"
            aria-label=emoji.emoji()
        >
            {emoji.emoji()}
        </span>
    }
}

/// Animated emoji with dynamic intensity based on a value
///
/// Useful for showing different animation intensities based on streak days, XP, etc.
#[component]
pub fn AnimatedEmojiWithIntensity(
    /// The type of emoji to display
    emoji: EmojiType,
    /// CSS size class
    #[prop(default = "text-2xl")]
    size: &'static str,
    /// Only animate when hovered
    #[prop(default = false)]
    hover_only: bool,
    /// Value to determine intensity (higher = stronger animation)
    #[prop(into)]
    value: Signal<i32>,
    /// Thresholds for intensity levels [subtle, normal, strong]
    #[prop(default = [1, 7, 30])]
    thresholds: [i32; 3],
    /// Additional CSS classes
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let intensity = move || {
        let v = value.get();
        if v >= thresholds[2] {
            AnimationIntensity::Strong
        } else if v >= thresholds[1] {
            AnimationIntensity::Normal
        } else if v >= thresholds[0] {
            AnimationIntensity::Subtle
        } else {
            AnimationIntensity::None
        }
    };

    let animation_ctx = use_animation_context_or_default();
    let (is_hovered, set_is_hovered) = signal(false);

    let computed_class = move || {
        let enabled = animation_ctx.is_enabled();
        let hovered = is_hovered.get();
        let current_intensity = intensity();
        let should_animate = enabled && current_intensity != AnimationIntensity::None && (!hover_only || hovered);

        let mut classes = vec![size.to_string()];
        
        if should_animate {
            classes.push(emoji.animation_class().to_string());
            if !current_intensity.class_modifier().is_empty() {
                classes.push(current_intensity.class_modifier().to_string());
            }
        }

        classes.push("transition-transform duration-200".to_string());

        if !class.is_empty() {
            classes.push(class.to_string());
        }

        classes.join(" ")
    };

    view! {
        <span
            class=computed_class
            on:mouseenter=move |_| set_is_hovered.set(true)
            on:mouseleave=move |_| set_is_hovered.set(false)
            role="img"
            aria-label=emoji.emoji()
        >
            {emoji.emoji()}
        </span>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emoji_type_emoji() {
        assert_eq!(EmojiType::Fire.emoji(), "🔥");
        assert_eq!(EmojiType::Trophy.emoji(), "🏆");
        assert_eq!(EmojiType::Sparkles.emoji(), "✨");
    }

    #[test]
    fn test_emoji_type_animation_class() {
        assert_eq!(EmojiType::Fire.animation_class(), "animate-emoji-flame");
        assert_eq!(EmojiType::Trophy.animation_class(), "animate-emoji-shine");
    }

    #[test]
    fn test_animation_intensity_modifier() {
        assert_eq!(AnimationIntensity::None.class_modifier(), "");
        assert_eq!(AnimationIntensity::Subtle.class_modifier(), "animation-subtle");
        assert_eq!(AnimationIntensity::Strong.class_modifier(), "animation-strong");
    }
}
