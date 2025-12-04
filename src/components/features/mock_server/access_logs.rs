//! Access Logs Section Component
//!
//! Displays the access logs from the mock server.
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
use crate::types::AccessLogEntry;

/// Access logs section component
#[component]
pub fn AccessLogsSection(
    logs: ReadSignal<Vec<AccessLogEntry>>,
    on_clear: Callback<leptos::ev::MouseEvent>,
) -> impl IntoView {
    view! {
        <div class="bg-slate-800 rounded-lg p-5 border border-slate-700/50">
            <div class="flex items-center justify-between mb-4">
                <h2 class="text-lg font-semibold text-dt-text flex items-center gap-2">
                    <Icon name="list".to_string() class="w-5 h-5 text-yellow-400".to_string() />
                    "Access Logs"
                </h2>
                <button
                    class="px-4 py-2 text-slate-400 hover:text-dt-text transition-colors"
                    on:click=move |ev| on_clear.run(ev)
                >
                    "Clear"
                </button>
            </div>
            <div class="max-h-64 overflow-y-auto space-y-1 font-mono text-sm">
                {move || {
                    let log_entries = logs.get();
                    if log_entries.is_empty() {
                        view! {
                            <div class="text-center py-8 text-dt-text-sub">
                                "No requests yet"
                            </div>
                        }.into_any()
                    } else {
                        log_entries.into_iter().map(|log| {
                            let status_color = if log.status_code < 300 {
                                "text-green-400"
                            } else if log.status_code < 400 {
                                "text-blue-400"
                            } else if log.status_code < 500 {
                                "text-yellow-400"
                            } else {
                                "text-red-400"
                            };

                            let time_str = log.timestamp.split('T').last().unwrap_or("").split('.').next().unwrap_or("").to_string();
                            let method = log.method.clone();
                            let path = log.path.clone();
                            let status_code = log.status_code;
                            let response_size_str = log.response_size.map(format_size).unwrap_or("-".to_string());
                            let response_time_str = format!("{}ms", log.response_time_ms);

                            view! {
                                <div class="flex items-center gap-4 py-1 px-2 hover:bg-slate-700/50 rounded">
                                    <span class="text-slate-500 w-20">{time_str}</span>
                                    <span class="text-gm-accent-purple w-12">{method}</span>
                                    <span class="flex-1 text-dt-text truncate">{path}</span>
                                    <span class=status_color>{status_code}</span>
                                    <span class="text-slate-500 w-20 text-right">{response_size_str}</span>
                                    <span class="text-slate-500 w-16 text-right">{response_time_str}</span>
                                </div>
                            }
                        }).collect_view().into_any()
                    }
                }}
            </div>
        </div>
    }
}

/// Format file size to human readable
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}
