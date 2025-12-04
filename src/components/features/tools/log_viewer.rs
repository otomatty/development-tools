use leptos::prelude::*;

use crate::types::{LogEntry, LogStream};

/// ログビューアコンポーネント
#[component]
pub fn LogViewer(logs: ReadSignal<Vec<LogEntry>>, visible: ReadSignal<bool>) -> impl IntoView {
    view! {
        <Show when=move || visible.get()>
            <div class="card p-4 mt-4">
                <div class="flex items-center justify-between mb-3">
                    <h3 class="text-sm font-medium text-dt-text-sub uppercase tracking-wider">
                        "Logs"
                    </h3>
                    <span class="text-xs text-dt-text-sub">
                        {move || format!("{} lines", logs.get().len())}
                    </span>
                </div>

                <div class="bg-slate-900 rounded-lg p-3 h-48 overflow-y-auto font-mono text-sm">
                    <Show
                        when=move || !logs.get().is_empty()
                        fallback=move || view! {
                            <div class="text-dt-text-sub text-center py-4">
                                "No logs yet..."
                            </div>
                        }
                    >
                        <For
                            each=move || logs.get()
                            key=|entry| entry.timestamp.clone()
                            children=move |entry: LogEntry| {
                                let class = match entry.stream {
                                    LogStream::Stdout => "text-slate-300",
                                    LogStream::Stderr => "text-red-400",
                                };
                                view! {
                                    <div class=format!("log-line {}", class)>
                                        {entry.line}
                                    </div>
                                }
                            }
                        />
                    </Show>
                </div>
            </div>
        </Show>
    }
}
