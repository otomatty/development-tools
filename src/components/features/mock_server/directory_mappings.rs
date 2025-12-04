//! Directory Mappings Section Component
//!
//! Manages directory mappings for the mock server.
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
use crate::types::DirectoryMapping;

/// Directory mappings section component
#[component]
pub fn DirectoryMappingsSection(
    mappings: ReadSignal<Vec<DirectoryMapping>>,
    show_add_mapping: ReadSignal<bool>,
    new_virtual_path: ReadSignal<String>,
    new_local_path: ReadSignal<String>,
    set_show_add_mapping: WriteSignal<bool>,
    set_new_virtual_path: WriteSignal<String>,
    set_new_local_path: WriteSignal<String>,
    on_add_mapping: Callback<leptos::ev::MouseEvent>,
    on_select_directory: Callback<leptos::ev::MouseEvent>,
    on_delete_mapping: Callback<i64>,
    on_toggle_mapping: Callback<(i64, bool)>,
    on_browse_files: Callback<DirectoryMapping>,
) -> impl IntoView {
    view! {
        <div class="bg-slate-800 rounded-lg p-5 border border-slate-700/50">
            <div class="flex items-center justify-between mb-4">
                <h2 class="text-lg font-semibold text-dt-text flex items-center gap-2">
                    <Icon name="folder".to_string() class="w-5 h-5 text-gm-accent-purple".to_string() />
                    "Directory Mappings"
                </h2>
                <button
                    class="flex items-center gap-2 px-4 py-2 bg-gm-accent-purple/20 text-gm-accent-purple rounded-lg hover:bg-gm-accent-purple/30 transition-colors"
                    on:click=move |_| set_show_add_mapping.set(true)
                >
                    <Icon name="plus".to_string() class="w-4 h-4".to_string() />
                    "Add Mapping"
                </button>
            </div>

            // Add mapping form
            <Show when=move || show_add_mapping.get()>
                <div class="mb-4 p-4 bg-slate-700/50 rounded-lg space-y-3">
                    <div class="flex items-center gap-4">
                        <div class="flex-1">
                            <label class="text-sm text-dt-text-sub block mb-1">"Virtual Path"</label>
                            <input
                                type="text"
                                class="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded text-dt-text focus:outline-none focus:border-gm-accent-cyan"
                                placeholder="/images"
                                prop:value=move || new_virtual_path.get()
                                on:input=move |ev| {
                                    let target = event_target::<web_sys::HtmlInputElement>(&ev);
                                    set_new_virtual_path.set(target.value());
                                }
                            />
                        </div>
                        <div class="flex-1">
                            <label class="text-sm text-dt-text-sub block mb-1">"Local Path"</label>
                            <div class="flex gap-2">
                                <input
                                    type="text"
                                    class="flex-1 px-3 py-2 bg-slate-700 border border-slate-600 rounded text-dt-text focus:outline-none focus:border-gm-accent-cyan"
                                    placeholder="/path/to/directory"
                                    prop:value=move || new_local_path.get()
                                    on:input=move |ev| {
                                        let target = event_target::<web_sys::HtmlInputElement>(&ev);
                                        set_new_local_path.set(target.value());
                                    }
                                />
                                <button
                                    class="px-3 py-2 bg-slate-600 hover:bg-slate-500 rounded text-dt-text transition-colors"
                                    on:click=move |ev| on_select_directory.run(ev)
                                    title="Browse"
                                >
                                    <Icon name="folder-open".to_string() class="w-5 h-5".to_string() />
                                </button>
                            </div>
                        </div>
                    </div>
                    <div class="flex justify-end gap-2">
                        <button
                            class="px-4 py-2 text-slate-400 hover:text-dt-text transition-colors"
                            on:click=move |_| set_show_add_mapping.set(false)
                        >
                            "Cancel"
                        </button>
                        <button
                            class="px-4 py-2 bg-gm-accent-cyan text-slate-900 rounded-lg hover:bg-gm-accent-cyan/80 transition-colors font-medium"
                            on:click=move |ev| on_add_mapping.run(ev)
                        >
                            "Add"
                        </button>
                    </div>
                </div>
            </Show>

            // Mappings list
            <div class="space-y-2">
                {move || {
                    let maps = mappings.get();
                    if maps.is_empty() {
                        view! {
                            <div class="text-center py-8 text-dt-text-sub">
                                <Icon name="folder".to_string() class="w-12 h-12 mx-auto mb-3 opacity-50".to_string() />
                                <p>"No mappings configured"</p>
                                <p class="text-sm">"Add a directory mapping to serve files"</p>
                            </div>
                        }.into_any()
                    } else {
                        maps.into_iter().map(|mapping| {
                            let mapping_id = mapping.id;
                            let mapping_clone = mapping.clone();
                            let mapping_clone2 = mapping.clone();
                            let mapping_for_browse = mapping.clone();
                            let enabled = mapping.enabled;

                            view! {
                                <div class=move || format!(
                                    "flex items-center gap-4 p-3 rounded-lg border transition-colors {}",
                                    if mapping.enabled {
                                        "bg-slate-700/50 border-slate-600"
                                    } else {
                                        "bg-slate-800/50 border-slate-700 opacity-60"
                                    }
                                )>
                                    // Enable/Disable toggle
                                    <button
                                        class=move || format!(
                                            "w-10 h-6 rounded-full relative transition-colors {}",
                                            if mapping_clone.enabled {
                                                "bg-gm-accent-cyan"
                                            } else {
                                                "bg-slate-600"
                                            }
                                        )
                                        on:click=move |_| on_toggle_mapping.run((mapping_id, enabled))
                                    >
                                        <span class=move || format!(
                                            "absolute top-1 w-4 h-4 bg-white rounded-full transition-transform {}",
                                            if mapping_clone2.enabled {
                                                "left-5"
                                            } else {
                                                "left-1"
                                            }
                                        )/>
                                    </button>

                                    // Virtual path
                                    <div class="w-32">
                                        <code class="text-gm-accent-cyan">{mapping.virtual_path.clone()}</code>
                                    </div>

                                    <Icon name="arrow-right".to_string() class="w-4 h-4 text-slate-500".to_string() />

                                    // Local path
                                    <div class="flex-1 truncate text-dt-text-sub">
                                        {mapping.local_path.clone()}
                                    </div>

                                    // Actions
                                    <div class="flex items-center gap-2">
                                        <button
                                            class="p-2 text-slate-400 hover:text-gm-accent-cyan rounded transition-colors"
                                            title="Browse files"
                                            on:click=move |_| on_browse_files.run(mapping_for_browse.clone())
                                        >
                                            <Icon name="eye".to_string() class="w-4 h-4".to_string() />
                                        </button>
                                        <button
                                            class="p-2 text-slate-400 hover:text-red-400 rounded transition-colors"
                                            title="Delete mapping"
                                            on:click=move |_| on_delete_mapping.run(mapping_id)
                                        >
                                            <Icon name="trash-2".to_string() class="w-4 h-4".to_string() />
                                        </button>
                                    </div>
                                </div>
                            }
                        }).collect_view().into_any()
                    }
                }}
            </div>
        </div>
    }
}
