//! Contribution graph component
//!
//! Displays GitHub-style contribution calendar (草グラフ) with hover cards
//! showing daily code statistics (additions/deletions).

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::tachys::view::any_view::AnyView;

use crate::tauri_api;
use crate::types::{CodeStatsResponse, DailyCodeStats, GitHubStats, RateLimitInfo};

/// Contribution graph component (GitHub草グラフ)
#[component]
pub fn ContributionGraph(
    github_stats: ReadSignal<Option<GitHubStats>>,
) -> impl IntoView {
    // コード統計データ
    let (code_stats, set_code_stats) = signal::<Option<CodeStatsResponse>>(None);
    let (is_loading_stats, set_is_loading_stats) = signal(false);
    let (is_syncing, set_is_syncing) = signal(false);
    let (rate_limit, set_rate_limit) = signal::<Option<RateLimitInfo>>(None);
    let (sync_error, set_sync_error) = signal::<Option<String>>(None);
    
    // ホバー状態
    let (hovered_date, set_hovered_date) = signal::<Option<String>>(None);
    let (hover_position, set_hover_position) = signal::<(i32, i32)>((0, 0));
    
    // 表示モード（コントリビューション or コード行数）
    let (show_code_lines, set_show_code_lines) = signal(false);
    
    // 自動同期中フラグ
    let (is_auto_syncing, set_is_auto_syncing) = signal(false);
    
    // 初回読み込み時にコード統計を取得
    Effect::new(move |_| {
        if github_stats.get().is_some() && code_stats.get().is_none() && !is_loading_stats.get() {
            set_is_loading_stats.set(true);
            spawn_local(async move {
                // まずキャッシュから取得を試みる
                match tauri_api::get_code_stats_summary("year").await {
                    Ok(stats) => {
                        set_code_stats.set(Some(stats));
                    }
                    Err(_e) => {
                        // キャッシュがない場合は自動同期をトリガー
                        set_is_auto_syncing.set(true);
                    }
                }
                // レート制限情報も取得
                if let Ok(info) = tauri_api::get_rate_limit_info().await {
                    set_rate_limit.set(Some(info));
                }
                set_is_loading_stats.set(false);
            });
        }
    });
    
    // 自動同期（キャッシュがない場合またはキャッシュが古い場合）
    Effect::new(move |_| {
        // キャッシュがなく、自動同期が必要な場合
        if is_auto_syncing.get() && !is_syncing.get() {
            // レート制限チェック
            let can_sync = rate_limit.get().map(|r| !r.is_critical).unwrap_or(true);
            if can_sync {
                set_is_syncing.set(true);
                spawn_local(async move {
                    match tauri_api::sync_code_stats().await {
                        Ok(_sync_result) => {
                            // 同期成功後、最新のデータを取得
                            if let Ok(stats) = tauri_api::get_code_stats_summary("year").await {
                                set_code_stats.set(Some(stats));
                            }
                        }
                        Err(e) => {
                            // 自動同期失敗時は警告のみ（エラー表示しない）
                            web_sys::console::warn_1(&format!("Auto-sync failed: {}", e).into());
                        }
                    }
                    // レート制限情報を更新
                    if let Ok(info) = tauri_api::get_rate_limit_info().await {
                        set_rate_limit.set(Some(info));
                    }
                    set_is_syncing.set(false);
                    set_is_auto_syncing.set(false);
                });
            } else {
                set_is_auto_syncing.set(false);
            }
        }
    });
    
    // コード統計を同期
    let on_sync_stats = move |_: leptos::ev::MouseEvent| {
        if is_syncing.get() {
            return;
        }
        
        // レート制限チェック - クリティカルな場合は警告を表示
        if let Some(info) = rate_limit.get() {
            if info.is_critical {
                set_sync_error.set(Some("⚠️ APIレート制限が残りわずかです。時間をおいてから再度お試しください。".to_string()));
                return;
            }
        }
        
        set_is_syncing.set(true);
        set_sync_error.set(None);
        spawn_local(async move {
            match tauri_api::sync_code_stats().await {
                Ok(_sync_result) => {
                    // 同期成功後、最新のデータを取得
                    if let Ok(stats) = tauri_api::get_code_stats_summary("year").await {
                        set_code_stats.set(Some(stats));
                    }
                    // 成功時はエラーをクリア
                    set_sync_error.set(None);
                }
                Err(e) => {
                    // エラーメッセージを解析してユーザーフレンドリーに変換
                    let error_msg = if e.contains("rate limit") || e.contains("API rate") {
                        "⚠️ GitHub APIのレート制限に達しました。1時間後にお試しください。".to_string()
                    } else if e.contains("Not logged in") || e.contains("token") {
                        "🔑 GitHubにログインしてください。".to_string()
                    } else if e.contains("network") || e.contains("connection") {
                        "🌐 ネットワーク接続を確認してください。".to_string()
                    } else {
                        format!("同期に失敗しました: {}", e)
                    };
                    set_sync_error.set(Some(error_msg));
                }
            }
            // レート制限情報を更新
            if let Ok(info) = tauri_api::get_rate_limit_info().await {
                set_rate_limit.set(Some(info));
            }
            set_is_syncing.set(false);
        });
    };
    
    // 日付でコード統計を検索
    let find_code_stats = move |date: &str| -> Option<DailyCodeStats> {
        code_stats.get().and_then(|stats| {
            stats.daily.iter().find(|d| d.date == date).cloned()
        })
    };
    
    view! {
        <div class="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-success/20 relative">
            <div class="flex items-center justify-between mb-4">
                <h3 class="text-xl font-gaming font-bold text-gm-success">
                    "📈 Contribution Graph"
                </h3>
                
                <div class="flex items-center gap-4">
                    // 表示切替ボタン
                    {move || {
                        if code_stats.get().is_some() {
                            view! {
                                <div class="flex items-center gap-2">
                                    <button
                                        class=move || format!(
                                            "px-3 py-1 text-xs rounded-lg transition-all {}",
                                            if !show_code_lines.get() {
                                                "bg-gm-success text-gm-bg-primary"
                                            } else {
                                                "bg-gm-bg-secondary text-dt-text-sub hover:bg-gm-bg-tertiary"
                                            }
                                        )
                                        on:click=move |_| set_show_code_lines.set(false)
                                    >
                                        "コントリビューション"
                                    </button>
                                    <button
                                        class=move || format!(
                                            "px-3 py-1 text-xs rounded-lg transition-all {}",
                                            if show_code_lines.get() {
                                                "bg-gm-accent-cyan text-gm-bg-primary"
                                            } else {
                                                "bg-gm-bg-secondary text-dt-text-sub hover:bg-gm-bg-tertiary"
                                            }
                                        )
                                        on:click=move |_| set_show_code_lines.set(true)
                                    >
                                        "コード行数"
                                    </button>
                                </div>
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }
                    }}
                    
                    // 同期ボタン
                    <button
                        class=move || {
                            let is_rate_limited = rate_limit.get().map(|r| r.is_critical).unwrap_or(false);
                            format!(
                                "px-3 py-1 text-xs rounded-lg transition-all flex items-center gap-1 {}",
                                if is_syncing.get() || is_loading_stats.get() || is_rate_limited {
                                    "bg-gm-bg-tertiary text-dt-text-sub cursor-not-allowed"
                                } else {
                                    "bg-gm-accent-purple text-white hover:bg-gm-accent-purple/80"
                                }
                            )
                        }
                        on:click=on_sync_stats
                        disabled=move || {
                            is_syncing.get() 
                                || is_loading_stats.get() 
                                || rate_limit.get().map(|r| r.is_critical).unwrap_or(false)
                        }
                        title=move || {
                            if rate_limit.get().map(|r| r.is_critical).unwrap_or(false) {
                                "APIレート制限のため同期できません"
                            } else {
                                "GitHubからコード統計を同期"
                            }
                        }
                    >
                        <span class=move || if is_syncing.get() { "animate-spin" } else { "" }>"🔄"</span>
                        {move || if is_syncing.get() { "同期中..." } else { "同期" }}
                    </button>
                    
                    // 年間合計表示
                    {move || {
                        github_stats.get().map(|stats| view! {
                            <div class="text-sm text-dt-text-sub">
                                <span class="text-gm-success font-bold">{stats.total_contributions}</span>
                                " contributions this year"
                            </div>
                        })
                    }}
                </div>
            </div>
            
            // 同期エラー表示
            <Show when=move || sync_error.get().is_some()>
                <div class="mb-4 p-2 bg-gm-error/20 border border-gm-error/50 rounded-lg text-gm-error text-sm">
                    {move || sync_error.get().unwrap_or_default()}
                </div>
            </Show>
            
            // レート制限情報表示
            {move || {
                rate_limit.get().map(|info| {
                    let rest_percent = info.rest_usage_percent();
                    let graphql_percent = info.graphql_usage_percent();
                    let is_warning = rest_percent > 80.0 || graphql_percent > 80.0;
                    let rest_class = if rest_percent > 80.0 { "font-bold" } else { "" };
                    let graphql_class = if graphql_percent > 80.0 { "font-bold" } else { "" };
                    let container_class = if is_warning { 
                        "mb-4 p-2 rounded-lg text-xs flex items-center gap-4 bg-gm-warning/20 text-gm-warning"
                    } else { 
                        "mb-4 p-2 rounded-lg text-xs flex items-center gap-4 bg-gm-bg-secondary text-dt-text-sub"
                    };
                    
                    view! {
                        <div class=container_class>
                            <span class="flex items-center gap-1">
                                "🔑 REST: "
                                <span class=rest_class>
                                    {info.rest_remaining}
                                </span>
                                "/"
                                {info.rest_limit}
                            </span>
                            <span class="flex items-center gap-1">
                                "🔷 GraphQL: "
                                <span class=graphql_class>
                                    {info.graphql_remaining}
                                </span>
                                "/"
                                {info.graphql_limit}
                            </span>
                            <Show when=move || is_warning>
                                <span class="text-gm-warning">"⚠️ APIリミットに注意"</span>
                            </Show>
                        </div>
                    }
                })
            }}

            // Graph
            {move || {
                if let Some(stats) = github_stats.get() {
                    if let Some(calendar) = stats.contribution_calendar {
                        let weeks = calendar.weeks.clone();
                        
                        // Take only last 52 weeks (1 year)
                        let weeks_len = weeks.len();
                        let display_weeks: Vec<_> = if weeks_len > 52 {
                            weeks.into_iter().skip(weeks_len - 52).collect()
                        } else {
                            weeks
                        };

                        view! {
                            <div class="overflow-x-auto">
                                // コード行数モードの場合は線グラフ、それ以外は草グラフ
                                {move || {
                                    if show_code_lines.get() {
                                        // 線グラフモード
                                        if let Some(code_stats_data) = code_stats.get() {
                                            code_lines_chart_view(
                                                code_stats_data,
                                                set_hovered_date,
                                                set_hover_position,
                                            )
                                        } else {
                                            view! {
                                                <div class="h-32 flex items-center justify-center text-dt-text-sub text-sm">
                                                    "コード統計を同期してください"
                                                </div>
                                            }.into_any()
                                        }
                                    } else {
                                        // 草グラフモード
                                        let weeks_for_view = display_weeks.clone();
                                        view! {
                                            <div class="flex gap-1 min-w-fit">
                                                {weeks_for_view.into_iter().map(|week| {
                                                    view! {
                                                        <div class="flex flex-col gap-1">
                                                            {week.contribution_days.into_iter().map(|day| {
                                                                let date = day.date.clone();
                                                                let date_for_hover = date.clone();
                                                                
                                                                // コントリビューションモード
                                                                let intensity = get_intensity(day.contribution_count);
                                                                let bg_class = match intensity {
                                                                    0 => "bg-gm-bg-secondary",
                                                                    1 => "bg-gm-success/20",
                                                                    2 => "bg-gm-success/40",
                                                                    3 => "bg-gm-success/60",
                                                                    _ => "bg-gm-success",
                                                                };
                                                                
                                                                let contribution_count = day.contribution_count;
                                                                
                                                                view! {
                                                                    <div
                                                                        class=format!("w-3 h-3 rounded-sm {} hover:ring-2 hover:ring-gm-accent-cyan transition-all cursor-pointer", bg_class)
                                                                        on:mouseenter=move |e| {
                                                                            set_hovered_date.set(Some(date_for_hover.clone()));
                                                                            let x = e.page_x();
                                                                            let y = e.page_y();
                                                                            set_hover_position.set((x, y));
                                                                        }
                                                                        on:mouseleave=move |_| {
                                                                            set_hovered_date.set(None);
                                                                        }
                                                                        title=format!("{}: {} contributions", date, contribution_count)
                                                                    />
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        }.into_any()
                                    }
                                }}
                                
                                // Legend
                                <div class="flex items-center justify-between mt-4">
                                    // コード統計サマリー（コード行数モード時）
                                    {move || {
                                        if show_code_lines.get() {
                                            if let Some(stats) = code_stats.get() {
                                                view! {
                                                    <div class="flex items-center gap-4 text-xs">
                                                        <span class="text-green-400">
                                                            <span class="font-bold">"+" {format_number(stats.monthly_total.additions)}</span>
                                                            " 追加"
                                                        </span>
                                                        <span class="text-red-400">
                                                            <span class="font-bold">"-" {format_number(stats.monthly_total.deletions)}</span>
                                                            " 削除"
                                                        </span>
                                                        <span class="text-dt-text-sub">
                                                            "(過去30日)"
                                                        </span>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! { <span></span> }.into_any()
                                            }
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }
                                    }}
                                    
                                    // カラーレジェンド（草グラフモードのみ）
                                    {move || {
                                        if !show_code_lines.get() {
                                            view! {
                                                <div class="flex items-center gap-2 text-xs text-dt-text-sub">
                                                    <span>"Less"</span>
                                                    <div class="w-3 h-3 rounded-sm bg-gm-bg-secondary"/>
                                                    <div class="w-3 h-3 rounded-sm bg-gm-success/20"/>
                                                    <div class="w-3 h-3 rounded-sm bg-gm-success/40"/>
                                                    <div class="w-3 h-3 rounded-sm bg-gm-success/60"/>
                                                    <div class="w-3 h-3 rounded-sm bg-gm-success"/>
                                                    <span>"More"</span>
                                                </div>
                                            }.into_any()
                                        } else {
                                            // 線グラフモードのレジェンド
                                            view! {
                                                <div class="flex items-center gap-4 text-xs text-dt-text-sub">
                                                    <span class="flex items-center gap-1">
                                                        <span class="w-3 h-0.5 bg-green-400 rounded"></span>
                                                        "追加"
                                                    </span>
                                                    <span class="flex items-center gap-1">
                                                        <span class="w-3 h-0.5 bg-red-400 rounded"></span>
                                                        "削除"
                                                    </span>
                                                </div>
                                            }.into_any()
                                        }
                                    }}
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="text-center py-8 text-dt-text-sub">
                                "No contribution data available"
                            </div>
                        }.into_any()
                    }
                } else {
                    view! {
                        <div class="text-center py-8 text-dt-text-sub">
                            <div class="animate-pulse">
                                "Loading contribution data..."
                            </div>
                        </div>
                    }.into_any()
                }
            }}
            
            // ホバーカード
            {move || {
                if let Some(date) = hovered_date.get() {
                    let (x, y) = hover_position.get();
                    let code_stat = find_code_stats(&date);
                    
                    // GitHub統計から該当日のコントリビューション数を取得
                    let contribution_count = github_stats.get()
                        .and_then(|s| s.contribution_calendar)
                        .and_then(|c| {
                            c.weeks.iter()
                                .flat_map(|w| &w.contribution_days)
                                .find(|d| d.date == date)
                                .map(|d| d.contribution_count)
                        })
                        .unwrap_or(0);
                    
                    view! {
                        <HoverCard
                            date=date
                            code_stats=code_stat
                            contribution_count=contribution_count
                            x=x
                            y=y
                        />
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}
        </div>
    }
}

/// ホバーカードコンポーネント
#[component]
fn HoverCard(
    date: String,
    code_stats: Option<DailyCodeStats>,
    contribution_count: i32,
    x: i32,
    y: i32,
) -> impl IntoView {
    // カード位置を計算（画面外にはみ出さないように調整）
    let card_style = format!(
        "position: fixed; left: {}px; top: {}px; transform: translate(-50%, -120%); z-index: 50;",
        x + 6, // セルの中心
        y
    );
    
    // 日付をフォーマット
    let formatted_date = format_date(&date);
    
    view! {
        <div
            class="bg-gm-bg-secondary/95 backdrop-blur-md border border-gm-success/30 rounded-lg shadow-xl p-3 min-w-48 pointer-events-none"
            style=card_style
        >
            // 日付ヘッダー
            <div class="text-sm font-medium text-gm-success mb-2 border-b border-gm-success/20 pb-1">
                {formatted_date}
            </div>
            
            // コントリビューション数
            <div class="flex items-center justify-between text-xs mb-1">
                <span class="text-dt-text-sub">"📊 コントリビューション"</span>
                <span class="font-bold text-gm-success">{format_number(contribution_count)}</span>
            </div>
            
            // コード統計（あれば表示）
            {move || {
                if let Some(ref stats) = code_stats {
                    let net = stats.net_change();
                    let net_class = if net >= 0 { "text-green-400" } else { "text-red-400" };
                    let net_sign = if net >= 0 { "+" } else { "" };
                    let additions_formatted = format_number(stats.additions);
                    let deletions_formatted = format_number(stats.deletions);
                    let net_formatted = format_number(net.abs());
                    let commits_formatted = format_number(stats.commits_count);
                    
                    view! {
                        <>
                            <div class="flex items-center justify-between text-xs mb-1">
                                <span class="text-dt-text-sub">"➕ 追加行"</span>
                                <span class="font-bold text-green-400">"+" {additions_formatted}</span>
                            </div>
                            <div class="flex items-center justify-between text-xs mb-1">
                                <span class="text-dt-text-sub">"➖ 削除行"</span>
                                <span class="font-bold text-red-400">"-" {deletions_formatted}</span>
                            </div>
                            <div class="flex items-center justify-between text-xs mb-1">
                                <span class="text-dt-text-sub">"📝 コミット"</span>
                                <span class="font-bold text-gm-accent-cyan">{commits_formatted}</span>
                            </div>
                            <div class="flex items-center justify-between text-xs border-t border-gm-success/20 pt-1 mt-1">
                                <span class="text-dt-text-sub">"📈 純増減"</span>
                                <span class=format!("font-bold {}", net_class)>
                                    {net_sign} {net_formatted}
                                </span>
                            </div>
                        </>
                    }.into_any()
                } else {
                    view! {
                        <div class="text-xs text-dt-text-sub italic">
                            "コード統計なし"
                        </div>
                    }.into_any()
                }
            }}
            
            // 下向き矢印
            <div class="absolute left-1/2 bottom-0 transform -translate-x-1/2 translate-y-full">
                <div class="w-0 h-0 border-l-4 border-r-4 border-t-4 border-l-transparent border-r-transparent border-t-gm-success/30"/>
            </div>
        </div>
    }
}

/// Calculate contribution intensity level (0-4)
fn get_intensity(count: i32) -> u8 {
    match count {
        0 => 0,
        1..=3 => 1,
        4..=6 => 2,
        7..=9 => 3,
        _ => 4,
    }
}

/// Format date string (YYYY-MM-DD) to Japanese format
fn format_date(date: &str) -> String {
    // Parse YYYY-MM-DD
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() == 3 {
        format!("{}年{}月{}日", parts[0], parts[1].trim_start_matches('0'), parts[2].trim_start_matches('0'))
    } else {
        date.to_string()
    }
}

/// Format number with thousand separators (e.g., 1234567 -> "1,234,567")
fn format_number(n: i32) -> String {
    let s = n.abs().to_string();
    let chars: Vec<char> = s.chars().rev().collect();
    let mut result = String::new();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }
    let formatted: String = result.chars().rev().collect();
    if n < 0 {
        format!("-{}", formatted)
    } else {
        formatted
    }
}

/// コード行数の線グラフビュー
fn code_lines_chart_view(
    code_stats: CodeStatsResponse,
    set_hovered_date: WriteSignal<Option<String>>,
    set_hover_position: WriteSignal<(i32, i32)>,
) -> AnyView {
    // 過去30日分のデータを取得（新しい順から古い順に並べ替え）
    let mut daily_data: Vec<_> = code_stats.daily.iter()
        .take(30)
        .cloned()
        .collect();
    daily_data.reverse(); // 古い順に並べ替え
    
    let data_len = daily_data.len();
    if data_len == 0 {
        return view! {
            <div class="h-32 flex items-center justify-center text-dt-text-sub text-sm">
                "データがありません"
            </div>
        }.into_any();
    }
    
    // グラフのサイズ
    let chart_width = 700.0_f64;
    let chart_height = 120.0_f64;
    let padding_left = 50.0_f64;
    let padding_right = 20.0_f64;
    let padding_top = 10.0_f64;
    let padding_bottom = 25.0_f64;
    
    let inner_width = chart_width - padding_left - padding_right;
    let inner_height = chart_height - padding_top - padding_bottom;
    
    // 最大値を計算
    let max_additions = daily_data.iter().map(|d| d.additions).max().unwrap_or(1).max(1);
    let max_deletions = daily_data.iter().map(|d| d.deletions).max().unwrap_or(1).max(1);
    let max_value = max_additions.max(max_deletions) as f64;
    
    // スケーリング関数
    let x_scale = |i: usize| -> f64 {
        padding_left + (i as f64 / (data_len - 1).max(1) as f64) * inner_width
    };
    
    let y_scale = |v: i32| -> f64 {
        padding_top + inner_height - (v as f64 / max_value) * inner_height
    };
    
    // 追加行のパスを生成（滑らかなベジェ曲線）
    let additions_path = generate_smooth_path(&daily_data, |d| d.additions, &x_scale, &y_scale);
    
    // 削除行のパスを生成（滑らかなベジェ曲線）
    let deletions_path = generate_smooth_path(&daily_data, |d| d.deletions, &x_scale, &y_scale);
    
    // グリッドライン
    let grid_lines: Vec<_> = (0..=4).map(|i| {
        let y = padding_top + (i as f64 / 4.0) * inner_height;
        let value = ((4 - i) as f64 / 4.0 * max_value) as i32;
        (y, value)
    }).collect();
    
    // X軸ラベル（日付）
    let x_labels: Vec<_> = daily_data.iter().enumerate()
        .filter(|(i, _)| i % 5 == 0 || *i == data_len - 1)
        .map(|(i, d)| {
            let x = x_scale(i);
            let date_parts: Vec<&str> = d.date.split('-').collect();
            let label = if date_parts.len() == 3 {
                format!("{}/{}", date_parts[1].trim_start_matches('0'), date_parts[2].trim_start_matches('0'))
            } else {
                d.date.clone()
            };
            (x, label)
        })
        .collect();
    
    // データポイント（ホバー用）
    let data_points: Vec<_> = daily_data.iter().enumerate().map(|(i, d)| {
        let x = x_scale(i);
        let add_y = y_scale(d.additions);
        let del_y = y_scale(d.deletions);
        (x, add_y, del_y, d.date.clone(), d.additions, d.deletions)
    }).collect();
    
    view! {
        <div class="relative">
            <svg
                width=format!("{}", chart_width)
                height=format!("{}", chart_height)
                class="overflow-visible"
            >
                // グリッドライン
                {grid_lines.iter().map(|(y, value)| {
                    let y_str = format!("{}", y);
                    let x1 = format!("{}", padding_left);
                    let x2 = format!("{}", chart_width - padding_right);
                    let label_x = format!("{}", padding_left - 5.0);
                    view! {
                        <g>
                            <line
                                x1=x1.clone()
                                y1=y_str.clone()
                                x2=x2
                                y2=y_str.clone()
                                stroke="currentColor"
                                stroke-opacity="0.1"
                                stroke-dasharray="4,4"
                            />
                            <text
                                x=label_x
                                y=y_str
                                fill="currentColor"
                                fill-opacity="0.5"
                                font-size="10"
                                text-anchor="end"
                                dominant-baseline="middle"
                            >
                                {format_number(*value)}
                            </text>
                        </g>
                    }
                }).collect_view()}
                
                // X軸ラベル
                {x_labels.iter().map(|(x, label)| {
                    let x_str = format!("{}", x);
                    let y_str = format!("{}", chart_height - 5.0);
                    let label_clone = label.clone();
                    view! {
                        <text
                            x=x_str
                            y=y_str
                            fill="currentColor"
                            fill-opacity="0.5"
                            font-size="9"
                            text-anchor="middle"
                        >
                            {label_clone}
                        </text>
                    }
                }).collect_view()}
                
                // 追加行の線（グラデーション付き）
                <defs>
                    <linearGradient id="additionsGradient" x1="0%" y1="0%" x2="0%" y2="100%">
                        <stop offset="0%" stop-color="#4ade80" stop-opacity="0.3"/>
                        <stop offset="100%" stop-color="#4ade80" stop-opacity="0.05"/>
                    </linearGradient>
                    <linearGradient id="deletionsGradient" x1="0%" y1="0%" x2="0%" y2="100%">
                        <stop offset="0%" stop-color="#f87171" stop-opacity="0.3"/>
                        <stop offset="100%" stop-color="#f87171" stop-opacity="0.05"/>
                    </linearGradient>
                </defs>
                
                // 追加行の線
                <path
                    d=additions_path.clone()
                    fill="none"
                    stroke="#4ade80"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                />
                
                // 削除行の線
                <path
                    d=deletions_path.clone()
                    fill="none"
                    stroke="#f87171"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                />
                
                // 各日のデータポイント（常に表示）
                {data_points.iter().map(|(x, add_y, del_y, date, additions, deletions)| {
                    let x_str = format!("{}", x);
                    let add_y_str = format!("{}", add_y);
                    let del_y_str = format!("{}", del_y);
                    let date_clone = date.clone();
                    let date_clone2 = date.clone();
                    let additions_val = *additions;
                    let deletions_val = *deletions;
                    
                    view! {
                        <g>
                            // 追加行のポイント（常に表示、ホバーで拡大）
                            <circle
                                cx=x_str.clone()
                                cy=add_y_str
                                r="3"
                                fill="#4ade80"
                                stroke="#166534"
                                stroke-width="1"
                                class="hover:r-5 cursor-pointer transition-all"
                                style="transition: r 0.15s ease-out;"
                                on:mouseenter=move |e| {
                                    set_hovered_date.set(Some(date_clone.clone()));
                                    let x = e.page_x();
                                    let y = e.page_y();
                                    set_hover_position.set((x, y));
                                }
                                on:mouseleave=move |_| {
                                    set_hovered_date.set(None);
                                }
                            >
                                <title>{format!("+{} 追加", format_number(additions_val))}</title>
                            </circle>
                            // 削除行のポイント（常に表示、ホバーで拡大）
                            <circle
                                cx=x_str
                                cy=del_y_str
                                r="3"
                                fill="#f87171"
                                stroke="#991b1b"
                                stroke-width="1"
                                class="hover:r-5 cursor-pointer transition-all"
                                style="transition: r 0.15s ease-out;"
                                on:mouseenter=move |e| {
                                    set_hovered_date.set(Some(date_clone2.clone()));
                                    let x = e.page_x();
                                    let y = e.page_y();
                                    set_hover_position.set((x, y));
                                }
                                on:mouseleave=move |_| {
                                    set_hovered_date.set(None);
                                }
                            >
                                <title>{format!("-{} 削除", format_number(deletions_val))}</title>
                            </circle>
                        </g>
                    }
                }).collect_view()}
            </svg>
        </div>
    }.into_any()
}

/// 滑らかなベジェ曲線パスを生成
fn generate_smooth_path<F>(
    data: &[DailyCodeStats],
    value_fn: F,
    x_scale: &impl Fn(usize) -> f64,
    y_scale: &impl Fn(i32) -> f64,
) -> String
where
    F: Fn(&DailyCodeStats) -> i32,
{
    if data.is_empty() {
        return String::new();
    }
    
    let points: Vec<(f64, f64)> = data.iter()
        .enumerate()
        .map(|(i, d)| (x_scale(i), y_scale(value_fn(d))))
        .collect();
    
    if points.len() == 1 {
        return format!("M {} {}", points[0].0, points[0].1);
    }
    
    let mut path = format!("M {} {}", points[0].0, points[0].1);
    
    // カットムル・ロム スプラインからベジェ曲線への変換
    for i in 0..points.len() - 1 {
        let p0 = if i > 0 { points[i - 1] } else { points[i] };
        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = if i + 2 < points.len() { points[i + 2] } else { points[i + 1] };
        
        // コントロールポイントを計算（張力 = 0.5）
        let tension = 0.5;
        let cp1x = p1.0 + (p2.0 - p0.0) * tension / 3.0;
        let cp1y = p1.1 + (p2.1 - p0.1) * tension / 3.0;
        let cp2x = p2.0 - (p3.0 - p1.0) * tension / 3.0;
        let cp2y = p2.1 - (p3.1 - p1.1) * tension / 3.0;
        
        path.push_str(&format!(" C {} {} {} {} {} {}", cp1x, cp1y, cp2x, cp2y, p2.0, p2.1));
    }
    
    path
}

