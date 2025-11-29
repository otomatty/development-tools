use leptos::prelude::*;

use crate::components::icons::Icon;
use crate::types::{AppPage, ToolInfo};

/// サイドバーコンポーネント
#[component]
pub fn Sidebar(
    tools: ReadSignal<Vec<ToolInfo>>,
    selected_tool: ReadSignal<Option<String>>,
    set_selected_tool: WriteSignal<Option<String>>,
    current_page: ReadSignal<AppPage>,
    set_current_page: WriteSignal<AppPage>,
    loading: ReadSignal<bool>,
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
                    on:click=move |_| {
                        set_current_page.set(AppPage::Home);
                        set_selected_tool.set(None);
                    }
                >
                    <Icon name="home".to_string() class="w-5 h-5".to_string() />
                    <span class="font-medium">"Home"</span>
                </button>

                // Tools
                <button
                    class=move || format!(
                        "w-full flex items-center gap-3 px-3 py-2.5 rounded-lg cursor-pointer transition-all duration-200 {}",
                        if current_page.get() == AppPage::Tools {
                            "bg-slate-800 text-dt-text border-l-2 border-dt-accent"
                        } else {
                            "text-slate-400 hover:bg-slate-800 hover:text-dt-text"
                        }
                    )
                    on:click=move |_| set_current_page.set(AppPage::Tools)
                >
                    <Icon name="wrench".to_string() class="w-5 h-5".to_string() />
                    <span class="font-medium">"Tools"</span>
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
                    on:click=move |_| {
                        set_current_page.set(AppPage::MockServer);
                        set_selected_tool.set(None);
                    }
                >
                    <Icon name="radio".to_string() class="w-5 h-5".to_string() />
                    <span class="font-medium">"Mock Server"</span>
                </button>
            </div>

            // ツール一覧 (Toolsページのとき表示)
            <Show when=move || current_page.get() == AppPage::Tools>
                <div class="flex-1 overflow-y-auto p-3 border-t border-slate-700/50">
                    <div class="text-xs font-medium text-dt-text-sub uppercase tracking-wider mb-3 px-2">
                        "Available Tools"
                    </div>

                <Show
                    when=move || !loading.get()
                    fallback=move || view! {
                        <div class="flex items-center justify-center py-8">
                            <div class="animate-spin w-6 h-6 border-2 border-dt-accent border-t-transparent rounded-full"/>
                        </div>
                    }
                >
                    <div class="space-y-1">
                        {move || {
                            tools.get().into_iter().map(|tool| {
                                let tool_name = tool.name.clone();
                                let tool_name_for_click = tool.name.clone();
                                let tool_display_name = tool.display_name.clone();
                                let tool_icon = tool.icon.clone().unwrap_or_else(|| "terminal".to_string());
                                
                                let is_selected = move || {
                                    selected_tool.get().as_ref() == Some(&tool_name)
                                };

                                view! {
                                    <button
                                        class=move || format!(
                                            "w-full flex items-center gap-3 px-3 py-2.5 rounded-lg cursor-pointer transition-all duration-200 {}",
                                            if is_selected() {
                                                "bg-slate-800 text-dt-text border-l-2 border-dt-accent"
                                            } else {
                                                "text-slate-400 hover:bg-slate-800 hover:text-dt-text"
                                            }
                                        )
                                        on:click={
                                            let name = tool_name_for_click.clone();
                                            move |_| set_selected_tool.set(Some(name.clone()))
                                        }
                                    >
                                        <Icon name=tool_icon.clone() class="w-5 h-5".to_string() />
                                        <span class="font-medium truncate">{tool_display_name.clone()}</span>
                                    </button>
                                }
                            }).collect_view()
                        }}
                    </div>
                </Show>

                    // ツールがない場合
                    <Show when=move || !loading.get() && tools.get().is_empty()>
                        <div class="text-center py-8 text-dt-text-sub">
                            <Icon name="package".to_string() class="w-12 h-12 mx-auto mb-3 opacity-50".to_string() />
                            <p class="text-sm">"No tools found"</p>
                            <p class="text-xs mt-1">"Add tool.json to tools/ directory"</p>
                        </div>
                    </Show>
                </div>
            </Show>

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
                        on:click=move |_| {
                            set_current_page.set(AppPage::Settings);
                            set_selected_tool.set(None);
                        }
                    >
                        <Icon name="settings".to_string() class="w-5 h-5".to_string() />
                    </button>
                </div>
            </div>
        </aside>
    }
}

