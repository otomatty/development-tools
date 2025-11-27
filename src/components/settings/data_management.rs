//! Data management settings component
//!
//! Allows users to manage cache, export data, and reset all data.

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::tauri_api;
use crate::types::DatabaseInfo;

/// Format bytes to human-readable string (KB, MB, GB)
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Reset confirmation dialog component with "RESET" input confirmation
#[component]
fn ResetConfirmDialog(
    visible: ReadSignal<bool>,
    on_confirm: impl Fn() + 'static + Clone + Send + Sync,
    on_cancel: impl Fn() + 'static + Clone + Send + Sync,
) -> impl IntoView {
    let (input_value, set_input_value) = signal(String::new());
    
    // Check if input matches "RESET"
    let is_confirm_enabled = Memo::new(move |_| input_value.get() == "RESET");
    
    // Clear input when dialog closes
    Effect::new(move |_| {
        if !visible.get() {
            set_input_value.set(String::new());
        }
    });
    
    view! {
        <Show when=move || visible.get()>
            <div 
                class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
                role="dialog"
                aria-modal="true"
                aria-labelledby="reset-dialog-title"
            >
                <div 
                    class="bg-gm-bg-card rounded-2xl border border-red-500/30 shadow-lg p-6 max-w-md w-full mx-4"
                >
                    <div class="flex items-center gap-3 mb-4">
                        <span class="text-3xl">"⚠️"</span>
                        <h3 
                            id="reset-dialog-title"
                            class="text-xl font-gaming font-bold text-white"
                        >
                            "データリセットの確認"
                        </h3>
                    </div>
                    
                    <div class="space-y-4 mb-6">
                        <p class="text-dt-text-sub">
                            "この操作により以下のデータが削除されます："
                        </p>
                        
                        <ul class="list-disc list-inside text-dt-text-sub space-y-1 pl-2">
                            <li>"経験値（XP）"</li>
                            <li>"レベル"</li>
                            <li>"バッジ"</li>
                            <li>"ストリーク記録"</li>
                            <li>"チャレンジ履歴"</li>
                            <li>"キャッシュデータ"</li>
                        </ul>
                        
                        <div class="p-3 bg-red-900/20 border border-red-500/30 rounded-lg">
                            <p class="text-red-200 text-sm font-bold">
                                "⚠️ この操作は取り消せません"
                            </p>
                        </div>
                        
                        <div class="space-y-2">
                            <label for="reset-confirm-input" class="text-white text-sm">
                                "続行するには「RESET」と入力してください："
                            </label>
                            <input
                                id="reset-confirm-input"
                                type="text"
                                class="w-full px-4 py-3 bg-gm-bg-primary border border-gm-accent-cyan/30 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-red-500/50 focus:border-red-500 placeholder-gray-500"
                                placeholder="RESET"
                                prop:value=move || input_value.get()
                                on:input=move |ev| set_input_value.set(event_target_value(&ev))
                                autocomplete="off"
                                spellcheck="false"
                            />
                        </div>
                    </div>
                    
                    <div class="flex gap-3 justify-end">
                        <button
                            class="px-4 py-2 rounded-lg bg-slate-700 hover:bg-slate-600 text-white transition-colors"
                            on:click={
                                let on_cancel = on_cancel.clone();
                                move |_| on_cancel()
                            }
                        >
                            "キャンセル"
                        </button>
                        <button
                            class=move || format!(
                                "px-4 py-2 rounded-lg text-white transition-colors {}",
                                if is_confirm_enabled.get() {
                                    "bg-red-600 hover:bg-red-500 cursor-pointer"
                                } else {
                                    "bg-red-900/50 cursor-not-allowed opacity-50"
                                }
                            )
                            disabled=move || !is_confirm_enabled.get()
                            on:click={
                                let on_confirm = on_confirm.clone();
                                move |_| {
                                    if is_confirm_enabled.get() {
                                        on_confirm()
                                    }
                                }
                            }
                        >
                            "リセットを実行"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

/// Data management settings component
#[component]
pub fn DataManagement() -> impl IntoView {
    let (db_info, set_db_info) = signal(Option::<DatabaseInfo>::None);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (success_message, set_success_message) = signal(None::<String>);
    let (clearing_cache, set_clearing_cache) = signal(false);
    let (exporting, set_exporting) = signal(false);
    let (resetting, set_resetting) = signal(false);
    let (show_reset_dialog, set_show_reset_dialog) = signal(false);
    
    // Store timeout handle for success message cleanup
    let (success_msg_handle, set_success_msg_handle) = signal(Option::<i32>::None);
    
    // Helper to clear success message timeout
    let clear_success_timeout = move || {
        if let Some(id) = success_msg_handle.get() {
            if let Some(window) = web_sys::window() {
                window.clear_timeout_with_handle(id);
            }
            set_success_msg_handle.set(None);
        }
    };
    
    // Helper to show success message with auto-hide
    let show_success = move |message: String| {
        clear_success_timeout();
        set_success_message.set(Some(message));
        
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
    };
    
    // Load database info on mount
    spawn_local(async move {
        match tauri_api::get_database_info().await {
            Ok(info) => {
                set_db_info.set(Some(info));
            }
            Err(e) => {
                set_error.set(Some(format!("データベース情報の取得に失敗しました: {}", e)));
            }
        }
        set_loading.set(false);
    });
    
    // Clear cache handler
    let on_clear_cache = move |_| {
        set_clearing_cache.set(true);
        set_error.set(None);
        
        spawn_local(async move {
            match tauri_api::clear_cache().await {
                Ok(result) => {
                    show_success(format!(
                        "キャッシュをクリアしました（{}エントリ、{}解放）",
                        result.cleared_entries,
                        format_bytes(result.freed_bytes)
                    ));
                    
                    // Refresh database info
                    if let Ok(info) = tauri_api::get_database_info().await {
                        set_db_info.set(Some(info));
                    }
                }
                Err(e) => {
                    set_error.set(Some(format!("キャッシュのクリアに失敗しました: {}", e)));
                }
            }
            set_clearing_cache.set(false);
        });
    };
    
    // Export data handler
    let on_export_data = move |_| {
        set_exporting.set(true);
        set_error.set(None);
        
        spawn_local(async move {
            match tauri_api::export_data().await {
                Ok(json_data) => {
                    // Create a downloadable file using data URL
                    if let Some(window) = web_sys::window() {
                        if let Some(document) = window.document() {
                            // Create download link using data URL
                            if let Ok(a) = document.create_element("a") {
                                let a: web_sys::HtmlAnchorElement = a.dyn_into().unwrap();
                                
                                // Use data URL for the JSON content
                                let encoded_data = js_sys::encode_uri_component(&json_data);
                                let data_url = format!("data:application/json;charset=utf-8,{}", encoded_data);
                                a.set_href(&data_url);
                                
                                // Generate filename with timestamp
                                let now = js_sys::Date::new_0();
                                let filename = format!(
                                    "development-tools-export-{:04}{:02}{:02}-{:02}{:02}{:02}.json",
                                    now.get_full_year(),
                                    now.get_month() + 1,
                                    now.get_date(),
                                    now.get_hours(),
                                    now.get_minutes(),
                                    now.get_seconds()
                                );
                                a.set_download(&filename);
                                
                                // Trigger download
                                a.click();
                                
                                show_success("データをエクスポートしました".to_string());
                            }
                        }
                    }
                }
                Err(e) => {
                    set_error.set(Some(format!("データのエクスポートに失敗しました: {}", e)));
                }
            }
            set_exporting.set(false);
        });
    };
    
    // Reset all data handler
    let on_reset_confirmed = move || {
        set_show_reset_dialog.set(false);
        set_resetting.set(true);
        set_error.set(None);
        
        spawn_local(async move {
            match tauri_api::reset_all_data().await {
                Ok(_) => {
                    show_success("全てのデータをリセットしました".to_string());
                    
                    // Refresh database info
                    if let Ok(info) = tauri_api::get_database_info().await {
                        set_db_info.set(Some(info));
                    }
                }
                Err(e) => {
                    set_error.set(Some(format!("データのリセットに失敗しました: {}", e)));
                }
            }
            set_resetting.set(false);
        });
    };
    
    let on_reset_cancel = move || {
        set_show_reset_dialog.set(false);
    };
    
    // Cleanup timeout on unmount
    on_cleanup(move || {
        clear_success_timeout();
    });
    
    view! {
        <div class="space-y-6">
            // Reset confirmation dialog
            <ResetConfirmDialog
                visible=show_reset_dialog
                on_confirm=on_reset_confirmed
                on_cancel=on_reset_cancel
            />
            
            // Loading state
            <Show when=move || loading.get()>
                <div class="text-center py-8 text-dt-text-sub">
                    "データ情報を読み込み中..."
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
            
            <Show when=move || !loading.get()>
                // Cache section
                <div class="space-y-3">
                    <h3 class="text-lg font-gaming font-bold text-white">
                        "キャッシュ"
                    </h3>
                    <div class="p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
                        <div class="flex items-center justify-between mb-4">
                            <div class="flex items-center gap-2">
                                <span class="text-2xl">"📦"</span>
                                <div>
                                    <span class="text-white font-medium block">
                                        "キャッシュサイズ"
                                    </span>
                                    <span class="text-gm-accent-cyan font-gaming">
                                        {move || db_info.get().map(|i| format_bytes(i.cache_size_bytes)).unwrap_or_else(|| "不明".to_string())}
                                    </span>
                                </div>
                            </div>
                        </div>
                        <button
                            class="w-full px-4 py-3 bg-slate-700 hover:bg-slate-600 rounded-lg text-white font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                            on:click=on_clear_cache
                            disabled=move || clearing_cache.get()
                            aria-busy=move || clearing_cache.get().to_string()
                        >
                            <span class=move || if clearing_cache.get() { "animate-spin" } else { "" }>
                                "🗑️"
                            </span>
                            {move || if clearing_cache.get() { "クリア中..." } else { "キャッシュをクリア" }}
                        </button>
                        <p class="mt-2 text-xs text-dt-text-sub">
                            "コントリビューショングラフなどのキャッシュを削除します"
                        </p>
                    </div>
                </div>
                
                // Divider
                <div class="border-t border-gm-accent-cyan/20"></div>
                
                // Data export section
                <div class="space-y-3">
                    <h3 class="text-lg font-gaming font-bold text-white">
                        "データエクスポート"
                    </h3>
                    <div class="p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
                        <p class="text-dt-text-sub mb-4">
                            "統計データをJSON形式でエクスポートします。"<br/>
                            "XP、バッジ、統計情報などが含まれます。"
                        </p>
                        <button
                            class="w-full px-4 py-3 bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple rounded-lg text-white font-gaming font-bold hover:opacity-90 transition-opacity disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                            on:click=on_export_data
                            disabled=move || exporting.get()
                            aria-busy=move || exporting.get().to_string()
                        >
                            <span class=move || if exporting.get() { "animate-spin" } else { "" }>
                                "📥"
                            </span>
                            {move || if exporting.get() { "エクスポート中..." } else { "データをエクスポート" }}
                        </button>
                    </div>
                </div>
                
                // Divider
                <div class="border-t border-gm-accent-cyan/20"></div>
                
                // Data reset section
                <div class="space-y-3">
                    <h3 class="text-lg font-gaming font-bold text-white">
                        "データリセット"
                    </h3>
                    <div class="p-4 bg-red-900/10 rounded-xl border border-red-500/30">
                        <div class="flex items-start gap-3 mb-4">
                            <span class="text-2xl">"⚠️"</span>
                            <div>
                                <span class="text-red-200 font-bold block">
                                    "全てのXP、バッジ、統計データを削除します"
                                </span>
                                <span class="text-red-200/70 text-sm">
                                    "この操作は取り消せません"
                                </span>
                            </div>
                        </div>
                        <button
                            class="w-full px-4 py-3 bg-red-600 hover:bg-red-500 rounded-lg text-white font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                            on:click=move |_| set_show_reset_dialog.set(true)
                            disabled=move || resetting.get()
                            aria-busy=move || resetting.get().to_string()
                        >
                            <span class=move || if resetting.get() { "animate-spin" } else { "" }>
                                "🗑️"
                            </span>
                            {move || if resetting.get() { "リセット中..." } else { "全データをリセット" }}
                        </button>
                    </div>
                </div>
                
                // Divider
                <div class="border-t border-gm-accent-cyan/20"></div>
                
                // Database info section
                <div class="space-y-3">
                    <h3 class="text-lg font-gaming font-bold text-white">
                        "データベース情報"
                    </h3>
                    <div class="p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
                        <div class="space-y-3">
                            <div class="flex items-center justify-between">
                                <span class="text-dt-text-sub">"パス"</span>
                                <span class="text-white text-sm font-mono truncate max-w-[200px]" title=move || db_info.get().map(|i| i.path.clone()).unwrap_or_default()>
                                    {move || {
                                        db_info.get().map(|i| {
                                            // Show only the last part of the path for readability
                                            i.path.split('/').last().unwrap_or(&i.path).to_string()
                                        }).unwrap_or_else(|| "不明".to_string())
                                    }}
                                </span>
                            </div>
                            <div class="flex items-center justify-between">
                                <span class="text-dt-text-sub">"データベースサイズ"</span>
                                <span class="text-gm-accent-cyan font-gaming">
                                    {move || db_info.get().map(|i| format_bytes(i.size_bytes)).unwrap_or_else(|| "不明".to_string())}
                                </span>
                            </div>
                            <div class="flex items-center justify-between">
                                <span class="text-dt-text-sub">"キャッシュサイズ"</span>
                                <span class="text-gm-accent-cyan font-gaming">
                                    {move || db_info.get().map(|i| format_bytes(i.cache_size_bytes)).unwrap_or_else(|| "不明".to_string())}
                                </span>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

