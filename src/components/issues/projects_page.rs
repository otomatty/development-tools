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
//!   └─ src/components/icons.rs

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::icons::Icon;
use crate::tauri_api;
use crate::types::{issue::{Project, RepositoryInfo}, AppPage};

/// Projects list page component
#[component]
pub fn ProjectsPage(set_current_page: WriteSignal<AppPage>) -> impl IntoView {
    let (projects, set_projects) = signal(Vec::<Project>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (show_create_modal, set_show_create_modal) = signal(false);
    let (new_project_name, set_new_project_name) = signal(String::new());
    let (new_project_description, set_new_project_description) = signal(String::new());
    let (creating, set_creating) = signal(false);
    
    // Repository selection state
    let (repositories, set_repositories) = signal(Vec::<RepositoryInfo>::new());
    let (repos_loading, set_repos_loading) = signal(false);
    let (selected_repo, set_selected_repo) = signal(Option::<RepositoryInfo>::None);
    let (repo_search_query, set_repo_search_query) = signal(String::new());

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

    // Create project handler - requires repository selection
    let create_project = move |_| {
        let repo_info = selected_repo.get();
        
        // Repository is required
        let repo = match repo_info {
            Some(r) => r,
            None => return,
        };
        
        let name = new_project_name.get();
        if name.trim().is_empty() {
            return;
        }

        set_creating.set(true);
        let description = new_project_description.get();
        let desc = if description.trim().is_empty() {
            None
        } else {
            Some(description.clone())
        };

        spawn_local(async move {
            match tauri_api::create_project(&name, desc.as_deref()).await {
                Ok(mut project) => {
                    // Link the repository
                    match tauri_api::link_repository(project.id, &repo.owner, &repo.name).await {
                        Ok(linked_project) => {
                            project = linked_project;
                        }
                        Err(e) => {
                            web_sys::console::error_1(&format!("Failed to link repository: {}", e).into());
                        }
                    }
                    
                    set_projects.update(|p| p.push(project));
                    set_show_create_modal.set(false);
                    set_new_project_name.set(String::new());
                    set_new_project_description.set(String::new());
                    set_selected_repo.set(None);
                    set_repo_search_query.set(String::new());
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_creating.set(false);
        });
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
            <Show when=move || show_create_modal.get()>
                <CreateProjectModal
                    repositories=repositories
                    repos_loading=repos_loading
                    selected_repo=selected_repo
                    set_selected_repo=set_selected_repo
                    repo_search_query=repo_search_query
                    set_repo_search_query=set_repo_search_query
                    new_project_name=new_project_name
                    set_new_project_name=set_new_project_name
                    new_project_description=new_project_description
                    set_new_project_description=set_new_project_description
                    creating=creating
                    on_create=create_project
                    on_close=move |_| {
                        set_show_create_modal.set(false);
                        set_new_project_name.set(String::new());
                        set_new_project_description.set(String::new());
                        set_selected_repo.set(None);
                        set_repo_search_query.set(String::new());
                    }
                    set_repositories=set_repositories
                    set_repos_loading=set_repos_loading
                />
            </Show>
        </div>
    }
}

/// Create project modal with repository selection
#[component]
fn CreateProjectModal(
    repositories: ReadSignal<Vec<RepositoryInfo>>,
    repos_loading: ReadSignal<bool>,
    selected_repo: ReadSignal<Option<RepositoryInfo>>,
    set_selected_repo: WriteSignal<Option<RepositoryInfo>>,
    repo_search_query: ReadSignal<String>,
    set_repo_search_query: WriteSignal<String>,
    new_project_name: ReadSignal<String>,
    set_new_project_name: WriteSignal<String>,
    new_project_description: ReadSignal<String>,
    set_new_project_description: WriteSignal<String>,
    creating: ReadSignal<bool>,
    on_create: impl Fn(web_sys::MouseEvent) + 'static + Copy,
    on_close: impl Fn(web_sys::MouseEvent) + 'static + Copy,
    set_repositories: WriteSignal<Vec<RepositoryInfo>>,
    set_repos_loading: WriteSignal<bool>,
) -> impl IntoView {
    // Load repositories on mount
    {
        let repos_empty = repositories.get().is_empty();
        let is_loading = repos_loading.get();
        if repos_empty && !is_loading {
            set_repos_loading.set(true);
            spawn_local(async move {
                match tauri_api::get_user_repositories().await {
                    Ok(repos) => {
                        set_repositories.set(repos);
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("Failed to load repositories: {}", e).into());
                    }
                }
                set_repos_loading.set(false);
            });
        }
    }
    
    // Filter repositories based on search query
    let filtered_repos = move || {
        let query = repo_search_query.get().to_lowercase();
        if query.is_empty() {
            repositories.get()
        } else {
            repositories
                .get()
                .into_iter()
                .filter(|r| {
                    r.full_name.to_lowercase().contains(&query)
                        || r.description
                            .as_ref()
                            .map(|d| d.to_lowercase().contains(&query))
                            .unwrap_or(false)
                })
                .collect()
        }
    };
    
    // Handle repository selection - auto-fill project name and description
    let on_repo_select = move |repo: RepositoryInfo| {
        // Use repository name as project name
        set_new_project_name.set(repo.name.clone());
        // Use repository description as project description
        set_new_project_description.set(repo.description.clone().unwrap_or_default());
        // Set selected repo
        set_selected_repo.set(Some(repo));
        // Clear search
        set_repo_search_query.set(String::new());
    };

    view! {
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-6">
            <div class="bg-dt-card border border-slate-700/50 rounded-lg w-full max-w-4xl h-full max-h-[calc(100vh-3rem)] flex flex-col">
                // Header
                <div class="p-6 border-b border-slate-700/50">
                    <h2 class="text-xl font-semibold text-dt-text">"Create New Project"</h2>
                    <p class="text-sm text-dt-text-sub mt-1">"Select a GitHub repository to create a project"</p>
                </div>
                
                // Content
                <div class="flex-1 overflow-hidden flex">
                    // Left: Repository list
                    <div class="w-1/2 border-r border-slate-700/50 flex flex-col">
                        <div class="p-4 border-b border-slate-700/50">
                            <div class="relative">
                                <Icon name="search".to_string() class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-dt-text-sub".to_string() />
                                <input
                                    type="text"
                                    class="w-full pl-10 pr-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-dt-text focus:border-gm-accent-cyan focus:outline-none"
                                    placeholder="Search repositories..."
                                    prop:value=move || repo_search_query.get()
                                    on:input=move |ev| set_repo_search_query.set(event_target_value(&ev))
                                />
                            </div>
                        </div>
                        
                        <div class="flex-1 overflow-y-auto">
                            // Loading state
                            <Show when=move || repos_loading.get()>
                                <div class="flex items-center justify-center py-12">
                                    <div class="animate-spin w-6 h-6 border-2 border-gm-accent-cyan border-t-transparent rounded-full"/>
                                </div>
                            </Show>
                            
                            // Repository list
                            <Show when=move || !repos_loading.get()>
                                {move || {
                                    let repos = filtered_repos();
                                    if repos.is_empty() {
                                        view! {
                                            <div class="px-4 py-12 text-center text-dt-text-sub">
                                                <Icon name="github".to_string() class="w-12 h-12 mx-auto mb-3 opacity-50".to_string() />
                                                <p>"No repositories found"</p>
                                            </div>
                                        }.into_any()
                                    } else {
                                        repos.into_iter().map(|repo| {
                                            let repo_clone = repo.clone();
                                            let repo_for_check = repo.clone();
                                            let repo_full_name = repo.full_name.clone();
                                            let repo_description = repo.description.clone();
                                            let repo_private = repo.private;
                                            let repo_issues_count = repo.open_issues_count;
                                            
                                            view! {
                                                <button
                                                    type="button"
                                                    class=move || format!(
                                                        "w-full p-4 text-left border-b border-slate-700/50 transition-colors {}",
                                                        if selected_repo.get().as_ref().map(|r| r.id == repo_for_check.id).unwrap_or(false) {
                                                            "bg-gm-accent-cyan/10 border-l-2 border-l-gm-accent-cyan"
                                                        } else {
                                                            "hover:bg-slate-800/50"
                                                        }
                                                    )
                                                    on:click=move |_| on_repo_select(repo_clone.clone())
                                                >
                                                    <div class="flex items-start gap-3">
                                                        <Icon name="github".to_string() class="w-5 h-5 text-dt-text-sub mt-0.5".to_string() />
                                                        <div class="flex-1 min-w-0">
                                                            <div class="flex items-center gap-2">
                                                                <span class="font-medium text-dt-text truncate">{repo_full_name}</span>
                                                                {if repo_private {
                                                                    Some(view! {
                                                                        <span class="px-1.5 py-0.5 text-xs bg-yellow-500/20 text-yellow-400 rounded">
                                                                            "Private"
                                                                        </span>
                                                                    })
                                                                } else {
                                                                    None
                                                                }}
                                                            </div>
                                                            {repo_description.map(|desc| view! {
                                                                <p class="text-sm text-dt-text-sub mt-1 line-clamp-2">{desc}</p>
                                                            })}
                                                            <div class="flex items-center gap-3 mt-2 text-xs text-dt-text-sub">
                                                                <span class="flex items-center gap-1">
                                                                    <Icon name="alert-circle".to_string() class="w-3 h-3".to_string() />
                                                                    {repo_issues_count} " open issues"
                                                                </span>
                                                            </div>
                                                        </div>
                                                    </div>
                                                </button>
                                            }
                                        }).collect_view().into_any()
                                    }
                                }}
                            </Show>
                        </div>
                    </div>
                    
                    // Right: Project details form
                    <div class="w-1/2 p-6 flex flex-col">
                        <Show
                            when=move || selected_repo.get().is_some()
                            fallback=move || view! {
                                <div class="flex-1 flex items-center justify-center text-center">
                                    <div>
                                        <Icon name="arrow-left".to_string() class="w-12 h-12 mx-auto text-dt-text-sub opacity-50 mb-4".to_string() />
                                        <h3 class="text-lg font-medium text-dt-text mb-2">"Select a Repository"</h3>
                                        <p class="text-sm text-dt-text-sub">"Choose a repository from the list to create a project"</p>
                                    </div>
                                </div>
                            }
                        >
                            <div class="space-y-6">
                                // Selected repository info
                                <div class="p-4 bg-slate-800/50 rounded-lg border border-slate-700/50">
                                    <div class="flex items-center gap-3">
                                        <Icon name="github".to_string() class="w-6 h-6 text-gm-accent-cyan".to_string() />
                                        <div>
                                            <p class="font-medium text-dt-text">{move || selected_repo.get().map(|r| r.full_name.clone()).unwrap_or_default()}</p>
                                            <p class="text-xs text-dt-text-sub">"Selected repository"</p>
                                        </div>
                                    </div>
                                </div>
                                
                                // Project Name
                                <div>
                                    <label class="block text-sm font-medium text-dt-text-sub mb-2">"Project Name"</label>
                                    <input
                                        type="text"
                                        class="w-full px-4 py-3 bg-slate-800 border border-slate-700 rounded-lg text-dt-text focus:border-gm-accent-cyan focus:outline-none"
                                        placeholder="My Awesome Project"
                                        prop:value=move || new_project_name.get()
                                        on:input=move |ev| set_new_project_name.set(event_target_value(&ev))
                                    />
                                </div>
                                
                                // Description
                                <div>
                                    <label class="block text-sm font-medium text-dt-text-sub mb-2">"Description"</label>
                                    <textarea
                                        class="w-full px-4 py-3 bg-slate-800 border border-slate-700 rounded-lg text-dt-text focus:border-gm-accent-cyan focus:outline-none resize-none"
                                        rows="4"
                                        placeholder="A brief description of your project..."
                                        prop:value=move || new_project_description.get()
                                        on:input=move |ev| set_new_project_description.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>
                        </Show>
                    </div>
                </div>

                // Footer
                <div class="p-6 border-t border-slate-700/50 flex justify-end gap-4">
                    <button
                        class="px-6 py-2.5 text-dt-text-sub hover:text-dt-text transition-colors"
                        on:click=on_close
                    >
                        "Cancel"
                    </button>
                    <button
                        class="px-6 py-2.5 bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple text-white rounded-lg hover:opacity-90 transition-opacity disabled:opacity-50 disabled:cursor-not-allowed"
                        disabled=move || creating.get() || selected_repo.get().is_none() || new_project_name.get().trim().is_empty()
                        on:click=on_create
                    >
                        {move || if creating.get() { "Creating..." } else { "Create Project" }}
                    </button>
                </div>
            </div>
        </div>
    }
}
