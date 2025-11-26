//! Badge grid component
//!
//! Displays earned and available badges.

use leptos::prelude::*;

use crate::types::{Badge, BadgeDefinition};

/// Badge grid component
#[component]
pub fn BadgeGrid(
    badges: ReadSignal<Vec<Badge>>,
    definitions: ReadSignal<Vec<BadgeDefinition>>,
) -> impl IntoView {
    view! {
        <div class="p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-badge-gold/20">
            <h3 class="text-xl font-gaming font-bold text-badge-gold mb-4">
                "🏅 Badges"
            </h3>

            <div class="grid grid-cols-4 gap-3">
                {move || {
                    let earned_badges = badges.get();
                    let all_definitions = definitions.get();
                    
                    all_definitions.into_iter().map(|def| {
                        let is_earned = earned_badges.iter().any(|b| b.badge_id == def.id);
                        
                        view! {
                            <BadgeItem
                                definition=def
                                earned=is_earned
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
    }
}

/// Individual badge item
#[component]
fn BadgeItem(
    definition: BadgeDefinition,
    earned: bool,
) -> impl IntoView {
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

