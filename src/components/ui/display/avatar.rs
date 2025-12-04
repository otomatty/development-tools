//! Avatar Component
//!
//! A reusable avatar component for displaying user profile images.
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this component):
//!   └─ src/components/ui/display/mod.rs
//!
//! Related Documentation:
//!   └─ Spec: ./display.spec.md

use leptos::prelude::*;

/// Avatar size variants
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum AvatarSize {
    /// Extra small avatar (w-6 h-6)
    XSmall,
    /// Small avatar (w-8 h-8)
    Small,
    /// Medium avatar (w-12 h-12) - default
    #[default]
    Medium,
    /// Large avatar (w-16 h-16)
    Large,
    /// Extra large avatar (w-20 h-20)
    XLarge,
}

impl AvatarSize {
    /// Get the CSS classes for this size
    fn classes(&self) -> &'static str {
        match self {
            AvatarSize::XSmall => "w-6 h-6",
            AvatarSize::Small => "w-8 h-8",
            AvatarSize::Medium => "w-12 h-12",
            AvatarSize::Large => "w-16 h-16",
            AvatarSize::XLarge => "w-20 h-20",
        }
    }

    /// Get the text size for fallback initials
    fn text_class(&self) -> &'static str {
        match self {
            AvatarSize::XSmall => "text-xs",
            AvatarSize::Small => "text-sm",
            AvatarSize::Medium => "text-lg",
            AvatarSize::Large => "text-xl",
            AvatarSize::XLarge => "text-2xl",
        }
    }

    /// Get the badge position offset
    fn badge_class(&self) -> &'static str {
        match self {
            AvatarSize::XSmall | AvatarSize::Small => "-bottom-1 -right-1",
            AvatarSize::Medium => "-bottom-1.5 -right-1.5",
            AvatarSize::Large | AvatarSize::XLarge => "-bottom-2 -right-2",
        }
    }
}

/// Avatar component
///
/// Displays a user avatar with optional fallback and badge.
///
/// # Props
///
/// - `src`: Optional image source URL
/// - `alt`: Alt text for the image
/// - `size`: Avatar size
/// - `fallback`: Fallback text (initials or emoji)
/// - `children`: Optional badge content (displayed at bottom-right)
/// - `class`: Additional CSS classes
/// - `border`: Whether to show a border
///
/// # Example
///
/// ```rust
/// view! {
///     <Avatar
///         src=user.avatar_url
///         alt="User avatar"
///         size=AvatarSize::Large
///         fallback="👤"
///         border=true
///     >
///         <div class="px-2 py-1 bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple rounded-lg text-white text-sm font-bold">
///             "Lv.5"
///         </div>
///     </Avatar>
/// }
/// ```
#[component]
pub fn Avatar(
    /// Optional image source URL
    #[prop(optional, into)]
    src: Option<String>,
    /// Alt text for the image
    #[prop(default = "Avatar")]
    alt: &'static str,
    /// Avatar size
    #[prop(default = AvatarSize::Medium)]
    size: AvatarSize,
    /// Fallback text (initials or emoji)
    #[prop(default = "👤")]
    fallback: &'static str,
    /// Optional badge content
    #[prop(optional)]
    children: Option<Children>,
    /// Additional CSS classes
    #[prop(default = "")]
    class: &'static str,
    /// Whether to show a border
    #[prop(default = false)]
    border: bool,
) -> impl IntoView {
    let size_class = size.classes();
    let text_class = size.text_class();
    let badge_position = size.badge_class();

    let border_class = if border {
        "border-2 border-gm-accent-cyan shadow-neon-cyan"
    } else {
        ""
    };

    let base_class = format!("{} rounded-2xl {} {}", size_class, border_class, class);

    view! {
        <div class="relative inline-block">
            {move || {
                if let Some(ref url) = src {
                    if !url.is_empty() {
                        return view! {
                            <img
                                src=url.clone()
                                alt=alt
                                class=format!("{} object-cover", base_class)
                            />
                        }.into_any();
                    }
                }

                // Fallback view
                view! {
                    <div class=format!(
                        "{} bg-gm-bg-secondary flex items-center justify-center",
                        base_class
                    )>
                        <span class=text_class>
                            {fallback}
                        </span>
                    </div>
                }.into_any()
            }}

            // Badge
            {children.map(|c| view! {
                <div class=format!("absolute {}", badge_position)>
                    {c()}
                </div>
            })}
        </div>
    }
}
