//! Input Component
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this component):
//!   ├─ src/components/ui/form/mod.rs
//!   └─ src/components/ui/mod.rs (re-export)
//! Related Documentation:
//!   ├─ Spec: ./form.spec.md
//!   └─ Tests: (embedded in this file)

use leptos::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Static counter for generating unique IDs
static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Generate a unique ID for form elements
fn generate_unique_id(prefix: &str) -> String {
    let id = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{}", prefix.to_lowercase().replace(' ', "-"), id)
}

/// Input type enum for different input variations
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum InputType {
    #[default]
    Text,
    Password,
    Number,
    Email,
    Url,
    Search,
}

impl InputType {
    pub fn as_str(&self) -> &'static str {
        match self {
            InputType::Text => "text",
            InputType::Password => "password",
            InputType::Number => "number",
            InputType::Email => "email",
            InputType::Url => "url",
            InputType::Search => "search",
        }
    }
}

/// Input size variants
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum InputSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl InputSize {
    pub fn class(&self) -> &'static str {
        match self {
            InputSize::Small => "px-2 py-1 text-sm",
            InputSize::Medium => "px-3 py-2 text-base",
            InputSize::Large => "px-4 py-3 text-lg",
        }
    }
}

/// Reusable text input component
///
/// ## Usage
///
/// ```rust
/// use leptos::prelude::*;
/// use crate::components::ui::form::{Input, InputType};
///
/// #[component]
/// fn Example() -> impl IntoView {
///     let value = RwSignal::new(String::new());
///     view! {
///         <Input
///             value=value
///             placeholder="Enter your name..."
///             input_type=InputType::Text
///         />
///     }
/// }
/// ```
#[component]
pub fn Input(
    /// The value signal (two-way binding)
    value: RwSignal<String>,
    /// Input type (text, password, number, etc.)
    #[prop(default = InputType::Text)]
    input_type: InputType,
    /// Placeholder text
    #[prop(optional)]
    placeholder: Option<&'static str>,
    /// Whether the input is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Size of the input
    #[prop(default = InputSize::Medium)]
    size: InputSize,
    /// Additional CSS classes
    #[prop(optional)]
    class: Option<&'static str>,
    /// Optional name attribute for forms
    #[prop(optional)]
    name: Option<&'static str>,
    /// Optional id attribute (supports dynamic String IDs)
    #[prop(optional)]
    id: Option<String>,
) -> impl IntoView {
    let base_class = "w-full bg-gm-bg-secondary border border-gm-border rounded-md \
                      text-dt-text-main placeholder-dt-text-sub/50 \
                      focus:outline-none focus:ring-2 focus:ring-gm-accent-cyan/50 focus:border-gm-accent-cyan \
                      transition-colors duration-200";

    let disabled_class = if disabled {
        "opacity-50 cursor-not-allowed"
    } else {
        ""
    };

    let size_class = size.class();
    let extra_class = class.unwrap_or("");

    let combined_class = format!(
        "{} {} {} {}",
        base_class, size_class, disabled_class, extra_class
    );

    view! {
        <input
            type=input_type.as_str()
            class=combined_class
            placeholder=placeholder.unwrap_or("")
            disabled=disabled
            name=name.unwrap_or("")
            id=id.as_deref().unwrap_or("")
            prop:value=move || value.get()
            on:input=move |ev| {
                let new_value = event_target_value(&ev);
                value.set(new_value);
            }
        />
    }
}

