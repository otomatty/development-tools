//! Modal Component
//!
//! A reusable modal component with overlay, ESC key support, and customizable content.
//! This is a pure container component - use ModalHeader, ModalBody, and ModalFooter
//! for structuring modal content.
//!
//! Features:
//! - Portal rendering to body element (avoids parent layout issues)
//! - AnimationContext support for fade-in/scale-in animations
//! - Customizable border color
//! - ESC key and overlay click to close
//! - Multiple size variants
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this component):
//!   └─ src/components/ui/dialog/mod.rs
//!
//! Dependencies:
//!   └─ src/components/animation_context.rs
//!
//! Related Documentation:
//!   └─ Issue: GitHub Issue #114

use leptos::ev;
use leptos::portal::Portal;
use leptos::prelude::*;

use crate::components::use_animation_context_or_default;

/// Modal size variants
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ModalSize {
    /// Small modal (max-w-sm)
    Small,
    /// Medium modal (max-w-md) - default
    #[default]
    Medium,
    /// Large modal (max-w-lg)
    Large,
    /// Extra large modal (max-w-xl)
    XLarge,
    /// 2XL modal (max-w-2xl)
    TwoXL,
    /// Full width modal (max-w-4xl)
    Full,
}

impl ModalSize {
    /// Get the CSS class for this size variant
    pub fn class(&self) -> &'static str {
        match self {
            ModalSize::Small => "max-w-sm",
            ModalSize::Medium => "max-w-md",
            ModalSize::Large => "max-w-lg",
            ModalSize::XLarge => "max-w-xl",
            ModalSize::TwoXL => "max-w-2xl",
            ModalSize::Full => "max-w-4xl",
        }
    }
}

/// Modal component
///
/// A pure container modal dialog with overlay background, ESC key support,
/// and optional click-outside-to-close behavior. Use ModalHeader, ModalBody,
/// and ModalFooter components to structure the modal content.
///
/// Features:
/// - AnimationContext integration for fade-in/scale-in animations
/// - Customizable border color via `border_class` prop
/// - Multiple size variants
/// - ESC key and overlay click handling
///
/// # Props
///
/// - `visible`: Signal controlling modal visibility
/// - `on_close`: Callback when modal should close
/// - `children`: Modal content (use ModalHeader, ModalBody, ModalFooter)
/// - `size`: Modal size variant
/// - `border_class`: Custom border CSS class (default: "border-slate-700/50")
/// - `close_on_overlay`: Whether clicking overlay closes modal (default: true)
/// - `close_on_escape`: Whether ESC key closes modal (default: true)
///
/// # Example
///
/// ```rust
/// let visible = RwSignal::new(false);
///
/// view! {
///     <Modal
///         visible=visible.read_only()
///         on_close=move || visible.set(false)
///         size=ModalSize::Large
///         border_class="border-2 border-badge-gold".to_string()
///     >
///         <ModalHeader on_close=move || visible.set(false)>
///             <h2 class="text-lg font-semibold">"My Modal"</h2>
///         </ModalHeader>
///         <ModalBody>
///             <p>"Modal content here"</p>
///         </ModalBody>
///         <ModalFooter>
///             <button on:click=move |_| visible.set(false)>"Close"</button>
///         </ModalFooter>
///     </Modal>
/// }
/// ```
#[component]
pub fn Modal<F>(
    /// Signal controlling modal visibility (accepts ReadSignal, Memo, or Signal)
    #[prop(into)]
    visible: Signal<bool>,
    /// Callback when modal should close
    on_close: F,
    /// Modal content (use ModalHeader, ModalBody, ModalFooter)
    children: ChildrenFn,
    /// Modal size variant
    #[prop(default = ModalSize::Medium)]
    size: ModalSize,
    /// Custom border CSS class
    #[prop(default = "border border-slate-700/50".to_string())]
    border_class: String,
    /// Whether clicking overlay closes modal
    #[prop(default = true)]
    close_on_overlay: bool,
    /// Whether ESC key closes modal
    #[prop(default = true)]
    close_on_escape: bool,
) -> impl IntoView
where
    F: Fn() + 'static + Clone + Send + Sync,
{
    let animation_ctx = use_animation_context_or_default();
    let on_close = std::sync::Arc::new(on_close);
    let on_close_for_key = on_close.clone();
    let on_close_for_overlay = on_close.clone();

    // Handle ESC key
    if close_on_escape {
        let _ = window_event_listener(ev::keydown, move |ev| {
            if visible.get_untracked() && ev.key() == "Escape" {
                on_close_for_key();
            }
        });
    }

    let size_class = size.class();
    // Store values for ChildrenFn requirements
    let on_close_stored = StoredValue::new(on_close_for_overlay);
    let border_class_stored = StoredValue::new(border_class);
    let children_stored = StoredValue::new(children);

    view! {
        // Show controls visibility, Portal renders to body when visible
        <Show when=move || visible.get()>
            <Portal>
                <div
                    class=move || format!(
                        "fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm {}",
                        animation_ctx.get_animation_class("animate-fade-in")
                    )
                    role="dialog"
                    aria-modal="true"
                    on:click={
                        let on_close_inner = on_close_stored.get_value();
                        move |_| {
                            if close_on_overlay {
                                on_close_inner();
                            }
                        }
                    }
                >
                    <div
                        class=format!(
                            "bg-dt-card {} rounded-2xl w-full {} mx-4 shadow-xl {}",
                            border_class_stored.get_value(),
                            size_class,
                            animation_ctx.get_animation_class("animate-scale-in")
                        )
                        on:click=|ev: ev::MouseEvent| ev.stop_propagation()
                    >
                        {children_stored.get_value()()}
                    </div>
                </div>
            </Portal>
        </Show>
    }
}

