use leptos::prelude::*;

use crate::components::icons::Icon;
use crate::types::{ResultSchema, ToolResult};

/// 結果表示コンポーネント
#[component]
pub fn ResultView(
    result: ReadSignal<Option<ToolResult>>,
    #[allow(unused_variables)]
    schema: ReadSignal<Option<ResultSchema>>,
) -> impl IntoView {
    let (active_tab, set_active_tab) = signal("summary".to_string());

    view! {
        <Show when=move || result.get().is_some()>
            <div class="card mt-4">
                // タブヘッダー
                <div class="flex border-b border-slate-700/50">
                    <TabButton 
                        label="Summary".to_string()
                        tab_id="summary".to_string()
                        active_tab=active_tab
                        on_click=set_active_tab
                    />
                    <TabButton 
                        label="Details".to_string()
                        tab_id="details".to_string()
                        active_tab=active_tab
                        on_click=set_active_tab
                    />
                    <TabButton 
                        label="Raw Output".to_string()
                        tab_id="raw".to_string()
                        active_tab=active_tab
                        on_click=set_active_tab
                    />
                </div>

                // タブコンテンツ
                <div class="p-4">
                    <Show when=move || active_tab.get() == "summary">
                        <SummaryViewInner result=result />
                    </Show>
                    <Show when=move || active_tab.get() == "details">
                        <DetailsViewInner result=result />
                    </Show>
                    <Show when=move || active_tab.get() == "raw">
                        <RawOutputViewInner result=result />
                    </Show>
                </div>
            </div>
        </Show>
    }
}

/// タブボタン
#[component]
fn TabButton(
    label: String,
    tab_id: String,
    active_tab: ReadSignal<String>,
    on_click: WriteSignal<String>,
) -> impl IntoView {
    let tab_id_clone = tab_id.clone();
    let is_active = move || active_tab.get() == tab_id_clone;

    view! {
        <button
            class=move || format!(
                "px-4 py-3 text-sm font-medium transition-colors {}",
                if is_active() {
                    "text-dt-accent border-b-2 border-dt-accent"
                } else {
                    "text-dt-text-sub hover:text-dt-text"
                }
            )
            on:click={
                let tab_id = tab_id.clone();
                move |_| on_click.set(tab_id.clone())
            }
        >
            {label}
        </button>
    }
}

