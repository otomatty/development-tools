//! XP History Page
//!
//! Displays the user's XP acquisition history.
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this component):
//!   └─ src/app.rs
//! Related Documentation:
//!   ├─ Spec: ./xp_history.spec.md
//!   └─ Types: src/types/gamification.rs

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::icons::Icon;
use crate::tauri_api;
use crate::types::{AppPage, XpHistoryEntry};

/// Get icon for action type
fn get_action_icon(action_type: &str) -> &'static str {
    match action_type {
        "commit" => "📝",
        "pull_request" => "🔀",
        "pull_request_merged" => "✅",
        "review" => "👀",
        "issue" => "📋",
        "issue_closed" => "✔️",
        "streak_bonus" => "🔥",
        "star" => "⭐",
        _ => "💫",
    }
}

/// Get display name for action type
fn get_action_display_name(action_type: &str) -> &'static str {
    match action_type {
        "commit" => "コミット",
        "pull_request" => "PR作成",
        "pull_request_merged" => "PRマージ",
        "review" => "レビュー",
        "issue" => "Issue作成",
        "issue_closed" => "Issueクローズ",
        "streak_bonus" => "ストリークボーナス",
        "star" => "スター獲得",
        _ => "その他",
    }
}

/// Get color class for action type
fn get_action_color_class(action_type: &str) -> &'static str {
    match action_type {
        "commit" => "text-blue-400",
        "pull_request" => "text-purple-400",
        "pull_request_merged" => "text-green-400",
        "review" => "text-yellow-400",
        "issue" => "text-orange-400",
        "issue_closed" => "text-emerald-400",
        "streak_bonus" => "text-red-400",
        "star" => "text-amber-400",
        _ => "text-gray-400",
    }
}

/// Format relative time from ISO8601 string
fn format_relative_time(created_at: &str) -> String {
    // Parse the date (handle both RFC3339 and simple date format)
    let now = js_sys::Date::new_0();
    let today = js_sys::Date::new_0();
    today.set_hours(0);
    today.set_minutes(0);
    today.set_seconds(0);
    today.set_milliseconds(0);

    let created_date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(created_at));
    let created_time = created_date.get_time();

    if created_time.is_nan() {
        return "不明".to_string();
    }

    let diff_ms = now.get_time() - created_time;
    let diff_days = (diff_ms / (1000.0 * 60.0 * 60.0 * 24.0)).floor() as i32;

    if diff_days == 0 {
        "今日".to_string()
    } else if diff_days == 1 {
        "昨日".to_string()
    } else if diff_days < 7 {
        format!("{}日前", diff_days)
    } else if diff_days < 30 {
        format!("{}週間前", diff_days / 7)
    } else if diff_days < 365 {
        format!("{}ヶ月前", diff_days / 30)
    } else {
        format!("{}年前", diff_days / 365)
    }
}

/// Format absolute time from ISO8601 string
fn format_absolute_time(created_at: &str) -> String {
    let created_date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(created_at));
    let created_time = created_date.get_time();

    if created_time.is_nan() {
        return "不明".to_string();
    }

    let year = created_date.get_full_year() as i32;
    let month = created_date.get_month() as i32 + 1; // 0-indexed
    let day = created_date.get_date() as i32;
    let hours = created_date.get_hours() as i32;
    let minutes = created_date.get_minutes() as i32;

    format!(
        "{}/{:02}/{:02} {:02}:{:02}",
        year,
        month,
        day,
        hours,
        minutes
    )
}

/// Get XP breakdown explanation for action type
fn get_xp_explanation(action_type: &str) -> Option<Vec<(&'static str, &'static str, i32)>> {
    match action_type {
        "github_sync" => {
            // For github_sync, we can show the possible breakdown based on XP rules
            Some(vec![
                ("📝", "コミット", 10),
                ("🔀", "PR作成", 25),
                ("✅", "PRマージ", 50),
                ("📋", "Issue作成", 5),
                ("✔️", "Issueクローズ", 10),
                ("👀", "レビュー", 15),
                ("⭐", "スター", 5),
            ])
        }
        "commit" => Some(vec![("📝", "コミット", 10)]),
        "pull_request" => Some(vec![("🔀", "PR作成", 25)]),
        "pull_request_merged" => Some(vec![("✅", "PRマージ", 50)]),
        "review" => Some(vec![("👀", "レビュー", 15)]),
        "issue" => Some(vec![("📋", "Issue作成", 5)]),
        "issue_closed" => Some(vec![("✔️", "Issueクローズ", 10)]),
        "star" => Some(vec![("⭐", "スター", 5)]),
        "streak_bonus" => None, // Streak bonus is percentage based
        _ => None,
    }
}

