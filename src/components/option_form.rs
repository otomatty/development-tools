use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::icons::Icon;
use crate::tauri_api;
use crate::types::{OptionType, OptionValues, ToolOption};

/// オプションフォームコンポーネント
#[component]
pub fn OptionForm(
    options: Vec<ToolOption>,
    values: ReadSignal<OptionValues>,
    set_values: WriteSignal<OptionValues>,
) -> impl IntoView {
    view! {
        <div class="space-y-4">
            {options.into_iter().map(|opt| {
                view! {
                    <OptionField option=opt values=values set_values=set_values />
                }
            }).collect_view()}
        </div>
    }
}

/// 個別のオプションフィールド
#[component]
fn OptionField(
    option: ToolOption,
    values: ReadSignal<OptionValues>,
    set_values: WriteSignal<OptionValues>,
) -> impl IntoView {
    let opt_name = option.name.clone();
    let opt_type = option.option_type.clone();
    let opt_description = option.description.clone();
    let opt_placeholder = option.placeholder.clone();
    let opt_options = option.options.clone();
    let opt_default = option.default.clone();
    let opt_required = option.required;

    view! {
        <div class="space-y-1.5">
            <label class="flex items-center gap-2 text-sm font-medium text-dt-text">
                {opt_description.clone()}
                {if opt_required {
                    view! { <span class="text-dt-error text-xs">"*"</span> }.into_any()
                } else {
                    view! {}.into_any()
                }}
            </label>

            {match opt_type {
                OptionType::Boolean => {
                    let name = opt_name.clone();

                    view! {
                        <BooleanField
                            name=name
                            values=values
                            set_values=set_values
                            default=opt_default.and_then(|v| v.as_bool()).unwrap_or(false)
                        />
                    }.into_any()
                }
                OptionType::Select => {
                    let name = opt_name.clone();
                    let options = opt_options.unwrap_or_default();
                    let default = opt_default.and_then(|v| v.as_str().map(|s| s.to_string()));

                    view! {
                        <SelectField
                            name=name
                            values=values
                            set_values=set_values
                            options=options
                            default=default
                        />
                    }.into_any()
                }
                OptionType::Path => {
                    let name = opt_name.clone();
                    let placeholder = opt_placeholder.unwrap_or_default();
                    let description = opt_description.clone();
                    let path_type = option.path_type.clone();

                    view! {
                        <PathField
                            name=name
                            values=values
                            set_values=set_values
                            placeholder=placeholder
                            description=description
                            path_type=path_type
                        />
                    }.into_any()
                }
                OptionType::Number => {
                    let name = opt_name.clone();
                    let placeholder = opt_placeholder.unwrap_or_default();
                    let default = opt_default.and_then(|v| v.as_f64());

                    view! {
                        <NumberField
                            name=name
                            values=values
                            set_values=set_values
                            placeholder=placeholder
                            default=default
                        />
                    }.into_any()
                }
                OptionType::String => {
                    let name = opt_name.clone();
                    let placeholder = opt_placeholder.unwrap_or_default();

                    view! {
                        <StringField
                            name=name
                            values=values
                            set_values=set_values
                            placeholder=placeholder
                        />
                    }.into_any()
                }
            }}
        </div>
    }
}

/// ブール値フィールド（トグルスイッチ）
#[component]
fn BooleanField(
    name: String,
    values: ReadSignal<OptionValues>,
    set_values: WriteSignal<OptionValues>,
    default: bool,
) -> impl IntoView {
    let name_for_btn = name.clone();
    let name_for_knob = name.clone();
    let name_for_click = name.clone();

    view! {
        <button
            type="button"
            class=move || {
                let checked = values.get()
                    .get(&name_for_btn)
                    .and_then(|v| v.as_bool())
                    .unwrap_or(default);
                format!(
                    "relative inline-flex h-6 w-11 items-center rounded-full transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-dt-accent focus:ring-offset-2 focus:ring-offset-dt-bg {}",
                    if checked { "bg-dt-accent" } else { "bg-slate-700" }
                )
            }
            on:click={
                let name = name_for_click.clone();
                move |_| {
                    let current = values.get()
                        .get(&name)
                        .and_then(|v| v.as_bool())
                        .unwrap_or(default);
                    set_values.update(|v| {
                        v.insert(name.clone(), serde_json::Value::Bool(!current));
                    });
                }
            }
        >
            <span
                class=move || {
                    let checked = values.get()
                        .get(&name_for_knob)
                        .and_then(|v| v.as_bool())
                        .unwrap_or(default);
                    format!(
                        "inline-block h-4 w-4 transform rounded-full bg-white shadow-lg transition-transform duration-200 ease-in-out {}",
                        if checked { "translate-x-6" } else { "translate-x-1" }
                    )
                }
            />
        </button>
    }
}

