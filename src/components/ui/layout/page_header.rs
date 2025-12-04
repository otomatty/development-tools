//! Page Header Component
//!
//! A reusable page header component with title, subtitle, and action area.
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this component):
//!   └─ src/components/ui/layout/mod.rs
//!
//! Related Documentation:
//!   └─ Spec: ./layout.spec.md

use leptos::prelude::*;

/// Page Header component
///
/// A consistent header component for pages with title, optional subtitle,
/// and optional action area (buttons, etc.).
///
/// # Props
///
/// - `title`: Page title text
/// - `subtitle`: Optional subtitle/description text
/// - `children`: Optional action area content (buttons, etc.)
/// - `class`: Additional CSS classes
///
/// # Example
///
/// ```rust
/// view! {
///     <PageHeader
///         title="Projects"
///         subtitle="Manage your GitHub issues with kanban boards"
///     >
///         <Button variant=ButtonVariant::Primary on_click=move |_| create_project()>
///             <Icon name="plus" class="w-5 h-5" />
///             "New Project"
///         </Button>
///     </PageHeader>
/// }
/// ```
#[component]
pub fn PageHeader(
    /// Page title text
    title: &'static str,
    /// Optional subtitle/description text
    #[prop(optional)]
    subtitle: Option<&'static str>,
    /// Optional action area content
    #[prop(optional)]
    children: Option<Children>,
    /// Additional CSS classes
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let combined_class = format!("flex items-center justify-between mb-6 {}", class);

    view! {
        <div class=combined_class>
            <div>
                <h1 class="text-2xl font-bold text-dt-text font-gaming">
                    {title}
                </h1>
                {subtitle.map(|s| view! {
                    <p class="text-dt-text-sub mt-1">{s}</p>
                })}
            </div>
            {children.map(|c| view! {
                <div class="flex items-center gap-3">
                    {c()}
                </div>
            })}
        </div>
    }
}

/// Page Header with dynamic title
///
/// Same as PageHeader but accepts String for dynamic titles.
#[component]
pub fn PageHeaderAction(
    /// Page title text (dynamic)
    title: String,
    /// Optional subtitle/description text
    #[prop(optional)]
    subtitle: Option<String>,
    /// Optional action area content
    #[prop(optional)]
    children: Option<Children>,
    /// Additional CSS classes
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let combined_class = format!("flex items-center justify-between mb-6 {}", class);

    view! {
        <div class=combined_class>
            <div>
                <h1 class="text-2xl font-bold text-dt-text font-gaming">
                    {title}
                </h1>
                {subtitle.map(|s| view! {
                    <p class="text-dt-text-sub mt-1">{s}</p>
                })}
            </div>
            {children.map(|c| view! {
                <div class="flex items-center gap-3">
                    {c()}
                </div>
            })}
        </div>
    }
}
