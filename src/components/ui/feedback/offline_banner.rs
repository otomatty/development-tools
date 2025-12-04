//! Offline Banner Component
//!
//! オフライン時に表示する警告バナー。
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   - src/components/ui/feedback/mod.rs
//!   - src/app.rs
//! Dependencies:
//!   - src/hooks/use_network_status.rs

use leptos::prelude::*;

use crate::hooks::try_use_network_status;

/// オフラインバナーコンポーネント
///
/// オフライン時に表示する警告バナー。
/// 最終オンライン時刻も表示する。
#[component]
pub fn OfflineBanner() -> impl IntoView {
    let network_ctx = try_use_network_status();

    view! {
        {move || {
            let ctx = network_ctx?;
            let state = ctx.state.get();

            if state.is_online {
                return None;
            }

            let last_online = state.last_online_at
                .as_ref()
                .map(|t| format!("最終オンライン: {}", format_timestamp(t)))
                .unwrap_or_default();

            Some(view! {
                <div class="bg-amber-500/90 text-amber-950 px-4 py-2 text-sm flex items-center justify-center gap-2">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                            d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                    </svg>
                    <span>"⚠️ オフラインモード - キャッシュデータを表示中"</span>
                    <span class="text-amber-800 text-xs">{last_online}</span>
                </div>
            })
        }}
    }
}

/// ISO 8601 タイムスタンプを人間が読みやすい形式に変換
fn format_timestamp(iso_string: &str) -> String {
    // 簡易的な実装：ISO文字列から日時部分を抽出
    // 例: "2025-11-30T12:34:56.789Z" -> "12:34"
    iso_string
        .split('T')
        .nth(1)
        .and_then(|time_part| time_part.get(0..5))
        .map(|s| s.to_string())
        .unwrap_or_else(|| iso_string.to_string())
}