/// サマリービュー（シグナルを使用）
#[component]
fn SummaryViewInner(
    result: ReadSignal<Option<ToolResult>>,
) -> impl IntoView {
    // JSONからサマリーデータを抽出
    let summary_data = move || {
        let res = result.get()?;
        let parsed = res.parsed_result.as_ref()?;
        
        // detectionsとsuspicious_filesを取得
        let detections = parsed.get("detections")?.as_array()?;
        let suspicious_files = parsed.get("suspicious_files")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);

        // 重要度別にカウント
        let critical_count = detections.iter()
            .filter(|d| d.get("severity").and_then(|s| s.as_str()) == Some("CRITICAL"))
            .count();
        let warning_count = detections.iter()
            .filter(|d| d.get("severity").and_then(|s| s.as_str()) == Some("WARNING"))
            .count();

        Some((critical_count, warning_count, suspicious_files))
    };

    view! {
        <div class="grid grid-cols-3 gap-4">
            {move || {
                if let Some((critical, warning, suspicious)) = summary_data() {
                    view! {
                        <SummaryCard 
                            label="Critical".to_string()
                            count=critical
                            color="red".to_string()
                            icon="alert-circle".to_string()
                        />
                        <SummaryCard 
                            label="Warning".to_string()
                            count=warning
                            color="yellow".to_string()
                            icon="alert-triangle".to_string()
                        />
                        <SummaryCard 
                            label="Suspicious".to_string()
                            count=suspicious
                            color="orange".to_string()
                            icon="file-warning".to_string()
                        />
                    }.into_any()
                } else {
                    view! {
                        <div class="col-span-3 text-center text-dt-text-sub py-8">
                            "No summary data available"
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

/// サマリーカード
#[component]
fn SummaryCard(
    label: String,
    count: usize,
    color: String,
    icon: String,
) -> impl IntoView {
    let (bg_class, text_class, border_class) = match color.as_str() {
        "red" => ("bg-red-500/10", "text-red-400", "border-red-500/30"),
        "yellow" => ("bg-yellow-500/10", "text-yellow-400", "border-yellow-500/30"),
        "orange" => ("bg-orange-500/10", "text-orange-400", "border-orange-500/30"),
        "green" => ("bg-green-500/10", "text-green-400", "border-green-500/30"),
        _ => ("bg-slate-500/10", "text-slate-400", "border-slate-500/30"),
    };

    view! {
        <div class=format!("rounded-xl p-4 border {} {}", border_class, bg_class)>
            <div class="flex items-center justify-between mb-2">
                <span class=format!("text-sm font-medium {}", text_class)>{label}</span>
                <Icon name=icon class=format!("w-5 h-5 {}", text_class) />
            </div>
            <div class=format!("text-3xl font-bold {}", text_class)>
                {count}
            </div>
        </div>
    }
}

/// 詳細ビュー（シグナルを使用）
#[component]
fn DetailsViewInner(
    result: ReadSignal<Option<ToolResult>>,
) -> impl IntoView {
    let detections = move || {
        result.get()
            .and_then(|r| r.parsed_result)
            .and_then(|p| p.get("detections").cloned())
            .and_then(|d| d.as_array().cloned())
            .unwrap_or_default()
    };

    view! {
        <div class="overflow-x-auto">
            <Show
                when=move || !detections().is_empty()
                fallback=move || view! {
                    <div class="text-center text-dt-text-sub py-8">
                        <Icon name="check".to_string() class="w-12 h-12 mx-auto mb-3 text-dt-success opacity-50".to_string() />
                        <p>"No detections found"</p>
                    </div>
                }
            >
                <table class="w-full text-sm">
                    <thead>
                        <tr class="text-left text-dt-text-sub border-b border-slate-700/50">
                            <th class="pb-3 font-medium">"Package"</th>
                            <th class="pb-3 font-medium">"Version"</th>
                            <th class="pb-3 font-medium">"Severity"</th>
                            <th class="pb-3 font-medium">"Source"</th>
                            <th class="pb-3 font-medium">"Location"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            detections().into_iter().map(|item| {
                                let package = item.get("package")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("-")
                                    .to_string();
                                let version = item.get("installed_version")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("-")
                                    .to_string();
                                let severity = item.get("severity")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("-")
                                    .to_string();
                                let source = item.get("source")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("-")
                                    .to_string();
                                let location = item.get("location")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("-")
                                    .to_string();
                                let location_title = location.clone();

                                let severity_class = match severity.as_str() {
                                    "CRITICAL" => "badge-critical",
                                    "WARNING" => "badge-warning",
                                    _ => "badge",
                                };

                                view! {
                                    <tr class="border-b border-slate-700/30 hover:bg-slate-800/50">
                                        <td class="py-3 font-medium text-dt-text">{package}</td>
                                        <td class="py-3 text-dt-text-sub">{version}</td>
                                        <td class="py-3">
                                            <span class=format!("badge {}", severity_class)>
                                                {severity}
                                            </span>
                                        </td>
                                        <td class="py-3 text-dt-text-sub">{source}</td>
                                        <td class="py-3 text-dt-text-sub text-xs font-mono truncate max-w-xs" title=location_title>
                                            {location}
                                        </td>
                                    </tr>
                                }
                            }).collect_view()
                        }}
                    </tbody>
                </table>
            </Show>
        </div>
    }
}

/// 生出力ビュー（シグナルを使用）
#[component]
fn RawOutputViewInner(result: ReadSignal<Option<ToolResult>>) -> impl IntoView {
    let output = move || {
        result.get().map(|r| {
            if !r.stdout.is_empty() {
                r.stdout
            } else if !r.stderr.is_empty() {
                r.stderr
            } else {
                "No output".to_string()
            }
        }).unwrap_or_else(|| "No output".to_string())
    };

    view! {
        <div class="bg-slate-900 rounded-lg p-4 overflow-auto max-h-96">
            <pre class="font-mono text-sm text-slate-300 whitespace-pre-wrap">
                {output}
            </pre>
        </div>
    }
}
