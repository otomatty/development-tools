//! Sync settings component
//!
//! Allows users to configure sync intervals, background sync, and startup sync options.

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::tauri_api;
use crate::types::{UpdateSettingsRequest, UserSettings, SyncIntervalOption};

/// Helper to clear a timeout handle stored in a signal
fn clear_timeout_signal(handle_signal: ReadSignal<Option<i32>>, set_handle_signal: WriteSignal<Option<i32>>) {
    if let Some(id) = handle_signal.get() {
        if let Some(window) = web_sys::window() {
            window.clear_timeout_with_handle(id);
        }
        set_handle_signal.set(None);
    }
}

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
    
    // Track initial load to avoid triggering auto-save on first load
    let (initial_load_complete, set_initial_load_complete) = signal(false);
    
    // Store timeout handles for cleanup (using signals for timeout IDs)
    let (debounce_handle, set_debounce_handle) = signal(Option::<i32>::None);
    let (success_msg_handle, set_success_msg_handle) = signal(Option::<i32>::None);

    // Load settings and sync intervals on mount (spawn_local, not Effect)
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
        set_initial_load_complete.set(true);
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
        
        // Clear any existing success message timeout
        clear_timeout_signal(success_msg_handle, set_success_msg_handle);

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
                            set_success_msg_handle.set(None);
                        });
                        if let Ok(id) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                            closure.as_ref().dyn_ref::<js_sys::Function>().expect("Closure should be a function"),
                            3000,
                        ) {
                            set_success_msg_handle.set(Some(id));
                        }
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
    Effect::new(move |_| {
        // Capture current settings value BEFORE creating the closure to avoid stale value
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
        clear_timeout_signal(debounce_handle, set_debounce_handle);
        
        // Debounce: save after 500ms of no changes
        if let Some(window) = web_sys::window() {
            let closure = wasm_bindgen::closure::Closure::once(move || {
                let update_request = UpdateSettingsRequest::from(&settings_to_save);
                spawn_local(async move {
                    match tauri_api::update_settings(&update_request).await {
                        Ok(_) => {
                            web_sys::console::log_1(&"Settings saved successfully".into());
                        }
                        Err(e) => {
                            web_sys::console::error_1(&format!("Failed to save settings: {}", e).into());
                            set_error.set(Some(format!("設定の保存に失敗しました: {}", e)));
                        }
                    }
                });
                set_debounce_handle.set(None);
            });
            if let Ok(id) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().dyn_ref::<js_sys::Function>().expect("Closure should be a function"),
                500,
            ) {
                set_debounce_handle.set(Some(id));
            }
            closure.forget();
        }
    });
    
    // Cleanup timeouts on component unmount
    on_cleanup(move || {
        clear_timeout_signal(debounce_handle, set_debounce_handle);
        clear_timeout_signal(success_msg_handle, set_success_msg_handle);
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
                                <h3 class="text-lg font-gaming font-bold text-white" id="sync-interval-label">
                                    "自動同期間隔"
                                </h3>
                                <div class="p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
                                    <select
                                        class="w-full px-4 py-3 bg-gm-bg-primary border border-gm-accent-cyan/30 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-gm-accent-cyan/50 focus:border-gm-accent-cyan cursor-pointer appearance-none"
                                        style="background-image: url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 20 20'%3E%3Cpath stroke='%2306b6d4' stroke-linecap='round' stroke-linejoin='round' stroke-width='1.5' d='M6 8l4 4 4-4'/%3E%3C/svg%3E\"); background-position: right 0.75rem center; background-repeat: no-repeat; background-size: 1.5em 1.5em; padding-right: 2.5rem;"
                                        aria-labelledby="sync-interval-label"
                                        on:change=move |ev| {
                                            match event_target_value(&ev).parse::<i32>() {
                                                Ok(value) => update_sync_interval(value),
                                                Err(e) => {
                                                    web_sys::console::error_1(&format!("Failed to parse interval value: {}", e).into());
                                                    // Keep current setting unchanged
                                                }
                                            }
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
                                            <span class="text-white block" id="background-sync-label">
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
                                            role="switch"
                                            aria-checked=move || current_settings.background_sync.to_string()
                                            aria-labelledby="background-sync-label"
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
                                            <span class="text-white block" id="sync-on-startup-label">
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
                                            role="switch"
                                            aria-checked=move || current_settings.sync_on_startup.to_string()
                                            aria-labelledby="sync-on-startup-label"
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
                                        aria-busy=move || syncing.get().to_string()
                                        aria-label="GitHubの統計情報を今すぐ同期"
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

