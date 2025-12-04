//! Projects Page Component
//!
//! Displays a list of all projects with options to create, edit, and delete projects.
//!
//! DEPENDENCY MAP:
//!
//! Parents:
//!   └─ src/components/issues/mod.rs
//! Dependencies:
//!   ├─ src/types/issue.rs
//!   ├─ src/tauri_api.rs
//!   ├─ src/components/icons.rs
//!   └─ src/components/issues/create_project_modal.rs

use leptos::prelude::*;
use leptos::task::spawn_local;

use super::CreateProjectModal;
use crate::components::icons::Icon;
use crate::tauri_api;
use crate::types::{issue::Project, AppPage};

/// Projects list page component
#[component]
pub fn ProjectsPage(set_current_page: WriteSignal<AppPage>) -> impl IntoView {
    let (projects, set_projects) = signal(Vec::<Project>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (show_create_modal, set_show_create_modal) = signal(false);

    // Load projects on mount
    {
        spawn_local(async move {
            match tauri_api::get_projects().await {
                Ok(list) => {
                    set_projects.set(list);
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        });
    }

    // Handle project created callback
    let on_project_created = move |project: Project| {
        set_projects.update(|p| p.push(project));
    };

    // Delete project handler
    let delete_project = move |project_id: i64| {
        spawn_local(async move {
            match tauri_api::delete_project(project_id).await {
                Ok(_) => {
                    set_projects.update(|p| p.retain(|proj| proj.id != project_id));
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
        });
    };

    view! {
        <div class="flex-1 p-6 overflow-y-auto">
            // Header
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h1 class="text-2xl font-bold text-dt-text font-gaming">"Projects"</h1>
                    <p class="text-dt-text-sub mt-1">"Manage your GitHub issues with kanban boards"</p>
                </div>
                <button
                    class="flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple text-white rounded-lg hover:opacity-90 transition-opacity"
                    on:click=move |_| set_show_create_modal.set(true)
                >
                    <Icon name="plus".to_string() class="w-5 h-5".to_string() />
                    <span>"New Project"</span>
                </button>
            </div>

            // Error message
            <Show when=move || error.get().is_some()>
                <div class="mb-4 p-4 bg-red-500/20 border border-red-500/50 rounded-lg text-red-400">
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            // Loading state
            <Show when=move || loading.get()>
                <div class="flex items-center justify-center py-12">
                    <div class="animate-spin w-8 h-8 border-2 border-gm-accent-cyan border-t-transparent rounded-full"/>
                </div>
            </Show>

            // Projects grid
            <Show when=move || !loading.get()>
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    {move || {
                        projects.get().into_iter().map(|project| {
                            let project_id_for_click = project.id;
                            let project_id_for_delete = project.id;
                            let project_name = project.name.clone();
                            let project_description = project.description.clone();
                            let project_repo_full_name = project.repo_full_name.clone();
                            let project_is_linked = project.is_linked();
                            let project_is_actions_setup = project.is_actions_setup;
                            let project_last_synced_at = project.last_synced_at.clone();

                            // Clone for inner closures
                            let repo_full_name_check = project_repo_full_name.clone();
                            let repo_full_name_display = project_repo_full_name.clone();
                            let description_check = project_description.clone();
                            let description_display = project_description.clone();
                            let last_synced_check = project_last_synced_at.clone();
                            let last_synced_display = project_last_synced_at.clone();

                            view! {
                                <div class="bg-dt-card border border-slate-700/50 rounded-lg p-4 hover:border-gm-accent-cyan/50 transition-colors cursor-pointer group">
                                    // Card content (clickable)
                                    <div
                                        class="flex-1"
                                        on:click=move |_| {
                                            set_current_page.set(AppPage::ProjectDetail(project_id_for_click));
                                        }
                                    >
                                        <div class="flex items-start justify-between">
                                            <div class="flex items-center gap-3">
                                                <div class="p-2 bg-gradient-to-br from-gm-accent-cyan/20 to-gm-accent-purple/20 rounded-lg">
                                                    <Icon name="kanban".to_string() class="w-5 h-5 text-gm-accent-cyan".to_string() />
                                                </div>
                                                <div>
                                                    <h3 class="font-semibold text-dt-text group-hover:text-gm-accent-cyan transition-colors">
                                                        {project_name}
                                                    </h3>
                                                    {repo_full_name_display.map(|name| view! {
                                                        <div class="flex items-center gap-1 text-xs text-dt-text-sub mt-1">
                                                            <Icon name="github".to_string() class="w-3 h-3".to_string() />
                                                            <span>{name}</span>
                                                        </div>
                                                    })}
                                                </div>
                                            </div>
                                        </div>

                                        {description_display.map(|desc| view! {
                                            <p class="text-sm text-dt-text-sub mt-3 line-clamp-2">
                                                {desc}
                                            </p>
                                        })}

                                        <div class="flex items-center justify-between mt-4 pt-3 border-t border-slate-700/50">
                                            // Status indicators
                                            <div class="flex items-center gap-3 text-xs text-dt-text-sub">
                                                {if project_is_linked {
                                                    view! {
                                                        <span class="flex items-center gap-1">
                                                            <span class="w-2 h-2 rounded-full bg-green-500"/>
                                                            "Linked"
                                                        </span>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <span class="flex items-center gap-1">
                                                            <span class="w-2 h-2 rounded-full bg-yellow-500"/>
                                                            "Not linked"
                                                        </span>
                                                    }.into_any()
                                                }}
                                                {if project_is_actions_setup {
                                                    Some(view! {
                                                        <span class="flex items-center gap-1">
                                                            <Icon name="check".to_string() class="w-3 h-3 text-green-500".to_string() />
                                                            "Actions"
                                                        </span>
                                                    })
                                                } else {
                                                    None
                                                }}
                                            </div>

                                            // Last synced
                                            {last_synced_display.map(|synced| view! {
                                                <span class="text-xs text-dt-text-sub">
                                                    "Synced: " {synced}
                                                </span>
                                            })}
                                        </div>
                                    </div>

                                    // Delete button (separate click handler)
                                    <div class="flex justify-end mt-2">
                                        <button
                                            class="p-1.5 text-slate-500 hover:text-red-400 hover:bg-red-400/10 rounded transition-colors opacity-0 group-hover:opacity-100"
                                            title="Delete project"
                                            on:click=move |e| {
                                                e.stop_propagation();
                                                delete_project(project_id_for_delete);
                                            }
                                        >
                                            <Icon name="trash".to_string() class="w-4 h-4".to_string() />
                                        </button>
                                    </div>
                                </div>
                            }
                        }).collect_view()
                    }}

                    // Empty state
                    <Show when=move || projects.get().is_empty()>
                        <div class="col-span-full text-center py-12">
                            <Icon name="kanban".to_string() class="w-16 h-16 mx-auto text-slate-600 mb-4".to_string() />
                            <h3 class="text-lg font-semibold text-dt-text mb-2">"No projects yet"</h3>
                            <p class="text-dt-text-sub mb-4">"Create a project to start managing your GitHub issues"</p>
                            <button
                                class="px-4 py-2 bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple text-white rounded-lg hover:opacity-90 transition-opacity"
                                on:click=move |_| set_show_create_modal.set(true)
                            >
                                "Create your first project"
                            </button>
                        </div>
                    </Show>
                </div>
            </Show>

            // Create project modal
            <CreateProjectModal
                visible=Signal::derive(move || show_create_modal.get())
                on_close=move || set_show_create_modal.set(false)
                on_created=on_project_created
            />
        </div>
    }
}
