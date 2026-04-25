use leptos::prelude::*;

use crate::components::icons::Icon;
use crate::types::AppPage;

/// サイドバーコンポーネント
#[component]
pub fn Sidebar(
    current_page: ReadSignal<AppPage>,
    set_current_page: WriteSignal<AppPage>,
) -> impl IntoView {
    view! {
        <aside class="w-64 bg-slate-900 border-r border-slate-700/50 flex flex-col h-full">
            // ヘッダー
            <div class="p-4 border-b border-slate-700/50">
                <div class="flex items-center gap-3">
                    <div class="p-2 bg-gradient-to-br from-gm-accent-cyan to-gm-accent-purple rounded-lg">
                        <Icon name="zap".to_string() class="w-6 h-6 text-white".to_string() />
                    </div>
                    <div>
                        <h1 class="text-lg font-semibold text-dt-text font-gaming">"Dev Tools"</h1>
                        <p class="text-xs text-dt-text-sub">"Level Up Your Dev"</p>
                    </div>
                </div>
            </div>

            // ナビゲーション
            <div class="p-3 space-y-1">
                // Home
                <button
                    class=move || format!(
                        "w-full flex items-center gap-3 px-3 py-2.5 rounded-lg cursor-pointer transition-all duration-200 {}",
                        if current_page.get() == AppPage::Home {
                            "bg-gradient-to-r from-gm-accent-cyan/20 to-gm-accent-purple/20 text-gm-accent-cyan border-l-2 border-gm-accent-cyan"
                        } else {
                            "text-slate-400 hover:bg-slate-800 hover:text-dt-text"
                        }
                    )
                    on:click=move |_| set_current_page.set(AppPage::Home)
                >
                    <Icon name="home".to_string() class="w-5 h-5".to_string() />
                    <span class="font-medium">"Home"</span>
                </button>

                // Projects (Issue管理)
                <button
                    class=move || format!(
                        "w-full flex items-center gap-3 px-3 py-2.5 rounded-lg cursor-pointer transition-all duration-200 {}",
                        if matches!(current_page.get(), AppPage::Projects | AppPage::ProjectDetail(_)) {
                            "bg-gradient-to-r from-gm-accent-cyan/20 to-gm-accent-purple/20 text-gm-accent-cyan border-l-2 border-gm-accent-cyan"
                        } else {
                            "text-slate-400 hover:bg-slate-800 hover:text-dt-text"
                        }
                    )
                    on:click=move |_| set_current_page.set(AppPage::Projects)
                >
                    <Icon name="kanban".to_string() class="w-5 h-5".to_string() />
                    <span class="font-medium">"Projects"</span>
                </button>

                // Separator
                <div class="my-2 border-t border-slate-700/50"/>

                // Mock Server
                <button
                    class=move || format!(
                        "w-full flex items-center gap-3 px-3 py-2.5 rounded-lg cursor-pointer transition-all duration-200 {}",
                        if current_page.get() == AppPage::MockServer {
                            "bg-gradient-to-r from-gm-accent-cyan/20 to-gm-accent-purple/20 text-gm-accent-cyan border-l-2 border-gm-accent-cyan"
                        } else {
                            "text-slate-400 hover:bg-slate-800 hover:text-dt-text"
                        }
                    )
                    on:click=move |_| set_current_page.set(AppPage::MockServer)
                >
                    <Icon name="radio".to_string() class="w-5 h-5".to_string() />
                    <span class="font-medium">"Mock Server"</span>
                </button>
            </div>

            // スペーサー
            <div class="flex-1"/>

            // フッター
            <div class="p-3 border-t border-slate-700/50">
                <div class="flex items-center justify-between">
                    <div class="text-xs text-dt-text-sub">
                    "v0.1.0"
                    </div>
                    // Settings button
                    <button
                        class=move || format!(
                            "p-2 rounded-lg transition-all duration-200 {}",
                            if current_page.get() == AppPage::Settings {
                                "bg-gm-accent-cyan/20 text-gm-accent-cyan"
                            } else {
                                "text-slate-400 hover:bg-slate-800 hover:text-dt-text"
                            }
                        )
                        title="Settings"
                        on:click=move |_| set_current_page.set(AppPage::Settings)
                    >
                        <Icon name="settings".to_string() class="w-5 h-5".to_string() />
                    </button>
                </div>
            </div>
        </aside>
    }
}
