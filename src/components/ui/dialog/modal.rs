//! Modal Component
//!
//! A reusable modal component with overlay, ESC key support, and customizable content.
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this component):
//!   └─ src/components/ui/dialog/mod.rs
//!
//! Related Documentation:
//!   └─ Issue: GitHub Issue #114

use leptos::ev;
use leptos::prelude::*;

use crate::components::icons::Icon;

/// Modal size variants
#[derive(Clone, Copy, PartialEq, Eq, Default)]
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
    fn class(&self) -> &'static str {
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
/// A flexible modal dialog with overlay background, ESC key support,
/// and optional click-outside-to-close behavior.
///
/// # Props
///
/// - `visible`: Signal controlling modal visibility
/// - `on_close`: Callback when modal should close
/// - `children`: Modal content
/// - `title`: Optional modal title
/// - `size`: Modal size variant
/// - `close_on_overlay`: Whether clicking overlay closes modal (default: true)
/// - `close_on_escape`: Whether ESC key closes modal (default: true)
/// - `show_close_button`: Whether to show close button in header (default: true)
///
/// # Example
///
/// ```rust
/// let (visible, set_visible) = signal(false);
///
/// view! {
///     <Modal
///         visible=visible
///         on_close=move || set_visible.set(false)
///         title="My Modal".to_string()
///         size=ModalSize::Large
///     >
///         <p>"Modal content here"</p>
///     </Modal>
/// }
/// ```
#[component]
pub fn Modal<F>(
    /// Signal controlling modal visibility
    visible: ReadSignal<bool>,
    /// Callback when modal should close
    on_close: F,
    /// Modal content
    children: ChildrenFn,
    /// Optional modal title
    #[prop(optional)]
    title: Option<String>,
    /// Modal size variant
    #[prop(default = ModalSize::Medium)]
    size: ModalSize,
    /// Whether clicking overlay closes modal
    #[prop(default = true)]
    close_on_overlay: bool,
    /// Whether ESC key closes modal
    #[prop(default = true)]
    close_on_escape: bool,
    /// Whether to show close button in header
    #[prop(default = true)]
    show_close_button: bool,
) -> impl IntoView
where
    F: Fn() + 'static + Clone + Send + Sync,
{
    let on_close_for_overlay = on_close.clone();
    let on_close_for_button = on_close.clone();
    let on_close_for_key = on_close.clone();

    // Handle ESC key
    if close_on_escape {
        let _ = window_event_listener(ev::keydown, move |ev| {
            if visible.get_untracked() && ev.key() == "Escape" {
                on_close_for_key();
            }
        });
    }

    // Handle overlay click
    let handle_overlay_click = move |_: ev::MouseEvent| {
        if close_on_overlay {
            on_close_for_overlay();
        }
    };

    // Prevent click propagation on modal content
    let stop_propagation = move |ev: ev::MouseEvent| {
        ev.stop_propagation();
    };

    let size_class = size.class();
    let title_clone = title.clone();
    let has_header = title.is_some() || show_close_button;

    view! {
        <Show when=move || visible.get()>
            <div
                class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
                role="dialog"
                aria-modal="true"
                aria-labelledby=title_clone.as_ref().map(|_| "modal-title")
                on:click=handle_overlay_click
            >
                <div
                    class=format!(
                        "bg-dt-card border border-slate-700/50 rounded-lg w-full {} mx-4 shadow-xl",
                        size_class
                    )
                    on:click=stop_propagation
                >
                    // Header (if title is provided or show_close_button is true)
                    {
                        let title_for_header = title.clone();
                        let on_close_btn = on_close_for_button.clone();

                        has_header.then(|| view! {
                            <div class="p-4 border-b border-slate-700/50 flex items-center justify-between">
                                {title_for_header.map(|t| view! {
                                    <h2 id="modal-title" class="text-lg font-semibold text-dt-text">
                                        {t}
                                    </h2>
                                })}
                                {show_close_button.then(|| view! {
                                    <button
                                        class="p-1.5 text-dt-text-sub hover:text-dt-text hover:bg-slate-800 rounded-lg transition-colors ml-auto"
                                        on:click=move |_| on_close_btn()
                                        aria-label="Close modal"
                                    >
                                        <Icon name="x".to_string() class="w-5 h-5".to_string() />
                                    </button>
                                })}
                            </div>
                        })
                    }

                    // Content
                    <div class="p-4">
                        {children()}
                    </div>
                </div>
            </div>
        </Show>
    }
}

/// Modal header component for custom headers
#[component]
pub fn ModalHeader(children: Children) -> impl IntoView {
    view! {
        <div class="p-4 border-b border-slate-700/50 flex items-center justify-between">
            {children()}
        </div>
    }
}

/// Modal body component for scrollable content
#[component]
pub fn ModalBody(children: Children, #[prop(default = "")] class: &'static str) -> impl IntoView {
    view! {
        <div class=format!("p-4 overflow-y-auto {}", class)>
            {children()}
        </div>
    }
}

/// Modal footer component for action buttons
#[component]
pub fn ModalFooter(children: Children) -> impl IntoView {
    view! {
        <div class="p-4 border-t border-slate-700/50 flex items-center justify-end gap-3">
            {children()}
        </div>
    }
}
