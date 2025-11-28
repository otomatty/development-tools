//! Settings reset component
//!
//! Allows users to reset all settings to defaults.

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::tauri_api;

/// Settings reset confirmation dialog
#[component]
fn SettingsResetDialog(
    visible: ReadSignal<bool>,
    on_confirm: impl Fn() + 'static + Clone + Send + Sync,
    on_cancel: impl Fn() + 'static + Clone + Send + Sync,
) -> impl IntoView {
    view! {
        <Show when=move || visible.get()>
            <div 
                class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
                role="dialog"
                aria-modal="true"
                aria-labelledby="settings-reset-dialog-title"
            >
                <div 
                    class="bg-gm-bg-card rounded-2xl border border-gm-accent-cyan/30 shadow-lg p-6 max-w-md w-full mx-4"
                >
                    <div class="flex items-center gap-3 mb-4">
                        <span class="text-3xl">{"\u{2699}\u{FE0F}"}</span>
                        <h3 
                            id="settings-reset-dialog-title"
                            class="text-xl font-gaming font-bold text-white"
                        >
                            "設定をリセットしますか？"
                        </h3>
                    </div>
                    
                    <div class="space-y-4 mb-6">
                        <p class="text-dt-text-sub">
                            "全ての設定がデフォルト値に戻ります。"
                        </p>
                        
                        <div class="p-3 bg-gm-accent-cyan/10 border border-gm-accent-cyan/30 rounded-lg">
                            <p class="text-gm-accent-cyan text-sm">
                                {"\u{2139}\u{FE0F}"}" XP・バッジ・統計データは削除されません"
                            </p>
                        </div>
                    </div>
                    
                    <div class="flex justify-end gap-3">
                        <button
                            class="px-4 py-2 rounded-lg border border-gm-accent-cyan/30 text-white hover:bg-gm-accent-cyan/10 transition-colors"
                            on:click={
                                let on_cancel = on_cancel.clone();
                                move |_| on_cancel()
                            }
                        >
                            "キャンセル"
                        </button>
                        <button
                            class="px-4 py-2 rounded-lg bg-gm-accent-cyan text-gm-bg-dark font-bold hover:bg-gm-accent-cyan/90 transition-colors"
                            on:click={
                                let on_confirm = on_confirm.clone();
                                move |_| on_confirm()
                            }
                        >
                            "リセット"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

/// Settings reset section component
#[component]
pub fn SettingsResetSection() -> impl IntoView {
    let (show_dialog, set_show_dialog) = signal(false);
    let (resetting, set_resetting) = signal(false);
    let (success_message, set_success_message) = signal(None::<String>);
    let (error_message, set_error_message) = signal(None::<String>);

    let on_reset_click = move |_| {
        set_show_dialog.set(true);
    };

    let on_confirm = move || {
        set_show_dialog.set(false);
        set_resetting.set(true);
        set_success_message.set(None);
        set_error_message.set(None);

        spawn_local(async move {
            match tauri_api::reset_settings().await {
                Ok(_) => {
                    set_success_message.set(Some("設定をリセットしました".to_string()));
                    // Clear success message after 3 seconds
                    gloo_timers::callback::Timeout::new(3000, move || {
                        set_success_message.set(None);
                    })
                    .forget();
                }
                Err(e) => {
                    set_error_message.set(Some(format!("設定のリセットに失敗しました: {}", e)));
                }
            }
            set_resetting.set(false);
        });
    };

    let on_cancel = move || {
        set_show_dialog.set(false);
    };

    view! {
        <div class="space-y-4">
            // Success message
            <Show when=move || success_message.get().is_some()>
                <div class="p-3 bg-green-900/20 border border-green-500/30 rounded-lg">
                    <p class="text-green-300 text-sm">
                        {"\u{2705}"}" "{move || success_message.get().unwrap_or_default()}
                    </p>
                </div>
            </Show>

            // Error message
            <Show when=move || error_message.get().is_some()>
                <div class="p-3 bg-red-900/20 border border-red-500/30 rounded-lg">
                    <p class="text-red-300 text-sm">
                        {move || error_message.get().unwrap_or_default()}
                    </p>
                </div>
            </Show>

            // Reset button section
            <div class="p-4 bg-gm-bg-darker/50 rounded-xl border border-gm-accent-cyan/20">
                <div class="flex items-center justify-between">
                    <div>
                        <h4 class="text-white font-semibold">
                            "全ての設定をリセット"
                        </h4>
                        <p class="text-dt-text-sub text-sm mt-1">
                            "設定をデフォルト値に戻します（データは削除されません）"
                        </p>
                    </div>
                    <button
                        class="px-4 py-2 rounded-lg border border-gm-accent-cyan/30 text-white hover:bg-gm-accent-cyan/10 transition-colors disabled:opacity-50"
                        on:click=on_reset_click
                        disabled=move || resetting.get()
                    >
                        {move || if resetting.get() { "リセット中..." } else { "リセット" }}
                    </button>
                </div>
            </div>

            // Confirmation dialog
            <SettingsResetDialog
                visible=show_dialog
                on_confirm=on_confirm
                on_cancel=on_cancel
            />
        </div>
    }
}