/// Input with label wrapper
///
/// ## Usage
///
/// ```rust
/// use leptos::prelude::*;
/// use crate::components::ui::form::LabeledInput;
///
/// #[component]
/// fn Example() -> impl IntoView {
///     let username = RwSignal::new(String::new());
///     view! {
///         <LabeledInput
///             value=username
///             label="Username"
///             required=true
///             description="Enter your unique username"
///         />
///     }
/// }
/// ```
#[component]
pub fn LabeledInput(
    /// The value signal
    value: RwSignal<String>,
    /// Label text
    label: &'static str,
    /// Input type
    #[prop(default = InputType::Text)]
    input_type: InputType,
    /// Placeholder text
    #[prop(optional)]
    placeholder: Option<&'static str>,
    /// Whether the field is required
    #[prop(default = false)]
    required: bool,
    /// Whether the input is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Optional description text
    #[prop(optional)]
    description: Option<&'static str>,
    /// Size of the input
    #[prop(default = InputSize::Medium)]
    size: InputSize,
) -> impl IntoView {
    // Generate a unique ID to ensure label-input association works correctly
    // even when multiple LabeledInput components have the same label
    let input_id = generate_unique_id(&format!("input-{}", label));

    view! {
        <div class="flex flex-col gap-1">
            <label
                for=input_id.clone()
                class="text-sm font-medium text-dt-text-main"
            >
                {label}
                {required.then(|| view! {
                    <span class="text-red-500 ml-1">"*"</span>
                })}
            </label>

            {description.map(|desc| view! {
                <span class="text-xs text-dt-text-sub">{desc}</span>
            })}

            <Input
                value=value
                input_type=input_type
                placeholder=placeholder.unwrap_or("")
                disabled=disabled
                size=size
                id=Some(input_id)
            />
        </div>
    }
}

/// Textarea component for multi-line input
///
/// ## Usage
///
/// ```rust
/// use leptos::prelude::*;
/// use crate::components::ui::form::Textarea;
///
/// #[component]
/// fn Example() -> impl IntoView {
///     let content = RwSignal::new(String::new());
///     view! {
///         <Textarea
///             value=content
///             placeholder="Enter your message..."
///             rows=4
///         />
///     }
/// }
/// ```
#[component]
pub fn Textarea(
    /// The value signal
    value: RwSignal<String>,
    /// Placeholder text
    #[prop(optional)]
    placeholder: Option<&'static str>,
    /// Number of rows
    #[prop(default = 3)]
    rows: u32,
    /// Whether the textarea is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Additional CSS classes
    #[prop(optional)]
    class: Option<&'static str>,
    /// Whether the textarea is resizable
    #[prop(default = true)]
    resizable: bool,
) -> impl IntoView {
    let base_class = "w-full bg-gm-bg-secondary border border-gm-border rounded-md \
                      text-dt-text-main placeholder-dt-text-sub/50 \
                      focus:outline-none focus:ring-2 focus:ring-gm-accent-cyan/50 focus:border-gm-accent-cyan \
                      transition-colors duration-200 px-3 py-2";

    let disabled_class = if disabled {
        "opacity-50 cursor-not-allowed"
    } else {
        ""
    };

    let resize_class = if resizable { "resize-y" } else { "resize-none" };

    let extra_class = class.unwrap_or("");

    let combined_class = format!(
        "{} {} {} {}",
        base_class, disabled_class, resize_class, extra_class
    );

    view! {
        <textarea
            class=combined_class
            placeholder=placeholder.unwrap_or("")
            rows=rows
            disabled=disabled
            prop:value=move || value.get()
            on:input=move |ev| {
                let new_value = event_target_value(&ev);
                value.set(new_value);
            }
        />
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TC-004: Input 基本レンダリング（型検証）
    #[test]
    fn test_input_type_as_str() {
        assert_eq!(InputType::Text.as_str(), "text");
        assert_eq!(InputType::Password.as_str(), "password");
        assert_eq!(InputType::Number.as_str(), "number");
        assert_eq!(InputType::Email.as_str(), "email");
        assert_eq!(InputType::Url.as_str(), "url");
        assert_eq!(InputType::Search.as_str(), "search");
    }

    // TC-005: Input サイズクラス
    #[test]
    fn test_input_size_class() {
        assert_eq!(InputSize::Small.class(), "px-2 py-1 text-sm");
        assert_eq!(InputSize::Medium.class(), "px-3 py-2 text-base");
        assert_eq!(InputSize::Large.class(), "px-4 py-3 text-lg");
    }

    // デフォルト値のテスト
    #[test]
    fn test_input_type_default() {
        let default_type = InputType::default();
        assert_eq!(default_type, InputType::Text);
    }

    #[test]
    fn test_input_size_default() {
        let default_size = InputSize::default();
        assert_eq!(default_size, InputSize::Medium);
    }
}
