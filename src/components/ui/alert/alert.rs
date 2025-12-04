//! Alert Component
//!
//! A reusable alert/banner component for displaying messages.
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this component):
//!   └─ src/components/ui/alert/mod.rs
//!
//! Related Documentation:
//!   └─ Spec: ./alert.spec.md

use leptos::prelude::*;

use crate::components::icons::Icon;

/// Alert style variants
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum AlertVariant {
    /// Success alert (green)
    Success,
    /// Warning alert (amber)
    Warning,
    /// Error alert (red)
    #[default]
    Error,
    /// Info alert (cyan)
    Info,
}

impl AlertVariant {
    /// Get the CSS classes for this variant
    fn classes(&self) -> (&'static str, &'static str, &'static str) {
        // Returns (bg_class, border_class, text_class)
        match self {
            AlertVariant::Success => ("bg-green-500/20", "border-green-500/50", "text-green-400"),
            AlertVariant::Warning => ("bg-amber-500/20", "border-amber-500/50", "text-amber-400"),
            AlertVariant::Error => ("bg-red-500/20", "border-red-500/50", "text-red-400"),
            AlertVariant::Info => (
                "bg-gm-accent-cyan/20",
                "border-gm-accent-cyan/50",
                "text-gm-accent-cyan",
            ),
        }
    }

    /// Get the icon for this variant
    fn icon(&self) -> &'static str {
        match self {
            AlertVariant::Success => "check-circle",
            AlertVariant::Warning => "alert-triangle",
            AlertVariant::Error => "x-circle",
            AlertVariant::Info => "info",
        }
    }
}

/// Alert component
///
/// Displays an alert message with optional title and dismiss button.
///
/// # Props
///
/// - `variant`: Alert style variant
/// - `message`: Alert message content
/// - `title`: Optional title text
/// - `dismissible`: Whether the alert can be dismissed
/// - `on_dismiss`: Callback when dismiss button is clicked
/// - `class`: Additional CSS classes
///
/// # Example
///
/// ```rust
/// view! {
///     <Alert
///         variant=AlertVariant::Error
///         message=error_message.into()
///         title="Error"
///         dismissible=true
///         on_dismiss=move || set_error.set(None)
///     />
/// }
/// ```
#[component]
pub fn Alert<F>(
    /// Alert style variant
    #[prop(default = AlertVariant::Error)]
    variant: AlertVariant,
    /// Alert message content
    #[prop(into)]
    message: Signal<String>,
    /// Optional title text
    #[prop(optional)]
    title: Option<&'static str>,
    /// Whether the alert can be dismissed
    #[prop(default = false)]
    dismissible: bool,
    /// Callback when dismiss button is clicked
    #[prop(optional)]
    on_dismiss: Option<F>,
    /// Additional CSS classes
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView
where
    F: Fn() + 'static + Clone + Send + Sync,
{
    let (bg_class, border_class, text_class) = variant.classes();
    let icon = variant.icon();

    let combined_class = format!(
        "flex items-start gap-3 p-4 {} border {} rounded-2xl {}",
        bg_class, border_class, class
    );

    view! {
        <div class=combined_class role="alert">
            // Icon
            <Icon
                name=icon.to_string()
                class=format!("w-5 h-5 {} flex-shrink-0 mt-0.5", text_class)
            />

            // Content
            <div class="flex-1 min-w-0">
                {title.map(|t| view! {
                    <h4 class=format!("font-semibold {} mb-1", text_class)>
                        {t}
                    </h4>
                })}
                <p class=format!("text-sm {}", text_class)>
                    {move || message.get()}
                </p>
            </div>

            // Dismiss button
            {dismissible.then(|| {
                let on_dismiss = on_dismiss.clone();
                view! {
                    <button
                        type="button"
                        class=format!(
                            "flex-shrink-0 p-1 {} hover:opacity-70 transition-opacity rounded-lg",
                            text_class
                        )
                        on:click=move |_| {
                            if let Some(ref callback) = on_dismiss {
                                callback();
                            }
                        }
                        aria-label="Dismiss"
                    >
                        <Icon name="x".to_string() class="w-4 h-4".to_string() />
                    </button>
                }
            })}
        </div>
    }
}

/// Banner component
///
/// A full-width banner for important announcements.
///
/// # Props
///
/// - `variant`: Banner style variant
/// - `message`: Banner message content
/// - `dismissible`: Whether the banner can be dismissed
/// - `on_dismiss`: Callback when dismiss button is clicked
///
/// # Example
///
/// ```rust
/// view! {
///     <Banner
///         variant=AlertVariant::Info
///         message="New features available!"
///         dismissible=true
///         on_dismiss=move || set_show_banner.set(false)
///     />
/// }
/// ```
#[component]
pub fn Banner<F>(
    /// Banner style variant
    #[prop(default = AlertVariant::Info)]
    variant: AlertVariant,
    /// Banner message content
    message: &'static str,
    /// Whether the banner can be dismissed
    #[prop(default = false)]
    dismissible: bool,
    /// Callback when dismiss button is clicked
    #[prop(optional)]
    on_dismiss: Option<F>,
) -> impl IntoView
where
    F: Fn() + 'static + Clone + Send + Sync,
{
    let (bg_class, border_class, text_class) = variant.classes();
    let icon = variant.icon();

    view! {
        <div
            class=format!(
                "flex items-center justify-center gap-3 px-4 py-3 {} border-b {} w-full",
                bg_class, border_class
            )
            role="alert"
        >
            <Icon
                name=icon.to_string()
                class=format!("w-5 h-5 {}", text_class)
            />
            <p class=format!("text-sm font-medium {}", text_class)>
                {message}
            </p>
            {dismissible.then(|| {
                let on_dismiss = on_dismiss.clone();
                view! {
                    <button
                        type="button"
                        class=format!(
                            "ml-auto p-1 {} hover:opacity-70 transition-opacity rounded-lg",
                            text_class
                        )
                        on:click=move |_| {
                            if let Some(ref callback) = on_dismiss {
                                callback();
                            }
                        }
                        aria-label="Dismiss"
                    >
                        <Icon name="x".to_string() class="w-4 h-4".to_string() />
                    </button>
                }
            })}
        </div>
    }
}
