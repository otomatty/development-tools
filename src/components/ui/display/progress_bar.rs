//! Progress Bar Component
//!
//! A reusable progress bar component with various styles.
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this component):
//!   └─ src/components/ui/display/mod.rs
//!
//! Related Documentation:
//!   └─ Spec: ./display.spec.md

use leptos::prelude::*;

/// Progress bar color variants
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ProgressBarVariant {
    /// Default gradient (cyan to purple)
    #[default]
    Default,
    /// Cyan color
    Cyan,
    /// Purple color
    Purple,
    /// Gold color
    Gold,
    /// Green color (success)
    Success,
    /// Red color (danger)
    Danger,
}

impl ProgressBarVariant {
    /// Get the CSS classes for this variant
    fn classes(&self) -> &'static str {
        match self {
            ProgressBarVariant::Default => {
                "bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple"
            }
            ProgressBarVariant::Cyan => "bg-gm-accent-cyan",
            ProgressBarVariant::Purple => "bg-gm-accent-purple",
            ProgressBarVariant::Gold => "bg-gm-accent-gold",
            ProgressBarVariant::Success => "bg-green-500",
            ProgressBarVariant::Danger => "bg-red-500",
        }
    }
}

/// Progress Bar component
///
/// Displays a progress bar with customizable appearance.
///
/// # Props
///
/// - `progress`: Progress value (0.0 to 100.0)
/// - `variant`: Color variant
/// - `show_label`: Whether to show the percentage label
/// - `label_position`: Label position ("inside", "outside", "top")
/// - `height`: Height class (default: "h-3")
/// - `animated`: Whether to animate changes
/// - `class`: Additional CSS classes
///
/// # Example
///
/// ```rust
/// view! {
///     <ProgressBar
///         progress=75.0
///         variant=ProgressBarVariant::Default
///         show_label=true
///         animated=true
///     />
/// }
/// ```
#[component]
pub fn ProgressBar(
    /// Progress value (0.0 to 100.0)
    #[prop(into)]
    progress: MaybeSignal<f64>,
    /// Color variant
    #[prop(default = ProgressBarVariant::Default)]
    variant: ProgressBarVariant,
    /// Whether to show the percentage label
    #[prop(default = false)]
    show_label: bool,
    /// Label position
    #[prop(default = "outside")]
    label_position: &'static str,
    /// Height class
    #[prop(default = "h-3")]
    height: &'static str,
    /// Whether to animate changes
    #[prop(default = true)]
    animated: bool,
    /// Additional CSS classes
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let transition_class = if animated {
        "transition-all duration-500 ease-out"
    } else {
        ""
    };

    let track_class = format!(
        "w-full {} bg-gm-bg-secondary rounded-full overflow-hidden {}",
        height, class
    );

    let bar_class = format!(
        "h-full {} rounded-full {}",
        variant.classes(),
        transition_class
    );

    // Clamp progress between 0 and 100
    let clamped_progress = move || {
        let p = progress.get();
        p.clamp(0.0, 100.0)
    };

    match label_position {
        "top" => view! {
            <div class="space-y-1">
                {show_label.then(|| view! {
                    <div class="flex justify-between text-sm">
                        <span class="text-dt-text-sub">"Progress"</span>
                        <span class="text-gm-accent-cyan font-gaming-mono">
                            {move || format!("{:.0}%", clamped_progress())}
                        </span>
                    </div>
                })}
                <div class=track_class.clone()>
                    <div
                        class=bar_class.clone()
                        style=move || format!("width: {}%", clamped_progress())
                    />
                </div>
            </div>
        }
        .into_any(),

        "inside" => view! {
            <div class=format!("{} relative", track_class)>
                <div
                    class=bar_class.clone()
                    style=move || format!("width: {}%", clamped_progress())
                />
                {show_label.then(|| view! {
                    <span class="absolute inset-0 flex items-center justify-center text-xs font-bold text-white">
                        {move || format!("{:.0}%", clamped_progress())}
                    </span>
                })}
            </div>
        }
        .into_any(),

        _ => {
            // "outside" (default)
            view! {
                <div class="flex items-center gap-3">
                    <div class=format!("{} flex-1", track_class)>
                        <div
                            class=bar_class.clone()
                            style=move || format!("width: {}%", clamped_progress())
                        />
                    </div>
                    {show_label.then(|| view! {
                        <span class="text-sm text-gm-accent-cyan font-gaming-mono min-w-[3rem] text-right">
                            {move || format!("{:.0}%", clamped_progress())}
                        </span>
                    })}
                </div>
            }
            .into_any()
        }
    }
}
