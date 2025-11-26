//! Account settings component
//!
//! Displays account information and provides logout functionality.

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::components::ConfirmDialog;
use crate::tauri_api;
use crate::types::{AppPage, AuthState};

/// Account settings component
#[component]
pub fn AccountSettings(
    auth_state: ReadSignal<AuthState>,
    set_auth_state: WriteSignal<AuthState>,
    set_current_page: WriteSignal<AppPage>,
) -> impl IntoView
{
    let (show_logout_dialog, set_show_logout_dialog) = signal(false);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (success_message, set_success_message) = signal(None::<String>);

    // Format date helper - extract date part from RFC3339 string
    let format_date = |date_str: Option<String>| {
        date_str
            .and_then(|s| {
                // RFC3339 format: "2024-11-26T15:30:00Z" -> extract "2024-11-26"
                s.split('T').next().map(|s| s.to_string())
            })
            .unwrap_or_else(|| "不明".to_string())
    };


    // Handle token validation
    let handle_validate_token = move || {
        set_loading.set(true);
        set_error.set(None);
        set_success_message.set(None);

        spawn_local(async move {
            match tauri_api::validate_token().await {
                Ok(true) => {
                    set_success_message.set(Some("トークンは有効です".to_string()));
                }
                Ok(false) => {
                    set_error.set(Some("トークンが無効です。再認証が必要です。".to_string()));
                }
                Err(e) => {
                    set_error.set(Some(format!("トークンの確認に失敗しました: {}", e)));
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="space-y-4">
            // Account info section
            {move || {
                let user = auth_state.get().user;
                if let Some(user_info) = user {
                    view! {
                        <div class="flex items-center gap-4 p-4 bg-gm-bg-card/50 rounded-xl border border-gm-accent-cyan/20">
                            // Avatar
                            <img
                                src=user_info.avatar_url.clone().unwrap_or_default()
                                alt="Avatar"
                                class="w-16 h-16 rounded-xl border-2 border-gm-accent-cyan"
                            />
                            <div class="flex-1">
                                <div class="text-white font-gaming font-bold text-lg">
                                    {"@"}{user_info.username.clone()}
                                </div>
                                <div class="text-dt-text-sub text-sm mt-1">
                                    {"GitHub ID: "}{user_info.github_id}
                                </div>
                                <div class="text-dt-text-sub text-sm">
                                    {"接続日: "}{format_date(user_info.created_at.clone())}
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="text-dt-text-sub text-center py-8">
                            "アカウント情報を取得できませんでした"
                        </div>
                    }.into_any()
                }
            }}

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

            // Action buttons
            <div class="flex gap-3">
                <button
                    class="flex-1 px-4 py-2 rounded-lg bg-gm-accent-cyan/20 hover:bg-gm-accent-cyan/30 text-gm-accent-cyan transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                    disabled=loading.get()
                    on:click=move |_| handle_validate_token()
                >
                    {if loading.get() {
                        "確認中..."
                    } else {
                        "🔄 トークンを確認"
                    }}
                </button>
                <button
                    class="flex-1 px-4 py-2 rounded-lg bg-gm-error/20 hover:bg-gm-error/30 text-gm-error transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                    disabled=loading.get()
                    on:click=move |_| set_show_logout_dialog.set(true)
                >
                    "🚪 ログアウト"
                </button>
            </div>

            // Note
            <div class="text-xs text-dt-text-sub p-3 bg-gm-bg-card/30 rounded-lg">
                "※ログアウトしてもXP・バッジ・統計データは保持されます"
            </div>

            // Logout confirmation dialog
            <ConfirmDialog
                title="ログアウトの確認".to_string()
                message="ログアウトしますか？トークンは削除されますが、XP・バッジ・統計データは保持されます。".to_string()
                confirm_label="ログアウト".to_string()
                cancel_label="キャンセル".to_string()
                visible=show_logout_dialog
                on_confirm=move |_| {
                    set_show_logout_dialog.set(false);
                    set_loading.set(true);
                    set_error.set(None);

                    spawn_local(async move {
                        match tauri_api::logout().await {
                            Ok(_) => {
                                set_auth_state.set(AuthState::default());
                                set_current_page.set(AppPage::Home);
                            }
                            Err(e) => {
                                set_error.set(Some(format!("ログアウトに失敗しました: {}", e)));
                            }
                        }
                        set_loading.set(false);
                    });
                }
                on_cancel=move |_| set_show_logout_dialog.set(false)
            />
        </div>
    }
}

