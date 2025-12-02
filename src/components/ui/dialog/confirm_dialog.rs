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

use leptos::prelude::*;

/// Confirmation dialog component
///
/// A specialized modal for confirming user actions with confirm/cancel buttons.
///
/// # Example
///
/// ```rust
/// let (show_dialog, set_show_dialog) = signal(false);
///
/// view! {
///     <ConfirmDialog
///         title="Delete Item".to_string()
///         message="Are you sure you want to delete this item?".to_string()
///         confirm_label="Delete".to_string()
///         cancel_label="Cancel".to_string()
///         visible=show_dialog
///         on_confirm=move |_| {
///             // Handle confirmation
///             set_show_dialog.set(false);
///         }
///         on_cancel=move |_| set_show_dialog.set(false)
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
) -> impl IntoView
where
    F: Fn(leptos::ev::MouseEvent) + 'static + Clone + Send + Sync,
    G: Fn(leptos::ev::MouseEvent) + 'static + Clone + Send + Sync,
{
    let title = title.clone();
    let message = message.clone();
    let confirm_label = confirm_label.clone();
    let cancel_label = cancel_label.clone();

    view! {
        <Show when=move || visible.get()>
            <div
                class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
                role="dialog"
                aria-modal="true"
                aria-labelledby="confirm-dialog-title"
                aria-describedby="confirm-dialog-message"
            >
                <div
                    id="confirm-dialog"
                    class="bg-gm-bg-card rounded-2xl border border-gm-accent-cyan/20 shadow-lg p-6 max-w-md w-full mx-4"
                >
                    <h3
                        id="confirm-dialog-title"
                        class="text-xl font-gaming font-bold text-white mb-4"
                    >
                        {title.clone()}
                    </h3>
                    <p
                        id="confirm-dialog-message"
                        class="text-dt-text-sub mb-6"
                    >
                        {message.clone()}
                    </p>
                    <div class="flex gap-3 justify-end">
                        <button
                            class="px-4 py-2 rounded-lg bg-slate-700 hover:bg-slate-600 text-white transition-colors"
                            on:click=on_cancel.clone()
                        >
                            {cancel_label.clone()}
                        </button>
                        <button
                            class="px-4 py-2 rounded-lg bg-gm-error hover:bg-red-600 text-white transition-colors"
                            on:click=on_confirm.clone()
                        >
                            {confirm_label.clone()}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
