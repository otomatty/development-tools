//! Shared Loading Components
//!
//! Common skeleton and loading state components shared across multiple pages.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   ├─ src/components/pages/mock_server/loading.rs
//!   ├─ src/components/pages/project_dashboard/loading.rs
//!   ├─ src/components/pages/projects/loading.rs
//!   ├─ src/components/pages/settings/loading.rs
//!   └─ src/components/pages/home/loading.rs
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

use leptos::prelude::*;

/// Generic loading spinner component
///
/// Displays an animated spinner while content is loading
#[component]
pub fn LoadingSpinner() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center h-full">
            <div class="animate-spin w-8 h-8 border-2 border-gm-accent-cyan border-t-transparent rounded-full"/>
        </div>
    }
}

/// Grid skeleton component
///
/// Creates a grid-based skeleton loader for list/grid views
///
/// # Arguments
/// * `items` - Number of skeleton items to display
/// * `cols` - Number of columns in the grid (e.g., "grid-cols-1 md:grid-cols-2 lg:grid-cols-3")
#[component]
pub fn GridSkeleton(
    items: usize,
    #[prop(default = "grid-cols-1 md:grid-cols-2 lg:grid-cols-3")] cols: &'static str,
) -> impl IntoView {
    view! {
        <div class=format!("grid {} gap-4 animate-pulse", cols)>
            {(0..items).map(|_| view! {
                <div class="bg-dt-card border border-slate-700/50 rounded-lg p-4">
                    <div class="flex items-center gap-3 mb-3">
                        <div class="w-10 h-10 bg-slate-700/50 rounded-lg"></div>
                        <div class="flex-1 space-y-2">
                            <div class="h-4 bg-slate-700/50 rounded w-32"></div>
                            <div class="h-3 bg-slate-700/50 rounded w-24"></div>
                        </div>
                    </div>
                    <div class="h-3 bg-slate-700/50 rounded w-full mb-2"></div>
                    <div class="h-3 bg-slate-700/50 rounded w-3/4 mb-4"></div>
                    <div class="flex items-center justify-between pt-3 border-t border-slate-700/50">
                        <div class="h-4 bg-slate-700/50 rounded w-16"></div>
                        <div class="h-4 bg-slate-700/50 rounded w-20"></div>
                    </div>
                </div>
            }).collect_view()}
        </div>
    }
}

/// List skeleton component
///
/// Creates a list-based skeleton loader for vertical layouts
///
/// # Arguments
/// * `items` - Number of skeleton items to display
#[component]
pub fn ListSkeleton(items: usize) -> impl IntoView {
    view! {
        <div class="space-y-4 animate-pulse">
            {(0..items).map(|_| view! {
                <div class="bg-dt-card border border-slate-700/50 rounded-lg p-4">
                    <div class="flex items-center gap-4">
                        <div class="w-12 h-12 bg-slate-700/50 rounded-lg flex-shrink-0"></div>
                        <div class="flex-1 space-y-2">
                            <div class="h-4 bg-slate-700/50 rounded w-32"></div>
                            <div class="h-3 bg-slate-700/50 rounded w-48"></div>
                        </div>
                    </div>
                </div>
            }).collect_view()}
        </div>
    }
}

/// Column skeleton component
///
/// Creates a multi-column layout skeleton (useful for kanban-like views)
///
/// # Arguments
/// * `columns` - Number of columns to display
/// * `items_per_column` - Number of items in each column
#[component]
pub fn ColumnsSkeleton(columns: usize, items_per_column: usize) -> impl IntoView {
    view! {
        <div class="flex gap-4 h-full overflow-x-auto animate-pulse">
            {(0..columns).map(|_| view! {
                <div class="flex-shrink-0 w-72 bg-slate-800/50 rounded-lg p-4">
                    <div class="h-5 bg-slate-700/50 rounded w-24 mb-4"></div>
                    <div class="space-y-3">
                        {(0..items_per_column).map(|_| view! {
                            <div class="bg-slate-700/30 rounded-lg p-3 space-y-2">
                                <div class="h-4 bg-slate-700/50 rounded w-full"></div>
                                <div class="h-3 bg-slate-700/50 rounded w-3/4"></div>
                                <div class="flex gap-2">
                                    <div class="h-5 bg-slate-700/50 rounded w-12"></div>
                                    <div class="h-5 bg-slate-700/50 rounded w-16"></div>
                                </div>
                            </div>
                        }).collect_view()}
                    </div>
                </div>
            }).collect_view()}
        </div>
    }
}

/// Accordion skeleton component
///
/// Creates a skeleton loader for accordion-style interfaces
///
/// # Arguments
/// * `items` - Number of accordion items to display
#[component]
pub fn AccordionSkeleton(items: usize) -> impl IntoView {
    view! {
        <div class="space-y-2 animate-pulse">
            {(0..items).map(|_| view! {
                <div class="bg-dt-card border border-slate-700/50 rounded-lg">
                    <div class="p-4 flex items-center justify-between">
                        <div class="flex items-center gap-3 flex-1">
                            <div class="w-5 h-5 bg-slate-700/50 rounded"></div>
                            <div class="h-4 bg-slate-700/50 rounded w-32"></div>
                        </div>
                        <div class="w-5 h-5 bg-slate-700/50 rounded"></div>
                    </div>
                </div>
            }).collect_view()}
        </div>
    }
}

/// Header skeleton component
///
/// Creates a header skeleton for page headers
#[component]
pub fn HeaderSkeleton() -> impl IntoView {
    view! {
        <div class="p-4 border-b border-slate-700/50 bg-dt-card/50 animate-pulse">
            <div class="flex items-center gap-4">
                <div class="w-10 h-10 bg-slate-700/50 rounded-lg"></div>
                <div class="space-y-2">
                    <div class="h-5 bg-slate-700/50 rounded w-32"></div>
                    <div class="h-3 bg-slate-700/50 rounded w-24"></div>
                </div>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loading_components_render() {
        // These tests verify that components compile and can be instantiated
        // Actual rendering tests would require a DOM environment
        let _ = LoadingSpinner;
        let _ = GridSkeleton;
        let _ = ListSkeleton;
        let _ = ColumnsSkeleton;
        let _ = AccordionSkeleton;
        let _ = HeaderSkeleton;
    }
}
