//! Create Project Modal Component
//!
//! Modal dialog for creating a new project with repository selection.
//!
//! DEPENDENCY MAP:
//!
//! Parents:
//!   └─ src/components/issues/projects_page.rs
//! Dependencies:
//!   ├─ src/types/issue.rs
//!   ├─ src/tauri_api.rs
//!   ├─ src/components/icons.rs
//!   └─ src/components/ui/dialog/modal.rs

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::icons::Icon;
use crate::components::ui::dialog::{Modal, ModalBody, ModalFooter, ModalHeader, ModalSize};
use crate::tauri_api;
use crate::types::issue::{Project, RepositoryInfo};

/// Create project modal with repository selection
#[component]
pub fn CreateProjectModal(
    #[prop(into)] visible: Signal<bool>,
    on_close: impl Fn() + 'static + Clone + Send + Sync,
    on_created: impl Fn(Project) + 'static + Clone + Send + Sync,
) -> impl IntoView {
    // Form state
    let (new_project_name, set_new_project_name) = signal(String::new());
    let (new_project_description, set_new_project_description) = signal(String::new());
    let (creating, set_creating) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    // Repository selection state
    let (repositories, set_repositories) = signal(Vec::<RepositoryInfo>::new());
    let (repos_loading, set_repos_loading) = signal(false);
    let (selected_repo, set_selected_repo) = signal(Option::<RepositoryInfo>::None);
    let (repo_search_query, set_repo_search_query) = signal(String::new());

    // Load repositories when modal becomes visible
    Effect::new(move |_| {
        if visible.get()
            && repositories.get_untracked().is_empty()
            && !repos_loading.get_untracked()
        {
            set_repos_loading.set(true);
            spawn_local(async move {
                match tauri_api::get_user_repositories().await {
                    Ok(repos) => {
                        set_repositories.set(repos);
                    }
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("Failed to load repositories: {}", e).into(),
                        );
                        set_error.set(Some(format!("Failed to load repositories: {}", e)));
                    }
                }
                set_repos_loading.set(false);
            });
        }
    });

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

    // Store callbacks for use in async contexts
    let on_close_stored = StoredValue::new(on_close.clone());
    let on_created_stored = StoredValue::new(on_created);
    let on_close_header = Callback::new(move |_: ()| on_close_stored.get_value()());

    // Reset form when closing
    let reset_form = move || {
        set_new_project_name.set(String::new());
        set_new_project_description.set(String::new());
        set_selected_repo.set(None);
        set_repo_search_query.set(String::new());
        set_error.set(None);
    };

    // Create project handler
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
        set_error.set(None);
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
                            web_sys::console::error_1(
                                &format!("Failed to link repository: {}", e).into(),
                            );
                        }
                    }

                    // Notify parent and close modal
                    on_created_stored.get_value()(project);
                    reset_form();
                    on_close_stored.get_value()();
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_creating.set(false);
        });
    };

    // Handle close with reset
    let handle_close = move || {
        reset_form();
        on_close_stored.get_value()();
    };

    view! {
        <Modal
            visible=visible
            on_close=handle_close.clone()
            size=ModalSize::Full
        >
            <ModalHeader on_close=on_close_header>
                <div>
                    <h2 class="text-xl font-semibold text-dt-text">"Create New Project"</h2>
                    <p class="text-sm text-dt-text-sub mt-1">"Select a GitHub repository to create a project"</p>
                </div>
            </ModalHeader>

            <ModalBody class="p-0 max-h-[60vh]">
                // Error message
                <Show when=move || error.get().is_some()>
                    <div class="mx-6 mt-4 p-4 bg-red-500/20 border border-red-500/50 rounded-lg text-red-400">
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>

                <div class="flex h-full min-h-[400px]">
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
            </ModalBody>

            <ModalFooter>
                <div class="flex justify-end gap-4 w-full">
                    <button
                        class="px-6 py-2.5 text-dt-text-sub hover:text-dt-text transition-colors"
                        on:click=move |_| handle_close()
                    >
                        "Cancel"
                    </button>
                    <button
                        class="px-6 py-2.5 bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple text-white rounded-lg hover:opacity-90 transition-opacity disabled:opacity-50 disabled:cursor-not-allowed"
                        disabled=move || creating.get() || selected_repo.get().is_none() || new_project_name.get().trim().is_empty()
                        on:click=create_project
                    >
                        {move || if creating.get() { "Creating..." } else { "Create Project" }}
                    </button>
                </div>
            </ModalFooter>
        </Modal>
    }
}
