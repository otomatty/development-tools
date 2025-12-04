//! Badge Component
//!
//! A reusable badge component for displaying status and labels.
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this component):
//!   └─ src/components/ui/badge/mod.rs
//!
//! Related Documentation:
//!   └─ Spec: ./badge.spec.md

use leptos::prelude::*;

/// Badge style variants
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum BadgeVariant {
    /// Success badge (green) - for positive states like "Linked", "Active"
    Success,
    /// Warning badge (yellow/amber) - for pending states like "Pending", "Not linked"
    Warning,
    /// Error badge (red) - for error states like "Failed", "Error"
    Error,
    /// Info badge (cyan) - for informational states
    #[default]
    Info,
    /// Neutral badge (gray) - for default/inactive states
    Neutral,
    /// Purple badge - for special states like "Premium", "Pro"
    Purple,
    /// Gold badge - for achievement or special status
    Gold,
}

impl BadgeVariant {
    /// Get the CSS classes for this variant
    fn classes(&self) -> &'static str {
        match self {
            BadgeVariant::Success => "bg-green-500/20 border-green-500/50 text-green-400",
            BadgeVariant::Warning => "bg-amber-500/20 border-amber-500/50 text-amber-400",
            BadgeVariant::Error => "bg-red-500/20 border-red-500/50 text-red-400",
            BadgeVariant::Info => {
                "bg-gm-accent-cyan/20 border-gm-accent-cyan/50 text-gm-accent-cyan"
            }
            BadgeVariant::Neutral => "bg-slate-500/20 border-slate-500/50 text-slate-400",
            BadgeVariant::Purple => {
                "bg-gm-accent-purple/20 border-gm-accent-purple/50 text-gm-accent-purple"
            }
            BadgeVariant::Gold => {
                "bg-gm-accent-gold/20 border-gm-accent-gold/50 text-gm-accent-gold"
            }
        }
    }

    /// Get the dot color class for this variant
    fn dot_class(&self) -> &'static str {
        match self {
            BadgeVariant::Success => "bg-green-500",
            BadgeVariant::Warning => "bg-amber-500",
            BadgeVariant::Error => "bg-red-500",
            BadgeVariant::Info => "bg-gm-accent-cyan",
            BadgeVariant::Neutral => "bg-slate-500",
            BadgeVariant::Purple => "bg-gm-accent-purple",
            BadgeVariant::Gold => "bg-gm-accent-gold",
        }
    }
}

/// Badge size variants
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum BadgeSize {
    /// Small badge (px-2 py-0.5 text-xs)
    Small,
    /// Medium badge (px-2.5 py-1 text-sm) - default
    #[default]
    Medium,
    /// Large badge (px-3 py-1.5 text-base)
    Large,
}

impl BadgeSize {
    /// Get the CSS classes for this size
    fn classes(&self) -> &'static str {
        match self {
            BadgeSize::Small => "px-2 py-0.5 text-xs",
            BadgeSize::Medium => "px-2.5 py-1 text-sm",
            BadgeSize::Large => "px-3 py-1.5 text-base",
        }
    }

    /// Get the dot size class for this badge size
    fn dot_size(&self) -> &'static str {
        match self {
            BadgeSize::Small => "w-1.5 h-1.5",
            BadgeSize::Medium => "w-2 h-2",
            BadgeSize::Large => "w-2.5 h-2.5",
        }
    }
}

/// Badge component
///
/// A flexible badge component for displaying status labels.
///
/// # Props
///
/// - `text`: Badge text content
/// - `variant`: Badge style variant
/// - `size`: Badge size
/// - `with_dot`: Whether to show a status dot
/// - `class`: Additional CSS classes
///
/// # Example
///
/// ```rust
/// view! {
///     <Badge
///         text="Active"
///         variant=BadgeVariant::Success
///         with_dot=true
///     />
/// }
/// ```
#[component]
pub fn Badge(
    /// Badge text content
    text: &'static str,
    /// Badge style variant
    #[prop(default = BadgeVariant::Info)]
    variant: BadgeVariant,
    /// Badge size
    #[prop(default = BadgeSize::Medium)]
    size: BadgeSize,
    /// Whether to show a status dot
    #[prop(default = false)]
    with_dot: bool,
    /// Additional CSS classes
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let base_classes = "inline-flex items-center gap-1.5 font-medium border rounded-2xl";

    let combined_class = format!(
        "{} {} {} {}",
        base_classes,
        variant.classes(),
        size.classes(),
        class
    );

    let dot_class = format!("{} {} rounded-full", size.dot_size(), variant.dot_class());

    view! {
        <span class=combined_class>
            {with_dot.then(|| view! {
                <span class=dot_class.clone()></span>
            })}
            {text}
        </span>
    }
}

