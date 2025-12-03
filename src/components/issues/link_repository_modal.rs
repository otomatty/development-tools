//! Link Repository Modal Component
//!
//! Modal dialog for linking a GitHub repository to a project.
//!
//! DEPENDENCY MAP:
//!
//! Parents:
//!   └─ src/components/issues/project_dashboard.rs
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

/// Link repository modal component
#[component]
pub fn LinkRepositoryModal(
    project_id: i64,
    #[prop(into)] visible: Signal<bool>,
    on_close: impl Fn() + 'static + Clone + Send + Sync,
    on_linked: impl Fn(Project) + 'static + Copy,
) -> impl IntoView {
    let (repositories, set_repositories) = signal(Vec::<RepositoryInfo>::new());
    let (loading, set_loading) = signal(true);
    let (linking, set_linking) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (search_query, set_search_query) = signal(String::new());
    let (linked_project, set_linked_project) = signal(Option::<Project>::None);

    // Load repositories on mount
    {
        spawn_local(async move {
            match tauri_api::get_user_repositories().await {
                Ok(repos) => {
                    set_repositories.set(repos);
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        });
    }

    // Watch for successful link
    Effect::new(move |_| {
        if let Some(project) = linked_project.get() {
            on_linked(project);
        }
    });

    // Filter repositories based on search
    let filtered_repos = move || {
        let query = search_query.get().to_lowercase();
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

    // Store on_close for use in ChildrenFn
    let on_close_stored = StoredValue::new(on_close.clone());
    let on_close_callback = Callback::new(move |_: ()| on_close_stored.get_value()());

    view! {
        <Modal
            visible=visible
            on_close=on_close.clone()
            size=ModalSize::XLarge
        >
            // Header
            <ModalHeader on_close=on_close_callback>
                <h2 class="text-lg font-semibold text-dt-text">"Link GitHub Repository"</h2>
            </ModalHeader>

            // Search
            <div class="p-4 border-b border-slate-700/50">
                <div class="relative">
                    <Icon name="search".to_string() class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-dt-text-sub".to_string() />
                    <input
                        type="text"
                        class="w-full pl-10 pr-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-dt-text focus:border-gm-accent-cyan focus:outline-none"
                        placeholder="Search repositories..."
                        prop:value=move || search_query.get()
                        on:input=move |ev| set_search_query.set(event_target_value(&ev))
                    />
                </div>
            </div>

            // Error message
            <Show when=move || error.get().is_some()>
                <div class="mx-4 mt-4 p-3 bg-red-500/20 border border-red-500/50 rounded-lg text-sm text-red-400">
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            // Repository list
            <ModalBody class="max-h-[50vh]">
                // Loading state
                <Show when=move || loading.get()>
                    <div class="flex items-center justify-center py-8">
                        <div class="animate-spin w-6 h-6 border-2 border-gm-accent-cyan border-t-transparent rounded-full"/>
                    </div>
                </Show>

                // Repository list
                <Show when=move || !loading.get()>
                    <div class="space-y-2">
                        {move || {
                            let repos = filtered_repos();
                            if repos.is_empty() {
                                view! {
                                    <div class="text-center py-8 text-dt-text-sub">
                                        <Icon name="github".to_string() class="w-12 h-12 mx-auto mb-3 opacity-50".to_string() />
                                        <p>"No repositories found"</p>
                                    </div>
                                }.into_any()
                            } else {
                                repos.into_iter().map(|repo| {
                                    let repo_owner = repo.owner.clone();
                                    let repo_name = repo.name.clone();
                                    let repo_display_name = repo.full_name.clone();
                                    let repo_description = repo.description.clone();
                                    let repo_private = repo.private;
                                    let repo_issues_count = repo.open_issues_count;

                                    view! {
                                        <button
                                            class="w-full p-3 bg-slate-800/50 border border-slate-700/50 rounded-lg hover:border-gm-accent-cyan/50 transition-colors text-left"
                                            disabled=move || linking.get()
                                            on:click=move |_| {
                                                set_linking.set(true);
                                                let owner = repo_owner.clone();
                                                let name = repo_name.clone();

                                                spawn_local(async move {
                                                    match tauri_api::link_repository(
                                                        project_id,
                                                        &owner,
                                                        &name,
                                                    ).await {
                                                        Ok(project) => {
                                                            set_linked_project.set(Some(project));
                                                        }
                                                        Err(e) => {
                                                            set_error.set(Some(e));
                                                            set_linking.set(false);
                                                        }
                                                    }
                                                });
                                            }
                                        >
                                            <div class="flex items-center gap-3">
                                                <Icon name="github".to_string() class="w-8 h-8 text-dt-text-sub".to_string() />
                                                <div class="flex-1 min-w-0">
                                                    <div class="flex items-center gap-2">
                                                        <span class="font-medium text-dt-text truncate">
                                                            {repo_display_name}
                                                        </span>
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
                                                        <p class="text-sm text-dt-text-sub truncate mt-0.5">
                                                            {desc}
                                                        </p>
                                                    })}
                                                    <div class="flex items-center gap-3 mt-1 text-xs text-dt-text-sub">
                                                        <span class="flex items-center gap-1">
                                                            <Icon name="alert-circle".to_string() class="w-3 h-3".to_string() />
                                                            {repo_issues_count} " issues"
                                                        </span>
                                                    </div>
                                                </div>
                                                <Icon name="arrow-right".to_string() class="w-5 h-5 text-dt-text-sub".to_string() />
                                            </div>
                                        </button>
                                    }
                                }).collect_view().into_any()
                            }
                        }}
                    </div>
                </Show>
            </ModalBody>

            // Footer
            <ModalFooter>
                <button
                    class="py-2 px-4 text-dt-text-sub hover:text-dt-text transition-colors"
                    on:click=move |_| on_close_stored.get_value()()
                >
                    "Cancel"
                </button>
            </ModalFooter>
        </Modal>
    }
}
