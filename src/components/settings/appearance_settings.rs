//! Appearance settings component
//!
//! Allows users to configure animation effects ON/OFF.

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::components::use_animation_context;
use crate::tauri_api;
use crate::types::{UpdateSettingsRequest, UserSettings};

/// Appearance settings component
#[component]
pub fn AppearanceSettings() -> impl IntoView {
    let (settings, set_settings) = signal(Option::<UserSettings>::None);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    // Track initial load to avoid triggering auto-save on first load
    let (initial_load_complete, set_initial_load_complete) = signal(false);

    // Store timeout handle for cleanup
    let (debounce_handle, set_debounce_handle) = signal(Option::<i32>::None);

    // Load settings on mount (only once)
    Effect::new(move |_| {
        // Only load once
        if initial_load_complete.get() {
            return;
        }
        
        spawn_local(async move {
            match tauri_api::get_settings().await {
                Ok(loaded_settings) => {
                    set_settings.set(Some(loaded_settings));
                }
                Err(e) => {
                    set_error.set(Some(format!("設定の読み込みに失敗しました: {}", e)));
                }
            }
            set_loading.set(false);
            set_initial_load_complete.set(true);
        });
    });

    // Get animation context to update global state
    let animation_context = use_animation_context();

    // Toggle animations
    let toggle_animations = move |_| {
        if let Some(mut current_settings) = settings.get() {
            current_settings.animations_enabled = !current_settings.animations_enabled;
            
            // Update global animation context immediately
            if let Some(ctx) = animation_context {
                ctx.set_enabled.set(current_settings.animations_enabled);
            }
            
            set_settings.set(Some(current_settings));
        }
    };

    // Helper to clear a timeout handle
    let clear_timeout = move || {
        if let Some(id) = debounce_handle.get() {
            if let Some(window) = web_sys::window() {
                window.clear_timeout_with_handle(id);
            }
            set_debounce_handle.set(None);
        }
    };

    // Auto-save when settings change with debouncing
    Effect::new(move |_| {
        let current_settings = settings.get();
        let is_loading = loading.get();
        let is_initial_load_complete = initial_load_complete.get();

        // Skip if settings are not loaded or initial load is not complete
        if current_settings.is_none() || is_loading || !is_initial_load_complete {
            return;
        }

        // Capture settings value for closure
        let settings_to_save = current_settings.unwrap();

        // Clear previous timeout if exists
        clear_timeout();

        // Debounce: save after 500ms of no changes
        if let Some(window) = web_sys::window() {
            let closure = wasm_bindgen::closure::Closure::once(move || {
                let update_request = UpdateSettingsRequest::from(&settings_to_save);
                spawn_local(async move {
                    match tauri_api::update_settings(&update_request).await {
                        Ok(_) => {
                            web_sys::console::log_1(&"Appearance settings saved successfully".into());
                        }
                        Err(e) => {
                            web_sys::console::error_1(
                                &format!("Failed to save appearance settings: {}", e).into(),
                            );
                            set_error.set(Some(format!("設定の保存に失敗しました: {}", e)));
                        }
                    }
                });
                set_debounce_handle.set(None);
            });
            if let Ok(id) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                closure
                    .as_ref()
                    .dyn_ref::<js_sys::Function>()
                    .expect("Closure should be a function"),
                500,
            ) {
                set_debounce_handle.set(Some(id));
            }
            closure.forget();
        }
    });

    // Cleanup timeout on component unmount
    on_cleanup(move || {
        clear_timeout();
    });

    view! {
        <div class="space-y-6">
            // Loading state
            <Show when=move || loading.get()>
                <div class="text-center py-8 text-dt-text-sub">
                    "設定を読み込み中..."
                </div>
            </Show>

            // Error message
            <Show when=move || error.get().is_some()>
                <div class="p-3 bg-red-900/30 border border-red-500/50 rounded-lg text-red-200 text-sm">
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            // Settings form
            <Show when=move || settings.get().is_some() && !loading.get()>
                {move || {
                    let current_settings = settings.get().unwrap();

                    view! {
                        // Animation toggle
                        <div class="space-y-3">
                            <div class="p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
                                <div class="flex items-center justify-between">
                                    <div class="flex-1">
                                        <span class="text-white block font-gaming font-bold" id="animations-label">
                                            "アニメーション効果"
                                        </span>
                                        <span class="text-sm text-dt-text-sub mt-1 block">
                                            "XP獲得、レベルアップ、バッジ獲得時の"<br/>
                                            "アニメーション効果を有効にする"
                                        </span>
                                    </div>
                                    <button
                                        class=move || format!(
                                            "relative w-12 h-6 rounded-full transition-colors duration-200 {}",
                                            if current_settings.animations_enabled {
                                                "bg-gm-accent-cyan"
                                            } else {
                                                "bg-slate-600"
                                            }
                                        )
                                        role="switch"
                                        aria-checked=move || current_settings.animations_enabled.to_string()
                                        aria-labelledby="animations-label"
                                        on:click=toggle_animations
                                    >
                                        <span
                                            class=move || format!(
                                                "absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform duration-200 {}",
                                                if current_settings.animations_enabled { "translate-x-6" } else { "translate-x-0" }
                                            )
                                        ></span>
                                    </button>
                                </div>
                            </div>

                            // Hint text
                            <div class="text-xs text-dt-text-sub p-3 bg-gm-bg-card/30 rounded-lg">
                                "※ OFFにするとパフォーマンスが向上する場合があります"
                            </div>

                            // Animation preview (when enabled)
                            <Show when=move || current_settings.animations_enabled>
                                <div class="p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
                                    <h4 class="text-sm font-gaming font-bold text-white mb-3">
                                        "プレビュー"
                                    </h4>
                                    <div class="flex items-center justify-center gap-4">
                                        <div class="text-3xl animate-bounce">"✨"</div>
                                        <div class="text-3xl animate-pulse">"🔥"</div>
                                        <div class="text-3xl animate-bounce-slow">"🏆"</div>
                                    </div>
                                </div>
                            </Show>
                        </div>
                    }
                }}
            </Show>
        </div>
    }
}

