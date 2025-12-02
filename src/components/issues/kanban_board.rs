//! Kanban Board Component
//!
//! Displays issues in a Linear-style kanban board layout with columns
//! for each status. Supports drag and drop for status changes using
//! mouse events (works in Tauri WebView).
//!
//! DEPENDENCY MAP:
//!
//! Parents:
//!   └─ src/components/issues/project_dashboard.rs
//! Dependencies:
//!   ├─ src/types/issue.rs
//!   └─ src/components/issues/issue_card.rs

use leptos::prelude::*;

use crate::components::icons::Icon;
use crate::components::issues::{IssueClickEvent, StatusChangeEvent};
use crate::types::issue::{IssueStatus, KanbanBoard as KanbanBoardType};
use crate::types::CachedIssue;

/// Drag state for mouse-based drag and drop
#[derive(Clone, Debug, PartialEq)]
pub struct MouseDragState {
    pub issue_number: i32,
    pub from_status: String,
    pub issue_title: String,
}

/// Kanban board component with mouse-based drag and drop support
/// Uses mousedown/mouseup instead of HTML5 drag events for Tauri compatibility
#[component]
pub fn KanbanBoard(
    board: ReadSignal<KanbanBoardType>,
    status_change_signal: WriteSignal<Option<StatusChangeEvent>>,
    issue_click_signal: WriteSignal<Option<IssueClickEvent>>,
) -> impl IntoView {
    // Track the currently dragged issue using mouse events
    let (dragging, set_dragging) = signal(Option::<MouseDragState>::None);
    // Track which column the mouse is currently hovering over
    let (hover_column, set_hover_column) = signal(Option::<String>::None);
    // Track mouse position for ghost card
    let (mouse_pos, set_mouse_pos) = signal((0i32, 0i32));

    // Global event handlers
    Effect::new(move |_| {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;

        let document = web_sys::window().unwrap().document().unwrap();

        // Global mousemove to track position for ghost card
        let set_mouse_pos_for_move = set_mouse_pos;
        let mousemove_handler = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            set_mouse_pos_for_move.set((e.client_x(), e.client_y()));
        }) as Box<dyn FnMut(_)>);

        document
            .add_event_listener_with_callback("mousemove", mousemove_handler.as_ref().unchecked_ref())
            .unwrap();

        // Global mouseup to handle drop or cancel
        let dragging_for_mouseup = dragging;
        let set_dragging_for_mouseup = set_dragging;
        let hover_column_for_mouseup = hover_column;
        let status_change_signal_for_mouseup = status_change_signal;
        let mouseup_handler = Closure::wrap(Box::new(move |_: web_sys::MouseEvent| {
            if let Some(drag_state) = dragging_for_mouseup.get_untracked() {
                if let Some(target_status) = hover_column_for_mouseup.get_untracked() {
                    if drag_state.from_status != target_status {
                        leptos::logging::log!(
                            "🎯 Drop! Moving issue #{} from {} to {}",
                            drag_state.issue_number,
                            drag_state.from_status,
                            target_status
                        );
                        status_change_signal_for_mouseup.set(Some(StatusChangeEvent {
                            issue_number: drag_state.issue_number,
                            new_status: target_status,
                        }));
                    } else {
                        leptos::logging::log!("🔄 Dropped on same column, no change");
                    }
                } else {
                    leptos::logging::log!("🚫 Drag cancelled (mouseup outside column)");
                }
                set_dragging_for_mouseup.set(None);
            }
        }) as Box<dyn FnMut(_)>);

        document
            .add_event_listener_with_callback("mouseup", mouseup_handler.as_ref().unchecked_ref())
            .unwrap();

        // Prevent default drag behavior globally during drag
        let dragging_for_selectstart = dragging;
        let selectstart_handler = Closure::wrap(Box::new(move |e: web_sys::Event| {
            if dragging_for_selectstart.get_untracked().is_some() {
                e.prevent_default();
            }
        }) as Box<dyn FnMut(_)>);

        document
            .add_event_listener_with_callback(
                "selectstart",
                selectstart_handler.as_ref().unchecked_ref(),
            )
            .unwrap();

        // Leak closures to keep them alive (TODO: [DEBT] proper cleanup with on_cleanup)
        mousemove_handler.forget();
        mouseup_handler.forget();
        selectstart_handler.forget();
    });

    view! {
        <div class="flex-1 h-full overflow-x-auto p-4 select-none relative">
            <div class="flex gap-4 h-full min-w-max">
                {IssueStatus::visible()
                    .into_iter()
                    .map(|status| {
                        view! {
                            <KanbanColumn
                                status=status
                                board=board
                                status_change_signal=status_change_signal
                                issue_click_signal=issue_click_signal
                                dragging=dragging
                                set_dragging=set_dragging
                                set_hover_column=set_hover_column
                                hover_column=hover_column
                            />
                        }
                    })
                    .collect_view()}
            </div>

            // Ghost card that follows the mouse during drag
            <Show when=move || dragging.get().is_some()>
                {move || {
                    let drag_state = dragging.get();
                    let (x, y) = mouse_pos.get();
                    view! {
                        <div
                            class="fixed pointer-events-none z-50 bg-gray-800 rounded-lg p-3 shadow-2xl border-2 border-gm-accent-cyan w-64 opacity-90"
                            style=move || format!("left: {}px; top: {}px; transform: translate(-50%, -50%);", x, y)
                        >
                            <div class="flex items-center gap-2 mb-1">
                                <span class="text-xs text-gray-400 font-mono">
                                    {"#"}{drag_state.as_ref().map(|d| d.issue_number).unwrap_or(0)}
                                </span>
                            </div>
                            <p class="text-sm text-white line-clamp-2">
                                {drag_state.as_ref().map(|d| d.issue_title.clone()).unwrap_or_default()}
                            </p>
                        </div>
                    }
                }}
            </Show>
        </div>
    }
}

