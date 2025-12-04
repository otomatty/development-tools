//! Project Dashboard Component
//!
//! Main dashboard for a single project with kanban board, repository linking,
//! and issue management features.
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
use crate::components::issues::{
    CreateIssueModal, IssueClickEvent, IssueDetailModal, IssueDetailStatusChange, KanbanBoard,
    LinkRepositoryModal, StatusChangeEvent,
};
use crate::tauri_api;
use crate::types::{
    issue::{CachedIssue, KanbanBoard as KanbanBoardType, Project},
    AppPage,
};

/// Project dashboard component
#[component]
pub fn ProjectDashboardPage(
    project_id: i64,
    set_current_page: WriteSignal<AppPage>,
) -> impl IntoView {
    let (project, set_project) = signal(Option::<Project>::None);
    let (kanban, set_kanban) = signal(KanbanBoardType::default());
    let (loading, set_loading) = signal(true);
    let (syncing, set_syncing) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (show_link_modal, set_show_link_modal) = signal(false);
    let (show_create_issue_modal, set_show_create_issue_modal) = signal(false);
    let (actions_yaml, set_actions_yaml) = signal(Option::<String>::None);
    let (show_actions_modal, set_show_actions_modal) = signal(false);
    let (selected_issue, set_selected_issue) = signal(Option::<CachedIssue>::None);

    // Load project and issues on mount
    {
        spawn_local(async move {
            // Load project
            match tauri_api::get_project(project_id).await {
                Ok(p) => {
                    set_project.set(Some(p));
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to load project: {}", e)));
                    set_loading.set(false);
                    return;
                }
            }

            // Load kanban board
            match tauri_api::get_kanban_board(project_id).await {
                Ok(board) => {
                    set_kanban.set(board);
                }
                Err(e) => {
                    // This might fail if no issues yet, which is fine
                    web_sys::console::log_1(&format!("Note: {}", e).into());
                }
            }

            set_loading.set(false);
        });
    }

    // Sync issues from GitHub
    let sync_issues = move |_| {
        set_syncing.set(true);
        spawn_local(async move {
            match tauri_api::sync_project_issues(project_id).await {
                Ok(issues) => {
                    // Rebuild kanban from synced issues
                    let board = KanbanBoardType::from_issues(issues);
                    set_kanban.set(board);

                    // Update project's last_synced_at
                    if let Ok(updated_project) = tauri_api::get_project(project_id).await {
                        set_project.set(Some(updated_project));
                    }
                }
                Err(e) => {
                    set_error.set(Some(format!("Sync failed: {}", e)));
                }
            }
            set_syncing.set(false);
        });
    };

    // Setup GitHub Actions
    let setup_actions = move |_| {
        spawn_local(async move {
            match tauri_api::setup_github_actions(project_id).await {
                Ok(yaml) => {
                    set_actions_yaml.set(Some(yaml));
                    set_show_actions_modal.set(true);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to generate Actions template: {}", e)));
                }
            }
        });
    };

    // Handle repository linked
    let on_repo_linked = move |updated_project: Project| {
        set_project.set(Some(updated_project));
        set_show_link_modal.set(false);
    };

    // Handle issue created
    let on_issue_created = move |issue: CachedIssue| {
        set_kanban.update(|board| {
            let status = issue.get_status();
            match status {
                crate::types::issue::IssueStatus::Backlog => board.backlog.push(issue),
                crate::types::issue::IssueStatus::Todo => board.todo.push(issue),
                crate::types::issue::IssueStatus::InProgress => board.in_progress.push(issue),
                crate::types::issue::IssueStatus::InReview => board.in_review.push(issue),
                crate::types::issue::IssueStatus::Done => board.done.push(issue),
                crate::types::issue::IssueStatus::Cancelled => board.cancelled.push(issue),
            }
        });
        set_show_create_issue_modal.set(false);
    };

    // Handle status change - using signal-based approach for Leptos thread safety
    let (status_change_event, set_status_change_event) = signal(Option::<StatusChangeEvent>::None);
    let (issue_click_event, set_issue_click_event) = signal(Option::<IssueClickEvent>::None);

    // React to issue click events
    Effect::new(move |_| {
        if let Some(event) = issue_click_event.get() {
            set_selected_issue.set(Some(event.issue));
            set_issue_click_event.set(None);
        }
    });

    // React to status change events
    Effect::new(move |_| {
        if let Some(event) = status_change_event.get() {
            let issue_number = event.issue_number;
            let new_status = event.new_status.clone();
            spawn_local(async move {
                if let Err(e) =
                    tauri_api::update_issue_status(project_id, issue_number, &new_status).await
                {
                    set_error.set(Some(format!("Failed to update status: {}", e)));
                } else {
                    // Refresh kanban board
                    if let Ok(board) = tauri_api::get_kanban_board(project_id).await {
                        set_kanban.set(board);
                    }
                }
            });
            // Clear the event after processing
            set_status_change_event.set(None);
        }
    });

    view! {
        <div class="flex-1 flex flex-col overflow-hidden">
            // Header
            <div class="p-4 border-b border-slate-700/50 bg-dt-card/50">
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-4">
                        // Back button
                        <button
                            class="p-2 text-dt-text-sub hover:text-dt-text hover:bg-slate-800 rounded-lg transition-colors"
                            on:click=move |_| set_current_page.set(AppPage::Projects)
                        >
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/>
                            </svg>
                        </button>

                        // Project info
                        <Show when=move || project.get().is_some()>
                            {move || {
                                let p = project.get().unwrap();
                                let repo_name = p.repo_full_name.clone();
                                view! {
                                    <div>
                                        <h1 class="text-xl font-bold text-dt-text font-gaming">{p.name}</h1>
                                        {repo_name.map(|name| view! {
                                            <div class="flex items-center gap-1 text-sm text-dt-text-sub">
                                                <Icon name="github".to_string() class="w-4 h-4".to_string() />
                                                <span>{name}</span>
                                            </div>
                                        })}
                                    </div>
                                }
                            }}
                        </Show>
                    </div>

                    // Actions
                    <div class="flex items-center gap-2">
                        // Link repository button (if not linked)
                        <Show when=move || project.get().map(|p| !p.is_linked()).unwrap_or(false)>
                            <button
                                class="flex items-center gap-2 px-3 py-1.5 text-sm text-dt-text-sub hover:text-dt-text border border-slate-700 hover:border-gm-accent-cyan rounded-lg transition-colors"
                                on:click=move |_| set_show_link_modal.set(true)
                            >
                                <Icon name="link".to_string() class="w-4 h-4".to_string() />
                                <span>"Link Repository"</span>
                            </button>
                        </Show>

                        // Setup Actions button (if linked but not setup)
                        <Show when=move || project.get().map(|p| p.is_linked() && !p.is_actions_setup).unwrap_or(false)>
                            <button
                                class="flex items-center gap-2 px-3 py-1.5 text-sm text-yellow-400 border border-yellow-400/50 hover:bg-yellow-400/10 rounded-lg transition-colors"
                                on:click=setup_actions
                            >
                                <Icon name="settings".to_string() class="w-4 h-4".to_string() />
                                <span>"Setup Actions"</span>
                            </button>
                        </Show>

                        // Sync button (if linked)
                        <Show when=move || project.get().map(|p| p.is_linked()).unwrap_or(false)>
                            <button
                                class="flex items-center gap-2 px-3 py-1.5 text-sm text-dt-text-sub hover:text-dt-text border border-slate-700 hover:border-gm-accent-cyan rounded-lg transition-colors disabled:opacity-50"
                                disabled=move || syncing.get()
                                on:click=sync_issues
                            >
                                {move || if syncing.get() {
                                    view! {
                                        <Icon name="refresh".to_string() class="w-4 h-4 animate-spin".to_string() />
                                    }.into_any()
                                } else {
                                    view! {
                                        <Icon name="refresh".to_string() class="w-4 h-4".to_string() />
                                    }.into_any()
                                }}
                                <span>{move || if syncing.get() { "Syncing..." } else { "Sync" }}</span>
                            </button>
                        </Show>

                        // Create issue button
                        <Show when=move || project.get().map(|p| p.is_linked()).unwrap_or(false)>
                            <button
                                class="flex items-center gap-2 px-3 py-1.5 text-sm bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple text-white rounded-lg hover:opacity-90 transition-opacity"
                                on:click=move |_| set_show_create_issue_modal.set(true)
                            >
                                <Icon name="plus".to_string() class="w-4 h-4".to_string() />
                                <span>"New Issue"</span>
                            </button>
                        </Show>
                    </div>
                </div>

                // Error message
                <Show when=move || error.get().is_some()>
                    <div class="mt-3 p-3 bg-red-500/20 border border-red-500/50 rounded-lg text-sm text-red-400">
                        {move || error.get().unwrap_or_default()}
                        <button
                            class="ml-2 underline"
                            on:click=move |_| set_error.set(None)
                        >
                            "Dismiss"
                        </button>
                    </div>
                </Show>
            </div>

            // Main content
            <div class="flex-1 overflow-hidden">
                // Loading state
                <Show when=move || loading.get()>
                    <div class="flex items-center justify-center h-full">
                        <div class="animate-spin w-8 h-8 border-2 border-gm-accent-cyan border-t-transparent rounded-full"/>
                    </div>
                </Show>

                // Not linked state
                <Show when=move || !loading.get() && project.get().map(|p| !p.is_linked()).unwrap_or(false)>
                    <div class="flex flex-col items-center justify-center h-full text-center px-4">
                        <Icon name="github".to_string() class="w-20 h-20 text-slate-600 mb-4".to_string() />
                        <h2 class="text-xl font-semibold text-dt-text mb-2">"Link a GitHub Repository"</h2>
                        <p class="text-dt-text-sub mb-6 max-w-md">
                            "Connect a GitHub repository to start managing issues with your kanban board."
                        </p>
                        <button
                            class="flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple text-white rounded-lg hover:opacity-90 transition-opacity"
                            on:click=move |_| set_show_link_modal.set(true)
                        >
                            <Icon name="link".to_string() class="w-5 h-5".to_string() />
                            <span>"Link Repository"</span>
                        </button>
                    </div>
                </Show>

                // Kanban board (when linked)
                <Show when=move || !loading.get() && project.get().map(|p| p.is_linked()).unwrap_or(false)>
                    <KanbanBoard
                        board=kanban
                        status_change_signal=set_status_change_event
                        issue_click_signal=set_issue_click_event
                    />
                </Show>
            </div>

            // Link Repository Modal
            {
                let visible = Memo::new(move |_| show_link_modal.get());
                view! {
                    <LinkRepositoryModal
                        project_id=project_id
                        visible=visible
                        on_close=move || set_show_link_modal.set(false)
                        on_linked=on_repo_linked
                    />
                }
            }

            // Create Issue Modal
            {
                let visible = Memo::new(move |_| show_create_issue_modal.get());
                view! {
                    <CreateIssueModal
                        project_id=project_id
                        visible=visible
                        on_close=move || set_show_create_issue_modal.set(false)
                        on_created=on_issue_created
                    />
                }
            }

            // GitHub Actions Setup Modal
            <Show when=move || show_actions_modal.get()>
                <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
                    <div class="bg-dt-card border border-slate-700/50 rounded-lg p-6 w-full max-w-2xl mx-4 max-h-[80vh] overflow-y-auto">
                        <div class="flex items-center justify-between mb-4">
                            <h2 class="text-xl font-semibold text-dt-text">"GitHub Actions Setup"</h2>
                            <button
                                class="p-1 text-dt-text-sub hover:text-dt-text"
                                on:click=move |_| set_show_actions_modal.set(false)
                            >
                                <Icon name="x".to_string() class="w-5 h-5".to_string() />
                            </button>
                        </div>

                        <p class="text-dt-text-sub mb-4">
                            "Copy the following YAML content and create a new file at "
                            <code class="px-1 py-0.5 bg-slate-800 rounded text-gm-accent-cyan">".github/workflows/issue-status.yml"</code>
                            " in your repository:"
                        </p>

                        <div class="relative">
                            <pre class="p-4 bg-slate-900 rounded-lg text-sm text-slate-300 overflow-x-auto">
                                <code>{move || actions_yaml.get().unwrap_or_default()}</code>
                            </pre>
                            <button
                                class="absolute top-2 right-2 p-2 text-dt-text-sub hover:text-dt-text bg-slate-800 rounded"
                                title="Copy to clipboard"
                                on:click=move |_| {
                                    if let Some(yaml) = actions_yaml.get() {
                                        if let Some(window) = web_sys::window() {
                                            let clipboard = window.navigator().clipboard();
                                            let _ = clipboard.write_text(&yaml);
                                        }
                                    }
                                }
                            >
                                <Icon name="clipboard-copy".to_string() class="w-4 h-4".to_string() />
                            </button>
                        </div>

                        <div class="mt-4 p-3 bg-yellow-500/10 border border-yellow-500/30 rounded-lg">
                            <p class="text-sm text-yellow-400">
                                <strong>"Note:"</strong>
                                " After adding this file to your repository, the workflow will automatically update issue status based on branch activity."
                            </p>
                        </div>

                        <div class="flex justify-end mt-6">
                            <button
                                class="px-4 py-2 bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple text-white rounded-lg hover:opacity-90 transition-opacity"
                                on:click=move |_| set_show_actions_modal.set(false)
                            >
                                "Got it!"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            // Issue Detail Modal
            {move || {
                let issue = selected_issue.get();
                let visible = Memo::new(move |_| selected_issue.get().is_some());
                let (detail_status_change, set_detail_status_change) = signal(Option::<IssueDetailStatusChange>::None);

                // Watch for status change from detail modal
                Effect::new(move |_| {
                    if let Some(event) = detail_status_change.get() {
                        // Trigger status change via the existing signal
                        set_status_change_event.set(Some(StatusChangeEvent {
                            issue_number: event.issue_number,
                            new_status: event.new_status,
                        }));
                        // Close the modal after status change
                        set_selected_issue.set(None);
                    }
                });

                issue.map(|issue| {
                    view! {
                        <IssueDetailModal
                            issue=issue
                            visible=visible
                            on_close=move || set_selected_issue.set(None)
                            status_change_signal=set_detail_status_change
                        />
                    }
                })
            }}
        </div>
    }
}
