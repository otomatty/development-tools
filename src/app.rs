use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use crate::components::{AnimationContext, HomePage, LogViewer, ResultView, Sidebar, ToolDetail};
use crate::components::settings::SettingsPage;
use crate::tauri_api;
use crate::types::{
    AppPage, AuthState, LogEntry, LogEvent, OptionValues, ToolConfig, ToolInfo, 
    ToolResult, ToolStatus, ToolStatusEvent,
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, handler: &Closure<dyn FnMut(JsValue)>) -> JsValue;
}

#[component]
pub fn App() -> impl IntoView {
    // ページ状態
    let (current_page, set_current_page) = signal(AppPage::Home);
    
    // 認証状態（SettingsPageで使用）
    let (auth_state, set_auth_state) = signal(AuthState::default());
    
    // アニメーション状態（グローバル）
    let animation_context = AnimationContext::new(true);
    provide_context(animation_context);
    
    // 認証状態とアニメーション設定を初期化
    {
        let animation_ctx = animation_context;
        spawn_local(async move {
            // 認証状態を取得
            match tauri_api::get_auth_state().await {
                Ok(state) => {
                    set_auth_state.set(state);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to get auth state: {}", e).into());
                }
            }
            
            // 設定を取得してアニメーション状態を更新
            match tauri_api::get_settings().await {
                Ok(settings) => {
                    animation_ctx.set_enabled.set(settings.animations_enabled);
                }
                Err(e) => {
                    // ログインしていない場合はエラーが出るが、デフォルト値を使用
                    web_sys::console::log_1(&format!("Settings not loaded (may not be logged in): {}", e).into());
                }
            }
        });
    }
    
    // ツール関連の状態管理
    let (tools, set_tools) = signal(Vec::<ToolInfo>::new());
    let (loading_tools, set_loading_tools) = signal(true);
    let (selected_tool_name, set_selected_tool_name) = signal(Option::<String>::None);
    let (tool_config, set_tool_config) = signal(Option::<ToolConfig>::None);
    let (option_values, set_option_values) = signal(OptionValues::new());
    let (running, set_running) = signal(false);
    let (logs, set_logs) = signal(Vec::<LogEntry>::new());
    let (result, set_result) = signal(Option::<ToolResult>::None);
    let (status, set_status) = signal(Option::<ToolStatus>::None);
    let (show_logs, set_show_logs) = signal(false);
    let (result_schema, set_result_schema) = signal(Option::<crate::types::ResultSchema>::None);
    let (trigger_run, set_trigger_run) = signal(0u32);

    // 初期化: ツール一覧を読み込み
    spawn_local(async move {
        match tauri_api::list_tools().await {
            Ok(tool_list) => {
                set_tools.set(tool_list);
            }
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to load tools: {}", e).into());
            }
        }
        set_loading_tools.set(false);
    });

    // イベントリスナーをセットアップ
    spawn_local(async move {
        // ログイベントリスナー
        let log_closure = Closure::new(move |event: JsValue| {
            if let Ok(payload) = js_sys::Reflect::get(&event, &"payload".into()) {
                if let Ok(log_event) = serde_wasm_bindgen::from_value::<LogEvent>(payload) {
                    set_logs.update(|logs| {
                        logs.push(LogEntry {
                            line: log_event.line,
                            stream: log_event.stream,
                            timestamp: log_event.timestamp,
                        });
                    });
                }
            }
        });
        let _ = listen("tool-log", &log_closure).await;
        log_closure.forget();

        // ステータスイベントリスナー
        let status_closure = Closure::new(move |event: JsValue| {
            if let Ok(payload) = js_sys::Reflect::get(&event, &"payload".into()) {
                if let Ok(status_event) = serde_wasm_bindgen::from_value::<ToolStatusEvent>(payload) {
                    set_status.set(Some(status_event.status.clone()));
                    
                    match status_event.status {
                        ToolStatus::Running => {
                            set_running.set(true);
                            set_show_logs.set(true);
                        }
                        ToolStatus::Completed | ToolStatus::Failed => {
                            set_running.set(false);
                            if let Some(res) = status_event.result {
                                set_result.set(Some(res));
                            }
                        }
                        ToolStatus::Cancelled => {
                            set_running.set(false);
                        }
                    }
                }
            }
        });
        let _ = listen("tool-status", &status_closure).await;
        status_closure.forget();
    });

    // ツール選択が変わったときにツール設定を読み込む
    Effect::new(move |prev_name: Option<Option<String>>| {
        let current_name = selected_tool_name.get();
        
        // 前回と同じ場合はスキップ
        if prev_name.as_ref() == Some(&current_name) {
            return current_name;
        }
        
        if let Some(name) = current_name.clone() {
            // 状態をリセット
            set_option_values.set(OptionValues::new());
            set_logs.set(Vec::new());
            set_result.set(None);
            set_status.set(None);
            set_show_logs.set(false);
            set_result_schema.set(None);

            spawn_local(async move {
                match tauri_api::get_tool_config(&name).await {
                    Ok(config) => {
                        // デフォルト値を設定
                        let mut defaults = HashMap::new();
                        for opt in &config.options {
                            if let Some(ref default) = opt.default {
                                defaults.insert(opt.name.clone(), default.clone());
                            }
                        }
                        set_option_values.set(defaults);
                        // スキーマを更新
                        let schema = config.result_parser.as_ref().and_then(|p| p.schema.clone());
                        set_result_schema.set(schema);
                        set_tool_config.set(Some(config));
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("Failed to load tool config: {}", e).into());
                    }
                }
            });
        }
        
        current_name
    });

    // 実行トリガーが変わったときにツールを実行
    Effect::new(move |prev_trigger: Option<u32>| {
        let current_trigger = trigger_run.get();
        
        // 初回または前回と同じ場合はスキップ
        if prev_trigger.is_none() || prev_trigger == Some(current_trigger) {
            return current_trigger;
        }
        
        if let Some(tool_name) = selected_tool_name.get_untracked() {
            set_logs.set(Vec::new());
            set_result.set(None);
            set_status.set(None);
            
            let options = option_values.get_untracked();
            spawn_local(async move {
                if let Err(e) = tauri_api::run_tool(&tool_name, &options).await {
                    web_sys::console::error_1(&format!("Failed to run tool: {}", e).into());
                    set_running.set(false);
                    set_status.set(Some(ToolStatus::Failed));
                }
            });
        }
        
        current_trigger
    });

    view! {
        <div class=move || {
            let base = "flex h-screen bg-dt-bg";
            if animation_context.enabled.get() {
                base.to_string()
            } else {
                format!("{} no-animation", base)
            }
        }>
            // サイドバー
            <Sidebar 
                tools=tools
                selected_tool=selected_tool_name
                set_selected_tool=set_selected_tool_name
                current_page=current_page
                set_current_page=set_current_page
                loading=loading_tools
            />

            // メインコンテンツ
            <main class="flex-1 flex flex-col overflow-hidden">
                // ページに応じてコンテンツを表示
                {move || match current_page.get() {
                    AppPage::Home => view! {
                        <HomePage set_current_page=set_current_page />
                    }.into_any(),
                    
                    AppPage::Tools => view! {
                        // 上部: ツール詳細・オプション
                        <div class="flex-1 overflow-y-auto">
                            <ToolDetail 
                                config=tool_config
                                option_values=option_values
                                set_option_values=set_option_values
                                trigger_run=set_trigger_run
                                running=running
                                status=status
                            />
                        </div>

                        // 下部: 結果表示
                        <Show when=move || result.get().is_some() || show_logs.get()>
                            <div class="border-t border-slate-700/50 bg-dt-card p-4 max-h-[50vh] overflow-y-auto">
                                // ログビューア
                                <LogViewer 
                                    logs=logs
                                    visible=show_logs
                                />

                                // 結果表示
                                <ResultView 
                                    result=result
                                    schema=result_schema
                                />
                            </div>
                        </Show>
                    }.into_any(),
                    
                    AppPage::Settings => view! {
                        <SettingsPage
                            auth_state=auth_state
                            set_auth_state=set_auth_state
                            set_current_page=set_current_page
                        />
                    }.into_any(),
                }}
            </main>
        </div>
    }
}
