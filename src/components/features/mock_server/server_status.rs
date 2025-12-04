//! Server Status Section Component
//!
//! Displays the current status of the mock server.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this component):
//!   └─ src/components/features/mock_server/mock_server_page.rs
//! Dependencies:
//!   └─ src/components/icons.rs
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

use leptos::prelude::*;
use web_sys::Event;

use crate::components::icons::Icon;
use crate::types::{MockServerConfig, MockServerState, ServerStatus};

/// Server status section component
#[component]
pub fn ServerStatusSection(
    server_state: ReadSignal<MockServerState>,
    config: ReadSignal<MockServerConfig>,
    on_port_change: Callback<Event>,
) -> impl IntoView {
    view! {
        <div class="bg-slate-800 rounded-lg p-5 border border-slate-700/50">
            <h2 class="text-lg font-semibold text-dt-text mb-4 flex items-center gap-2">
                <Icon name="activity".to_string() class="w-5 h-5 text-gm-accent-cyan".to_string() />
                "Server Status"
            </h2>
            <div class="grid grid-cols-3 gap-4">
                <div>
                    <div class="text-sm text-dt-text-sub mb-1">"Status"</div>
                    <div class=move || format!(
                        "flex items-center gap-2 font-medium {}",
                        if server_state.get().status == ServerStatus::Running {
                            "text-green-400"
                        } else {
                            "text-slate-400"
                        }
                    )>
                        <span class=move || format!(
                            "w-2 h-2 rounded-full {}",
                            if server_state.get().status == ServerStatus::Running {
                                "bg-green-400 animate-pulse"
                            } else {
                                "bg-slate-500"
                            }
                        )/>
                        {move || if server_state.get().status == ServerStatus::Running {
                            "Running"
                        } else {
                            "Stopped"
                        }}
                    </div>
                </div>
                <div>
                    <div class="text-sm text-dt-text-sub mb-1">"Port"</div>
                    <input
                        type="number"
                        class="w-24 px-3 py-1.5 bg-slate-700 border border-slate-600 rounded text-dt-text focus:outline-none focus:border-gm-accent-cyan"
                        value=move || config.get().port.to_string()
                        disabled=move || server_state.get().status == ServerStatus::Running
                        on:change=move |ev| on_port_change.run(ev)
                    />
                </div>
                <div>
                    <div class="text-sm text-dt-text-sub mb-1">"URL"</div>
                    <a
                        href=move || server_state.get().url.clone()
                        target="_blank"
                        class="text-gm-accent-cyan hover:underline"
                    >
                        {move || server_state.get().url.clone()}
                    </a>
                </div>
            </div>
        </div>
    }
}
