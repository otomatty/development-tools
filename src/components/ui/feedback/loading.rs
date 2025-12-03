//! Loading component
//!
//! A loading spinner/indicator component with optional text.
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this component):
//!   └─ src/components/ui/feedback/mod.rs
//! Related Documentation:
//!   ├─ Spec: ./feedback.spec.md
//!   └─ Issue: #115 Phase 2 フォーム・フィードバックコンポーネントの移動

use leptos::prelude::*;

/// Loading size
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum LoadingSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl LoadingSize {
    fn spinner_class(&self) -> &'static str {
        match self {
            LoadingSize::Small => "w-4 h-4",
            LoadingSize::Medium => "w-8 h-8",
            LoadingSize::Large => "w-12 h-12",
        }
    }

    fn text_class(&self) -> &'static str {
        match self {
            LoadingSize::Small => "text-xs",
            LoadingSize::Medium => "text-sm",
            LoadingSize::Large => "text-base",
        }
    }
}

/// Loading spinner component
///
/// ## Usage
///
/// ```rust
/// use crate::components::ui::feedback::{Loading, LoadingSize};
///
/// view! {
///     <Loading size=LoadingSize::Medium text="読み込み中..." />
/// }
/// ```
#[component]
pub fn Loading(
    /// Size of the loading spinner
    #[prop(default = LoadingSize::Medium)]
    size: LoadingSize,
    /// Optional text to display below the spinner
    #[prop(optional)]
    text: Option<&'static str>,
) -> impl IntoView {
    let spinner_size = size.spinner_class();
    let text_size = size.text_class();

    view! {
        <div class="flex flex-col items-center justify-center gap-2">
            // Spinner
            <div
                class=format!(
                    "{} border-2 border-gm-accent-cyan/30 border-t-gm-accent-cyan rounded-full animate-spin",
                    spinner_size
                )
                role="status"
                aria-label="Loading"
            ></div>

            // Optional text
            {text.map(|t| view! {
                <span class=format!("{} text-dt-text-sub", text_size)>
                    {t}
                </span>
            })}
        </div>
    }
}

/// Inline loading indicator (small, inline with text)
#[component]
pub fn InlineLoading(
    /// Optional text to display next to the spinner
    #[prop(optional)]
    text: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div class="inline-flex items-center gap-2">
            <div
                class="w-4 h-4 border-2 border-gm-accent-cyan/30 border-t-gm-accent-cyan rounded-full animate-spin"
                role="status"
                aria-label="Loading"
            ></div>
            {text.map(|t| view! {
                <span class="text-sm text-dt-text-sub">{t}</span>
            })}
        </div>
    }
}

/// Full page loading overlay
#[component]
pub fn LoadingOverlay(
    /// Whether the overlay is visible
    visible: impl Fn() -> bool + Send + Sync + 'static,
    /// Optional text to display
    #[prop(optional)]
    text: Option<&'static str>,
) -> impl IntoView {
    view! {
        <Show when=visible>
            <div class="fixed inset-0 z-50 flex items-center justify-center bg-gm-bg-primary/80 backdrop-blur-sm">
                {text.map_or_else(
                    || view! { <Loading size=LoadingSize::Large /> }.into_any(),
                    |t| view! { <Loading size=LoadingSize::Large text=t /> }.into_any()
                )}
            </div>
        </Show>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TC-007: Loading 基本レンダリング
    #[test]
    fn test_loading_size_medium_class() {
        let size = LoadingSize::Medium;
        assert_eq!(size.spinner_class(), "w-8 h-8");
    }

    // TC-009: Loading サイズ small
    #[test]
    fn test_loading_size_small_class() {
        let size = LoadingSize::Small;
        assert_eq!(size.spinner_class(), "w-4 h-4");
    }

    #[test]
    fn test_loading_size_large_class() {
        let size = LoadingSize::Large;
        assert_eq!(size.spinner_class(), "w-12 h-12");
    }

    #[test]
    fn test_loading_size_default() {
        let size = LoadingSize::default();
        assert_eq!(size, LoadingSize::Medium);
    }
}