/// 選択フィールド（ドロップダウン）
#[component]
fn SelectField(
    name: String,
    values: ReadSignal<OptionValues>,
    set_values: WriteSignal<OptionValues>,
    options: Vec<String>,
    default: Option<String>,
) -> impl IntoView {
    let name_for_change = name.clone();
    let name_for_select = name.clone();
    let default_for_select = default.clone();

    view! {
        <div class="relative">
            <select
                class="select-field pr-10"
                on:change={
                    let name = name_for_change.clone();
                    move |ev| {
                        let value = event_target_value(&ev);
                        set_values.update(|v| {
                            v.insert(name.clone(), serde_json::Value::String(value));
                        });
                    }
                }
            >
                {options.into_iter().map(|opt| {
                    let opt_display = opt.clone();
                    let opt_value = opt.clone();
                    let name_for_check = name_for_select.clone();
                    let default_for_check = default_for_select.clone();

                    let is_selected = move || {
                        let current = values.get()
                            .get(&name_for_check)
                            .and_then(|v| v.as_str().map(|s| s.to_string()))
                            .or_else(|| default_for_check.clone())
                            .unwrap_or_default();
                        current == opt_value
                    };

                    view! {
                        <option value=opt.clone() selected=is_selected>
                            {opt_display}
                        </option>
                    }
                }).collect_view()}
            </select>
            <div class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
                <svg class="w-4 h-4 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
                </svg>
            </div>
        </div>
    }
}

/// パスフィールド（ファイル/ディレクトリ選択）
#[component]
fn PathField(
    name: String,
    values: ReadSignal<OptionValues>,
    set_values: WriteSignal<OptionValues>,
    placeholder: String,
    description: String,
    path_type: Option<String>,
) -> impl IntoView {
    let name_for_value = name.clone();
    let name_for_input = name.clone();
    let name_for_click = name.clone();
    let placeholder_for_click = placeholder.clone();
    let description_for_click = description.clone();
    let path_type_for_click = path_type.clone();

    let current_value = move || {
        values
            .get()
            .get(&name_for_value)
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default()
    };

    // ブラウズボタンのクリックハンドラー
    let on_browse_click = move |_| {
        let name = name_for_click.clone();
        let placeholder = placeholder_for_click.clone();
        let description = description_for_click.clone();
        let path_type_opt = path_type_for_click.clone();

        // path_typeが指定されていればそれを使用、なければプレースホルダーから推測（デフォルトはdirectory）
        let path_type = path_type_opt.unwrap_or_else(|| {
            if placeholder.to_lowercase().contains("file") {
                "file".to_string()
            } else {
                "directory".to_string()
            }
        });

        spawn_local(async move {
            match tauri_api::select_path(&path_type, Some(&description), None).await {
                Ok(Some(selected_path)) => {
                    set_values.update(|v| {
                        v.insert(name.clone(), serde_json::Value::String(selected_path));
                    });
                }
                Ok(None) => {
                    // ユーザーがキャンセルした場合は何もしない
                }
                Err(e) => {
                    leptos::logging::error!("Failed to select path: {}", e);
                }
            }
        });
    };

    view! {
        <div class="flex gap-2">
            <input
                type="text"
                class="input-field flex-1"
                placeholder=placeholder
                prop:value=current_value
                on:input={
                    let name = name_for_input.clone();
                    move |ev| {
                        let value = event_target_value(&ev);
                        set_values.update(|v| {
                            v.insert(name.clone(), serde_json::Value::String(value));
                        });
                    }
                }
            />
            <button
                type="button"
                class="btn-secondary px-3"
                title="Browse"
                on:click=on_browse_click
            >
                <Icon name="folder".to_string() class="w-5 h-5".to_string() />
            </button>
        </div>
    }
}

/// 数値フィールド
#[component]
fn NumberField(
    name: String,
    values: ReadSignal<OptionValues>,
    set_values: WriteSignal<OptionValues>,
    placeholder: String,
    default: Option<f64>,
) -> impl IntoView {
    let name_for_value = name.clone();
    let name_for_input = name.clone();

    let current_value = move || {
        values
            .get()
            .get(&name_for_value)
            .and_then(|v| v.as_f64())
            .or(default)
            .map(|n| n.to_string())
            .unwrap_or_default()
    };

    view! {
        <input
            type="number"
            class="input-field"
            placeholder=placeholder
            prop:value=current_value
            on:input={
                let name = name_for_input.clone();
                move |ev| {
                    let value = event_target_value(&ev);
                    if let Ok(n) = value.parse::<f64>() {
                        set_values.update(|v| {
                            v.insert(name.clone(), serde_json::Value::Number(
                                serde_json::Number::from_f64(n).unwrap()
                            ));
                        });
                    }
                }
            }
        />
    }
}

/// 文字列フィールド
#[component]
fn StringField(
    name: String,
    values: ReadSignal<OptionValues>,
    set_values: WriteSignal<OptionValues>,
    placeholder: String,
) -> impl IntoView {
    let name_for_value = name.clone();
    let name_for_input = name.clone();

    let current_value = move || {
        values
            .get()
            .get(&name_for_value)
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default()
    };

    view! {
        <input
            type="text"
            class="input-field"
            placeholder=placeholder
            prop:value=current_value
            on:input={
                let name = name_for_input.clone();
                move |ev| {
                    let value = event_target_value(&ev);
                    set_values.update(|v| {
                        v.insert(name.clone(), serde_json::Value::String(value));
                    });
                }
            }
        />
    }
}
