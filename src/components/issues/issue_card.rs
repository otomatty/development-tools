/**
 * Issue Card Component
 *
 * Individual issue card displayed in the Kanban board
 * Supports drag and drop for status changes
 *
 * DEPENDENCY MAP:
 *
 * Parents:
 *   └─ src/components/issues/kanban_board.rs
 * Dependencies:
 *   └─ src/types/issue.rs (CachedIssue)
 */

use leptos::prelude::*;
use crate::types::CachedIssue;
use crate::components::icons::Icon;

/// Status change event data
#[derive(Clone, Debug)]
pub struct StatusChangeEvent {
    pub issue_number: i32,
    pub new_status: String,
}

/// Issue click event data
#[derive(Clone, Debug)]
pub struct IssueClickEvent {
    pub issue: CachedIssue,
}

/// Drag start event data
#[derive(Clone, Debug)]
pub struct DragStartEvent {
    pub issue_number: i32,
    pub current_status: String,
}

/// Issue card with status dropdown and drag support
/// Uses WriteSignals to report status changes and clicks back to parent
#[component]
pub fn IssueCard(
    issue: CachedIssue,
    #[prop(optional)] status_change_signal: Option<WriteSignal<Option<StatusChangeEvent>>>,
    issue_click_signal: WriteSignal<Option<IssueClickEvent>>,
    #[prop(optional)] drag_signal: Option<WriteSignal<Option<DragStartEvent>>>,
) -> impl IntoView {
    let (show_dropdown, set_show_dropdown) = signal(false);
    let issue_clone = issue.clone();
    
    let statuses = vec![
        ("backlog", "Backlog", "bg-gray-500"),
        ("todo", "Todo", "bg-blue-500"),
        ("in-progress", "In Progress", "bg-yellow-500"),
        ("in-review", "In Review", "bg-purple-500"),
        ("done", "Done", "bg-green-500"),
        ("cancelled", "Cancelled", "bg-red-500"),
    ];
    
    let current_status = issue.status.clone();
    let issue_url = issue.html_url.clone().unwrap_or_default();
    let issue_title = issue.title.clone();
    let issue_number = issue.number;
    let issue_body = issue.body.clone();
    let issue_assignee = issue.assignee_login.clone();
    
    // Status badge color
    let status_color = match current_status.as_str() {
        "backlog" => "bg-gray-500",
        "todo" => "bg-blue-500",
        "in-progress" => "bg-yellow-500",
        "in-review" => "bg-purple-500",
        "done" => "bg-green-500",
        "cancelled" => "bg-red-500",
        _ => "bg-gray-500",
    };
    
    view! {
        <div 
            class="bg-gray-800 rounded-lg p-3 shadow-md hover:shadow-lg transition-all border cursor-grab border-gray-700 hover:border-gm-accent-cyan/50"
            draggable="true"
            on:dragstart={
                let issue_number_for_drag = issue_number;
                let current_status_for_drag = issue.status.clone();
                move |e: web_sys::DragEvent| {
                    leptos::logging::log!("🚀 dragstart fired for issue #{}", issue_number_for_drag);
                    
                    // CRITICAL: Must set data on DataTransfer for drop to work!
                    // Without this, browsers won't allow drops
                    if let Some(dt) = e.data_transfer() {
                        // Set some data - the actual value doesn't matter since we use signals
                        let _ = dt.set_data("text/plain", &issue_number_for_drag.to_string());
                        dt.set_effect_allowed("move");
                        leptos::logging::log!("📋 DataTransfer set: {}", issue_number_for_drag);
                    } else {
                        leptos::logging::log!("❌ No DataTransfer available!");
                    }
                    
                    // Notify parent about drag start via signal
                    if let Some(signal) = drag_signal {
                        signal.set(Some(DragStartEvent {
                            issue_number: issue_number_for_drag,
                            current_status: current_status_for_drag.clone(),
                        }));
                    }
                }
            }
            on:dragend=move |_| {
                leptos::logging::log!("🏁 dragend fired");
                // Clear drag state in parent
                if let Some(signal) = drag_signal {
                    signal.set(None);
                }
            }
            on:click={
                let issue_for_click = issue_clone.clone();
                move |_| {
                    issue_click_signal.set(Some(IssueClickEvent {
                        issue: issue_for_click.clone(),
                    }));
                }
            }
        >
            // Header with issue number and GitHub link
            <div class="flex items-center justify-between mb-2">
                <span class="text-xs text-gray-400 font-mono">
                    {"#"}{issue_number}
                </span>
                <a
                    href={issue_url}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="text-gray-400 hover:text-white transition-colors"
                    on:click=move |e| e.stop_propagation()
                >
                    <Icon name="github".to_string() class="w-4 h-4".to_string() />
                </a>
            </div>
            
            // Title
            <h4 class="text-sm font-medium text-white mb-2 line-clamp-2">
                {issue_title}
            </h4>
            
            // Body preview (if exists)
            {issue_body.map(|body| {
                view! {
                    <p class="text-xs text-gray-400 mb-2 line-clamp-2">
                        {body}
                    </p>
                }
            })}
            
            // Footer with status and assignee
            <div class="flex items-center justify-between mt-2 pt-2 border-t border-gray-700">
                // Status dropdown
                <div class="relative">
                    <button
                        class=format!("px-2 py-1 rounded text-xs text-white {} hover:opacity-80 transition-opacity flex items-center gap-1", status_color)
                        on:click=move |e| {
                            e.stop_propagation();
                            set_show_dropdown.update(|v| *v = !*v);
                        }
                    >
                        {current_status.replace("-", " ")}
                        <svg class="w-3 h-3 rotate-90" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                        </svg>
                    </button>
                    
                    // Dropdown menu
                    <Show when=move || show_dropdown.get()>
                        <div class="absolute left-0 bottom-full mb-1 bg-gray-900 rounded-lg shadow-lg border border-gray-700 py-1 z-50 min-w-[120px]">
                            {statuses.iter().map(|(value, label, color)| {
                                let value = value.to_string();
                                let label = *label;
                                let color = *color;
                                view! {
                                    <button
                                        class="w-full px-3 py-1.5 text-left text-xs text-white hover:bg-gray-800 flex items-center gap-2"
                                        on:click={
                                            let value = value.clone();
                                            move |e| {
                                                e.stop_propagation();
                                                if let Some(signal) = status_change_signal {
                                                    Set::set(&signal, Some(StatusChangeEvent {
                                                        issue_number,
                                                        new_status: value.clone(),
                                                    }));
                                                }
                                                set_show_dropdown.set(false);
                                            }
                                        }
                                    >
                                        <span class=format!("w-2 h-2 rounded-full {}", color)></span>
                                        {label}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </Show>
                </div>
                
                // Assignee
                {issue_assignee.map(|assignee| {
                    let assignee_display = assignee.clone();
                    view! {
                        <span class="text-xs text-gray-400 truncate max-w-[80px]" title={assignee}>
                            {"@"}{assignee_display}
                        </span>
                    }
                })}
            </div>
        </div>
    }
}
