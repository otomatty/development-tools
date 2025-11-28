//! Toggle switch component
//!
//! A reusable toggle switch component with smooth animations.

use leptos::prelude::*;

/// Toggle switch size
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ToggleSwitchSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl ToggleSwitchSize {
    fn button_class(&self) -> &'static str {
        match self {
            ToggleSwitchSize::Small => "w-10 h-5",
            ToggleSwitchSize::Medium => "w-12 h-6",
            ToggleSwitchSize::Large => "w-14 h-7",
        }
    }

    fn knob_class(&self) -> &'static str {
        match self {
            ToggleSwitchSize::Small => "w-3 h-3",
            ToggleSwitchSize::Medium => "w-4 h-4",
            ToggleSwitchSize::Large => "w-5 h-5",
        }
    }

    fn translate_class(&self) -> &'static str {
        match self {
            ToggleSwitchSize::Small => "translate-x-5",
            ToggleSwitchSize::Medium => "translate-x-6",
            ToggleSwitchSize::Large => "translate-x-7",
        }
    }
}

/// Toggle switch component props
#[component]
pub fn ToggleSwitch(
    /// Whether the toggle is on
    enabled: bool,
    /// Callback when toggle is clicked
    on_toggle: impl Fn() + 'static + Clone + Send + Sync,
    /// Optional label for accessibility
    #[prop(optional)] label_id: Option<&'static str>,
    /// Size of the toggle
    #[prop(default = ToggleSwitchSize::Medium)] size: ToggleSwitchSize,
    /// Whether the toggle is disabled
    #[prop(default = false)] disabled: bool,
) -> impl IntoView {
    let on_toggle_click = on_toggle.clone();
    let on_toggle_key = on_toggle.clone();
    
    let button_size = size.button_class();
    let knob_size = size.knob_class();
    let translate = size.translate_class();
    
    view! {
        <button
            class=move || format!(
                "relative {} rounded-full transition-all duration-300 ease-in-out {} {} focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gm-bg-primary",
                button_size,
                if enabled {
                    "bg-gradient-to-r from-gm-accent-cyan to-gm-accent-cyan/80 shadow-[0_0_10px_rgba(0,255,255,0.3)] focus:ring-gm-accent-cyan"
                } else {
                    "bg-slate-600 hover:bg-slate-500 focus:ring-slate-500"
                },
                if disabled { "opacity-50 cursor-not-allowed" } else { "cursor-pointer" }
            )
            type="button"
            role="switch"
            aria-checked=move || enabled.to_string()
            aria-labelledby=label_id.unwrap_or_default()
            disabled=disabled
            tabindex="0"
            on:click=move |_| {
                if !disabled {
                    on_toggle_click();
                }
            }
            on:keydown=move |ev| {
                if !disabled && (ev.key() == "Enter" || ev.key() == " ") {
                    ev.prevent_default();
                    on_toggle_key();
                }
            }
        >
            // Knob
            <span
                class=move || format!(
                    "absolute top-1 left-1 {} bg-white rounded-full shadow-md transition-all duration-300 ease-in-out {}",
                    knob_size,
                    if enabled { translate } else { "translate-x-0" }
                )
            ></span>
            
            // Glow effect when enabled
            <Show when=move || enabled>
                <span class="absolute inset-0 rounded-full bg-gm-accent-cyan/20 animate-pulse"></span>
            </Show>
        </button>
    }
}

/// Toggle switch with label component
#[component]
pub fn LabeledToggle(
    /// The label text
    label: String,
    /// Optional description text
    #[prop(optional)] description: Option<String>,
    /// Whether the toggle is on
    enabled: bool,
    /// Callback when toggle is clicked
    on_toggle: impl Fn() + 'static + Clone + Send + Sync,
    /// Size of the toggle
    #[prop(default = ToggleSwitchSize::Medium)] size: ToggleSwitchSize,
    /// Whether the toggle is disabled
    #[prop(default = false)] disabled: bool,
) -> impl IntoView {
    let label_id = format!("toggle-label-{}", label.replace(" ", "-").to_lowercase());
    let label_id_static: &'static str = Box::leak(label_id.clone().into_boxed_str());
    
    view! {
        <div class="flex items-center justify-between p-3 rounded-lg hover:bg-gm-bg-card/30 transition-colors">
            <div class="flex-1">
                <span 
                    class="text-white block font-gaming" 
                    id=label_id_static
                >
                    {label}
                </span>
                {description.map(|desc| view! {
                    <span class="text-sm text-dt-text-sub mt-1 block">
                        {desc}
                    </span>
                })}
            </div>
            <ToggleSwitch
                enabled=enabled
                on_toggle=on_toggle
                label_id=label_id_static
                size=size
                disabled=disabled
            />
        </div>
    }
}
