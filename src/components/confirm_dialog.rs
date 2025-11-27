//! Confirmation dialog component
//!
//! A reusable confirmation dialog component for confirming actions.

use leptos::prelude::*;

/// Confirmation dialog component
#[component]
pub fn ConfirmDialog<F, G>(
    title: String,
    message: String,
    confirm_label: String,
    cancel_label: String,
    visible: ReadSignal<bool>,
    on_confirm: F,
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
