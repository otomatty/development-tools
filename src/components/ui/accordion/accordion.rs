//! Accordion Component
//!
//! A reusable accordion component for collapsible content sections.
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this component):
//!   └─ src/components/ui/accordion/mod.rs
//!
//! Related Documentation:
//!   └─ Spec: ./accordion.spec.md

use leptos::prelude::*;

use crate::components::icons::Icon;

/// Accordion item data structure
#[derive(Clone)]
pub struct AccordionItem {
    /// Unique identifier for the item
    pub id: String,
    /// Title text
    pub title: String,
    /// Optional icon name
    pub icon: Option<String>,
}

/// Single Accordion Section component
///
/// A single collapsible section with title, icon, and content.
///
/// # Props
///
/// - `title`: Section title
/// - `icon`: Optional icon name
/// - `expanded`: Whether the section is expanded
/// - `on_toggle`: Callback when the section header is clicked
/// - `children`: Section content
/// - `max_height`: Maximum height when expanded
/// - `class`: Additional CSS classes
///
/// # Example
///
/// ```rust
/// let expanded = RwSignal::new(false);
///
/// view! {
///     <AccordionSection
///         title="Settings"
///         icon="settings"
///         expanded=expanded.read_only()
///         on_toggle=move || expanded.update(|v| *v = !*v)
///     >
///         <p>"Settings content here"</p>
///     </AccordionSection>
/// }
/// ```
#[component]
pub fn AccordionSection<F>(
    /// Section title
    title: String,
    /// Optional icon name
    #[prop(optional)]
    icon: Option<&'static str>,
    /// Whether the section is expanded
    #[prop(into)]
    expanded: Signal<bool>,
    /// Callback when the section header is clicked
    on_toggle: F,
    /// Section content
    children: Children,
    /// Maximum height when expanded
    #[prop(default = "500px")]
    max_height: &'static str,
    /// Additional CSS classes
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView
where
    F: Fn() + 'static + Clone + Send + Sync,
{
    let section_id = format!(
        "accordion-section-{}",
        title.replace(' ', "-").to_lowercase()
    );
    let content_id = format!("{}-content", section_id);

    let on_toggle_click = on_toggle.clone();
    let on_toggle_key = on_toggle.clone();

    let combined_class = format!(
        "bg-gm-bg-card/80 backdrop-blur-sm rounded-2xl border border-gm-accent-cyan/20 \
         shadow-lg overflow-hidden transition-all duration-300 \
         hover:border-gm-accent-cyan/40 hover:shadow-gm-accent-cyan/10 {}",
        class
    );

    view! {
        <div class=combined_class>
            // Header button
            <button
                class="w-full px-6 py-4 flex items-center justify-between text-left \
                       hover:bg-gm-accent-cyan/10 transition-all duration-200 group \
                       focus:outline-none focus:ring-2 focus:ring-inset focus:ring-gm-accent-cyan"
                type="button"
                on:click=move |_| on_toggle_click()
                on:keydown=move |ev| {
                    if ev.key() == "Enter" || ev.key() == " " {
                        ev.prevent_default();
                        on_toggle_key();
                    }
                }
                aria-expanded=move || expanded.get()
                aria-controls=content_id.clone()
                id=section_id.clone()
            >
                <div class="flex items-center gap-3">
                    {icon.map(|i| view! {
                        <span class="text-gm-accent-cyan group-hover:scale-110 transition-transform duration-200">
                            <Icon name=i.to_string() class="w-5 h-5".to_string() />
                        </span>
                    })}
                    <span class="text-lg font-gaming font-bold text-white \
                                 group-hover:text-gm-accent-cyan transition-colors duration-200">
                        {title}
                    </span>
                </div>
                <span
                    class="text-gm-accent-cyan transition-transform duration-300 ease-in-out"
                    style:transform=move || if expanded.get() { "rotate(180deg)" } else { "rotate(0deg)" }
                    aria-hidden="true"
                >
                    <Icon name="chevron-down".to_string() class="w-5 h-5".to_string() />
                </span>
            </button>

            // Content area
            <div
                id=content_id
                role="region"
                aria-labelledby=section_id
                class="overflow-hidden transition-all duration-300 ease-in-out"
                style:max-height=move || if expanded.get() { max_height } else { "0px" }
                style:opacity=move || if expanded.get() { "1" } else { "0" }
            >
                <div class="px-6 pb-6 pt-2">
                    {children()}
                </div>
            </div>
        </div>
    }
}

/// Accordion component
///
/// A container for multiple accordion sections with optional single/multiple expand mode.
///
/// # Props
///
/// - `children`: Accordion sections
/// - `allow_multiple`: Whether multiple sections can be expanded at once
/// - `class`: Additional CSS classes
/// - `gap`: Gap between sections
///
/// # Example
///
/// ```rust
/// view! {
///     <Accordion allow_multiple=false gap="gap-4">
///         <AccordionSection title="Section 1" icon="settings" ...>
///             // content
///         </AccordionSection>
///         <AccordionSection title="Section 2" icon="user" ...>
///             // content
///         </AccordionSection>
///     </Accordion>
/// }
/// ```
#[component]
pub fn Accordion(
    /// Accordion sections
    children: Children,
    /// Gap between sections
    #[prop(default = "gap-4")]
    gap: &'static str,
    /// Additional CSS classes
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let combined_class = format!("flex flex-col {} {}", gap, class);

    view! {
        <div class=combined_class>
            {children()}
        </div>
    }
}
