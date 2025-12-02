//! Card Component
//!
//! A reusable card container component with multiple style variants.
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this component):
//!   └─ src/components/ui/card/mod.rs
//!
//! Related Documentation:
//!   └─ Issue: GitHub Issue #114

use leptos::prelude::*;

/// Card style variants
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum CardVariant {
    /// Default card style (bg-dt-card)
    #[default]
    Default,
    /// Gaming-style card with blur effect (bg-gm-bg-card/80 backdrop-blur-sm)
    Gaming,
    /// Gaming card with cyan accent border
    GamingCyan,
    /// Gaming card with gold accent border
    GamingGold,
    /// Gaming card with purple accent border
    GamingPurple,
    /// Elevated card with stronger shadow
    Elevated,
    /// Simple card without background
    Ghost,
}

impl CardVariant {
    /// Get the CSS classes for this variant
    fn classes(&self) -> &'static str {
        match self {
            CardVariant::Default => "bg-dt-card border border-slate-700/50 rounded-lg",
            CardVariant::Gaming => {
                "bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-slate-700/30"
            }
            CardVariant::GamingCyan => {
                "bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-cyan/20"
            }
            CardVariant::GamingGold => {
                "bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-gold/20"
            }
            CardVariant::GamingPurple => {
                "bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-purple/20"
            }
            CardVariant::Elevated => "bg-dt-card border border-slate-700/50 rounded-lg shadow-lg",
            CardVariant::Ghost => "rounded-lg",
        }
    }
}

/// Card component
///
/// A flexible container component that can be styled with different variants.
///
/// # Props
///
/// - `children`: The content to render inside the card
/// - `variant`: The style variant to use (default: `CardVariant::Default`)
/// - `class`: Additional CSS classes to apply
/// - `padding`: Whether to apply default padding (default: true)
///
/// # Example
///
/// ```rust
/// view! {
///     <Card variant=CardVariant::GamingCyan>
///         <h3>"Card Title"</h3>
///         <p>"Card content goes here"</p>
///     </Card>
/// }
/// ```
#[component]
pub fn Card(
    /// The content to render inside the card
    children: Children,
    /// The style variant to use
    #[prop(default = CardVariant::Default)]
    variant: CardVariant,
    /// Additional CSS classes to apply
    #[prop(default = "")]
    class: &'static str,
    /// Whether to apply default padding (p-6)
    #[prop(default = true)]
    padding: bool,
) -> impl IntoView {
    let padding_class = if padding { "p-6" } else { "" };

    // Filter empty classes and join to avoid extra whitespace
    let classes = [variant.classes(), padding_class, class]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    view! {
        <div class=classes>
            {children()}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_variant_classes() {
        assert!(CardVariant::Default.classes().contains("bg-dt-card"));
        assert!(CardVariant::Gaming.classes().contains("backdrop-blur-sm"));
        assert!(CardVariant::GamingCyan.classes().contains("gm-accent-cyan"));
        assert!(CardVariant::GamingGold.classes().contains("gm-accent-gold"));
        assert!(CardVariant::GamingPurple
            .classes()
            .contains("gm-accent-purple"));
        assert!(CardVariant::Elevated.classes().contains("shadow-lg"));
        assert_eq!(CardVariant::Ghost.classes(), "rounded-lg");
    }

    #[test]
    fn test_card_variant_default() {
        assert_eq!(CardVariant::default(), CardVariant::Default);
    }
}
