//! Badge grid component
//!
//! Displays earned badges and near-completion badges with progress.
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this component):
//!   └─ src/components/home/mod.rs
//!
//! Dependencies:
//!   ├─ src/components/ui/dialog/modal.rs
//!   ├─ src/components/animation_context.rs
//!   ├─ src/tauri_api.rs
//!   └─ src/types/gamification.rs

use leptos::prelude::*;

use crate::components::ui::dialog::{Modal, ModalBody, ModalHeader, ModalSize};
use crate::tauri_api;
use crate::types::BadgeWithProgress;

/// Badge grid component with progress information
#[component]
pub fn BadgeGrid() -> impl IntoView {
    // Fetch badges with progress
    let badges_resource =
        LocalResource::new(move || async move { tauri_api::get_badges_with_progress().await });

    // Selected badge for modal
    let (selected_badge, set_selected_badge) = signal(Option::<BadgeWithProgress>::None);

    view! {
        <div class="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-badge-gold/20">
            <Suspense fallback=move || {
                view! {
                    <div class="animate-pulse space-y-4">
                        <div class="h-6 w-32 bg-slate-700 rounded"></div>
                        <div class="grid grid-cols-4 gap-3">
                            {(0..8).map(|_| view! {
                                <div class="h-16 bg-slate-700 rounded-xl"></div>
                            }).collect_view()}
                        </div>
                    </div>
                }
            }>
                {move || {
                    badges_resource.get().map(|wrapped_result| {
                        match wrapped_result.take() {
                            Ok(badges) => {
                                let earned: Vec<BadgeWithProgress> = badges.iter()
                                    .filter(|b| b.earned)
                                    .cloned()
                                    .collect();
                                let near_completion: Vec<BadgeWithProgress> = badges.iter()
                                    .filter(|b| !b.earned && b.progress.as_ref().map(|p| p.progress_percent >= 50.0).unwrap_or(false))
                                    .cloned()
                                    .collect();
                                let total_count = badges.len();
                                let earned_count = earned.len();

                                // Pre-render sections
                                let earned_section = if !earned.is_empty() {
                                    let earned_len = earned.len();
                                    Some(view! {
                                        <div class="mb-6">
                                            <h4 class="text-sm font-bold text-gm-success mb-3 flex items-center gap-2">
                                                <span>"✓"</span>
                                                <span>"Unlocked"</span>
                                                <span class="text-dt-text-sub font-normal">
                                                    {format!("({})", earned_len)}
                                                </span>
                                            </h4>
                                            <div class="grid grid-cols-5 gap-3">
                                                {earned.into_iter().map(|badge| {
                                                    let badge_for_click = badge.clone();
                                                    view! {
                                                        <BadgeItem
                                                            badge=badge
                                                            on_click=move |_| {
                                                                set_selected_badge.set(Some(badge_for_click.clone()));
                                                            }
                                                        />
                                                    }
                                                }).collect_view()}
                                            </div>
                                        </div>
                                    })
                                } else {
                                    None
                                };

                                let near_completion_section = if !near_completion.is_empty() {
                                    let near_completion_len = near_completion.len();
                                    Some(view! {
                                        <div>
                                            <h4 class="text-sm font-bold text-gm-accent-cyan mb-3 flex items-center gap-2">
                                                <span>"🎯"</span>
                                                <span>"Almost There"</span>
                                                <span class="text-dt-text-sub font-normal">
                                                    {format!("({})", near_completion_len)}
                                                </span>
                                            </h4>
                                            <div class="space-y-2">
                                                {near_completion.into_iter().map(|badge| {
                                                    let badge_for_click = badge.clone();
                                                    view! {
                                                        <NearCompletionBadgeItem
                                                            badge=badge
                                                            on_click=move |_| {
                                                                set_selected_badge.set(Some(badge_for_click.clone()));
                                                            }
                                                        />
                                                    }
                                                }).collect_view()}
                                            </div>
                                        </div>
                                    })
                                } else {
                                    None
                                };

                                let empty_section = if earned_section.is_none() && near_completion_section.is_none() {
                                    Some(view! {
                                        <div class="text-center py-8 text-dt-text-sub">
                                            <span class="text-4xl mb-2 block">"🏅"</span>
                                            <p>"No badges yet. Keep coding to earn your first badge!"</p>
                                        </div>
                                    })
                                } else {
                                    None
                                };

                                view! {
                                    // Header
                                    <div class="flex items-center justify-between mb-4">
                                        <h3 class="text-xl font-gaming font-bold text-badge-gold">
                                            "🏅 Badges"
                                        </h3>
                                        <span class="text-sm text-dt-text-sub">
                                            {format!("{} / {} unlocked", earned_count, total_count)}
                                        </span>
                                    </div>

                                    {earned_section}
                                    {near_completion_section}
                                    {empty_section}
                                }.into_any()
                            },
                            Err(e) => {
                                view! {
                                    <div class="text-center py-4 text-gm-error">
                                        {format!("Failed to load badges: {}", e)}
                                    </div>
                                }.into_any()
                            }
                        }
                    })
                }}
            </Suspense>
        </div>

        // Badge detail modal
        <BadgeDetailModal
            badge_info=selected_badge
            on_close=move || set_selected_badge.set(None)
        />
    }
}

