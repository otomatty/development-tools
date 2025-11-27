//! Badge grid component
//!
//! Displays earned and available badges.

use leptos::prelude::*;

use crate::components::use_animation_context_or_default;
use crate::types::{Badge, BadgeDefinition};

/// Badge grid component
#[component]
pub fn BadgeGrid(
    badges: ReadSignal<Vec<Badge>>,
    definitions: ReadSignal<Vec<BadgeDefinition>>,
) -> impl IntoView {
    // Selected badge for modal
    let (selected_badge, set_selected_badge) = signal(Option::<(BadgeDefinition, bool)>::None);
    
    view! {
        <div class="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-badge-gold/20">
            <h3 class="text-xl font-gaming font-bold text-badge-gold mb-4">
                "🏅 Badges"
            </h3>

            // Badge count
            <div class="mb-4 text-sm text-dt-text-sub">
                {move || {
                    let earned_count = badges.get().len();
                    let total_count = definitions.get().len();
                    format!("{} / {} unlocked", earned_count, total_count)
                }}
            </div>

            <div class="grid grid-cols-4 gap-3">
                {move || {
                    let earned_badges = badges.get();
                    let all_definitions = definitions.get();
                    
                    all_definitions.into_iter().map(|def| {
                        let is_earned = earned_badges.iter().any(|b| b.badge_id == def.id);
                        let def_clone = def.clone();
                        
                        view! {
                            <BadgeItem
                                definition=def
                                earned=is_earned
                                on_click=move |_| {
                                    set_selected_badge.set(Some((def_clone.clone(), is_earned)));
                                }
                            />
                        }
                    }).collect_view()
                }}
            </div>

            // Empty state
            <Show when=move || definitions.get().is_empty()>
                <div class="text-center py-4 text-dt-text-sub">
                    "No badges defined yet"
                </div>
            </Show>
        </div>
        
        // Badge detail modal
        <BadgeDetailModal
            badge_info=selected_badge
            on_close=move || set_selected_badge.set(None)
        />
    }
}

/// Individual badge item
#[component]
fn BadgeItem<F>(
    definition: BadgeDefinition,
    earned: bool,
    on_click: F,
) -> impl IntoView 
where
    F: Fn(leptos::ev::MouseEvent) + 'static + Clone,
{
    let rarity_class = match definition.rarity.as_str() {
        "bronze" => "border-badge-bronze text-badge-bronze",
        "silver" => "border-badge-silver text-badge-silver",
        "gold" => "border-badge-gold text-badge-gold shadow-neon-cyan",
        "platinum" => "border-badge-platinum text-badge-platinum shadow-neon-purple",
        _ => "border-slate-600 text-slate-400",
    };

    let opacity_class = if earned { "" } else { "opacity-30 grayscale" };
    
    // Clone values before using in view
    let title = format!("{}: {}", definition.name, definition.description);
    let name = definition.name.clone();
    let rarity = definition.rarity.clone();
    let icon = definition.icon.clone();

    view! {
        <div
            class=format!(
                "relative p-3 rounded-xl border-2 {} {} transition-all duration-200 hover:scale-105 cursor-pointer group",
                rarity_class,
                opacity_class
            )
            title=title
            on:click=on_click
        >
            // Badge icon
            <div class="text-center">
                <span class="text-3xl">{icon}</span>
            </div>

            // Badge name (on hover)
            <div class="absolute inset-0 flex items-center justify-center bg-gm-bg-card/90 rounded-xl opacity-0 group-hover:opacity-100 transition-opacity p-2">
                <div class="text-center">
                    <div class="text-xs font-bold text-white truncate">{name}</div>
                    <div class="text-[10px] text-dt-text-sub truncate">{rarity}</div>
                </div>
            </div>

            // Earned indicator
            <Show when=move || earned>
                <div class="absolute -top-1 -right-1 w-4 h-4 bg-gm-success rounded-full flex items-center justify-center">
                    <span class="text-[10px]">"✓"</span>
                </div>
            </Show>
        </div>
    }
}

/// Badge detail modal
#[component]
fn BadgeDetailModal<F>(
    badge_info: ReadSignal<Option<(BadgeDefinition, bool)>>,
    on_close: F,
) -> impl IntoView 
where
    F: Fn() + 'static + Clone + Send + Sync,
{
    // Get animation context with default
    let animation_ctx = use_animation_context_or_default();

    view! {
        <Show when=move || badge_info.get().is_some()>
            {
                let on_close = on_close.clone();
                move || {
                    let (def, earned) = badge_info.get().unwrap();
                    let on_close_overlay = on_close.clone();
                    let on_close_button = on_close.clone();
                    
                    let (border_class, text_class) = match def.rarity.as_str() {
                        "bronze" => ("border-badge-bronze", "text-badge-bronze"),
                        "silver" => ("border-badge-silver", "text-badge-silver"),
                        "gold" => ("border-badge-gold", "text-badge-gold"),
                        "platinum" => ("border-badge-platinum", "text-badge-platinum"),
                        _ => ("border-slate-600", "text-slate-400"),
                    };
                    
                    let category_label = match def.badge_type.as_str() {
                        "milestone" => "🏁 Milestone",
                        "streak" => "🔥 Streak",
                        "collaboration" => "🤝 Collaboration",
                        "quality" => "✨ Quality",
                        "challenge" => "🎯 Challenge",
                        _ => "📌 Other",
                    };
                    
                    view! {
                        // Overlay
                        <div 
                            class=move || format!("fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center {}", animation_ctx.get_animation_class("animate-fade-in"))
                            on:click=move |_| on_close_overlay()
                        >
                            // Modal content
                            <div 
                                class=move || format!(
                                    "relative p-6 bg-gm-bg-card rounded-2xl border-2 {} max-w-sm w-full mx-4 {}",
                                    border_class, animation_ctx.get_animation_class("animate-scale-in")
                                )
                                on:click=|ev| ev.stop_propagation()
                            >
                                // Close button
                                <button
                                    class="absolute top-4 right-4 text-dt-text-sub hover:text-white transition-colors"
                                    on:click=move |_| on_close_button()
                                >
                                    "✕"
                                </button>
                                
                                // Content
                                <div class="text-center space-y-4">
                                    // Badge icon
                                    <div class=if earned { "" } else { "opacity-50 grayscale" }>
                                        <span class="text-7xl">{def.icon.clone()}</span>
                                    </div>
                                    
                                    // Badge name
                                    <h3 class=format!("text-2xl font-gaming font-bold {}", text_class)>
                                        {def.name.clone()}
                                    </h3>
                                    
                                    // Description
                                    <p class="text-dt-text-sub">
                                        {def.description.clone()}
                                    </p>
                                    
                                    // Category and rarity
                                    <div class="flex justify-center gap-4 text-sm">
                                        <span class="px-3 py-1 rounded-full bg-slate-800/50 text-dt-text-sub">
                                            {category_label}
                                        </span>
                                        <span class=format!("px-3 py-1 rounded-full bg-slate-800/50 uppercase font-bold {}", text_class)>
                                            {def.rarity.clone()}
                                        </span>
                                    </div>
                                    
                                    // Status
                                    <div class="pt-2">
                                        {if earned {
                                            view! {
                                                <div class="flex items-center justify-center gap-2 text-gm-success">
                                                    <span>"✓"</span>
                                                    <span class="font-bold">"Unlocked!"</span>
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
                            </div>
                        </div>
                    }
                }
            }
        </Show>
    }
}

