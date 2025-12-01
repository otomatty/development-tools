//! Toast notification component
//!
//! Displays temporary success/error/info messages that auto-hide.

use leptos::prelude::*;

/// Toast type for different message styles
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ToastType {
    Success,
    Error,
    Info,
    Warning,
}

impl Default for ToastType {
    fn default() -> Self {
        Self::Info
    }
}

/// Toast notification component
#[component]
pub fn Toast(
    /// Whether the toast is visible
    visible: ReadSignal<bool>,
    /// The message to display
    message: Signal<String>,
    /// The type of toast (success, error, info, warning)
    #[prop(default = ToastType::Info)]
    toast_type: ToastType,
) -> impl IntoView {
    let (icon, bg_class, border_class, text_class, glow_class) = match toast_type {
        ToastType::Success => (
            "✓",
            "bg-green-900/90",
            "border-green-500/50",
            "text-green-200",
            "shadow-[0_0_15px_rgba(34,197,94,0.3)]",
        ),
        ToastType::Error => (
            "✗",
            "bg-red-900/90",
            "border-red-500/50",
            "text-red-200",
            "shadow-[0_0_15px_rgba(239,68,68,0.3)]",
        ),
        ToastType::Info => (
            "ℹ",
            "bg-gm-accent-cyan/20",
            "border-gm-accent-cyan/50",
            "text-gm-accent-cyan",
            "shadow-[0_0_15px_rgba(6,182,212,0.3)]",
        ),
        ToastType::Warning => (
            "⚠",
            "bg-amber-900/90",
            "border-amber-500/50",
            "text-amber-200",
            "shadow-[0_0_15px_rgba(245,158,11,0.3)]",
        ),
    };

    view! {
        <Show when=move || visible.get()>
            <div
                class=format!(
                    "fixed bottom-6 right-6 z-50 flex items-center gap-3 px-5 py-3 rounded-xl {} border {} backdrop-blur-sm animate-slideInUp {}",
                    bg_class, border_class, glow_class
                )
                role="alert"
                aria-live="polite"
            >
                <span class=format!("text-lg font-bold {}", text_class)>
                    {icon}
                </span>
                <span class=format!("font-gaming {}", text_class)>
                    {move || message.get()}
                </span>
            </div>
        </Show>
    }
}

/// Inline toast for settings panels (shows within the component)
#[component]
pub fn InlineToast<F>(
    /// Whether the toast is visible
    visible: F,
    /// The message to display
    message: Signal<String>,
    /// The type of toast (success, error, info, warning)
    #[prop(default = ToastType::Success)]
    toast_type: ToastType,
) -> impl IntoView
where
    F: Fn() -> bool + Copy + Send + Sync + 'static,
{
    let (icon, bg_class, border_class, text_class) = match toast_type {
        ToastType::Success => (
            "✓",
            "bg-green-900/30",
            "border-green-500/50",
            "text-green-200",
        ),
        ToastType::Error => ("✗", "bg-red-900/30", "border-red-500/50", "text-red-200"),
        ToastType::Info => (
            "ℹ",
            "bg-gm-accent-cyan/10",
            "border-gm-accent-cyan/30",
            "text-gm-accent-cyan",
        ),
        ToastType::Warning => (
            "⚠",
            "bg-amber-900/30",
            "border-amber-500/50",
            "text-amber-200",
        ),
    };

    view! {
        <Show when=visible>
            <div
                class=format!(
                    "flex items-center gap-2 px-4 py-2.5 rounded-lg {} border {} animate-fadeIn",
                    bg_class, border_class
                )
                role="alert"
                aria-live="polite"
            >
                <span class=format!("text-sm font-bold {}", text_class)>
                    {icon}
                </span>
                <span class=format!("text-sm {}", text_class)>
                    {move || message.get()}
                </span>
            </div>
        </Show>
    }
}

/// Save status indicator that shows auto-save status
#[component]
pub fn SaveStatusIndicator(
    /// Whether settings are currently being saved
    saving: ReadSignal<bool>,
    /// Whether the last save was successful (None = no save yet)
    last_save_success: ReadSignal<Option<bool>>,
) -> impl IntoView {
    view! {
        <div class="flex items-center gap-2 text-xs">
            <Show
                when=move || saving.get()
                fallback=move || {
                    match last_save_success.get() {
                        Some(true) => view! {
                            <div class="flex items-center gap-1.5 text-green-400 animate-fadeIn">
                                <span class="w-2 h-2 rounded-full bg-green-400"></span>
                                <span>"保存済み"</span>
                            </div>
                        }.into_any(),
                        Some(false) => view! {
                            <div class="flex items-center gap-1.5 text-red-400 animate-fadeIn">
                                <span class="w-2 h-2 rounded-full bg-red-400"></span>
                                <span>"保存失敗"</span>
                            </div>
                        }.into_any(),
                        None => view! {
                            <></>
                        }.into_any(),
                    }
                }
            >
                <div class="flex items-center gap-1.5 text-gm-accent-cyan animate-pulse">
                    <span class="w-2 h-2 rounded-full bg-gm-accent-cyan"></span>
                    <span>"保存中..."</span>
                </div>
            </Show>
        </div>
    }
}
