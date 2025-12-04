//! Cache Indicator Component
//!
//! Shows when data is being displayed from cache (offline mode).
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this component):
//!   └─ src/components/pages/home_page.rs
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

use leptos::prelude::*;

/// Cache indicator component - shows when data is from cache (offline mode)
#[component]
pub fn CacheIndicator(
    data_from_cache: ReadSignal<bool>,
    cache_timestamp: ReadSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <Show when=move || data_from_cache.get()>
            <div class="p-3 bg-gm-warning/20 border border-gm-warning/50 rounded-lg flex items-center gap-3">
                <svg class="w-5 h-5 text-gm-warning flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
                <div class="flex-1">
                    <p class="text-gm-warning text-sm font-medium">
                        "キャッシュデータを表示中"
                    </p>
                    <p class="text-gm-text-secondary text-xs">
                        {move || {
                            cache_timestamp.get()
                                .map(|ts| format!("最終更新: {}", ts))
                                .unwrap_or_else(|| "オンライン復帰時に自動更新されます".to_string())
                        }}
                    </p>
                </div>
            </div>
        </Show>
    }
}