/// Single kanban column component
#[component]
fn KanbanColumn(
    status: IssueStatus,
    board: ReadSignal<KanbanBoardType>,
    status_change_signal: WriteSignal<Option<StatusChangeEvent>>,
    issue_click_signal: WriteSignal<Option<IssueClickEvent>>,
    dragging: ReadSignal<Option<MouseDragState>>,
    set_dragging: WriteSignal<Option<MouseDragState>>,
    set_hover_column: WriteSignal<Option<String>>,
    hover_column: ReadSignal<Option<String>>,
) -> impl IntoView {
    let status_name = status.display_name();
    let status_value: &'static str = match status {
        IssueStatus::Backlog => "backlog",
        IssueStatus::Todo => "todo",
        IssueStatus::InProgress => "in-progress",
        IssueStatus::InReview => "in-review",
        IssueStatus::Done => "done",
        IssueStatus::Cancelled => "cancelled",
    };
    let status_color = match status {
        IssueStatus::Backlog => "bg-gray-400",
        IssueStatus::Todo => "bg-blue-500",
        IssueStatus::InProgress => "bg-yellow-500",
        IssueStatus::InReview => "bg-purple-500",
        IssueStatus::Done => "bg-green-500",
        IssueStatus::Cancelled => "bg-gray-500",
    };

    // Check if we're dragging from a different column (valid drop target)
    let is_valid_drop_target = move || {
        dragging
            .get()
            .map(|d| d.from_status != status_value)
            .unwrap_or(false)
    };

    // Check if dragging at all
    let is_dragging = move || dragging.get().is_some();

    // Check if this column is currently being hovered
    let is_hovered = move || {
        hover_column
            .get()
            .map(|h| h == status_value)
            .unwrap_or(false)
    };

    view! {
        <div
            class=move || {
                let dragging_active = is_dragging();
                let valid_target = is_valid_drop_target();
                let hovered = is_hovered();

                format!(
                    "flex flex-col w-72 h-full rounded-lg border-2 transition-all duration-200 {}",
                    if dragging_active && valid_target && hovered {
                        // Actively hovering over a valid drop target - inner glow effect
                        "bg-gm-accent-cyan/20 border-gm-accent-cyan shadow-[inset_0_0_20px_rgba(0,255,255,0.3)] ring-2 ring-gm-accent-cyan/50 ring-inset"
                    } else if dragging_active && valid_target {
                        // Valid drop target but not hovered
                        "bg-slate-900/50 border-gm-accent-cyan/50 border-dashed"
                    } else if dragging_active && !valid_target {
                        // Source column (where drag started)
                        "bg-slate-900/30 border-slate-700/30 opacity-60"
                    } else {
                        // Normal state
                        "bg-slate-900/50 border-slate-700/50"
                    },
                )
            }
            // Track mouse enter/leave for drop target
            on:mouseenter=move |_| {
                if is_dragging() {
                    leptos::logging::log!("📍 Mouse entered column: {}", status_value);
                    set_hover_column.set(Some(status_value.to_string()));
                }
            }
            on:mouseleave=move |_| {
                if is_dragging() {
                    leptos::logging::log!("📍 Mouse left column: {}", status_value);
                    set_hover_column.set(None);
                }
            }
        >
            // Column header
            <div class=move || {
                format!(
                    "p-3 border-b transition-colors {}",
                    if is_dragging() && is_valid_drop_target() && is_hovered() {
                        "border-gm-accent-cyan/50"
                    } else {
                        "border-slate-700/50"
                    }
                )
            }>
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-2">
                        <span class=format!("w-2 h-2 rounded-full {}", status_color) />
                        <span class="font-medium text-dt-text">{status_name}</span>
                    </div>
                    <span class="text-sm text-dt-text-sub bg-slate-800 px-2 py-0.5 rounded">
                        {move || board.get().count(status)}
                    </span>
                </div>
            </div>

            // Column content - with filtering for completed statuses
            <CompletedColumnContent
                status=status
                status_value=status_value
                board=board
                issue_click_signal=issue_click_signal
                status_change_signal=status_change_signal
                dragging=dragging
                set_dragging=set_dragging
            />

            // Drop indicator
            <Show when=move || is_dragging() && is_valid_drop_target()>
                <div class="p-2">
                    <div class="h-1 bg-gm-accent-cyan rounded-full animate-pulse" />
                </div>
            </Show>
        </div>
    }
}

