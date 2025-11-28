//! Application information component
//!
//! Displays application version, build info, and provides links to resources.

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::tauri_api;
use crate::types::AppInfo;

/// Application information component
#[component]
pub fn AppInfoSection() -> impl IntoView {
    let (app_info, set_app_info) = signal(Option::<AppInfo>::None);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (opening_url, set_opening_url) = signal(false);

    // Load app info on mount
    Effect::new(move |_| {
        spawn_local(async move {
            match tauri_api::get_app_info().await {
                Ok(info) => {
                    set_app_info.set(Some(info));
                }
                Err(e) => {
                    set_error.set(Some(format!("アプリ情報の取得に失敗しました: {}", e)));
                }
            }
            set_loading.set(false);
        });
    });

    // Open GitHub repository
    let open_github = move |_| {
        set_opening_url.set(true);
        spawn_local(async move {
            if let Err(e) = tauri_api::open_external_url("https://github.com/otomatty/development-tools").await {
                web_sys::console::error_1(&format!("Failed to open URL: {}", e).into());
            }
            set_opening_url.set(false);
        });
    };

    view! {
        <div class="space-y-6">
            // Loading state
            <Show when=move || loading.get()>
                <div class="flex items-center justify-center py-8">
                    <div class="animate-spin w-8 h-8 border-4 border-gm-accent-cyan/30 border-t-gm-accent-cyan rounded-full"></div>
                </div>
            </Show>

            // Error state
            <Show when=move || error.get().is_some()>
                <div class="p-4 bg-red-900/20 border border-red-500/30 rounded-xl">
                    <p class="text-red-300 text-sm">
                        {move || error.get().unwrap_or_default()}
                    </p>
                </div>
            </Show>

            // App info display
            <Show when=move || app_info.get().is_some() && !loading.get()>
                {move || {
                    let info = app_info.get().unwrap_or_default();
                    view! {
                        <div class="space-y-6">
                            // App name and icon
                            <div class="flex items-center gap-4">
                                <div class="w-16 h-16 rounded-xl bg-gradient-to-br from-gm-accent-cyan to-gm-accent-purple flex items-center justify-center">
                                    <span class="text-3xl">{"\u{1F6E0}\u{FE0F}"}</span>
                                </div>
                                <div>
                                    <h3 class="text-xl font-gaming font-bold text-white">
                                        "Development Tools"
                                    </h3>
                                    <p class="text-dt-text-sub">
                                        "開発者向けツールコレクション"
                                    </p>
                                </div>
                            </div>

                            // Version info
                            <div class="grid grid-cols-2 gap-4">
                                <InfoItem label="バージョン" value=info.version.clone() />
                                <InfoItem label="ビルド日時" value=info.build_date.clone() />
                                <InfoItem label="Tauri" value=info.tauri_version.clone() />
                                <InfoItem label="Leptos" value=info.leptos_version.clone() />
                                <InfoItem label="Rust" value=info.rust_version.clone() />
                            </div>

                            // Action buttons
                            <div class="flex flex-wrap gap-3 pt-4 border-t border-gm-accent-cyan/20">
                                // License info button (placeholder)
                                <button
                                    class="flex items-center gap-2 px-4 py-2 bg-gm-bg-darker rounded-lg border border-gm-accent-cyan/30 text-white hover:bg-gm-accent-cyan/10 transition-colors"
                                    on:click=move |_| {
                                        // TODO: Show license modal
                                        web_sys::console::log_1(&"License info clicked".into());
                                    }
                                >
                                    <span>{"\u{1F4C4}"}</span>
                                    <span>"ライセンス情報"</span>
                                </button>

                                // GitHub repo button
                                <button
                                    class="flex items-center gap-2 px-4 py-2 bg-gm-bg-darker rounded-lg border border-gm-accent-cyan/30 text-white hover:bg-gm-accent-cyan/10 transition-colors disabled:opacity-50"
                                    on:click=open_github
                                    disabled=move || opening_url.get()
                                >
                                    <span>{"\u{1F419}"}</span>
                                    <span>
                                        {move || if opening_url.get() { "開いています..." } else { "GitHubリポジトリ" }}
                                    </span>
                                </button>
                            </div>
                        </div>
                    }
                }}
            </Show>
        </div>
    }
}

/// Info item component for displaying version/build info
#[component]
fn InfoItem(
    label: &'static str,
    value: String,
) -> impl IntoView {
    view! {
        <div class="p-3 bg-gm-bg-darker/50 rounded-lg">
            <dt class="text-xs text-dt-text-sub uppercase tracking-wider">
                {label}
            </dt>
            <dd class="mt-1 text-white font-mono text-sm">
                {value}
            </dd>
        </div>
    }
}
