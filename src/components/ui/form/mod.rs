//! Form Components
//!
//! Reusable form input components for building user interfaces.
//!
//! ## Components
//!
//! - `ToggleSwitch` - ON/OFF toggle switch with animations
//! - `LabeledToggle` - Toggle switch with label
//! - `OptionForm` - Tool option input form
//! - `Input` - Generic text input
//! - `LabeledInput` - Input with label wrapper
//! - `Textarea` - Multi-line text input
//!
//! ## Usage
//!
//! ```rust
//! use crate::components::ui::form::{ToggleSwitch, ToggleSwitchSize, Input, InputType};
//!
//! view! {
//!     <ToggleSwitch
//!         enabled=true
//!         on_toggle=|| log!("toggled")
//!         size=ToggleSwitchSize::Medium
//!     />
//! }
//! ```
//!
//! ## Related Documentation
//!
//! - Spec: `./form.spec.md`
//! - Parent Issue: #115

mod input;
mod option_form;
mod toggle_switch;

pub use input::{Input, InputSize, InputType, LabeledInput, Textarea};
pub use option_form::OptionForm;
pub use toggle_switch::{LabeledToggle, ToggleSwitch, ToggleSwitchSize};
