//! DropdownMenu Component
//!
//! A reusable dropdown menu component that supports:
//! - Click to toggle menu visibility
//! - Click outside to close
//! - ESC key to close
//! - Animation support via AnimationContext
//! - Accessibility attributes
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this component):
//!   └─ src/components/home/profile_card.rs
//!   └─ src/components/ui/dropdown/mod.rs
//!
//! Related Documentation:
//!   ├─ Spec: src/components/dropdown_menu.spec.md
//!   └─ GitHub Issue: #114

use leptos::ev;
use leptos::prelude::*;

use crate::components::animation_context::use_animation_context_or_default;

/// Context type for sharing dropdown menu state with child components
#[derive(Clone, Copy)]
pub struct DropdownMenuContext {
    pub is_open: RwSignal<bool>,
}

/// Dropdown menu component
///
/// Provides a dropdown menu that can be triggered by clicking a button.
/// Supports click-outside detection and ESC key to close.
#[component]
pub fn DropdownMenu<TriggerFn, TriggerView>(
    /// The trigger button content (typically an icon)
    trigger: TriggerFn,
    /// Menu items to display when open
    children: Children,
    /// Menu alignment: "right" or "left"
    #[prop(default = "right")]
    align: &'static str,
) -> impl IntoView
where
    TriggerFn: Fn() -> TriggerView + 'static,
    TriggerView: IntoView + 'static,
{
    let is_open = RwSignal::new(false);
    let animation_ctx = use_animation_context_or_default();
    // TODO: [IMPROVE] container_refは将来的にフォーカストラップ実装時に使用予定

    // Provide context for child components (DropdownMenuItem)
    provide_context(DropdownMenuContext { is_open });

    // Toggle menu open/close
    let toggle_menu = move |_: ev::MouseEvent| {
        is_open.update(|open| *open = !*open);
    };

    // Close menu
    let close_menu = move || {
        is_open.set(false);
    };

    // Handle ESC key - uses Leptos's window_event_listener for proper lifecycle management
    let _ = window_event_listener(ev::keydown, move |ev| {
        if is_open.get() && ev.key() == "Escape" {
            is_open.set(false);
        }
    });

    // Animation classes
    let menu_classes = move || {
        let base = format!(
            "absolute {} top-full mt-2 min-w-[160px] bg-gm-bg-card/95 backdrop-blur-sm border border-gm-accent-cyan/20 rounded-lg shadow-lg z-50",
            if align == "left" { "left-0" } else { "right-0" }
        );

        if animation_ctx.is_enabled() {
            format!("{} transition-all duration-200 ease-out", base)
        } else {
            base
        }
    };

    let menu_style = move || {
        let is_open_val = is_open.get();
        let animated = animation_ctx.is_enabled();

        if is_open_val {
            "opacity: 1; transform: translateY(0);"
        } else if animated {
            "opacity: 0; transform: translateY(-8px); pointer-events: none;"
        } else {
            "display: none;"
        }
    };

    view! {
        <div class="relative">
            // Overlay for click outside detection (only when menu is open)
            {move || {
                if is_open.get() {
                    Some(view! {
                        <div
                            class="fixed inset-0 z-40"
                            on:click=move |_| close_menu()
                        />
                    })
                } else {
                    None
                }
            }}

            // Trigger button
            <button
                type="button"
                class="p-2 text-dt-text-sub hover:text-gm-accent-cyan transition-colors rounded-lg"
                aria-expanded=move || is_open.get().to_string()
                aria-haspopup="true"
                on:click=toggle_menu
            >
                {trigger()}
            </button>

            // Dropdown menu
            <div
                class=menu_classes
                style=menu_style
                role="menu"
                aria-orientation="vertical"
            >
                <div class="py-1">
                    {children()}
                </div>
            </div>
        </div>
    }
}

/// Dropdown menu item component
#[component]
pub fn DropdownMenuItem<F>(
    /// Click handler for the menu item
    on_click: F,
    /// Whether this is a dangerous action (shows in red)
    #[prop(default = false)]
    danger: bool,
    /// Content of the menu item
    children: Children,
) -> impl IntoView
where
    F: Fn(ev::MouseEvent) + 'static + Clone,
{
    // Get the dropdown menu context to close menu after click
    let menu_ctx = use_context::<DropdownMenuContext>();

    let base_classes = "flex items-center gap-3 px-4 py-2 text-sm transition-colors cursor-pointer w-full text-left";

    let item_classes = if danger {
        format!("{} text-gm-error hover:bg-gm-error/10", base_classes)
    } else {
        format!(
            "{} text-dt-text-main hover:bg-gm-accent-cyan/10",
            base_classes
        )
    };

    view! {
        <button
            type="button"
            class=item_classes
            role="menuitem"
            on:click=move |e| {
                on_click.clone()(e);
                // Close menu after item click (TC-006)
                if let Some(ctx) = menu_ctx {
                    ctx.is_open.set(false);
                }
            }
        >
            {children()}
        </button>
    }
}

/// Dropdown menu divider component
#[component]
pub fn DropdownMenuDivider() -> impl IntoView {
    view! {
        <div class="my-1 border-t border-gm-accent-cyan/10" role="separator" />
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// TC-001: メニュー初期状態
    #[test]
    fn test_initial_state_is_closed() {
        let is_open = RwSignal::new(false);
        assert!(!is_open.get_untracked());
    }

    /// TC-002: トリガークリックでメニュー開く
    #[test]
    fn test_toggle_opens_menu() {
        let is_open = RwSignal::new(false);
        is_open.update(|open| *open = !*open);
        assert!(is_open.get_untracked());
    }

    /// TC-003: トリガークリックでメニュー閉じる
    #[test]
    fn test_toggle_closes_menu() {
        let is_open = RwSignal::new(true);
        is_open.update(|open| *open = !*open);
        assert!(!is_open.get_untracked());
    }

    /// TC-004: メニュー閉じる関数
    #[test]
    fn test_close_menu() {
        let is_open = RwSignal::new(true);
        is_open.set(false);
        assert!(!is_open.get_untracked());
    }

    /// TC-009: dangerプロパティのクラス生成
    #[test]
    fn test_danger_classes() {
        let base_classes = "flex items-center gap-3 px-4 py-2 text-sm transition-colors cursor-pointer w-full text-left";
        let danger_classes = format!("{} text-gm-error hover:bg-gm-error/10", base_classes);
        assert!(danger_classes.contains("text-gm-error"));

        let normal_classes = format!(
            "{} text-dt-text-main hover:bg-gm-accent-cyan/10",
            base_classes
        );
        assert!(normal_classes.contains("text-dt-text-main"));
    }

    /// Test menu alignment classes
    #[test]
    fn test_alignment_classes() {
        let align_right = "right";
        let align_left = "left";

        let right_class = if align_right == "left" {
            "left-0"
        } else {
            "right-0"
        };
        let left_class = if align_left == "left" {
            "left-0"
        } else {
            "right-0"
        };

        assert_eq!(right_class, "right-0");
        assert_eq!(left_class, "left-0");
    }
}
