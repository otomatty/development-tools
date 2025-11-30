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

    /// Get accessible aria-label for screen readers
    pub fn aria_label(&self) -> &'static str {
        match self {
            Self::Fire => "streak fire",
            Self::Trophy => "trophy achievement",
            Self::Star => "star rating",
            Self::Target => "goal target",
            Self::Muscle => "strength milestone",
            Self::Crown => "crown achievement",
            Self::Party => "celebration",
            Self::Sparkles => "sparkles effect",
            Self::Rocket => "progress rocket",
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

/// Build CSS classes for animated emoji
///
/// Helper function to avoid code duplication between components.
fn build_emoji_classes(
    is_animation_enabled: bool,
    is_hovered: bool,
    hover_only: bool,
    emoji: EmojiType,
    intensity: AnimationIntensity,
    size: &str,
    custom_class: &str,
) -> String {
    let should_animate = is_animation_enabled
        && intensity != AnimationIntensity::None
        && (!hover_only || is_hovered);

    let mut classes = vec![size.to_string()];

    if should_animate {
        classes.push(emoji.animation_class().to_string());
        if !intensity.class_modifier().is_empty() {
            classes.push(intensity.class_modifier().to_string());
        }
    }

    // Add transition for smooth animation start/stop
    classes.push("transition-transform duration-200".to_string());

    if !custom_class.is_empty() {
        classes.push(custom_class.to_string());
    }

    classes.join(" ")
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
        build_emoji_classes(
            animation_ctx.is_enabled(),
            is_hovered.get(),
            hover_only,
            emoji,
            intensity,
            size,
            class,
        )
    };

    view! {
        <span
            class=computed_class
            on:mouseenter=move |_| set_is_hovered.set(true)
            on:mouseleave=move |_| set_is_hovered.set(false)
            role="img"
            aria-label=emoji.aria_label()
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
        build_emoji_classes(
            animation_ctx.is_enabled(),
            is_hovered.get(),
            hover_only,
            emoji,
            intensity(),
            size,
            class,
        )
    };

    view! {
        <span
            class=computed_class
            on:mouseenter=move |_| set_is_hovered.set(true)
            on:mouseleave=move |_| set_is_hovered.set(false)
            role="img"
            aria-label=emoji.aria_label()
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
    fn test_emoji_type_aria_label() {
        assert_eq!(EmojiType::Fire.aria_label(), "streak fire");
        assert_eq!(EmojiType::Trophy.aria_label(), "trophy achievement");
        assert_eq!(EmojiType::Sparkles.aria_label(), "sparkles effect");
    }

    #[test]
    fn test_animation_intensity_modifier() {
        assert_eq!(AnimationIntensity::None.class_modifier(), "");
        assert_eq!(AnimationIntensity::Subtle.class_modifier(), "animation-subtle");
        assert_eq!(AnimationIntensity::Strong.class_modifier(), "animation-strong");
    }
}
