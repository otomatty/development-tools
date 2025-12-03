//! Confirmation dialog component
//!
//! A reusable confirmation dialog component for confirming actions.
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this component):
//!   └─ src/components/ui/dialog/mod.rs
//!
//! Related Documentation:
//!   └─ Issue: GitHub Issue #114
//!
//! TODO: [DEBT] このコンポーネントをModalコンポーネントベースにリファクタリングする
//! 現在はオーバーレイロジックが重複しているため、Modal + ModalHeader/Body/Footer を
//! 使用した実装に置き換えることで、コードの重複を削減し一貫性を高める。
//! See: GitHub PR #119 review comments

use leptos::prelude::*;

/// Confirmation dialog component
///
/// A specialized modal for confirming user actions with confirm/cancel buttons.
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
) -> impl IntoView
where
    F: Fn(leptos::ev::MouseEvent) + 'static + Clone + Send + Sync,
    G: Fn(leptos::ev::MouseEvent) + 'static + Clone + Send + Sync,
{
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
