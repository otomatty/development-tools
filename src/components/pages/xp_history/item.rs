//! XP History Item Component
//!
//! Individual XP history entry with expandable details.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   └─ src/components/pages/xp_history/mod.rs
//! Dependencies:
//!   ├─ src/components/icons.rs
//!   └─ src/types/gamification.rs
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

use leptos::prelude::*;

use super::utils::{
    format_absolute_time, format_relative_time, get_action_color_class, get_action_display_name,
    get_action_icon,
};
use crate::components::icons::Icon;
use crate::types::XpHistoryEntry;

/// XP History item component with accordion
#[component]
pub fn XpHistoryItem(entry: XpHistoryEntry) -> impl IntoView {
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

            // Expanded details
            <div
                class="overflow-hidden transition-all duration-200"
                class=("max-h-0", move || !expanded.get())
                class=("max-h-96", move || expanded.get())
            >
                <div class="px-4 pb-4 pt-0 border-t border-slate-700/30 bg-slate-800/20">
                    <div class="pt-4 space-y-3">
                        // Detail grid
                        <div class="grid grid-cols-2 gap-4 text-sm">
                            <div>
                                <div class="text-dt-text-sub text-xs mb-1">"アクションタイプ"</div>
                                <div class="text-dt-text font-mono">{action_type.clone()}</div>
                            </div>
                            <div>
                                <div class="text-dt-text-sub text-xs mb-1">"獲得XP"</div>
                                <div class="text-gm-success font-gaming-mono font-bold">
                                    "+" {xp_amount} " XP"
                                </div>
                            </div>
                            <div>
                                <div class="text-dt-text-sub text-xs mb-1">"取得日時"</div>
                                <div class="text-dt-text">{absolute_time}</div>
                            </div>
                            <div>
                                <div class="text-dt-text-sub text-xs mb-1">"履歴ID"</div>
                                <div class="text-dt-text font-mono text-xs">{entry_id}</div>
                            </div>
                        </div>

                        // XP Breakdown for github_sync
                        {if is_github_sync {
                            if let Some(ref bd) = breakdown {
                                Some(view! {
                                    <XpBreakdownSection breakdown=bd.clone() />
                                }.into_any())
                            } else {
                                Some(view! {
                                    <XpReferenceSection />
                                }.into_any())
                            }
                        } else {
                            None
                        }}

                        // Streak bonus explanation
                        {if is_streak_bonus {
                            Some(view! {
                                <StreakBonusSection xp_amount=xp_amount />
                            })
                        } else {
                            None
                        }}

                        // GitHub Event ID
                        {github_event_id.map(|event_id| view! {
                            <div>
                                <div class="text-dt-text-sub text-xs mb-1">"GitHub Event ID"</div>
                                <div class="text-dt-text font-mono text-xs break-all bg-slate-900/50 p-2 rounded">
                                    {event_id}
                                </div>
                            </div>
                        })}

                        // Description
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

/// XP Breakdown section for github_sync entries
#[component]
fn XpBreakdownSection(breakdown: crate::types::XpBreakdown) -> impl IntoView {
    let bd = breakdown;
    view! {
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
    }
}

/// XP Reference section (fallback when breakdown is not available)
#[component]
fn XpReferenceSection() -> impl IntoView {
    view! {
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
                </div>
            </div>
        </div>
    }
}

/// Streak bonus section
#[component]
fn StreakBonusSection(xp_amount: i32) -> impl IntoView {
    view! {
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
    }
}
