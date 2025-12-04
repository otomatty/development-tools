//! CORS Settings Section Component
//!
//! Displays and allows toggling of CORS settings.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this component):
//!   └─ src/components/features/mock_server/mock_server_page.rs
//! Dependencies:
//!   └─ src/components/icons.rs
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

use leptos::prelude::*;

use crate::components::icons::Icon;
use crate::types::{CorsMode, MockServerConfig};

/// CORS settings section component
#[component]
pub fn CorsSettingsSection(
    config: ReadSignal<MockServerConfig>,
    on_toggle_cors: Callback<leptos::ev::MouseEvent>,
) -> impl IntoView {
    view! {
        <div class="bg-slate-800 rounded-lg p-5 border border-slate-700/50">
            <div class="flex items-center justify-between mb-4">
                <h2 class="text-lg font-semibold text-dt-text flex items-center gap-2">
                    <Icon name="shield".to_string() class="w-5 h-5 text-green-400".to_string() />
                    "CORS Settings"
                </h2>
                <button
                    class=move || format!(
                        "px-4 py-2 rounded-lg font-medium transition-colors {}",
                        if config.get().cors_mode == CorsMode::Simple {
                            "bg-green-500/20 text-green-400"
                        } else {
                            "bg-orange-500/20 text-orange-400"
                        }
                    )
                    on:click=move |ev| on_toggle_cors.run(ev)
                >
                    {move || if config.get().cors_mode == CorsMode::Simple {
                        "Simple"
                    } else {
                        "Advanced"
                    }}
                </button>
            </div>
            <div class="text-dt-text-sub">
                {move || if config.get().cors_mode == CorsMode::Simple {
                    "✅ All origins allowed (*)"
                } else {
                    "⚙️ Custom CORS configuration"
                }}
            </div>
        </div>
    }
}