/// XP History item component with accordion
#[component]
fn XpHistoryItem(entry: XpHistoryEntry) -> impl IntoView {
    let (expanded, set_expanded) = signal(false);

    let icon = get_action_icon(&entry.action_type);
    let action_name = get_action_display_name(&entry.action_type);
    let color_class = get_action_color_class(&entry.action_type);
    let relative_time = format_relative_time(&entry.created_at);
    let absolute_time = format_absolute_time(&entry.created_at);
    let xp_amount = entry.xp_amount;
    let description = entry.description.clone();
    let description_for_expanded = entry.description.clone();
    let action_type = entry.action_type.clone();
    let action_type_for_breakdown = entry.action_type.clone();
    let github_event_id = entry.github_event_id.clone();
    let entry_id = entry.id;
    let is_github_sync = entry.action_type == "github_sync";
    let is_streak_bonus = entry.action_type == "streak_bonus";
    let breakdown = entry.breakdown.clone();

    view! {
        <div class="bg-gm-bg-card/50 rounded-xl border border-slate-700/30 hover:border-gm-accent-cyan/30 transition-all duration-200 overflow-hidden">
            // Main row (clickable)
            <button
                class="w-full flex items-center gap-4 p-4 text-left cursor-pointer"
                on:click=move |_| set_expanded.update(|e| *e = !*e)
            >
                // Icon
                <div class="flex-shrink-0 w-12 h-12 flex items-center justify-center bg-slate-800/50 rounded-xl text-2xl">
                    {icon}
                </div>

                // Content
                <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2">
                        <span class=format!("font-medium {}", color_class)>
                            {action_name}
                        </span>
                        <span class="text-dt-text-sub text-sm">
                            {relative_time}
                        </span>
                    </div>
                    {description.clone().map(|desc| view! {
                        <p class="text-dt-text-sub text-sm mt-1 truncate">
                            {desc}
                        </p>
                    })}
                </div>

                // XP Amount
                <div class="flex-shrink-0 text-right">
                    <span class="text-xl font-gaming-mono font-bold text-gm-success">
                        "+" {xp_amount}
                    </span>
                    <span class="text-gm-accent-cyan text-sm ml-1">"XP"</span>
                </div>

                // Expand indicator
                <div class="flex-shrink-0 text-dt-text-sub transition-transform duration-200"
                    class=("rotate-180", move || expanded.get())
                >
                    <Icon name="chevron-down".to_string() class="w-5 h-5".to_string() />
                </div>
            </button>

            // Expanded details (using CSS for show/hide to avoid closure issues)
            <div
                class="overflow-hidden transition-all duration-200"
                class=("max-h-0", move || !expanded.get())
                class=("max-h-96", move || expanded.get())
            >
                <div class="px-4 pb-4 pt-0 border-t border-slate-700/30 bg-slate-800/20">
                    <div class="pt-4 space-y-3">
                        // Detail grid
                        <div class="grid grid-cols-2 gap-4 text-sm">
                            // Action Type
                            <div>
                                <div class="text-dt-text-sub text-xs mb-1">"アクションタイプ"</div>
                                <div class="text-dt-text font-mono">{action_type.clone()}</div>
                            </div>

                            // XP Amount
                            <div>
                                <div class="text-dt-text-sub text-xs mb-1">"獲得XP"</div>
                                <div class="text-gm-success font-gaming-mono font-bold">
                                    "+" {xp_amount} " XP"
                                </div>
                            </div>

                            // Absolute Time
                            <div>
                                <div class="text-dt-text-sub text-xs mb-1">"取得日時"</div>
                                <div class="text-dt-text">{absolute_time}</div>
                            </div>

                            // Entry ID
                            <div>
                                <div class="text-dt-text-sub text-xs mb-1">"履歴ID"</div>
                                <div class="text-dt-text font-mono text-xs">{entry_id}</div>
                            </div>
                        </div>

                        // XP Breakdown - show actual data if available, otherwise show reference
                        {if is_github_sync {
                            if let Some(ref bd) = breakdown {
                                // Show actual breakdown data from database
                                Some(view! {
                                    <div class="mt-4">
                                        <div class="text-dt-text-sub text-xs mb-2">"XP計算内訳"</div>
                                        <div class="bg-slate-900/50 rounded-lg p-3">
                                            <div class="grid grid-cols-2 sm:grid-cols-4 gap-2 text-xs">
                                                {(bd.commits_xp > 0).then(|| view! {
                                                    <div class="flex items-center gap-1.5 p-2 bg-slate-800/50 rounded">
                                                        <span>"📝"</span>
                                                        <span class="text-dt-text-sub">"コミット"</span>
                                                        <span class="text-gm-success font-mono ml-auto">"+" {bd.commits_xp}</span>
                                                    </div>
                                                })}
                                                {(bd.prs_created_xp > 0).then(|| view! {
                                                    <div class="flex items-center gap-1.5 p-2 bg-slate-800/50 rounded">
                                                        <span>"🔀"</span>
                                                        <span class="text-dt-text-sub">"PR作成"</span>
                                                        <span class="text-gm-success font-mono ml-auto">"+" {bd.prs_created_xp}</span>
                                                    </div>
                                                })}
                                                {(bd.prs_merged_xp > 0).then(|| view! {
                                                    <div class="flex items-center gap-1.5 p-2 bg-slate-800/50 rounded">
                                                        <span>"✅"</span>
                                                        <span class="text-dt-text-sub">"PRマージ"</span>
                                                        <span class="text-gm-success font-mono ml-auto">"+" {bd.prs_merged_xp}</span>
                                                    </div>
                                                })}
                                                {(bd.reviews_xp > 0).then(|| view! {
                                                    <div class="flex items-center gap-1.5 p-2 bg-slate-800/50 rounded">
                                                        <span>"👀"</span>
                                                        <span class="text-dt-text-sub">"レビュー"</span>
                                                        <span class="text-gm-success font-mono ml-auto">"+" {bd.reviews_xp}</span>
                                                    </div>
                                                })}
                                                {(bd.issues_created_xp > 0).then(|| view! {
                                                    <div class="flex items-center gap-1.5 p-2 bg-slate-800/50 rounded">
                                                        <span>"📋"</span>
                                                        <span class="text-dt-text-sub">"Issue作成"</span>
                                                        <span class="text-gm-success font-mono ml-auto">"+" {bd.issues_created_xp}</span>
                                                    </div>
                                                })}
                                                {(bd.issues_closed_xp > 0).then(|| view! {
                                                    <div class="flex items-center gap-1.5 p-2 bg-slate-800/50 rounded">
                                                        <span>"✔️"</span>
                                                        <span class="text-dt-text-sub">"Issueクローズ"</span>
                                                        <span class="text-gm-success font-mono ml-auto">"+" {bd.issues_closed_xp}</span>
                                                    </div>
                                                })}
                                                {(bd.stars_xp > 0).then(|| view! {
                                                    <div class="flex items-center gap-1.5 p-2 bg-slate-800/50 rounded">
                                                        <span>"⭐"</span>
                                                        <span class="text-dt-text-sub">"スター"</span>
                                                        <span class="text-gm-success font-mono ml-auto">"+" {bd.stars_xp}</span>
                                                    </div>
                                                })}
                                                {(bd.streak_bonus_xp > 0).then(|| view! {
                                                    <div class="flex items-center gap-1.5 p-2 bg-slate-800/50 rounded">
                                                        <span>"🔥"</span>
                                                        <span class="text-dt-text-sub">"ストリーク"</span>
                                                        <span class="text-gm-success font-mono ml-auto">"+" {bd.streak_bonus_xp}</span>
                                                    </div>
                                                })}
                                            </div>
                                            <div class="flex items-center justify-end gap-2 mt-2 pt-2 border-t border-slate-700/30">
                                                <span class="text-dt-text-sub text-xs">"合計"</span>
                                                <span class="text-gm-success font-gaming-mono font-bold">"+" {bd.total_xp} " XP"</span>
                                            </div>
                                        </div>
                                    </div>
                                }.into_any())
                            } else {
                                // Fallback: show XP rate reference when breakdown data is not available
                                Some(view! {
                                    <div class="mt-4">
                                        <div class="text-dt-text-sub text-xs mb-2">"XP計算内訳（単価参考）"</div>
                                        <div class="bg-slate-900/50 rounded-lg p-3">
                                            <div class="grid grid-cols-2 sm:grid-cols-4 gap-2 text-xs">
                                                <div class="flex items-center gap-1.5 p-2 bg-slate-800/50 rounded">
                                                    <span>"📝"</span>
                                                    <span class="text-dt-text-sub">"コミット"</span>
                                                    <span class="text-gm-accent-cyan font-mono ml-auto">"10"</span>
                                                </div>
                                                <div class="flex items-center gap-1.5 p-2 bg-slate-800/50 rounded">
                                                    <span>"🔀"</span>
                                                    <span class="text-dt-text-sub">"PR作成"</span>
                                                    <span class="text-gm-accent-cyan font-mono ml-auto">"25"</span>
                                                </div>
                                                <div class="flex items-center gap-1.5 p-2 bg-slate-800/50 rounded">
                                                    <span>"✅"</span>
                                                    <span class="text-dt-text-sub">"PRマージ"</span>
                                                    <span class="text-gm-accent-cyan font-mono ml-auto">"50"</span>
                                                </div>
                                                <div class="flex items-center gap-1.5 p-2 bg-slate-800/50 rounded">
                                                    <span>"👀"</span>
                                                    <span class="text-dt-text-sub">"レビュー"</span>
                                                    <span class="text-gm-accent-cyan font-mono ml-auto">"15"</span>
                                                </div>
                                                <div class="flex items-center gap-1.5 p-2 bg-slate-800/50 rounded">
                                                    <span>"📋"</span>
                                                    <span class="text-dt-text-sub">"Issue作成"</span>
                                                    <span class="text-gm-accent-cyan font-mono ml-auto">"5"</span>
                                                </div>
                                                <div class="flex items-center gap-1.5 p-2 bg-slate-800/50 rounded">
                                                    <span>"✔️"</span>
                                                    <span class="text-dt-text-sub">"Issueクローズ"</span>
                                                    <span class="text-gm-accent-cyan font-mono ml-auto">"10"</span>
                                                </div>
                                                <div class="flex items-center gap-1.5 p-2 bg-slate-800/50 rounded">
                                                    <span>"⭐"</span>
                                                    <span class="text-dt-text-sub">"スター"</span>
                                                    <span class="text-gm-accent-cyan font-mono ml-auto">"5"</span>
                                                </div>
                                                <div class="flex items-center gap-1.5 p-2 bg-slate-800/50 rounded">
                                                    <span>"🔥"</span>
                                                    <span class="text-dt-text-sub">"ストリーク"</span>
                                                    <span class="text-gm-accent-cyan font-mono ml-auto">"%"</span>
                                                </div>
                                            </div>
                                            <p class="text-dt-text-sub text-xs mt-2 italic">
                                                "※ 過去の履歴のため詳細内訳は記録されていません"
                                            </p>
                                        </div>
                                    </div>
                                }.into_any())
                            }
                        } else {
                            None
                        }}

                        // Single action XP explanation (not github_sync)
                        {if !is_github_sync && !is_streak_bonus {
                            get_xp_explanation(&action_type_for_breakdown).map(|rules| {
                                let rule = rules.first();
                                rule.map(|(icon, name, unit_xp)| {
                                    let count = if *unit_xp > 0 { xp_amount / unit_xp } else { 0 };
                                    view! {
                                        <div class="mt-4">
                                            <div class="text-dt-text-sub text-xs mb-2">"XP計算"</div>
                                            <div class="bg-slate-900/50 rounded-lg p-3 flex items-center gap-3">
                                                <span class="text-2xl">{*icon}</span>
                                                <div class="flex-1">
                                                    <div class="text-dt-text font-medium">{*name}</div>
                                                    <div class="text-dt-text-sub text-xs">
                                                        {format!("{} × {} = {} XP", count, unit_xp, xp_amount)}
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    }
                            })
                        })
                    } else {
                        None
                    }}                        // Streak bonus explanation
                        {if is_streak_bonus {
                            Some(view! {
                                <div class="mt-4">
                                    <div class="text-dt-text-sub text-xs mb-2">"ストリークボーナス"</div>
                                    <div class="bg-slate-900/50 rounded-lg p-3">
                                        <div class="flex items-center gap-3">
                                            <span class="text-3xl">"🔥"</span>
                                            <div class="flex-1">
                                                <div class="text-gm-warning font-bold">
                                                    "+" {xp_amount} " XP"
                                                </div>
                                                <div class="text-dt-text-sub text-xs mt-1">
                                                    "連続活動日数に応じたボーナスXP"
                                                </div>
                                            </div>
                                        </div>
                                        <p class="text-dt-text-sub text-xs mt-2 italic">
                                            "※ 最大10日間で+10%のボーナス（累積）"
                                        </p>
                                    </div>
                                </div>
                            })
                        } else {
                            None
                        }}

                        // GitHub Event ID (if exists)
                        {github_event_id.map(|event_id| view! {
                            <div>
                                <div class="text-dt-text-sub text-xs mb-1">"GitHub Event ID"</div>
                                <div class="text-dt-text font-mono text-xs break-all bg-slate-900/50 p-2 rounded">
                                    {event_id}
                                </div>
                            </div>
                        })}

                        // Description (full, if exists)
                        {description_for_expanded.map(|desc| view! {
                            <div>
                                <div class="text-dt-text-sub text-xs mb-1">"詳細"</div>
                                <div class="text-dt-text text-sm bg-slate-900/50 p-2 rounded">
                                    {desc}
                                </div>
                            </div>
                        })}
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Loading skeleton for XP history items
#[component]
fn XpHistorySkeleton() -> impl IntoView {
    view! {
        <div class="space-y-3">
            {(0..5).map(|_| view! {
                <div class="flex items-center gap-4 p-4 bg-gm-bg-card/50 rounded-xl border border-slate-700/30 animate-pulse">
                    <div class="w-12 h-12 bg-slate-700/50 rounded-xl"></div>
                    <div class="flex-1 space-y-2">
                        <div class="h-4 bg-slate-700/50 rounded w-24"></div>
                        <div class="h-3 bg-slate-700/50 rounded w-48"></div>
                    </div>
                    <div class="h-6 bg-slate-700/50 rounded w-16"></div>
                </div>
            }).collect_view()}
        </div>
    }
}

/// Empty state component
#[component]
fn EmptyState() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center py-16 text-center">
            <div class="w-20 h-20 mb-6 flex items-center justify-center bg-slate-800/50 rounded-full text-4xl">
                "📜"
            </div>
            <h3 class="text-xl font-gaming font-bold text-dt-text mb-2">
                "まだ履歴がありません"
            </h3>
            <p class="text-dt-text-sub max-w-md">
                "GitHubで活動すると、ここにXP獲得履歴が表示されます。"
                <br />
                "コミット、PR作成、レビューなどでXPを獲得しましょう！"
            </p>
        </div>
    }
}

/// XP History Page component
#[component]
pub fn XpHistoryPage(set_current_page: WriteSignal<AppPage>) -> impl IntoView {
    // State
    let (xp_history, set_xp_history) = signal(Vec::<XpHistoryEntry>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);

    // Calculate total XP from history
    let total_xp = Memo::new(move |_| {
        xp_history.get().iter().map(|e| e.xp_amount).sum::<i32>()
    });

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

                    // Total XP badge (only show when not loading and has data)
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
