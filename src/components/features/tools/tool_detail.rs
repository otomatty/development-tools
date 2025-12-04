use leptos::prelude::*;

use crate::components::icons::{Icon, Spinner};
use crate::components::ui::form::OptionForm;
use crate::types::{OptionValues, ToolConfig, ToolStatus};

/// ツール詳細コンポーネント
#[component]
pub fn ToolDetail(
    config: ReadSignal<Option<ToolConfig>>,
    option_values: ReadSignal<OptionValues>,
    set_option_values: WriteSignal<OptionValues>,
    trigger_run: WriteSignal<u32>,
    running: ReadSignal<bool>,
    status: ReadSignal<Option<ToolStatus>>,
) -> impl IntoView {
    view! {
        <div class="p-6 h-full overflow-y-auto">
            <Show
                when=move || config.get().is_some()
                fallback=move || view! {
                    <div class="flex flex-col items-center justify-center h-full text-dt-text-sub">
                        <Icon name="terminal".to_string() class="w-16 h-16 mb-4 opacity-50".to_string() />
                        <p class="text-lg">"Select a tool to get started"</p>
                        <p class="text-sm mt-2">"Choose from the sidebar on the left"</p>
                    </div>
                }
            >
                {move || {
                    let cfg = config.get().unwrap();
                    let icon = cfg.icon.clone().unwrap_or_else(|| "terminal".to_string());

                    view! {
                        <div class="space-y-6">
                            // ヘッダー
                            <div class="flex items-start gap-4">
                                <div class="p-3 bg-dt-accent/20 rounded-xl">
                                    <Icon name=icon class="w-8 h-8 text-dt-accent".to_string() />
                                </div>
                                <div class="flex-1">
                                    <h2 class="text-2xl font-bold text-dt-text">
                                        {cfg.display_name.clone()}
                                    </h2>
                                    <p class="text-dt-text-sub mt-1">
                                        {cfg.description.clone()}
                                    </p>
                                    <div class="flex items-center gap-4 mt-2">
                                        <span class="text-xs text-dt-text-sub">
                                            "v" {cfg.version.clone()}
                                        </span>
                                        {cfg.category.clone().map(|cat| view! {
                                            <span class="badge badge-success">
                                                {cat}
                                            </span>
                                        })}
                                    </div>
                                </div>
                            </div>

                            // オプションフォーム
                            <div class="card p-4">
                                <h3 class="text-sm font-medium text-dt-text-sub uppercase tracking-wider mb-4">
                                    "Options"
                                </h3>
                                <OptionForm
                                    options=cfg.options.clone()
                                    values=option_values
                                    set_values=set_option_values
                                />
                            </div>

                            // 実行ボタン
                            <div class="flex items-center gap-4">
                                <button
                                    class="btn-primary"
                                    disabled=move || running.get()
                                    on:click=move |_| trigger_run.update(|v| *v += 1)
                                >
                                    {move || if running.get() {
                                        view! {
                                            <Spinner class="w-5 h-5".to_string() />
                                            <span>"Running..."</span>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <Icon name="play".to_string() class="w-5 h-5".to_string() />
                                            <span>"Run Scanner"</span>
                                        }.into_any()
                                    }}
                                </button>

                                // ステータス表示
                                {move || {
                                    match status.get() {
                                        Some(ToolStatus::Completed) => view! {
                                            <div class="flex items-center gap-2 text-dt-success">
                                                <Icon name="check".to_string() class="w-5 h-5".to_string() />
                                                <span>"Completed"</span>
                                            </div>
                                        }.into_any(),
                                        Some(ToolStatus::Failed) => view! {
                                            <div class="flex items-center gap-2 text-dt-error">
                                                <Icon name="x".to_string() class="w-5 h-5".to_string() />
                                                <span>"Failed"</span>
                                            </div>
                                        }.into_any(),
                                        _ => view! {}.into_any(),
                                    }
                                }}
                            </div>
                        </div>
                    }
                }}
            </Show>
        </div>
    }
}
