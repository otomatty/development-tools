//! Project Card Component
//!
//! Card component for displaying project information in a grid.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this component):
//!   └─ src/components/pages/projects_page.rs
//! Dependencies:
//!   ├─ src/components/icons.rs
//!   └─ src/types/issue.rs
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

use leptos::prelude::*;

use crate::components::icons::Icon;
use crate::types::issue::Project;

/// Project card component
#[component]
pub fn ProjectCard(
    project: Project,
    on_click: Callback<i64>,
    on_delete: Callback<i64>,
) -> impl IntoView {
    let project_id = project.id;
    let project_id_for_delete = project.id;
    let project_name = project.name.clone();
    let project_description = project.description.clone();
    let project_repo_full_name = project.repo_full_name.clone();
    let project_is_linked = project.is_linked();
    let project_is_actions_setup = project.is_actions_setup;
    let project_last_synced_at = project.last_synced_at.clone();

    let repo_full_name_display = project_repo_full_name.clone();
    let description_display = project_description.clone();
    let last_synced_display = project_last_synced_at.clone();

    view! {
        <div class="bg-dt-card border border-slate-700/50 rounded-lg p-4 hover:border-gm-accent-cyan/50 transition-colors cursor-pointer group">
            // Card content (clickable)
            <div
                class="flex-1"
                on:click=move |_| on_click.run(project_id)
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
                        on_delete.run(project_id_for_delete);
                    }
                >
                    <Icon name="trash".to_string() class="w-4 h-4".to_string() />
                </button>
            </div>
        </div>
    }
}

/// Empty state for projects page
#[component]
pub fn ProjectsEmptyState(on_create: Callback<()>) -> impl IntoView {
    view! {
        <div class="col-span-full text-center py-12">
            <Icon name="kanban".to_string() class="w-16 h-16 mx-auto text-slate-600 mb-4".to_string() />
            <h3 class="text-lg font-semibold text-dt-text mb-2">"No projects yet"</h3>
            <p class="text-dt-text-sub mb-4">"Create a project to start managing your GitHub issues"</p>
            <button
                class="px-4 py-2 bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple text-white rounded-lg hover:opacity-90 transition-opacity"
                on:click=move |_| on_create.run(())
            >
                "Create your first project"
            </button>
        </div>
    }
}