/// Individual earned badge item (compact)
#[component]
fn BadgeItem<F>(badge: BadgeWithProgress, on_click: F) -> impl IntoView
where
    F: Fn(leptos::ev::MouseEvent) + 'static + Clone,
{
    let rarity_class = match badge.rarity.as_str() {
        "bronze" => "border-badge-bronze text-badge-bronze",
        "silver" => "border-badge-silver text-badge-silver",
        "gold" => "border-badge-gold text-badge-gold shadow-neon-cyan",
        "platinum" => "border-badge-platinum text-badge-platinum shadow-neon-purple",
        _ => "border-slate-600 text-slate-400",
    };

    let title = format!("{}: {}", badge.name, badge.description);

    view! {
        <div
            class=format!(
                "relative p-3 rounded-xl border-2 {} transition-all duration-200 hover:scale-105 cursor-pointer group",
                rarity_class
            )
            title=title
            on:click=on_click
        >
            // Badge icon
            <div class="text-center">
                <span class="text-2xl">{badge.icon.clone()}</span>
            </div>

            // Badge name (on hover)
            <div class="absolute inset-0 flex items-center justify-center bg-gm-bg-card/90 rounded-xl opacity-0 group-hover:opacity-100 transition-opacity p-2">
                <div class="text-center">
                    <div class="text-xs font-bold text-white truncate">{badge.name.clone()}</div>
                </div>
            </div>
        </div>
    }
}

/// Near completion badge item with progress bar
#[component]
fn NearCompletionBadgeItem<F>(badge: BadgeWithProgress, on_click: F) -> impl IntoView
where
    F: Fn(leptos::ev::MouseEvent) + 'static + Clone,
{
    let rarity_class = match badge.rarity.as_str() {
        "bronze" => "border-badge-bronze/50",
        "silver" => "border-badge-silver/50",
        "gold" => "border-badge-gold/50",
        "platinum" => "border-badge-platinum/50",
        _ => "border-slate-600/50",
    };

    let progress_bar_class = match badge.rarity.as_str() {
        "bronze" => "bg-badge-bronze",
        "silver" => "bg-badge-silver",
        "gold" => "bg-badge-gold",
        "platinum" => "bg-badge-platinum",
        _ => "bg-slate-600",
    };

    let progress = badge.progress.as_ref();
    let progress_percent = progress.map(|p| p.progress_percent).unwrap_or(0.0);
    let current_value = progress.map(|p| p.current_value).unwrap_or(0);
    let target_value = progress.map(|p| p.target_value).unwrap_or(0);

    view! {
        <div
            class=format!(
                "flex items-center gap-3 p-3 rounded-xl border {} bg-slate-800/30 hover:bg-slate-800/50 transition-colors cursor-pointer",
                rarity_class
            )
            on:click=on_click
        >
            // Badge icon
            <div class="flex-shrink-0 opacity-60">
                <span class="text-2xl">{badge.icon.clone()}</span>
            </div>

            // Badge info and progress
            <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between mb-1">
                    <span class="text-sm font-medium text-white truncate">{badge.name.clone()}</span>
                    <span class="text-xs text-dt-text-sub ml-2">
                        {format!("{}/{}", current_value, target_value)}
                    </span>
                </div>

                // Progress bar
                <div class="h-1.5 bg-slate-700 rounded-full overflow-hidden">
                    <div
                        class=format!("h-full {} transition-all duration-300", progress_bar_class)
                        style=format!("width: {}%", progress_percent.min(100.0))
                    ></div>
                </div>
            </div>

            // Progress percentage
            <div class="flex-shrink-0 text-sm font-bold text-gm-accent-cyan">
                {format!("{}%", progress_percent.round() as i32)}
            </div>
        </div>
    }
}