/// Days to show for completed issues
const COMPLETED_ISSUES_DAYS: i64 = 7;

/// Column content with special handling for completed statuses (Done/Cancelled)
/// Shows only issues updated within the last week, with "Show more" option
#[component]
fn CompletedColumnContent(
    status: IssueStatus,
    status_value: &'static str,
    board: ReadSignal<KanbanBoardType>,
    issue_click_signal: WriteSignal<Option<IssueClickEvent>>,
    status_change_signal: WriteSignal<Option<StatusChangeEvent>>,
    dragging: ReadSignal<Option<MouseDragState>>,
    set_dragging: WriteSignal<Option<MouseDragState>>,
) -> impl IntoView {
    let is_completed_status = matches!(status, IssueStatus::Done | IssueStatus::Cancelled);
    let (show_all, set_show_all) = signal(false);

    // Derive is_dragging and is_valid_drop_target from the dragging signal
    let is_dragging = move || dragging.get().is_some();
    let is_valid_drop_target = move || {
        dragging
            .get()
            .map(|d| d.from_status != status_value)
            .unwrap_or(false)
    };

    view! {
        <div class="flex-1 overflow-y-auto p-2 space-y-2 min-h-[200px]">
            {move || {
                let issues = board.get();
                let column_issues = issues.get_issues(status);

                if column_issues.is_empty() {
                    view! {
                        <div class=move || {
                            format!(
                                "flex items-center justify-center h-20 text-sm italic rounded-lg transition-colors {}",
                                if is_dragging() && is_valid_drop_target() {
                                    "text-gm-accent-cyan bg-gm-accent-cyan/5 border-2 border-dashed border-gm-accent-cyan"
                                } else {
                                    "text-dt-text-sub"
                                },
                            )
                        }>
                            {move || {
                                if is_dragging() && is_valid_drop_target() {
                                    "Drop here to move"
                                } else {
                                    "No issues"
                                }
                            }}
                        </div>
                    }
                        .into_any()
                } else if is_completed_status && !show_all.get() {
                    // For completed statuses, filter to recent issues
                    let (recent_issues, older_count): (Vec<_>, usize) = {
                        let recent: Vec<_> = column_issues
                            .iter()
                            .filter(|issue| issue.is_updated_within_days(COMPLETED_ISSUES_DAYS))
                            .cloned()
                            .collect();
                        let older = column_issues.len() - recent.len();
                        (recent, older)
                    };

                    view! {
                        <div class="space-y-2">
                            // Recent issues
                            {recent_issues
                                .into_iter()
                                .map(|issue| {
                                    view! {
                                        <IssueCardDraggable
                                            issue=issue
                                            issue_click_signal=issue_click_signal
                                            status_change_signal=status_change_signal
                                            dragging=dragging
                                            set_dragging=set_dragging
                                        />
                                    }
                                })
                                .collect_view()}

                            // "Show more" button if there are older issues
                            {if older_count > 0 {
                                Some(view! {
                                    <button
                                        class="w-full py-2 px-3 text-sm text-dt-text-sub hover:text-dt-text 
                                               bg-slate-800/50 hover:bg-slate-700/50 rounded-lg 
                                               border border-slate-700/50 hover:border-slate-600/50
                                               transition-colors flex items-center justify-center gap-2"
                                        on:click=move |_| set_show_all.set(true)
                                    >
                                        <Icon name="chevron-down" class="w-4 h-4".to_string() />
                                        <span>{format!("{} older issues", older_count)}</span>
                                    </button>
                                })
                            } else {
                                None
                            }}
                        </div>
                    }
                        .into_any()
                } else {
                    // Show all issues (non-completed or expanded)
                    view! {
                        <div class="space-y-2">
                            // Collapse button for completed statuses when expanded
                            <Show when=move || is_completed_status && show_all.get()>
                                <button
                                    class="w-full py-1.5 px-3 text-xs text-dt-text-sub hover:text-dt-text 
                                           bg-slate-800/30 hover:bg-slate-700/30 rounded-lg 
                                           border border-slate-700/30 hover:border-slate-600/30
                                           transition-colors flex items-center justify-center gap-1"
                                    on:click=move |_| set_show_all.set(false)
                                >
                                    <Icon name="chevron-up" class="w-3 h-3".to_string() />
                                    <span>"Show recent only"</span>
                                </button>
                            </Show>

                            {column_issues
                                .iter()
                                .map(|issue| {
                                    view! {
                                        <IssueCardDraggable
                                            issue=issue.clone()
                                            issue_click_signal=issue_click_signal
                                            status_change_signal=status_change_signal
                                            dragging=dragging
                                            set_dragging=set_dragging
                                        />
                                    }
                                })
                                .collect_view()}
                        </div>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}

/// Issue card with mouse-based drag support
#[component]
fn IssueCardDraggable(
    issue: CachedIssue,
    issue_click_signal: WriteSignal<Option<IssueClickEvent>>,
    status_change_signal: WriteSignal<Option<StatusChangeEvent>>,
    dragging: ReadSignal<Option<MouseDragState>>,
    set_dragging: WriteSignal<Option<MouseDragState>>,
) -> impl IntoView {
    let (is_mouse_down, set_is_mouse_down) = signal(false);
    let issue_clone = issue.clone();

    let issue_url = issue.html_url.clone().unwrap_or_default();
    let issue_title = issue.title.clone();
    let issue_title_for_drag = issue.title.clone();
    let issue_title_for_title_drag = issue.title.clone();
    let issue_number = issue.number;
    let issue_body = issue.body.clone();
    let issue_assignee = issue.assignee_login.clone();
    let issue_status_for_drag = issue.status.clone();
    let issue_status_for_title_drag = issue.status.clone();

    // Check if this card is being dragged
    let is_being_dragged = move || {
        dragging
            .get()
            .map(|d| d.issue_number == issue_number)
            .unwrap_or(false)
    };

    view! {
        <div
            class=move || {
                format!(
                    "bg-gray-800 rounded-lg p-3 shadow-md transition-all border select-none {}",
                    if is_being_dragged() {
                        "opacity-30 border-gm-accent-cyan scale-95 cursor-grabbing"
                    } else if is_mouse_down.get() {
                        "cursor-grabbing border-gm-accent-cyan/50"
                    } else {
                        "cursor-grab border-gray-700 hover:border-gm-accent-cyan/50 hover:shadow-lg"
                    },
                )
            }
            // Prevent text selection during drag
            on:selectstart=move |e: web_sys::Event| {
                e.prevent_default();
            }
            // Prevent native drag
            draggable="false"
            // Mouse down = start potential drag
            on:mousedown={
                let issue_status = issue_status_for_drag.clone();
                let title = issue_title_for_drag.clone();
                move |e: web_sys::MouseEvent| {
                    // Prevent text selection
                    e.prevent_default();
                    // Only start drag on primary button
                    if e.button() == 0 {
                        leptos::logging::log!("🚀 Drag start: issue #{} from {}", issue_number, issue_status);
                        set_is_mouse_down.set(true);
                        set_dragging.set(Some(MouseDragState {
                            issue_number,
                            from_status: issue_status.clone(),
                            issue_title: title.clone(),
                        }));
                    }
                }
            }
            // Mouse up = end drag or click
            on:mouseup=move |_| {
                set_is_mouse_down.set(false);
            }
            // Mouse leave while dragging = continue drag
            on:mouseleave=move |_| {
                // Don't reset if we're dragging
                if !is_being_dragged() {
                    set_is_mouse_down.set(false);
                }
            }
        >
            // Header with issue number, detail button, and GitHub link
            <div class="flex items-center justify-between mb-2">
                <span class="text-xs text-gray-400 font-mono">{"#"} {issue_number}</span>
                <div class="flex items-center gap-1" on:mousedown=move |e| e.stop_propagation()>
                    // Detail button
                    <button
                        class="p-1 text-gray-400 hover:text-gm-accent-cyan hover:bg-gray-700 rounded transition-all"
                        title="View details"
                        on:click={
                            let issue_for_click = issue_clone.clone();
                            move |e: web_sys::MouseEvent| {
                                e.stop_propagation();
                                leptos::logging::log!("🔍 Detail button clicked for issue #{}", issue_number);
                                issue_click_signal.set(Some(IssueClickEvent {
                                    issue: issue_for_click.clone(),
                                }));
                            }
                        }
                    >
                        <Icon name="expand".to_string() class="w-4 h-4".to_string() />
                    </button>
                    // GitHub link
                    <a
                        href=issue_url.clone()
                        target="_blank"
                        rel="noopener noreferrer"
                        class="p-1 text-gray-400 hover:text-white hover:bg-gray-700 rounded transition-all"
                        title="Open in GitHub"
                        on:click=move |e: web_sys::MouseEvent| e.stop_propagation()
                    >
                        <Icon name="github".to_string() class="w-4 h-4".to_string() />
                    </a>
                </div>
            </div>

            // Title - draggable only (detail opens via button)
            <h4
                class="text-sm font-medium text-white mb-2 line-clamp-2"
                on:mousedown={
                    let issue_status = issue_status_for_title_drag.clone();
                    let title = issue_title_for_title_drag.clone();
                    move |e: web_sys::MouseEvent| {
                        // Start drag
                        e.prevent_default();
                        if e.button() == 0 {
                            leptos::logging::log!("🚀 Drag start from title: issue #{} from {}", issue_number, issue_status);
                            set_is_mouse_down.set(true);
                            set_dragging.set(Some(MouseDragState {
                                issue_number,
                                from_status: issue_status.clone(),
                                issue_title: title.clone(),
                            }));
                        }
                        e.stop_propagation();
                    }
                }
            >
                {issue_title}
            </h4>

            // Body preview
            {issue_body.map(|body| {
                view! { <p class="text-xs text-gray-400 mb-2 line-clamp-2">{body}</p> }
            })}

            // Footer - Assignee only (status changed via drag & drop)
            {issue_assignee.map(|assignee| {
                let assignee_display = assignee.clone();
                view! {
                    <div class="mt-2 pt-2 border-t border-gray-700">
                        <span class="text-xs text-gray-400 truncate max-w-[80px]" title=assignee>
                            {"@"} {assignee_display}
                        </span>
                    </div>
                }
            })}
        </div>
    }
}
