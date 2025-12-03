//! Issue Detail Modal Component
//!
//! Modal dialog displaying detailed information about an issue.
//! Shows title, body, labels, assignee, status, and provides
//! a link to the GitHub issue.
//!
//! DEPENDENCY MAP:
//!
//! Parents:
//!   └─ src/components/issues/project_dashboard.rs
//! Dependencies:
//!   ├─ src/types/issue.rs
//!   ├─ src/components/icons.rs
//!   └─ src/components/ui/dialog/modal.rs

use leptos::prelude::*;

use crate::components::icons::Icon;
use crate::components::ui::dialog::{Modal, ModalBody, ModalFooter, ModalHeader, ModalSize};
use crate::types::issue::{CachedIssue, IssueStatus};
use crate::utils::render_markdown;

/// Event emitted when issue status is changed from the detail modal
#[derive(Clone, Debug)]
pub struct IssueDetailStatusChange {
    pub issue_number: i32,
    pub new_status: String,
}

/// Issue detail modal component
/// Uses WriteSignal for callbacks to ensure thread safety in Leptos
#[component]
pub fn IssueDetailModal(
    issue: CachedIssue,
    #[prop(into)] visible: Signal<bool>,
    on_close: impl Fn() + 'static + Clone + Send + Sync,
    status_change_signal: WriteSignal<Option<IssueDetailStatusChange>>,
) -> impl IntoView {
    let status = issue.get_status();
    let labels = issue.get_labels();
    let priority = issue.get_priority();
    let issue_number = issue.number;

    // Status options for dropdown
    let statuses = StoredValue::new(vec![
        (IssueStatus::Backlog, "Backlog", "bg-gray-400"),
        (IssueStatus::Todo, "Todo", "bg-blue-500"),
        (IssueStatus::InProgress, "In Progress", "bg-yellow-500"),
        (IssueStatus::InReview, "In Review", "bg-purple-500"),
        (IssueStatus::Done, "Done", "bg-green-500"),
        (IssueStatus::Cancelled, "Cancelled", "bg-gray-500"),
    ]);

    let (show_status_dropdown, set_show_status_dropdown) = signal(false);

    // Format dates - Store in StoredValue for ChildrenFn
    let created_at = StoredValue::new(
        issue
            .github_created_at
            .clone()
            .map(|d| format_date(&d))
            .unwrap_or_else(|| "Unknown".to_string()),
    );
    let updated_at = StoredValue::new(
        issue
            .github_updated_at
            .clone()
            .map(|d| format_date(&d))
            .unwrap_or_else(|| "Unknown".to_string()),
    );

    // Store on_close for use in ChildrenFn
    let on_close_stored = StoredValue::new(on_close.clone());
    let on_close_callback = Callback::new(move |_: ()| on_close_stored.get_value()());

    // Store issue data for use in ChildrenFn
    let issue_title = StoredValue::new(issue.title.clone());
    let issue_number_display = issue.number;
    let issue_is_open = issue.is_open();
    let issue_body = StoredValue::new(issue.body.clone());
    let issue_html_url = StoredValue::new(issue.html_url.clone());
    let assignee_login = StoredValue::new(issue.assignee_login.clone());
    let assignee_avatar_url = StoredValue::new(issue.assignee_avatar_url.clone());
    let labels_stored = StoredValue::new(labels.clone());

    view! {
        <Modal
            visible=visible
            on_close=on_close.clone()
            size=ModalSize::TwoXL
        >
            // Header
            <ModalHeader on_close=on_close_callback>
                <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 mb-1">
                        <span class="text-sm text-dt-text-sub font-mono">
                            {"#"}{issue_number_display}
                        </span>
                        // State badge
                        <span class=format!(
                            "px-2 py-0.5 text-xs rounded-full {}",
                            if issue_is_open { "bg-green-500/20 text-green-400" } else { "bg-purple-500/20 text-purple-400" }
                        )>
                            {if issue_is_open { "Open" } else { "Closed" }}
                        </span>
                    </div>
                    <h2 class="text-xl font-semibold text-dt-text break-words">
                        {issue_title.get_value()}
                    </h2>
                </div>
            </ModalHeader>

            // Content (scrollable)
            <ModalBody class="max-h-[60vh]">
                <div class="space-y-6">
                    // Metadata grid
                    <div class="grid grid-cols-2 gap-4">
                        // Status
                        <div>
                            <label class="block text-xs text-dt-text-sub mb-1">"Status"</label>
                            <div class="relative">
                                <button
                                    class=format!(
                                        "flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm text-white {} hover:opacity-80 transition-opacity w-full justify-between",
                                        status.color_class()
                                    )
                                    on:click=move |_| set_show_status_dropdown.update(|v| *v = !*v)
                                >
                                    <span>{status.display_name()}</span>
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
                                    </svg>
                                </button>

                                <Show when=move || show_status_dropdown.get()>
                                    <div class="absolute top-full left-0 mt-1 w-full bg-slate-900 rounded-lg shadow-lg border border-slate-700 py-1 z-10">
                                        {statuses.get_value().iter().map(|(s, label, color)| {
                                            let status_value = match s {
                                                IssueStatus::Backlog => "backlog",
                                                IssueStatus::Todo => "todo",
                                                IssueStatus::InProgress => "in-progress",
                                                IssueStatus::InReview => "in-review",
                                                IssueStatus::Done => "done",
                                                IssueStatus::Cancelled => "cancelled",
                                            };
                                            let value = status_value.to_string();
                                            let label = *label;
                                            let color = *color;
                                            view! {
                                                <button
                                                    class="w-full px-3 py-2 text-left text-sm text-white hover:bg-slate-800 flex items-center gap-2"
                                                    on:click={
                                                        let value = value.clone();
                                                        move |_| {
                                                            status_change_signal.set(Some(IssueDetailStatusChange {
                                                                issue_number,
                                                                new_status: value.clone(),
                                                            }));
                                                            set_show_status_dropdown.set(false);
                                                        }
                                                    }
                                                >
                                                    <span class=format!("w-3 h-3 rounded-full {}", color)></span>
                                                    {label}
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                </Show>
                            </div>
                        </div>

                        // Priority
                        <div>
                            <label class="block text-xs text-dt-text-sub mb-1">"Priority"</label>
                            <div class="flex items-center gap-2 px-3 py-1.5 bg-slate-800 rounded-lg text-sm">
                                {priority.map(|p| {
                                    view! {
                                        <div class="flex items-center gap-1">
                                            <span>{p.emoji()}</span>
                                            <span class={p.color_class()}>{p.display_name()}</span>
                                        </div>
                                    }.into_any()
                                }).unwrap_or_else(|| view! {
                                    <span class="text-dt-text-sub">"Not set"</span>
                                }.into_any())}
                            </div>
                        </div>

                        // Assignee
                        <div>
                            <label class="block text-xs text-dt-text-sub mb-1">"Assignee"</label>
                            <div class="flex items-center gap-2 px-3 py-1.5 bg-slate-800 rounded-lg text-sm">
                                {move || assignee_login.get_value().clone().map(|login| {
                                    let avatar_url = assignee_avatar_url.get_value();
                                    let login_display = login.clone();
                                    view! {
                                        <div class="flex items-center gap-2">
                                            {avatar_url.map(|url| view! {
                                                <img
                                                    src={url}
                                                    class="w-5 h-5 rounded-full"
                                                    alt={login.clone()}
                                                />
                                            })}
                                            <span class="text-dt-text">{"@"}{login_display}</span>
                                        </div>
                                    }.into_any()
                                }).unwrap_or_else(|| view! {
                                    <span class="text-dt-text-sub">"Unassigned"</span>
                                }.into_any())}
                            </div>
                        </div>

                        // Dates
                        <div>
                            <label class="block text-xs text-dt-text-sub mb-1">"Created"</label>
                            <div class="px-3 py-1.5 bg-slate-800 rounded-lg text-sm text-dt-text">
                                {created_at.get_value()}
                            </div>
                        </div>
                    </div>

                    // Labels
                    {
                        let labels_for_display = labels_stored.get_value();
                        let has_labels = !labels_for_display.is_empty();
                        view! {
                            <Show when=move || has_labels>
                                <div>
                                    <label class="block text-xs text-dt-text-sub mb-2">"Labels"</label>
                                    <div class="flex flex-wrap gap-2">
                                        {labels_for_display.iter().map(|label| {
                                            // Filter out status labels
                                            if label.starts_with("status:") || label.starts_with("priority:") {
                                                return view! { <></> }.into_any();
                                            }
                                            view! {
                                                <span class="px-2 py-1 bg-slate-700 rounded-full text-xs text-dt-text">
                                                    {label.clone()}
                                                </span>
                                            }.into_any()
                                        }).collect_view()}
                                    </div>
                                </div>
                            </Show>
                        }
                    }

                    // Description
                    <div>
                        <label class="block text-xs text-dt-text-sub mb-2">"Description"</label>
                        <div class="p-4 bg-slate-800/50 rounded-lg border border-slate-700/50">
                            {move || issue_body.get_value().clone().map(|body| {
                                if body.trim().is_empty() {
                                    view! {
                                        <p class="text-dt-text-sub italic">"No description provided."</p>
                                    }.into_any()
                                } else {
                                    // Render Markdown to HTML
                                    let html_content = render_markdown(&body);
                                    view! {
                                        <div
                                            class="markdown-body"
                                            inner_html=html_content
                                        />
                                    }.into_any()
                                }
                            }).unwrap_or_else(|| view! {
                                <p class="text-dt-text-sub italic">"No description provided."</p>
                            }.into_any())}
                        </div>
                    </div>

                    // Timestamps
                    <div class="text-xs text-dt-text-sub flex items-center gap-4">
                        <span>"Updated: "{updated_at.get_value()}</span>
                    </div>
                </div>
            </ModalBody>

            // Footer
            <ModalFooter>
                {move || issue_html_url.get_value().clone().map(|url| {
                    view! {
                        <a
                            href={url}
                            target="_blank"
                            rel="noopener noreferrer"
                            class="flex items-center gap-2 px-3 py-1.5 text-sm text-dt-text-sub hover:text-dt-text border border-slate-700 hover:border-gm-accent-cyan rounded-lg transition-colors mr-auto"
                        >
                            <Icon name="github".to_string() class="w-4 h-4".to_string() />
                            <span>"View on GitHub"</span>
                        </a>
                    }
                })}
                <button
                    class="px-4 py-1.5 text-sm bg-slate-800 hover:bg-slate-700 text-dt-text rounded-lg transition-colors"
                    on:click=move |_| on_close_stored.get_value()()
                >
                    "Close"
                </button>
            </ModalFooter>
        </Modal>
    }
}

/// Format ISO date string to a more readable format
fn format_date(iso_string: &str) -> String {
    // Parse the ISO date string and format it
    // Simple implementation - just show the date part
    if let Some(date_part) = iso_string.split('T').next() {
        date_part.to_string()
    } else {
        iso_string.to_string()
    }
}
