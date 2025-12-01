//! Notification settings component
//!
//! Allows users to configure notification methods and individual notification types.

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use super::toggle_switch::ToggleSwitch;
use crate::tauri_api;
use crate::types::{NotificationMethod, UpdateSettingsRequest, UserSettings};

/// Notification settings component
#[component]
pub fn NotificationSettings() -> impl IntoView {
    let (settings, set_settings) = signal(Option::<UserSettings>::None);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (success_message, set_success_message) = signal(None::<String>);

    // Track initial load to avoid triggering auto-save on first load
    let (initial_load_complete, set_initial_load_complete) = signal(false);

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

    // Update notification method
    let update_notification_method = move |method: NotificationMethod| {
        if let Some(mut current_settings) = settings.get() {
            current_settings.notification_method = method.as_str().to_string();
            set_settings.set(Some(current_settings));
        }
    };

    // Toggle individual notification setting
    let toggle_notification = move |field: &'static str| {
        if let Some(mut current_settings) = settings.get() {
            match field {
                "xp_gain" => current_settings.notify_xp_gain = !current_settings.notify_xp_gain,
                "level_up" => current_settings.notify_level_up = !current_settings.notify_level_up,
                "badge_earned" => {
                    current_settings.notify_badge_earned = !current_settings.notify_badge_earned
                }
                "streak_update" => {
                    current_settings.notify_streak_update = !current_settings.notify_streak_update
                }
                "streak_milestone" => {
                    current_settings.notify_streak_milestone =
                        !current_settings.notify_streak_milestone
                }
                _ => {}
            }
            set_settings.set(Some(current_settings));
        }
    };

    // Toggle all notifications on
    let toggle_all_on = move |_| {
        if let Some(mut current_settings) = settings.get() {
            current_settings.notify_xp_gain = true;
            current_settings.notify_level_up = true;
            current_settings.notify_badge_earned = true;
            current_settings.notify_streak_update = true;
            current_settings.notify_streak_milestone = true;
            set_settings.set(Some(current_settings));
        }
    };

    // Toggle all notifications off
    let toggle_all_off = move |_| {
        if let Some(mut current_settings) = settings.get() {
            current_settings.notify_xp_gain = false;
            current_settings.notify_level_up = false;
            current_settings.notify_badge_earned = false;
            current_settings.notify_streak_update = false;
            current_settings.notify_streak_milestone = false;
            set_settings.set(Some(current_settings));
        }
    };

    // Store timeout handle for debouncing
    let (timeout_id, set_timeout_id) = signal(None::<i32>);

    // Helper to clear timeout (uses untrack to avoid Effect dependency)
    let clear_timeout_untracked = move || {
        leptos::prelude::untrack(|| {
            if let Some(id) = timeout_id.get() {
                if let Some(window) = web_sys::window() {
                    let _ = window.clear_timeout_with_handle(id);
                }
                set_timeout_id.set(None);
            }
        });
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

        // Clear previous timeout if exists (untracked to avoid dependency loop)
        clear_timeout_untracked();

        // Debounce: save after 500ms of no changes (untracked to avoid dependency loop)
        leptos::prelude::untrack(|| {
            if let Some(window) = web_sys::window() {
                let closure = wasm_bindgen::closure::Closure::once(move || {
                    let update_request = UpdateSettingsRequest::from(&settings_to_save);
                    spawn_local(async move {
                        match tauri_api::update_settings(&update_request).await {
                            Ok(_) => {
                                web_sys::console::log_1(
                                    &"Notification settings saved successfully".into(),
                                );
                            }
                            Err(e) => {
                                web_sys::console::error_1(
                                    &format!("Failed to save settings: {}", e).into(),
                                );
                                set_error.set(Some(format!("設定の保存に失敗しました: {}", e)));
                            }
                        }
                    });
                    set_timeout_id.set(None);
                });
                if let Ok(id) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure
                        .as_ref()
                        .dyn_ref::<js_sys::Function>()
                        .expect("Closure should be a function"),
                    500,
                ) {
                    set_timeout_id.set(Some(id));
                }
                closure.forget();
            }
        });
    });

    // Cleanup timeout on component unmount
    on_cleanup(move || {
        clear_timeout_untracked();
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

            // Success message
            <Show when=move || success_message.get().is_some()>
                <div class="p-3 bg-green-900/30 border border-green-500/50 rounded-lg text-green-200 text-sm">
                    {move || success_message.get().unwrap_or_default()}
                </div>
            </Show>

            // Settings form
            <Show when=move || settings.get().is_some() && !loading.get()>
                {
                    move || {
                        let current_settings = settings.get().unwrap();
                        let current_method = NotificationMethod::from_str(&current_settings.notification_method);

                        view! {
                            // Notification method selection
                            <div class="space-y-3">
                                <h3 class="text-lg font-gaming font-bold text-white">
                                    "通知方法"
                                </h3>
                                <div class="space-y-2 p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
                                    {[
                                        (NotificationMethod::AppOnly, "アプリ内のみ"),
                                        (NotificationMethod::OsOnly, "OSネイティブのみ"),
                                        (NotificationMethod::Both, "両方"),
                                        (NotificationMethod::None, "通知なし"),
                                    ].into_iter().map(move |(method, label)| {
                                        let is_selected = current_method == method;
                                        let method_clone = method;
                                        let update_method = update_notification_method.clone();

                                        view! {
                                            <label
                                                class=move || format!(
                                                    "flex items-center gap-3 p-3 rounded-lg cursor-pointer transition-colors {}",
                                                    if is_selected {
                                                        "bg-gm-accent-cyan/20 border border-gm-accent-cyan/50"
                                                    } else {
                                                        "hover:bg-gm-bg-card/30"
                                                    }
                                                )
                                            >
                                                <input
                                                    type="radio"
                                                    name="notification_method"
                                                    checked=is_selected
                                                    on:change=move |_| update_method(method_clone)
                                                    class="w-4 h-4 text-gm-accent-cyan bg-gm-bg-card border-gm-accent-cyan/50 focus:ring-gm-accent-cyan focus:ring-2"
                                                />
                                                <span class="text-white">
                                                    {label}
                                                </span>
                                            </label>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>

                            // Divider
                            <div class="border-t border-gm-accent-cyan/20"></div>

                            // Individual notification settings
                            <div class="space-y-3">
                                <div class="flex items-center justify-between">
                                    <h3 class="text-lg font-gaming font-bold text-white">
                                        "通知の種類"
                                    </h3>
                                    <div class="flex gap-2">
                                        <button
                                            class="px-3 py-1 text-sm rounded-lg bg-gm-accent-cyan/20 hover:bg-gm-accent-cyan/30 text-gm-accent-cyan transition-colors"
                                            on:click=toggle_all_on
                                        >
                                            "全てON"
                                        </button>
                                        <button
                                            class="px-3 py-1 text-sm rounded-lg bg-slate-700/50 hover:bg-slate-700/70 text-white transition-colors"
                                            on:click=toggle_all_off
                                        >
                                            "全てOFF"
                                        </button>
                                    </div>
                                </div>
                                <div class="space-y-2 p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
                                    {[
                                        ("xp_gain", "XP獲得通知", current_settings.notify_xp_gain),
                                        ("level_up", "レベルアップ通知", current_settings.notify_level_up),
                                        ("badge_earned", "バッジ獲得通知", current_settings.notify_badge_earned),
                                        ("streak_update", "ストリーク更新通知", current_settings.notify_streak_update),
                                        ("streak_milestone", "ストリークマイルストーン", current_settings.notify_streak_milestone),
                                    ].into_iter().map(move |(field, label, enabled)| {
                                        let field_str = field;
                                        let toggle_fn = toggle_notification.clone();

                                        view! {
                                            <div class="flex items-center justify-between p-3 rounded-lg hover:bg-gm-bg-card/30 transition-colors">
                                                <span class="text-white font-gaming">
                                                    {label}
                                                </span>
                                                <ToggleSwitch
                                                    enabled=enabled
                                                    on_toggle=move || toggle_fn(field_str)
                                                />
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                        }
                    }
                }
            </Show>
        </div>
    }
}
