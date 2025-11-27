//! Sync settings component
//!
//! Allows users to configure sync intervals, background sync, and startup sync options.

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::tauri_api;
use crate::types::{UpdateSettingsRequest, UserSettings, SyncIntervalOption};

/// Sync settings component
#[component]
pub fn SyncSettings() -> impl IntoView {
    let (settings, set_settings) = signal(Option::<UserSettings>::None);
    let (sync_intervals, set_sync_intervals) = signal(Vec::<SyncIntervalOption>::new());
    let (loading, set_loading) = signal(true);
    let (syncing, set_syncing) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (success_message, set_success_message) = signal(None::<String>);
    let (last_sync_time, set_last_sync_time) = signal(None::<String>);

    // Load settings and sync intervals on mount
    Effect::new(move |_| {
        set_loading.set(true);
        set_error.set(None);
        set_success_message.set(None);

        spawn_local(async move {
            // Load sync intervals from backend
            match tauri_api::get_sync_intervals().await {
                Ok(intervals) => {
                    set_sync_intervals.set(intervals);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to load sync intervals: {}", e).into());
                    // Use fallback intervals
                    set_sync_intervals.set(vec![
                        SyncIntervalOption { value: 5, label: "5分".to_string() },
                        SyncIntervalOption { value: 15, label: "15分".to_string() },
                        SyncIntervalOption { value: 30, label: "30分".to_string() },
                        SyncIntervalOption { value: 60, label: "1時間".to_string() },
                        SyncIntervalOption { value: 180, label: "3時間".to_string() },
                        SyncIntervalOption { value: 0, label: "手動のみ".to_string() },
                    ]);
                }
            }

            // Load user settings
            match tauri_api::get_settings().await {
                Ok(loaded_settings) => {
                    set_settings.set(Some(loaded_settings));
                }
                Err(e) => {
                    set_error.set(Some(format!("設定の読み込みに失敗しました: {}", e)));
                }
            }
            set_loading.set(false);
        });
    });

    // Update sync interval
    let update_sync_interval = move |interval: i32| {
        if let Some(mut current_settings) = settings.get() {
            current_settings.sync_interval_minutes = interval;
            set_settings.set(Some(current_settings));
        }
    };

    // Toggle background sync
    let toggle_background_sync = move |_| {
        if let Some(mut current_settings) = settings.get() {
            current_settings.background_sync = !current_settings.background_sync;
            set_settings.set(Some(current_settings));
        }
    };

    // Toggle sync on startup
    let toggle_sync_on_startup = move |_| {
        if let Some(mut current_settings) = settings.get() {
            current_settings.sync_on_startup = !current_settings.sync_on_startup;
            set_settings.set(Some(current_settings));
        }
    };

    // Manual sync
    let on_manual_sync = move |_| {
        set_syncing.set(true);
        set_error.set(None);
        set_success_message.set(None);

        spawn_local(async move {
            match tauri_api::sync_github_stats().await {
                Ok(sync_result) => {
                    // Update last sync time
                    let now = js_sys::Date::new_0();
                    let time_str = format!(
                        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                        now.get_full_year(),
                        now.get_month() + 1,
                        now.get_date(),
                        now.get_hours(),
                        now.get_minutes(),
                        now.get_seconds()
                    );
                    set_last_sync_time.set(Some(time_str));
                    
                    let xp_msg = if sync_result.xp_gained > 0 {
                        format!(" (+{} XP)", sync_result.xp_gained)
                    } else {
                        String::new()
                    };
                    set_success_message.set(Some(format!("同期が完了しました{}", xp_msg)));
                    
                    // Auto-hide success message after 3 seconds
                    if let Some(window) = web_sys::window() {
                        let closure = wasm_bindgen::closure::Closure::once(move || {
                            set_success_message.set(None);
                        });
                        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                            closure.as_ref().dyn_ref::<js_sys::Function>().expect("Closure should be a function"),
                            3000,
                        );
                        closure.forget();
                    }
                }
                Err(e) => {
                    set_error.set(Some(format!("同期に失敗しました: {}", e)));
                }
            }
            set_syncing.set(false);
        });
    };

    // Auto-save when settings change with debouncing
    let (timeout_id, set_timeout_id) = signal(None::<i32>);
    Effect::new(move |_| {
        let current_settings = settings.get();
        if current_settings.is_some() && !loading.get() {
            // Clear previous timeout if exists
            if let Some(id) = timeout_id.get() {
                if let Some(window) = web_sys::window() {
                    let _ = window.clear_timeout_with_handle(id);
                }
            }
            
            // Debounce: save after 500ms of no changes
            let set_timeout_id_clone = set_timeout_id.clone();
            let closure = wasm_bindgen::closure::Closure::once(move || {
                if let Some(current_settings) = settings.get() {
                    let update_request = UpdateSettingsRequest::from(&current_settings);
                    let set_error = set_error.clone();
                    spawn_local(async move {
                        match tauri_api::update_settings(&update_request).await {
                            Ok(_) => {
                                // Success - settings saved silently
                            }
                            Err(e) => {
                                web_sys::console::error_1(&format!("Failed to save settings: {}", e).into());
                                set_error.set(Some(format!("設定の保存に失敗しました: {}", e)));
                            }
                        }
                    });
                }
                set_timeout_id_clone.set(None);
            });
            if let Some(window) = web_sys::window() {
                let id = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().dyn_ref::<js_sys::Function>().expect("Closure should be a function"),
                    500,
                ).expect("Failed to set timeout");
                set_timeout_id.set(Some(id));
                closure.forget();
            }
        }
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
                        let intervals = sync_intervals.get();

                        view! {
                            // Sync interval selection
                            <div class="space-y-3">
                                <h3 class="text-lg font-gaming font-bold text-white">
                                    "自動同期間隔"
                                </h3>
                                <div class="p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
                                    <select
                                        class="w-full px-4 py-3 bg-gm-bg-primary border border-gm-accent-cyan/30 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-gm-accent-cyan/50 focus:border-gm-accent-cyan cursor-pointer appearance-none"
                                        style="background-image: url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 20 20'%3E%3Cpath stroke='%2306b6d4' stroke-linecap='round' stroke-linejoin='round' stroke-width='1.5' d='M6 8l4 4 4-4'/%3E%3C/svg%3E\"); background-position: right 0.75rem center; background-repeat: no-repeat; background-size: 1.5em 1.5em; padding-right: 2.5rem;"
                                        on:change=move |ev| {
                                            let value: i32 = event_target_value(&ev).parse().unwrap_or(60);
                                            update_sync_interval(value);
                                        }
                                    >
                                        {intervals.iter().map(|interval| {
                                            let value = interval.value;
                                            let label = interval.label.clone();
                                            let is_selected = current_settings.sync_interval_minutes == value;
                                            view! {
                                                <option
                                                    value=value.to_string()
                                                    selected=is_selected
                                                >
                                                    {label}
                                                </option>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </select>
                                    <p class="mt-2 text-sm text-dt-text-sub">
                                        {move || {
                                            if current_settings.sync_interval_minutes == 0 {
                                                "自動同期は無効です。手動で同期を実行してください。"
                                            } else {
                                                "GitHubの統計情報を自動的に取得する間隔を設定します。"
                                            }
                                        }}
                                    </p>
                                </div>
                            </div>

                            // Divider
                            <div class="border-t border-gm-accent-cyan/20"></div>

                            // Toggle settings
                            <div class="space-y-3">
                                <h3 class="text-lg font-gaming font-bold text-white">
                                    "同期オプション"
                                </h3>
                                <div class="space-y-2 p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
                                    // Background sync toggle
                                    <div class="flex items-center justify-between p-3 rounded-lg hover:bg-gm-bg-card/30 transition-colors">
                                        <div class="flex-1">
                                            <span class="text-white block">
                                                "バックグラウンド同期"
                                            </span>
                                            <span class="text-sm text-dt-text-sub">
                                                "アプリがバックグラウンドにある時も同期を続ける"
                                            </span>
                                        </div>
                                        <button
                                            class=move || format!(
                                                "relative w-12 h-6 rounded-full transition-colors duration-200 {}",
                                                if current_settings.background_sync {
                                                    "bg-gm-accent-cyan"
                                                } else {
                                                    "bg-slate-600"
                                                }
                                            )
                                            on:click=toggle_background_sync
                                        >
                                            <span
                                                class=move || format!(
                                                    "absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform duration-200 {}",
                                                    if current_settings.background_sync { "translate-x-6" } else { "translate-x-0" }
                                                )
                                            ></span>
                                        </button>
                                    </div>

                                    // Sync on startup toggle
                                    <div class="flex items-center justify-between p-3 rounded-lg hover:bg-gm-bg-card/30 transition-colors">
                                        <div class="flex-1">
                                            <span class="text-white block">
                                                "起動時に同期"
                                            </span>
                                            <span class="text-sm text-dt-text-sub">
                                                "アプリ起動時に自動的に同期を実行する"
                                            </span>
                                        </div>
                                        <button
                                            class=move || format!(
                                                "relative w-12 h-6 rounded-full transition-colors duration-200 {}",
                                                if current_settings.sync_on_startup {
                                                    "bg-gm-accent-cyan"
                                                } else {
                                                    "bg-slate-600"
                                                }
                                            )
                                            on:click=toggle_sync_on_startup
                                        >
                                            <span
                                                class=move || format!(
                                                    "absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform duration-200 {}",
                                                    if current_settings.sync_on_startup { "translate-x-6" } else { "translate-x-0" }
                                                )
                                            ></span>
                                        </button>
                                    </div>
                                </div>
                            </div>

                            // Divider
                            <div class="border-t border-gm-accent-cyan/20"></div>

                            // Manual sync section
                            <div class="space-y-3">
                                <h3 class="text-lg font-gaming font-bold text-white">
                                    "手動同期"
                                </h3>
                                <div class="p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
                                    // Last sync time
                                    <Show when=move || last_sync_time.get().is_some()>
                                        <div class="mb-4 text-sm text-dt-text-sub">
                                            <span class="font-medium">"最終同期: "</span>
                                            <span>{move || last_sync_time.get().unwrap_or_default()}</span>
                                        </div>
                                    </Show>

                                    // Sync button
                                    <button
                                        class="w-full px-4 py-3 bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple rounded-lg text-white font-gaming font-bold hover:opacity-90 transition-opacity disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                                        on:click=on_manual_sync
                                        disabled=move || syncing.get()
                                    >
                                        <span class=move || if syncing.get() { "animate-spin" } else { "" }>
                                            "🔄"
                                        </span>
                                        {move || if syncing.get() { "同期中..." } else { "今すぐ同期" }}
                                    </button>
                                    <p class="mt-2 text-sm text-dt-text-sub text-center">
                                        "GitHubの統計情報を今すぐ取得します"
                                    </p>
                                </div>
                            </div>
                        }
                    }
                }
            </Show>
        </div>
    }
}

