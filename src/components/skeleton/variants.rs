//! Skeleton variants
//!
//! Provides specialized skeleton components for common UI patterns.

use super::Skeleton;
use leptos::prelude::*;

/// Skeleton for avatar/rounded elements
#[component]
pub fn SkeletonAvatar(
    /// Size of the avatar (CSS value)
    #[prop(default = "3rem")]
    size: &'static str,
) -> impl IntoView {
    view! {
        <Skeleton
            width=size
            height=size
            rounded="rounded-xl"
        />
    }
}

/// Skeleton for text lines
#[allow(dead_code)]
#[component]
pub fn SkeletonText(
    /// Number of lines to display
    #[prop(default = 1)]
    lines: usize,
    /// Width of each line (can vary for natural look)
    #[prop(default = "100%")]
    width: &'static str,
    /// Height of each line
    #[prop(default = "0.875rem")]
    height: &'static str,
    /// Gap between lines
    #[prop(default = "0.5rem")]
    gap: &'static str,
) -> impl IntoView {
    let line_widths = match lines {
        1 => vec![width],
        2 => vec!["100%", "75%"],
        3 => vec!["100%", "85%", "60%"],
        _ => {
            let mut widths = vec!["100%"; lines];
            if lines > 1 {
                widths[lines - 1] = "60%";
            }
            widths
        }
    };

    view! {
        <div class="flex flex-col" style=format!("gap: {};", gap)>
            {line_widths.into_iter().map(|w| {
                view! {
                    <Skeleton
                        width=w
                        height=height
                        rounded="rounded"
                    />
                }
            }).collect_view()}
        </div>
    }
}

/// Skeleton for card containers
#[component]
pub fn SkeletonCard(
    /// Card content (children)
    children: Children,
    /// Additional CSS classes for the card
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <div class=format!(
            "p-6 bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-slate-700/30 {}",
            class
        )>
            {children()}
        </div>
    }
}

/// Skeleton for stat display items
#[component]
pub fn SkeletonStat(
    /// Show icon placeholder
    #[prop(default = true)]
    show_icon: bool,
) -> impl IntoView {
    view! {
        <div class="p-4 bg-gm-bg-secondary/50 rounded-xl border border-slate-700/30">
            <div class="flex items-center gap-3">
                <Show when=move || show_icon>
                    <Skeleton
                        width="2rem"
                        height="2rem"
                        rounded="rounded"
                    />
                </Show>
                <div class="flex-1 space-y-2">
                    <Skeleton
                        width="3rem"
                        height="1.25rem"
                        rounded="rounded"
                    />
                    <Skeleton
                        width="5rem"
                        height="0.75rem"
                        rounded="rounded"
                    />
                </div>
            </div>
        </div>
    }
}

/// Skeleton for badge items
#[component]
pub fn SkeletonBadge() -> impl IntoView {
    view! {
        <div class="p-3 bg-gm-bg-secondary/30 rounded-xl border border-slate-700/30 flex flex-col items-center gap-2">
            <Skeleton
                width="2.5rem"
                height="2.5rem"
                rounded="rounded-lg"
            />
            <Skeleton
                width="3rem"
                height="0.5rem"
                rounded="rounded"
            />
        </div>
    }
}

/// Skeleton for contribution graph
#[component]
pub fn SkeletonGraph(
    /// Number of weeks to show (default: 52 for full year)
    #[prop(default = 52)]
    weeks: usize,
    /// Number of days per week
    #[prop(default = 7)]
    days: usize,
) -> impl IntoView {
    view! {
        <div class="flex gap-1 overflow-hidden">
            {(0..weeks).map(|_| {
                view! {
                    <div class="flex flex-col gap-1">
                        {(0..days).map(|_| {
                            view! {
                                <Skeleton
                                    width="0.75rem"
                                    height="0.75rem"
                                    rounded="rounded-sm"
                                />
                            }
                        }).collect_view()}
                    </div>
                }
            }).collect_view()}
        </div>
    }
}
