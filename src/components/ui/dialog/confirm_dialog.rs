//! Confirmation dialog component
//!
//! A reusable confirmation dialog component for confirming actions.
//! Built on top of the Modal component.
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this component):
//!   └─ src/components/ui/dialog/mod.rs
//!
//! Dependencies:
//!   └─ src/components/ui/dialog/modal.rs
//!
//! Related Documentation:
//!   └─ Issue: GitHub Issue #114

use super::modal::{Modal, ModalBody, ModalFooter, ModalHeader, ModalSize};
use leptos::prelude::*;

/// Confirmation dialog component
///
/// A specialized modal for confirming user actions with confirm/cancel buttons.
/// Built on top of the Modal component for consistent behavior.
///
/// # Example
///
/// ```rust
/// let visible = RwSignal::new(false);
///
/// view! {
///     <ConfirmDialog
///         title="Delete Item".to_string()
///         message="Are you sure you want to delete this item?".to_string()
///         confirm_label="Delete".to_string()
///         cancel_label="Cancel".to_string()
///         visible=visible.read_only()
///         on_confirm=move |_| {
///             // Handle confirmation
///             visible.set(false);
///         }
///         on_cancel=move |_| visible.set(false)
///     />
/// }
/// ```
#[component]
pub fn ConfirmDialog<F, G>(
    /// Dialog title
    title: String,
    /// Dialog message/description
    message: String,
    /// Label for confirm button
    confirm_label: String,
    /// Label for cancel button
    cancel_label: String,
    /// Signal controlling dialog visibility
    visible: ReadSignal<bool>,
    /// Callback when confirm button is clicked
    on_confirm: F,
    /// Callback when cancel button is clicked
    on_cancel: G,
    /// Whether to close on overlay click (default: false for confirm dialogs)
    #[prop(default = false)]
    close_on_overlay: bool,
) -> impl IntoView
where
    F: Fn(leptos::ev::MouseEvent) + 'static + Clone + Send + Sync,
    G: Fn(leptos::ev::MouseEvent) + 'static + Clone + Send + Sync,
{
    // Clone values for use in the view closure (Modal uses ChildrenFn which may call children multiple times)
    let title = StoredValue::new(title);
    let message = StoredValue::new(message);
    let confirm_label = StoredValue::new(confirm_label);
    let cancel_label = StoredValue::new(cancel_label);
    let on_confirm = StoredValue::new(on_confirm);
    let on_cancel = StoredValue::new(on_cancel);

    view! {
        <Modal
            visible=visible
            on_close=move || {
                // No-op: ConfirmDialog requires explicit confirm/cancel button clicks
            }
            size=ModalSize::Medium
            close_on_overlay=close_on_overlay
            close_on_escape=false
        >
            <ModalHeader>
                <h3
                    id="confirm-dialog-title"
                    class="text-xl font-gaming font-bold text-white"
                >
                    {title.get_value()}
                </h3>
            </ModalHeader>
            <ModalBody>
                <p
                    id="confirm-dialog-message"
                    class="text-dt-text-sub"
                >
                    {message.get_value()}
                </p>
            </ModalBody>
            <ModalFooter>
                <button
                    class="px-4 py-2 rounded-lg bg-slate-700 hover:bg-slate-600 text-white transition-colors"
                    on:click=move |ev| on_cancel.get_value()(ev)
                >
                    {cancel_label.get_value()}
                </button>
                <button
                    class="px-4 py-2 rounded-lg bg-gm-error hover:bg-red-600 text-white transition-colors"
                    on:click=move |ev| on_confirm.get_value()(ev)
                >
                    {confirm_label.get_value()}
                </button>
            </ModalFooter>
        </Modal>
    }
}
