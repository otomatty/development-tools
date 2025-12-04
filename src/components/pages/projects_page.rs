//! Projects Page Component
//!
//! Displays a list of all projects with options to create, edit, and delete projects.
//!
//! DEPENDENCY MAP:
//!
//! Parents:
//!   └─ src/components/pages/mod.rs
//! Dependencies:
//!   ├─ src/types/issue.rs
//!   ├─ src/tauri_api.rs
//!   ├─ src/components/icons.rs
//!   ├─ src/components/features/issues/create_project_modal.rs
//!   └─ src/components/features/issues/project_card.rs

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::features::issues::{CreateProjectModal, ProjectCard, ProjectsEmptyState};
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
    let delete_project = Callback::new(move |project_id: i64| {
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
    });

    // Navigate to project detail
    let navigate_to_project = Callback::new(move |project_id: i64| {
        set_current_page.set(AppPage::ProjectDetail(project_id));
    });

    // Show create modal
    let show_create = Callback::new(move |_: ()| {
        set_show_create_modal.set(true);
    });

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
                            view! {
                                <ProjectCard
                                    project=project
                                    on_click=navigate_to_project
                                    on_delete=delete_project
                                />
                            }
                        }).collect_view()
                    }}

                    // Empty state
                    <Show when=move || projects.get().is_empty()>
                        <ProjectsEmptyState on_create=show_create />
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
