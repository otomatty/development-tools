use leptos::prelude::*;

use crate::components::icons::Icon;
use crate::types::{ResultSchema, ToolResult};

/// 結果表示コンポーネント
#[component]
pub fn ResultView(
    result: ReadSignal<Option<ToolResult>>,
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
                        <SummaryViewInner result=result schema=schema />
                    </Show>
                    <Show when=move || active_tab.get() == "details">
                        <DetailsViewInner result=result schema=schema />
                    </Show>
                    <Show when=move || active_tab.get() == "raw">
                        <RawOutputViewInner result=result />
                    </Show>
                </div>
            </div>
        </Show>
    }
}

/// JSONPathライクなパス（例: $.summary.by_type.TODO）から値を取得
fn get_value_by_path(json: &serde_json::Value, path: &str) -> Option<serde_json::Value> {
    // ルート要素自体を返すケース
    if path == "$" {
        return Some(json.clone());
    }

    // $. で始まるパスを正規化
    let path = path.strip_prefix("$.").unwrap_or(path);

    // 空パスの場合はルート要素を返す
    if path.is_empty() {
        return Some(json.clone());
    }

    let mut current = json;
    // 空のパス部分をフィルタリング（連続ドット対策）
    for part in path.split('.').filter(|p| !p.is_empty()) {
        match current {
            serde_json::Value::Object(map) => {
                current = map.get(part)?;
            }
            serde_json::Value::Array(arr) => {
                let idx: usize = part.parse().ok()?;
                current = arr.get(idx)?;
            }
            _ => return None,
        }
    }
    Some(current.clone())
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

/// サマリービュー（スキーマを使用して汎用的に表示）
#[component]
fn SummaryViewInner(
    result: ReadSignal<Option<ToolResult>>,
    schema: ReadSignal<Option<ResultSchema>>,
) -> impl IntoView {
    // スキーマからサマリー定義を取得し、各項目の値を計算
    let summary_items = move || {
        let res = result.get()?;
        let parsed = res.parsed_result.as_ref()?;
        let schema = schema.get()?;
        let summary_def = schema.summary?;

        let items: Vec<(String, usize, String, String)> = summary_def
            .iter()
            .filter_map(|item| {
                let value = get_value_by_path(parsed, &item.path)?;
                let count = match &value {
                    serde_json::Value::Number(n) => n.as_u64().unwrap_or(0) as usize,
                    serde_json::Value::Array(arr) => arr.len(),
                    _ => 0,
                };

                Some((
                    item.label.clone(),
                    count,
                    item.color.clone().unwrap_or_else(|| "slate".to_string()),
                    item.icon.clone().unwrap_or_else(|| "info".to_string()),
                ))
            })
            .collect();

        if items.is_empty() {
            None
        } else {
            Some(items)
        }
    };

    view! {
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
            {move || {
                if let Some(items) = summary_items() {
                    items.into_iter().map(|(label, count, color, icon)| {
                        view! {
                            <SummaryCard
                                label=label
                                count=count
                                color=color
                                icon=icon
                            />
                        }
                    }).collect_view().into_any()
                } else {
                    // スキーマがない場合はフォールバック表示
                    view! {
                        <div class="col-span-4 text-center text-dt-text-sub py-8">
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
fn SummaryCard(label: String, count: usize, color: String, icon: String) -> impl IntoView {
    let (bg_class, text_class, border_class) = match color.as_str() {
        "red" => ("bg-red-500/10", "text-red-400", "border-red-500/30"),
        "yellow" => (
            "bg-yellow-500/10",
            "text-yellow-400",
            "border-yellow-500/30",
        ),
        "orange" => (
            "bg-orange-500/10",
            "text-orange-400",
            "border-orange-500/30",
        ),
        "green" => ("bg-green-500/10", "text-green-400", "border-green-500/30"),
        "blue" => ("bg-blue-500/10", "text-blue-400", "border-blue-500/30"),
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

/// 詳細ビュー（スキーマを使用して汎用的に表示）
#[component]
fn DetailsViewInner(
    result: ReadSignal<Option<ToolResult>>,
    schema: ReadSignal<Option<ResultSchema>>,
) -> impl IntoView {
    // スキーマからアイテムリストとカラム定義を取得
    let details_data = move || {
        let res = result.get()?;
        let parsed = res.parsed_result.as_ref()?;
        let schema = schema.get()?;
        let details_config = schema.details?;

        // アイテムリストを取得
        let items = get_value_by_path(parsed, &details_config.items)?
            .as_array()?
            .clone();

        Some((items, details_config.columns))
    };

    view! {
        <div class="overflow-x-auto">
            {move || {
                if let Some((items, columns)) = details_data() {
                    if items.is_empty() {
                        view! {
                            <div class="text-center text-dt-text-sub py-8">
                                <Icon name="check".to_string() class="w-12 h-12 mx-auto mb-3 text-dt-success opacity-50".to_string() />
                                <p>"No items found"</p>
                            </div>
                        }.into_any()
                    } else {
                        let columns_clone = columns.clone();
                        view! {
                            <table class="w-full text-sm">
                                <thead>
                                    <tr class="text-left text-dt-text-sub border-b border-slate-700/50">
                                        {columns.iter().map(|col| {
                                            let style = col.width.as_ref()
                                                .map(|w| format!("width: {}", w))
                                                .unwrap_or_default();
                                            view! {
                                                <th class="pb-3 font-medium" style=style>
                                                    {col.label.clone()}
                                                </th>
                                            }
                                        }).collect_view()}
                                    </tr>
                                </thead>
                                <tbody>
                                    {items.into_iter().map(|item| {
                                        // 参照を使用して不要なクローンを回避
                                        let cols = &columns_clone;
                                        view! {
                                            <tr class="border-b border-slate-700/30 hover:bg-slate-800/50">
                                                {cols.iter().map(|col| {
                                                    let value = item.get(&col.key)
                                                        .map(|v| {
                                                            match v {
                                                                serde_json::Value::String(s) => s.clone(),
                                                                serde_json::Value::Number(n) => n.to_string(),
                                                                _ => v.to_string(),
                                                            }
                                                        })
                                                        .unwrap_or_else(|| "-".to_string());

                                                    // type列は特別なスタイリング
                                                    let is_type_col = col.key == "type";
                                                    let cell_class = if is_type_col {
                                                        get_type_badge_class(&value)
                                                    } else {
                                                        "text-dt-text-sub".to_string()
                                                    };

                                                    view! {
                                                        <td class="py-3">
                                                            {if is_type_col {
                                                                view! {
                                                                    <span class=format!("badge {}", cell_class)>
                                                                        {value}
                                                                    </span>
                                                                }.into_any()
                                                            } else {
                                                                view! {
                                                                    <span class=cell_class>
                                                                        {value}
                                                                    </span>
                                                                }.into_any()
                                                            }}
                                                        </td>
                                                    }
                                                }).collect_view()}
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        }.into_any()
                    }
                } else {
                    view! {
                        <div class="text-center text-dt-text-sub py-8">
                            <Icon name="info".to_string() class="w-12 h-12 mx-auto mb-3 opacity-50".to_string() />
                            <p>"No details schema defined"</p>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

/// type列のバッジクラスを取得
fn get_type_badge_class(type_value: &str) -> String {
    match type_value.to_uppercase().as_str() {
        "FIXME" => "badge-critical".to_string(),
        "TODO" => "badge-info".to_string(),
        "HACK" => "badge-warning".to_string(),
        "XXX" => "badge-warning".to_string(),
        "NOTE" => "badge-success".to_string(),
        "CRITICAL" => "badge-critical".to_string(),
        "WARNING" => "badge-warning".to_string(),
        _ => "badge".to_string(),
    }
}

/// 生出力ビュー（シグナルを使用）
#[component]
fn RawOutputViewInner(result: ReadSignal<Option<ToolResult>>) -> impl IntoView {
    let output = move || {
        result
            .get()
            .map(|r| {
                if !r.stdout.is_empty() {
                    r.stdout
                } else if !r.stderr.is_empty() {
                    r.stderr
                } else {
                    "No output".to_string()
                }
            })
            .unwrap_or_else(|| "No output".to_string())
    };

    view! {
        <div class="bg-slate-900 rounded-lg p-4 overflow-auto max-h-96">
            <pre class="font-mono text-sm text-slate-300 whitespace-pre-wrap">
                {output}
            </pre>
        </div>
    }
}
