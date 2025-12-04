//! Mock Server Page Component
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this component):
//!   └─ src/components/mock_server/mod.rs
//! Dependencies:
//!   ├─ src/tauri_api.rs
//!   └─ src/types.rs

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;

use crate::components::icons::Icon;
use crate::tauri_api;
use crate::types::{
    AccessLogEntry, CorsMode, CreateMappingRequest, DirectoryMapping, FileInfo, MockServerConfig,
    MockServerState, ServerStatus, UpdateConfigRequest, UpdateMappingRequest,
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, handler: &Closure<dyn FnMut(JsValue)>) -> JsValue;
}

/// Mock Server Page
#[component]
pub fn MockServerPage() -> impl IntoView {
    // State
    let (server_state, set_server_state) = signal(MockServerState::default());
    let (config, set_config) = signal(MockServerConfig::default());
    let (mappings, set_mappings) = signal(Vec::<DirectoryMapping>::new());
    let (logs, set_logs) = signal(Vec::<AccessLogEntry>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (starting, set_starting) = signal(false);

    // For mapping form
    let (show_add_mapping, set_show_add_mapping) = signal(false);
    let (new_virtual_path, set_new_virtual_path) = signal(String::new());
    let (new_local_path, set_new_local_path) = signal(String::new());

    // File browser state
    let (selected_mapping_id, set_selected_mapping_id) = signal(Option::<i64>::None);
    let (files, set_files) = signal(Vec::<FileInfo>::new());
    let (browsing_path, set_browsing_path) = signal(Option::<String>::None);

    // Load initial data
    {
        spawn_local(async move {
            // Load state, config, and mappings in parallel
            let (state_result, config_result, mappings_result) = futures::join!(
                tauri_api::get_mock_server_state(),
                tauri_api::get_mock_server_config(),
                tauri_api::get_mock_server_mappings()
            );

            if let Ok(state) = state_result {
                set_server_state.set(state);
            }
            if let Ok(cfg) = config_result {
                set_config.set(cfg);
            }
            if let Ok(maps) = mappings_result {
                set_mappings.set(maps);
            }

            set_loading.set(false);
        });
    }

    // Setup log event listener
    {
        spawn_local(async move {
            let closure = Closure::new(move |event: JsValue| {
                if let Ok(payload) = js_sys::Reflect::get(&event, &"payload".into()) {
                    if let Ok(log_entry) = serde_wasm_bindgen::from_value::<AccessLogEntry>(payload)
                    {
                        set_logs.update(|logs| {
                            logs.insert(0, log_entry);
                            // Keep only last 100 logs
                            if logs.len() > 100 {
                                logs.truncate(100);
                            }
                        });
                    }
                }
            });
            let _ = listen("mock-server-log", &closure).await;
            closure.forget();
        });
    }

    // Start server handler
    let start_server = move |_| {
        set_starting.set(true);
        set_error.set(None);
        spawn_local(async move {
            match tauri_api::start_mock_server().await {
                Ok(state) => {
                    set_server_state.set(state);
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_starting.set(false);
        });
    };

    // Stop server handler
    let stop_server = move |_| {
        set_starting.set(true);
        set_error.set(None);
        spawn_local(async move {
            match tauri_api::stop_mock_server().await {
                Ok(state) => {
                    set_server_state.set(state);
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_starting.set(false);
        });
    };

    // Update port handler
    let update_port = move |ev: web_sys::Event| {
        let target = ev.target().unwrap();
        let input: web_sys::HtmlInputElement = target.unchecked_into();
        if let Ok(port) = input.value().parse::<u16>() {
            spawn_local(async move {
                let request = UpdateConfigRequest {
                    port: Some(port),
                    cors_mode: None,
                    cors_origins: None,
                    cors_methods: None,
                    cors_headers: None,
                    cors_max_age: None,
                    show_directory_listing: None,
                };
                if let Ok(cfg) = tauri_api::update_mock_server_config(request).await {
                    set_config.set(cfg);
                }
            });
        }
    };

    // Add mapping handler
    let add_mapping = move |_| {
        let virtual_path = new_virtual_path.get();
        let local_path = new_local_path.get();

        if virtual_path.is_empty() || local_path.is_empty() {
            return;
        }

        spawn_local(async move {
            let request = CreateMappingRequest {
                virtual_path,
                local_path,
            };
            match tauri_api::create_mock_server_mapping(request).await {
                Ok(mapping) => {
                    set_mappings.update(|m| m.push(mapping));
                    set_new_virtual_path.set(String::new());
                    set_new_local_path.set(String::new());
                    set_show_add_mapping.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
        });
    };

    // Select directory handler
    let select_directory = move |_| {
        spawn_local(async move {
            if let Ok(Some(path)) = tauri_api::select_mock_server_directory().await {
                set_new_local_path.set(path);
            }
        });
    };

    // Delete mapping handler
    let delete_mapping = move |id: i64| {
        spawn_local(async move {
            if tauri_api::delete_mock_server_mapping(id).await.is_ok() {
                set_mappings.update(|m| m.retain(|mapping| mapping.id != id));
                if selected_mapping_id.get() == Some(id) {
                    set_selected_mapping_id.set(None);
                    set_files.set(Vec::new());
                }
            }
        });
    };

    // Toggle mapping enabled handler
    let toggle_mapping = move |id: i64, enabled: bool| {
        spawn_local(async move {
            let request = UpdateMappingRequest {
                id,
                virtual_path: None,
                local_path: None,
                enabled: Some(!enabled),
            };
            if let Ok(updated) = tauri_api::update_mock_server_mapping(request).await {
                set_mappings.update(|m| {
                    if let Some(mapping) = m.iter_mut().find(|m| m.id == id) {
                        *mapping = updated;
                    }
                });
            }
        });
    };

    // Browse files handler
    let browse_files = move |mapping: DirectoryMapping| {
        set_selected_mapping_id.set(Some(mapping.id));
        set_browsing_path.set(Some(mapping.local_path.clone()));
        spawn_local(async move {
            if let Ok(file_list) = tauri_api::list_mock_server_directory(&mapping.local_path).await
            {
                set_files.set(file_list);
            }
        });
    };

    // Navigate to subdirectory
    let navigate_to = move |path: String| {
        set_browsing_path.set(Some(path.clone()));
        spawn_local(async move {
            if let Ok(file_list) = tauri_api::list_mock_server_directory(&path).await {
                set_files.set(file_list);
            }
        });
    };

    // Clear logs handler
    let clear_logs = move |_| {
        set_logs.set(Vec::new());
    };

    // Toggle CORS mode
    let toggle_cors_mode = move |_| {
        let current_mode = config.get().cors_mode;
        let new_mode = match current_mode {
            CorsMode::Simple => CorsMode::Advanced,
            CorsMode::Advanced => CorsMode::Simple,
        };

        spawn_local(async move {
            let request = UpdateConfigRequest {
                port: None,
                cors_mode: Some(new_mode),
                cors_origins: None,
                cors_methods: None,
                cors_headers: None,
                cors_max_age: None,
                show_directory_listing: None,
            };
            if let Ok(cfg) = tauri_api::update_mock_server_config(request).await {
                set_config.set(cfg);
            }
        });
    };

    view! {
        <div class="h-full flex flex-col bg-slate-900">
            // Header
            <div class="flex items-center justify-between p-6 border-b border-slate-700/50">
                <div class="flex items-center gap-3">
                    <div class="p-2 bg-gradient-to-br from-gm-accent-cyan to-gm-accent-purple rounded-lg">
                        <Icon name="radio".to_string() class="w-6 h-6 text-white".to_string() />
                    </div>
                    <div>
                        <h1 class="text-2xl font-bold text-dt-text font-gaming">"Mock Server"</h1>
                        <p class="text-sm text-dt-text-sub">"Static file server for local development"</p>
                    </div>
                </div>

                // Start/Stop Button
                <button
                    class=move || format!(
                        "flex items-center gap-2 px-6 py-3 rounded-lg font-semibold transition-all duration-200 {}",
                        if starting.get() {
                            "bg-slate-700 text-slate-400 cursor-not-allowed"
                        } else if server_state.get().status == ServerStatus::Running {
                            "bg-red-600 hover:bg-red-700 text-white"
                        } else {
                            "bg-gm-accent-cyan hover:bg-gm-accent-cyan/80 text-slate-900"
                        }
                    )
                    disabled=move || starting.get()
                    on:click=move |ev| {
                        if server_state.get().status == ServerStatus::Running {
                            stop_server(ev);
                        } else {
                            start_server(ev);
                        }
                    }
                >
                    {move || {
                        if starting.get() {
                            view! {
                                <div class="animate-spin w-5 h-5 border-2 border-current border-t-transparent rounded-full"/>
                                <span>"Processing..."</span>
                            }.into_any()
                        } else if server_state.get().status == ServerStatus::Running {
                            view! {
                                <Icon name="square".to_string() class="w-5 h-5".to_string() />
                                <span>"Stop Server"</span>
                            }.into_any()
                        } else {
                            view! {
                                <Icon name="play".to_string() class="w-5 h-5".to_string() />
                                <span>"Start Server"</span>
                            }.into_any()
                        }
                    }}
                </button>
            </div>

            // Error message
            <Show when=move || error.get().is_some()>
                <div class="mx-6 mt-4 p-4 bg-red-500/20 border border-red-500/50 rounded-lg text-red-400">
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            // Main content
            <div class="flex-1 overflow-y-auto p-6 space-y-6">
                // Loading state
                <Show when=move || loading.get()>
                    <div class="flex items-center justify-center py-12">
                        <div class="animate-spin w-8 h-8 border-3 border-gm-accent-cyan border-t-transparent rounded-full"/>
                    </div>
                </Show>

                <Show when=move || !loading.get()>
                    // Server Status Section
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
                                    on:change=update_port
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

                    // Directory Mappings Section
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
                                                on:click=select_directory
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
                                        on:click=add_mapping
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
                                                    on:click=move |_| toggle_mapping(mapping_id, enabled)
                                                >
                                                    <span class=move || format!(
                                                        "absolute top-1 w-4 h-4 bg-white rounded-full transition-transform {}",
                                                        if mapping_clone.enabled {
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
                                                        on:click=move |_| browse_files(mapping_for_browse.clone())
                                                    >
                                                        <Icon name="eye".to_string() class="w-4 h-4".to_string() />
                                                    </button>
                                                    <button
                                                        class="p-2 text-slate-400 hover:text-red-400 rounded transition-colors"
                                                        title="Delete mapping"
                                                        on:click=move |_| delete_mapping(mapping_id)
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

                    // CORS Settings Section
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
                                on:click=toggle_cors_mode
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

                    // Access Logs Section
                    <div class="bg-slate-800 rounded-lg p-5 border border-slate-700/50">
                        <div class="flex items-center justify-between mb-4">
                            <h2 class="text-lg font-semibold text-dt-text flex items-center gap-2">
                                <Icon name="list".to_string() class="w-5 h-5 text-yellow-400".to_string() />
                                "Access Logs"
                            </h2>
                            <button
                                class="px-4 py-2 text-slate-400 hover:text-dt-text transition-colors"
                                on:click=clear_logs
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

                                        // Clone values to avoid borrowing issues
                                        let time_str = log.timestamp.split('T').last().unwrap_or("").split('.').next().unwrap_or("").to_string();
                                        let method = log.method.clone();
                                        let path = log.path.clone();
                                        let status_code = log.status_code;
                                        let response_size_str = log.response_size.map(|s| format_size(s)).unwrap_or("-".to_string());
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

                    // File Browser Section (shown when a mapping is selected)
                    <Show when=move || selected_mapping_id.get().is_some()>
                        <div class="bg-slate-800 rounded-lg p-5 border border-slate-700/50">
                            <div class="flex items-center justify-between mb-4">
                                <h2 class="text-lg font-semibold text-dt-text flex items-center gap-2">
                                    <Icon name="file".to_string() class="w-5 h-5 text-blue-400".to_string() />
                                    "File Browser"
                                </h2>
                                <button
                                    class="text-slate-400 hover:text-dt-text transition-colors"
                                    on:click=move |_| {
                                        set_selected_mapping_id.set(None);
                                        set_files.set(Vec::new());
                                    }
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
                                                        navigate_to(file_path.clone());
                                                    }
                                                }
                                            >
                                                <Icon
                                                    name=if file.is_directory { "folder".to_string() } else { "file".to_string() }
                                                    class=if file.is_directory { "w-5 h-5 text-yellow-400".to_string() } else { "w-5 h-5 text-slate-400".to_string() }
                                                />
                                                <span class="flex-1 text-dt-text">{file.name}</span>
                                                <span class="text-slate-500 text-sm">
                                                    {file.size.map(|s| format_size(s)).unwrap_or_default()}
                                                </span>
                                            </div>
                                        }
                                    }).collect_view()
                                }}
                            </div>
                        </div>
                    </Show>
                </Show>
            </div>
        </div>
    }
}

/// Format file size to human readable
fn format_size(bytes: u64) -> String {
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