/// Modal header component for custom headers
///
/// Provides a consistent header layout with optional close button.
///
/// # Props
///
/// - `children`: Header content (title, etc.)
/// - `on_close`: Optional callback for close button. If provided, displays "✕" button.
#[component]
pub fn ModalHeader(
    children: Children,
    /// Optional callback for close button
    #[prop(optional)]
    on_close: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="p-4 border-b border-slate-700/50 flex items-center justify-between">
            <div class="flex-1 min-w-0">
                {children()}
            </div>
            {on_close.map(|close_fn| {
                view! {
                    <button
                        class="p-1.5 text-dt-text-sub hover:text-white hover:bg-slate-800 rounded-lg transition-colors flex-shrink-0"
                        on:click=move |_| close_fn.run(())
                    >
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                        </svg>
                    </button>
                }
            })}
        </div>
    }
}

/// Modal body component for scrollable content
#[component]
pub fn ModalBody(children: Children, #[prop(default = "")] class: &'static str) -> impl IntoView {
    // Filter empty class to avoid trailing space
    let classes = ["p-4 overflow-y-auto", class]
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

/// Modal footer component for action buttons
///
/// # Props
///
/// - `children`: Footer content (action buttons, links, etc.)
#[component]
pub fn ModalFooter(children: Children) -> impl IntoView {
    view! {
        <div class="p-4 border-t border-slate-700/50 flex items-center justify-end gap-3">
            {children()}
        </div>
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modal_size_classes() {
        assert_eq!(ModalSize::Small.class(), "max-w-sm");
        assert_eq!(ModalSize::Medium.class(), "max-w-md");
        assert_eq!(ModalSize::Large.class(), "max-w-lg");
        assert_eq!(ModalSize::XLarge.class(), "max-w-xl");
        assert_eq!(ModalSize::TwoXL.class(), "max-w-2xl");
        assert_eq!(ModalSize::Full.class(), "max-w-4xl");
    }

    #[test]
    fn test_modal_size_default() {
        assert_eq!(ModalSize::default(), ModalSize::Medium);
    }
}
