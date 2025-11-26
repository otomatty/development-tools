//! Contribution graph component
//!
//! Displays GitHub-style contribution calendar (草グラフ).

use leptos::prelude::*;

use crate::types::GitHubStats;

/// Contribution graph component (GitHub草グラフ)
#[component]
pub fn ContributionGraph(
    github_stats: ReadSignal<Option<GitHubStats>>,
) -> impl IntoView {
    view! {
        <div class="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-success/20">
            <div class="flex items-center justify-between mb-4">
                <h3 class="text-xl font-gaming font-bold text-gm-success">
                    "📈 Contribution Graph"
                </h3>
                
                {move || {
                    github_stats.get().map(|stats| view! {
                        <div class="text-sm text-dt-text-sub">
                            <span class="text-gm-success font-bold">{stats.total_contributions}</span>
                            " contributions this year"
                        </div>
                    })
                }}
            </div>

            // Graph
            {move || {
                if let Some(stats) = github_stats.get() {
                    if let Some(calendar) = stats.contribution_calendar {
                        let weeks = calendar.weeks;
                        
                        // Take only last 52 weeks (1 year)
                        let weeks_len = weeks.len();
                        let display_weeks: Vec<_> = if weeks_len > 52 {
                            weeks.into_iter().skip(weeks_len - 52).collect()
                        } else {
                            weeks
                        };

                        view! {
                            <div class="overflow-x-auto">
                                <div class="flex gap-1 min-w-fit">
                                    {display_weeks.into_iter().map(|week| {
                                        view! {
                                            <div class="flex flex-col gap-1">
                                                {week.contribution_days.into_iter().map(|day| {
                                                    let intensity = get_intensity(day.contribution_count);
                                                    let bg_class = match intensity {
                                                        0 => "bg-gm-bg-secondary",
                                                        1 => "bg-gm-success/20",
                                                        2 => "bg-gm-success/40",
                                                        3 => "bg-gm-success/60",
                                                        _ => "bg-gm-success",
                                                    };
                                                    
                                                    view! {
                                                        <div
                                                            class=format!("w-3 h-3 rounded-sm {} hover:ring-1 hover:ring-gm-accent-cyan transition-all cursor-pointer", bg_class)
                                                            title=format!("{}: {} contributions", day.date, day.contribution_count)
                                                        />
                                                    }
                                                }).collect_view()}
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                                
                                // Legend
                                <div class="flex items-center justify-end gap-2 mt-4 text-xs text-dt-text-sub">
                                    <span>"Less"</span>
                                    <div class="w-3 h-3 rounded-sm bg-gm-bg-secondary"/>
                                    <div class="w-3 h-3 rounded-sm bg-gm-success/20"/>
                                    <div class="w-3 h-3 rounded-sm bg-gm-success/40"/>
                                    <div class="w-3 h-3 rounded-sm bg-gm-success/60"/>
                                    <div class="w-3 h-3 rounded-sm bg-gm-success"/>
                                    <span>"More"</span>
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