/// Badge detail modal
#[component]
fn BadgeDetailModal<F>(
    badge_info: ReadSignal<Option<BadgeWithProgress>>,
    on_close: F,
) -> impl IntoView
where
    F: Fn() + 'static + Clone + Send + Sync,
{
    // Create a derived signal for visibility
    let visible = Memo::new(move |_| badge_info.get().is_some());

    // Store values for use in ChildrenFn closure
    let on_close_stored = StoredValue::new(on_close.clone());
    let badge_info_stored = StoredValue::new(badge_info);

    // Get border class based on current badge
    let border_class = move || {
        badge_info
            .get()
            .map(|b| match b.rarity.as_str() {
                "bronze" => "border-2 border-badge-bronze",
                "silver" => "border-2 border-badge-silver",
                "gold" => "border-2 border-badge-gold",
                "platinum" => "border-2 border-badge-platinum",
                _ => "border border-slate-600",
            })
            .unwrap_or("border border-slate-600")
            .to_string()
    };

    view! {
        <Modal
            visible=visible
            on_close=on_close.clone()
            size=ModalSize::Small
            border_class=border_class()
        >
            {move || {
                let badge_signal = badge_info_stored.get_value();
                let on_close_fn = on_close_stored.get_value();

                badge_signal.get().map(|badge| {
                    let text_class = match badge.rarity.as_str() {
                        "bronze" => "text-badge-bronze",
                        "silver" => "text-badge-silver",
                        "gold" => "text-badge-gold",
                        "platinum" => "text-badge-platinum",
                        _ => "text-slate-400",
                    };

                    let progress_bar_class = match badge.rarity.as_str() {
                        "bronze" => "bg-badge-bronze",
                        "silver" => "bg-badge-silver",
                        "gold" => "bg-badge-gold",
                        "platinum" => "bg-badge-platinum",
                        _ => "bg-slate-600",
                    };

                    let category_label = match badge.badge_type.as_str() {
                        "milestone" => "🏁 Milestone",
                        "streak" => "🔥 Streak",
                        "consistency" => "📅 Consistency",
                        "collaboration" => "🤝 Collaboration",
                        "quality" => "✨ Quality",
                        "challenge" => "🎯 Challenge",
                        "level" => "⭐ Level",
                        "stars" => "🌟 Stars",
                        "language" => "🌍 Language",
                        _ => "📌 Other",
                    };

                    let progress = badge.progress.clone();
                    let earned = badge.earned;
                    let on_close_callback = Callback::new(move |_: ()| on_close_fn());

                    view! {
                        <ModalHeader on_close=on_close_callback>
                            <h3 class=format!("text-xl font-gaming font-bold {}", text_class)>
                                {badge.name.clone()}
                            </h3>
                        </ModalHeader>
                        <ModalBody class="text-center">
                            <div class="space-y-4">
                                // Badge icon
                                <div class=if earned { "" } else { "opacity-50 grayscale" }>
                                    <span class="text-7xl">{badge.icon.clone()}</span>
                                </div>

                                // Description
                                <p class="text-dt-text-sub">
                                    {badge.description.clone()}
                                </p>

                                // Category and rarity
                                <div class="flex justify-center gap-4 text-sm">
                                    <span class="px-3 py-1 rounded-full bg-slate-800/50 text-dt-text-sub">
                                        {category_label}
                                    </span>
                                    <span class=format!(
                                        "px-3 py-1 rounded-full bg-slate-800/50 uppercase font-bold {}",
                                        text_class
                                    )>
                                        {badge.rarity.clone()}
                                    </span>
                                </div>

                                // Status and progress
                                <div class="pt-2">
                                    {if earned {
                                        view! {
                                            <div class="flex items-center justify-center gap-2 text-gm-success">
                                                <span>"✓"</span>
                                                <span class="font-bold">"Unlocked!"</span>
                                            </div>
                                        }.into_any()
                                    } else if let Some(prog) = progress {
                                        let progress_bar_class = progress_bar_class.to_string();
                                        view! {
                                            <div class="space-y-2">
                                                <div class="flex justify-between text-sm">
                                                    <span class="text-dt-text-sub">"Progress"</span>
                                                    <span class="text-white font-medium">
                                                        {format!("{}/{}", prog.current_value, prog.target_value)}
                                                    </span>
                                                </div>
                                                <div class="h-2 bg-slate-700 rounded-full overflow-hidden">
                                                    <div
                                                        class=format!("h-full {} transition-all duration-500", progress_bar_class)
                                                        style=format!("width: {}%", prog.progress_percent.min(100.0))
                                                    ></div>
                                                </div>
                                                <div class="text-sm text-gm-accent-cyan font-bold">
                                                    {format!("{}% complete", prog.progress_percent.round() as i32)}
                                                </div>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="text-dt-text-sub italic">
                                                "Not yet unlocked"
                                            </div>
                                        }.into_any()
                                    }}
                                </div>
                            </div>
                        </ModalBody>
                    }
                })
            }}
        </Modal>
    }
}
