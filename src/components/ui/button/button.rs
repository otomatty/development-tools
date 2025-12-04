//! Button Component
//!
//! A reusable button component with multiple variants and sizes.
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this component):
//!   └─ src/components/ui/button/mod.rs
//!
//! Related Documentation:
//!   └─ Spec: ./button.spec.md

use leptos::prelude::*;

/// Button style variants
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ButtonVariant {
    /// Primary button with gradient background (cyan to purple)
    #[default]
    Primary,
    /// Secondary button with border and transparent background
    Secondary,
    /// Ghost button with transparent background, visible on hover
    Ghost,
    /// Danger button for destructive actions (red)
    Danger,
    /// Success button for positive actions (green)
    Success,
    /// Outline button with cyan border
    Outline,
}

impl ButtonVariant {
    /// Get the CSS classes for this variant
    fn classes(&self) -> &'static str {
        match self {
            ButtonVariant::Primary => {
                "bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple text-white \
                 hover:opacity-90 hover:shadow-[0_0_15px_rgba(6,182,212,0.4)] \
                 active:opacity-80 focus:ring-gm-accent-cyan"
            }
            ButtonVariant::Secondary => {
                "bg-gm-bg-secondary border border-slate-600 text-dt-text \
                 hover:bg-slate-700 hover:border-slate-500 \
                 active:bg-slate-600 focus:ring-slate-500"
            }
            ButtonVariant::Ghost => {
                "bg-transparent text-dt-text-sub \
                 hover:bg-slate-800 hover:text-dt-text \
                 active:bg-slate-700 focus:ring-slate-500"
            }
            ButtonVariant::Danger => {
                "bg-red-500/20 border border-red-500/50 text-red-400 \
                 hover:bg-red-500/30 hover:border-red-500 hover:text-red-300 \
                 active:bg-red-500/40 focus:ring-red-500"
            }
            ButtonVariant::Success => {
                "bg-green-500/20 border border-green-500/50 text-green-400 \
                 hover:bg-green-500/30 hover:border-green-500 hover:text-green-300 \
                 active:bg-green-500/40 focus:ring-green-500"
            }
            ButtonVariant::Outline => {
                "bg-transparent border border-gm-accent-cyan/50 text-gm-accent-cyan \
                 hover:bg-gm-accent-cyan/10 hover:border-gm-accent-cyan \
                 active:bg-gm-accent-cyan/20 focus:ring-gm-accent-cyan"
            }
        }
    }
}

/// Button size variants
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ButtonSize {
    /// Small button (px-3 py-1.5 text-sm)
    Small,
    /// Medium button (px-4 py-2 text-base) - default
    #[default]
    Medium,
    /// Large button (px-6 py-3 text-lg)
    Large,
}

impl ButtonSize {
    /// Get the CSS classes for this size
    fn classes(&self) -> &'static str {
        match self {
            ButtonSize::Small => "px-3 py-1.5 text-sm gap-1.5",
            ButtonSize::Medium => "px-4 py-2 text-base gap-2",
            ButtonSize::Large => "px-6 py-3 text-lg gap-2.5",
        }
    }

    /// Get the icon size class for this button size
    fn icon_class(&self) -> &'static str {
        match self {
            ButtonSize::Small => "w-4 h-4",
            ButtonSize::Medium => "w-5 h-5",
            ButtonSize::Large => "w-6 h-6",
        }
    }
}

/// Button component
///
/// A flexible button component with multiple variants and sizes.
///
/// # Props
///
/// - `children`: Button content (text and/or icons)
/// - `variant`: Button style variant
/// - `size`: Button size
/// - `disabled`: Whether the button is disabled
/// - `full_width`: Whether the button should take full width
/// - `class`: Additional CSS classes
/// - `on_click`: Click event handler
/// - `button_type`: HTML button type attribute
///
/// # Example
///
/// ```rust
/// view! {
///     <Button
///         variant=ButtonVariant::Primary
///         size=ButtonSize::Medium
///         on_click=move |_| log::info!("Clicked!")
///     >
///         <Icon name="plus" class="w-5 h-5" />
///         "Add Item"
///     </Button>
/// }
/// ```
#[component]
pub fn Button<F>(
    /// Button content
    children: Children,
    /// Button style variant
    #[prop(default = ButtonVariant::Primary)]
    variant: ButtonVariant,
    /// Button size
    #[prop(default = ButtonSize::Medium)]
    size: ButtonSize,
    /// Whether the button is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Whether the button should take full width
    #[prop(default = false)]
    full_width: bool,
    /// Additional CSS classes
    #[prop(default = "")]
    class: &'static str,
    /// Click event handler
    on_click: F,
    /// HTML button type attribute
    #[prop(default = "button")]
    button_type: &'static str,
) -> impl IntoView
where
    F: Fn(leptos::ev::MouseEvent) + 'static + Clone + Send + Sync,
{
    let base_classes = "inline-flex items-center justify-center font-medium \
                        rounded-2xl transition-all duration-200 \
                        focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gm-bg-primary \
                        disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none";

    let width_class = if full_width { "w-full" } else { "" };

    let combined_class = format!(
        "{} {} {} {} {}",
        base_classes,
        variant.classes(),
        size.classes(),
        width_class,
        class
    );

    view! {
        <button
            type=button_type
            class=combined_class
            disabled=disabled
            on:click=on_click
        >
            {children()}
        </button>
    }
}

/// Icon-only button component
///
/// A compact button for icon-only actions.
///
/// # Props
///
/// - `children`: Icon content
/// - `variant`: Button style variant
/// - `size`: Button size
/// - `disabled`: Whether the button is disabled
/// - `label`: Accessibility label (aria-label)
/// - `class`: Additional CSS classes
/// - `on_click`: Click event handler
///
/// # Example
///
/// ```rust
/// view! {
///     <IconButton
///         variant=ButtonVariant::Ghost
///         label="Delete item"
///         on_click=move |_| delete_item()
///     >
///         <Icon name="trash" class="w-5 h-5" />
///     </IconButton>
/// }
/// ```
#[component]
pub fn IconButton<F>(
    /// Icon content
    children: Children,
    /// Button style variant
    #[prop(default = ButtonVariant::Ghost)]
    variant: ButtonVariant,
    /// Button size
    #[prop(default = ButtonSize::Medium)]
    size: ButtonSize,
    /// Whether the button is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Accessibility label
    label: &'static str,
    /// Additional CSS classes
    #[prop(default = "")]
    class: &'static str,
    /// Click event handler
    on_click: F,
) -> impl IntoView
where
    F: Fn(leptos::ev::MouseEvent) + 'static + Clone + Send + Sync,
{
    let base_classes = "inline-flex items-center justify-center \
                        rounded-2xl transition-all duration-200 \
                        focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gm-bg-primary \
                        disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none";

    let size_class = match size {
        ButtonSize::Small => "p-1.5",
        ButtonSize::Medium => "p-2",
        ButtonSize::Large => "p-3",
    };

    let combined_class = format!(
        "{} {} {} {}",
        base_classes,
        variant.classes(),
        size_class,
        class
    );

    view! {
        <button
            type="button"
            class=combined_class
            disabled=disabled
            aria-label=label
            title=label
            on:click=on_click
        >
            {children()}
        </button>
    }
}
