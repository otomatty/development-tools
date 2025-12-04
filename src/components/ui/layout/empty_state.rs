//! Empty State Component
//!
//! A component for displaying empty/no-data states with icon, title, and description.
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this component):
//!   └─ src/components/ui/layout/mod.rs
//!
//! Related Documentation:
//!   └─ Spec: ./layout.spec.md

use leptos::prelude::*;

use crate::components::icons::Icon;

/// Empty State component
///
/// Displays a centered message when there's no data to show.
///
/// # Props
///
/// - `icon`: Icon name to display
/// - `title`: Main message title
/// - `description`: Detailed description text
/// - `children`: Optional action area (buttons, links)
/// - `class`: Additional CSS classes
///
/// # Example
///
/// ```rust
/// view! {
///     <EmptyState
///         icon="folder-open"
///         title="No projects yet"
///         description="Create your first project to get started"
///     >
///         <Button variant=ButtonVariant::Primary on_click=move |_| create_project()>
///             "Create Project"
///         </Button>
///     </EmptyState>
/// }
/// ```
#[component]
pub fn EmptyState(
    /// Icon name to display
    icon: &'static str,
    /// Main message title
    title: &'static str,
    /// Detailed description text
    description: &'static str,
    /// Optional action area content
    #[prop(optional)]
    children: Option<Children>,
    /// Additional CSS classes
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let combined_class = format!(
        "flex flex-col items-center justify-center py-12 text-center {}",
        class
    );

    view! {
        <div class=combined_class>
            // Icon container
            <div class="p-4 bg-gm-bg-card/80 rounded-2xl border border-slate-700/50 mb-4">
                <Icon
                    name=icon.to_string()
                    class="w-12 h-12 text-dt-text-sub".to_string()
                />
            </div>

            // Title
            <h3 class="text-lg font-semibold text-dt-text mb-2">
                {title}
            </h3>

            // Description
            <p class="text-dt-text-sub text-sm max-w-md mb-6">
                {description}
            </p>

            // Action area
            {children.map(|c| view! {
                <div class="flex items-center gap-3">
                    {c()}
                </div>
            })}
        </div>
    }
}