/// Dynamic Badge component
///
/// A badge component that accepts dynamic text (String or Signal).
///
/// # Props
///
/// - `text`: Badge text content (dynamic)
/// - `variant`: Badge style variant
/// - `size`: Badge size
/// - `with_dot`: Whether to show a status dot
/// - `class`: Additional CSS classes
#[component]
pub fn DynamicBadge(
    /// Badge text content (dynamic)
    text: String,
    /// Badge style variant
    #[prop(default = BadgeVariant::Info)]
    variant: BadgeVariant,
    /// Badge size
    #[prop(default = BadgeSize::Medium)]
    size: BadgeSize,
    /// Whether to show a status dot
    #[prop(default = false)]
    with_dot: bool,
    /// Additional CSS classes
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let base_classes = "inline-flex items-center gap-1.5 font-medium border rounded-2xl";

    let combined_class = format!(
        "{} {} {} {}",
        base_classes,
        variant.classes(),
        size.classes(),
        class
    );

    let dot_class = format!("{} {} rounded-full", size.dot_size(), variant.dot_class());

    view! {
        <span class=combined_class>
            {with_dot.then(|| view! {
                <span class=dot_class.clone()></span>
            })}
            {text}
        </span>
    }
}

/// Status Badge component
///
/// A pre-configured badge specifically for status indicators.
/// Always shows a dot and uses appropriate variants for common statuses.
///
/// # Props
///
/// - `status`: Status type (determines variant and text)
/// - `size`: Badge size
/// - `class`: Additional CSS classes
///
/// # Example
///
/// ```rust
/// view! {
///     <StatusBadge status=Status::Linked />
/// }
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    /// Linked/Connected status (green)
    Linked,
    /// Not linked status (yellow)
    NotLinked,
    /// Active status (green)
    Active,
    /// Inactive status (gray)
    Inactive,
    /// Pending status (yellow)
    Pending,
    /// Error status (red)
    Error,
    /// Success status (green)
    Success,
    /// Warning status (yellow)
    Warning,
    /// Syncing status (cyan)
    Syncing,
    /// Offline status (gray)
    Offline,
    /// Online status (green)
    Online,
}

impl Status {
    /// Get the display text for this status
    fn text(&self) -> &'static str {
        match self {
            Status::Linked => "Linked",
            Status::NotLinked => "Not linked",
            Status::Active => "Active",
            Status::Inactive => "Inactive",
            Status::Pending => "Pending",
            Status::Error => "Error",
            Status::Success => "Success",
            Status::Warning => "Warning",
            Status::Syncing => "Syncing",
            Status::Offline => "Offline",
            Status::Online => "Online",
        }
    }

    /// Get the badge variant for this status
    fn variant(&self) -> BadgeVariant {
        match self {
            Status::Linked | Status::Active | Status::Success | Status::Online => {
                BadgeVariant::Success
            }
            Status::NotLinked | Status::Pending | Status::Warning => BadgeVariant::Warning,
            Status::Error => BadgeVariant::Error,
            Status::Syncing => BadgeVariant::Info,
            Status::Inactive | Status::Offline => BadgeVariant::Neutral,
        }
    }
}

#[component]
pub fn StatusBadge(
    /// Status type
    status: Status,
    /// Badge size
    #[prop(default = BadgeSize::Small)]
    size: BadgeSize,
    /// Additional CSS classes
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let text = status.text();
    let variant = status.variant();

    view! {
        <Badge
            text=text
            variant=variant
            size=size
            with_dot=true
            class=class
        />
    }
}
