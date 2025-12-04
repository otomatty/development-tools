//! File Browser Section Component
//!
//! Displays a file browser for the selected directory mapping.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this component):
//!   └─ src/components/features/mock_server/mock_server_page.rs
//! Dependencies:
//!   └─ src/components/icons.rs
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

use leptos::prelude::*;

use super::access_logs::format_size;
use crate::components::icons::Icon;
use crate::types::FileInfo;

/// File browser section component
#[component]
pub fn FileBrowserSection(
    selected_mapping_id: ReadSignal<Option<i64>>,
    files: ReadSignal<Vec<FileInfo>>,
    browsing_path: ReadSignal<Option<String>>,
    on_navigate: Callback<String>,
    on_close: Callback<leptos::ev::MouseEvent>,
) -> impl IntoView {
    view! {
        <Show when=move || selected_mapping_id.get().is_some()>
            <div class="bg-slate-800 rounded-lg p-5 border border-slate-700/50">
                <div class="flex items-center justify-between mb-4">
                    <h2 class="text-lg font-semibold text-dt-text flex items-center gap-2">
                        <Icon name="file".to_string() class="w-5 h-5 text-blue-400".to_string() />
                        "File Browser"
                    </h2>
                    <button
                        class="text-slate-400 hover:text-dt-text transition-colors"
                        on:click=move |ev| on_close.run(ev)
                    >
                        <Icon name="x".to_string() class="w-5 h-5".to_string() />
                    </button>
                </div>
                <div class="text-sm text-dt-text-sub mb-3">
                    {move || browsing_path.get().unwrap_or_default()}
                </div>
                <div class="max-h-64 overflow-y-auto space-y-1">
                    {move || {
                        files.get().into_iter().map(|file| {
                            let file_path = file.path.clone();
                            let is_dir = file.is_directory;

                            view! {
                                <div
                                    class=move || format!(
                                        "flex items-center gap-3 py-2 px-3 rounded hover:bg-slate-700/50 {}",
                                        if is_dir { "cursor-pointer" } else { "" }
                                    )
                                    on:click=move |_| {
                                        if is_dir {
                                            on_navigate.run(file_path.clone());
                                        }
                                    }
                                >
                                    <Icon
                                        name=if file.is_directory { "folder".to_string() } else { "file".to_string() }
                                        class=if file.is_directory { "w-5 h-5 text-yellow-400".to_string() } else { "w-5 h-5 text-slate-400".to_string() }
                                    />
                                    <span class="flex-1 text-dt-text">{file.name}</span>
                                    <span class="text-slate-500 text-sm">
                                        {file.size.map(format_size).unwrap_or_default()}
                                    </span>
                                </div>
                            }
                        }).collect_view()
                    }}
                </div>
            </div>
        </Show>
    }
}
