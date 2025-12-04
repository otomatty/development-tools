//! Create Issue Modal Component
//!
//! Modal dialog for creating a new GitHub issue.
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

use crate::components::ui::dialog::{Modal, ModalBody, ModalFooter, ModalHeader, ModalSize};
use crate::tauri_api;
use crate::types::issue::CachedIssue;

/// Create issue modal component
#[component]
pub fn CreateIssueModal(
    project_id: i64,
    #[prop(into)] visible: Signal<bool>,
    on_close: impl Fn() + 'static + Clone + Send + Sync,
    on_created: impl Fn(CachedIssue) + 'static + Copy,
) -> impl IntoView {
    let (title, set_title) = signal(String::new());
    let (body, set_body) = signal(String::new());
    let (status, set_status) = signal("backlog".to_string());
    let (priority, set_priority) = signal(Option::<String>::None);
    let (creating, set_creating) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (created_issue, set_created_issue) = signal(Option::<CachedIssue>::None);

    // Store on_close for use in ChildrenFn
    let on_close_stored = StoredValue::new(on_close.clone());
    let on_close_callback = Callback::new(move |_: ()| on_close_stored.get_value()());

    // Watch for successful creation
    Effect::new(move |_| {
        if let Some(issue) = created_issue.get() {
            on_created(issue);
        }
    });

    let create_issue = move |_| {
        let title_val = title.get();
        if title_val.trim().is_empty() {
            set_error.set(Some("Title is required".to_string()));
            return;
        }

        set_creating.set(true);
        set_error.set(None);

        let body_val = body.get();
        let body_opt = if body_val.trim().is_empty() {
            None
        } else {
            Some(body_val.clone())
        };
        let status_val = status.get();
        let priority_val = priority.get();

        spawn_local(async move {
            match tauri_api::create_github_issue(
                project_id,
                &title_val,
                body_opt.as_deref(),
                Some(&status_val),
                priority_val.as_deref(),
            )
            .await
            {
                Ok(issue) => {
                    set_created_issue.set(Some(issue));
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_creating.set(false);
                }
            }
        });
    };

    view! {
        <Modal
            visible=visible
            on_close=on_close.clone()
            size=ModalSize::Large
        >
            // Header
            <ModalHeader on_close=on_close_callback>
                <h2 class="text-lg font-semibold text-dt-text">"Create New Issue"</h2>
            </ModalHeader>

            // Form
            <ModalBody>
                <div class="space-y-4">
                    // Error message
                    <Show when=move || error.get().is_some()>
                        <div class="p-3 bg-red-500/20 border border-red-500/50 rounded-lg text-sm text-red-400">
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </Show>

                    // Title
                    <div>
                        <label class="block text-sm font-medium text-dt-text-sub mb-1">"Title" <span class="text-red-400">"*"</span></label>
                        <input
                            type="text"
                            class="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-dt-text focus:border-gm-accent-cyan focus:outline-none"
                            placeholder="Issue title..."
                            prop:value=move || title.get()
                            on:input=move |ev| set_title.set(event_target_value(&ev))
                        />
                    </div>

                    // Body
                    <div>
                        <label class="block text-sm font-medium text-dt-text-sub mb-1">"Description"</label>
                        <textarea
                            class="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-dt-text focus:border-gm-accent-cyan focus:outline-none resize-none"
                            rows="4"
                            placeholder="Describe the issue..."
                            prop:value=move || body.get()
                            on:input=move |ev| set_body.set(event_target_value(&ev))
                        />
                    </div>

                    // Status and Priority row
                    <div class="grid grid-cols-2 gap-4">
                        // Status
                        <div>
                            <label class="block text-sm font-medium text-dt-text-sub mb-1">"Status"</label>
                            <select
                                class="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-dt-text focus:border-gm-accent-cyan focus:outline-none"
                                prop:value=move || status.get()
                                on:change=move |ev| set_status.set(event_target_value(&ev))
                            >
                                <option value="backlog">"Backlog"</option>
                                <option value="todo">"Todo"</option>
                                <option value="in-progress">"In Progress"</option>
                                <option value="in-review">"In Review"</option>
                            </select>
                        </div>

                        // Priority
                        <div>
                            <label class="block text-sm font-medium text-dt-text-sub mb-1">"Priority"</label>
                            <select
                                class="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-dt-text focus:border-gm-accent-cyan focus:outline-none"
                                prop:value=move || priority.get().unwrap_or_default()
                                on:change=move |ev| {
                                    let val = event_target_value(&ev);
                                    set_priority.set(if val.is_empty() { None } else { Some(val) });
                                }
                            >
                                <option value="">"No priority"</option>
                                <option value="high">"🔴 High"</option>
                                <option value="medium">"🟡 Medium"</option>
                                <option value="low">"🟢 Low"</option>
                            </select>
                        </div>
                    </div>
                </div>
            </ModalBody>

            // Footer
            <ModalFooter>
                <button
                    class="px-4 py-2 text-dt-text-sub hover:text-dt-text transition-colors"
                    on:click=move |_| on_close_stored.get_value()()
                >
                    "Cancel"
                </button>
                <button
                    class="px-4 py-2 bg-gradient-to-r from-gm-accent-cyan to-gm-accent-purple text-white rounded-lg hover:opacity-90 transition-opacity disabled:opacity-50"
                    disabled=move || creating.get() || title.get().trim().is_empty()
                    on:click=create_issue
                >
                    {move || if creating.get() { "Creating..." } else { "Create Issue" }}
                </button>
            </ModalFooter>
        </Modal>
    }
}
