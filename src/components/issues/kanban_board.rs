//! Kanban Board Component
//!
//! Displays issues in a Linear-style kanban board layout with columns
//! for each status.
//!
//! DEPENDENCY MAP:
//!
//! Parents:
//!   └─ src/components/issues/project_dashboard.rs
//! Dependencies:
//!   ├─ src/types/issue.rs
//!   └─ src/components/issues/issue_card.rs

use leptos::prelude::*;

use crate::components::issues::{IssueCard, StatusChangeEvent};
use crate::types::issue::{IssueStatus, KanbanBoard as KanbanBoardType};

/// Kanban board component
#[component]
pub fn KanbanBoard(
    board: ReadSignal<KanbanBoardType>,
    status_change_signal: WriteSignal<Option<StatusChangeEvent>>,
) -> impl IntoView {
    view! {
        <div class="flex-1 overflow-x-auto p-4">
            <div class="flex gap-4 h-full min-w-max">
                // Render visible columns (excluding Cancelled)
                {IssueStatus::visible().into_iter().map(|status| {
                    view! {
                        <KanbanColumn
                            status=status
                            board=board
                            status_change_signal=status_change_signal
                        />
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

/// Single kanban column component
#[component]
fn KanbanColumn(
    status: IssueStatus,
    board: ReadSignal<KanbanBoardType>,
    status_change_signal: WriteSignal<Option<StatusChangeEvent>>,
) -> impl IntoView {
    let status_name = status.display_name();
    let status_color = match status {
        IssueStatus::Backlog => "bg-gray-400",
        IssueStatus::Todo => "bg-blue-500",
        IssueStatus::InProgress => "bg-yellow-500",
        IssueStatus::InReview => "bg-purple-500",
        IssueStatus::Done => "bg-green-500",
        IssueStatus::Cancelled => "bg-gray-500",
    };

    view! {
        <div class="flex flex-col w-72 bg-slate-900/50 rounded-lg border border-slate-700/50">
            // Column header
            <div class="p-3 border-b border-slate-700/50">
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-2">
                        <span class=format!("w-2 h-2 rounded-full {}", status_color)/>
                        <span class="font-medium text-dt-text">{status_name}</span>
                    </div>
                    <span class="text-sm text-dt-text-sub bg-slate-800 px-2 py-0.5 rounded">
                        {move || board.get().count(status)}
                    </span>
                </div>
            </div>

            // Column content (scrollable)
            <div class="flex-1 overflow-y-auto p-2 space-y-2 min-h-[200px]">
                {move || {
                    let issues = board.get();
                    let column_issues = issues.get_issues(status);
                    
                    if column_issues.is_empty() {
                        view! {
                            <div class="flex items-center justify-center h-20 text-sm text-dt-text-sub italic">
                                "No issues"
                            </div>
                        }.into_any()
                    } else {
                        column_issues.iter().map(|issue| {
                            view! {
                                <IssueCard 
                                    issue=issue.clone()
                                    status_change_signal=status_change_signal
                                />
                            }
                        }).collect_view().into_any()
                    }
                }}
            </div>
        </div>
    }
}
