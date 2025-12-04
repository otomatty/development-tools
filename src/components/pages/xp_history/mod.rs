//! XP History Page Module
//!
//! Displays the user's XP acquisition history with detailed breakdown.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   ├─ src/components/pages/mod.rs
//!   └─ src/app.rs
//! Children:
//!   ├─ utils.rs - Utility functions for formatting
//!   ├─ item.rs - XpHistoryItem component
//!   └─ skeleton.rs - Loading skeleton and empty state
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

mod item;
mod skeleton;
pub mod utils;

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::icons::Icon;
use crate::tauri_api;
use crate::types::{AppPage, XpHistoryEntry};

pub use item::XpHistoryItem;
pub use skeleton::{EmptyState, XpHistorySkeleton};

/// XP History Page component
#[component]
pub fn XpHistoryPage(set_current_page: WriteSignal<AppPage>) -> impl IntoView {
    // State
    let (xp_history, set_xp_history) = signal(Vec::<XpHistoryEntry>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);

    // Calculate total XP from history
    let total_xp = Memo::new(move |_| xp_history.get().iter().map(|e| e.xp_amount).sum::<i32>());

    // Load XP history on mount
    spawn_local(async move {
        match tauri_api::get_xp_history(Some(20)).await {
            Ok(history) => {
                set_xp_history.set(history);
            }
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to load XP history: {}", e).into());
                set_error.set(Some(format!("履歴の読み込みに失敗しました: {}", e)));
            }
        }
        set_loading.set(false);
    });

    view! {
        <div class="flex-1 overflow-y-auto">
            <div class="max-w-4xl mx-auto p-6">
                // Header
                <div class="flex items-center gap-4 mb-8">
                    // Back button
                    <button
                        class="p-2 rounded-lg bg-slate-800/50 hover:bg-slate-700/50 text-dt-text-sub hover:text-dt-text transition-colors"
                        on:click=move |_| set_current_page.set(AppPage::Home)
                    >
                        <Icon name="arrow-left".to_string() class="w-5 h-5".to_string() />
                    </button>

                    <div class="flex-1">
                        <h1 class="text-2xl font-gaming font-bold text-dt-text flex items-center gap-3">
                            <span class="text-3xl">"📜"</span>
                            "XP獲得履歴"
                        </h1>
                        <p class="text-dt-text-sub mt-1">
                            "最近のXP獲得履歴を確認できます"
                        </p>
                    </div>

                    // Total XP badge
                    <Show when=move || !loading.get() && !xp_history.get().is_empty()>
                        <div class="px-4 py-2 bg-gm-bg-card rounded-xl border border-gm-accent-cyan/30">
                            <div class="text-xs text-dt-text-sub">"表示中の合計"</div>
                            <div class="text-lg font-gaming-mono font-bold text-gm-success">
                                "+" {move || total_xp.get()} " XP"
                            </div>
                        </div>
                    </Show>
                </div>

                // Error state
                <Show when=move || error.get().is_some()>
                    <div class="p-4 mb-6 bg-red-500/10 border border-red-500/30 rounded-xl">
                        <div class="flex items-center gap-3 text-red-400">
                            <span class="text-xl">"⚠️"</span>
                            <span>{move || error.get().unwrap_or_default()}</span>
                        </div>
                    </div>
                </Show>

                // Content
                <Show
                    when=move || !loading.get()
                    fallback=move || view! { <XpHistorySkeleton /> }
                >
                    <Show
                        when=move || !xp_history.get().is_empty()
                        fallback=move || view! { <EmptyState /> }
                    >
                        <div class="space-y-3">
                            <For
                                each=move || xp_history.get()
                                key=|entry| entry.id
                                children=move |entry| {
                                    view! { <XpHistoryItem entry=entry /> }
                                }
                            />
                        </div>
                    </Show>
                </Show>
            </div>
        </div>
    }
}
