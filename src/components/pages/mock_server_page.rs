//! Mock Server Page Component
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this component):
//!   └─ src/components/pages/mock_server_page.rs
//! Dependencies:
//!   ├─ src/tauri_api.rs
//!   ├─ features/mock_server/server_status.rs
//!   ├─ features/mock_server/directory_mappings.rs
//!   ├─ features/mock_server/cors_settings.rs
//!   ├─ features/mock_server/access_logs.rs
//!   └─ features/mock_server/file_browser.rs
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;

use crate::components::features::mock_server::{
    AccessLogsSection, CorsSettingsSection, DirectoryMappingsSection, FileBrowserSection,
    ServerStatusSection,
};
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

    // Event handlers
    let start_server = move |_| {
        set_starting.set(true);
        set_error.set(None);
        spawn_local(async move {
            match tauri_api::start_mock_server().await {
                Ok(state) => set_server_state.set(state),
                Err(e) => set_error.set(Some(e)),
            }
            set_starting.set(false);
        });
    };

    let stop_server = move |_| {
        set_starting.set(true);
        set_error.set(None);
        spawn_local(async move {
            match tauri_api::stop_mock_server().await {
                Ok(state) => set_server_state.set(state),
                Err(e) => set_error.set(Some(e)),
            }
            set_starting.set(false);
        });
    };

    let update_port = Callback::new(move |ev: web_sys::Event| {
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
    });

    let add_mapping = Callback::new(move |_: leptos::ev::MouseEvent| {
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
                Err(e) => set_error.set(Some(e)),
            }
        });
    });

    let select_directory = Callback::new(move |_: leptos::ev::MouseEvent| {
        spawn_local(async move {
            if let Ok(Some(path)) = tauri_api::select_mock_server_directory().await {
                set_new_local_path.set(path);
            }
        });
    });

    let delete_mapping = Callback::new(move |id: i64| {
        spawn_local(async move {
            if tauri_api::delete_mock_server_mapping(id).await.is_ok() {
                set_mappings.update(|m| m.retain(|mapping| mapping.id != id));
                if selected_mapping_id.get() == Some(id) {
                    set_selected_mapping_id.set(None);
                    set_files.set(Vec::new());
                }
            }
        });
    });

    let toggle_mapping = Callback::new(move |(id, enabled): (i64, bool)| {
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
    });

    let browse_files = Callback::new(move |mapping: DirectoryMapping| {
        set_selected_mapping_id.set(Some(mapping.id));
        set_browsing_path.set(Some(mapping.local_path.clone()));
        spawn_local(async move {
            if let Ok(file_list) = tauri_api::list_mock_server_directory(&mapping.local_path).await
            {
                set_files.set(file_list);
            }
        });
    });

    let navigate_to = Callback::new(move |path: String| {
        set_browsing_path.set(Some(path.clone()));
        spawn_local(async move {
            if let Ok(file_list) = tauri_api::list_mock_server_directory(&path).await {
                set_files.set(file_list);
            }
        });
    });

    let clear_logs = Callback::new(move |_: leptos::ev::MouseEvent| {
        set_logs.set(Vec::new());
    });

    let toggle_cors_mode = Callback::new(move |_: leptos::ev::MouseEvent| {
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
    });

    let close_file_browser = Callback::new(move |_: leptos::ev::MouseEvent| {
        set_selected_mapping_id.set(None);
        set_files.set(Vec::new());
    });

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
                    <ServerStatusSection
                        server_state=server_state
                        config=config
                        on_port_change=update_port
                    />

                    // Directory Mappings Section
                    <DirectoryMappingsSection
                        mappings=mappings
                        show_add_mapping=show_add_mapping
                        new_virtual_path=new_virtual_path
                        new_local_path=new_local_path
                        set_show_add_mapping=set_show_add_mapping
                        set_new_virtual_path=set_new_virtual_path
                        set_new_local_path=set_new_local_path
                        on_add_mapping=add_mapping
                        on_select_directory=select_directory
                        on_delete_mapping=delete_mapping
                        on_toggle_mapping=toggle_mapping
                        on_browse_files=browse_files
                    />

                    // CORS Settings Section
                    <CorsSettingsSection
                        config=config
                        on_toggle_cors=toggle_cors_mode
                    />

                    // Access Logs Section
                    <AccessLogsSection
                        logs=logs
                        on_clear=clear_logs
                    />

                    // File Browser Section
                    <FileBrowserSection
                        selected_mapping_id=selected_mapping_id
                        files=files
                        browsing_path=browsing_path
                        on_navigate=navigate_to
                        on_close=close_file_browser
                    />
                </Show>
            </div>
        </div>
    }
}
