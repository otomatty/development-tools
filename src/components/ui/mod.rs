//! UI Components Module
//!
//! This module contains reusable UI primitives that form the building blocks
//! of the application's user interface.
//!
//! ## Structure
//!
//! - `card/` - Card container components
//! - `dialog/` - Modal and dialog components
//! - `dropdown/` - Dropdown menu components
//! - `feedback/` - Toast, notification, and loading components
//! - `form/` - Form input components (toggle, input, etc.)
//! - `skeleton/` - Loading skeleton components
//!
//! ## Usage
//!
//! ```rust
//! use crate::components::ui::{Card, CardVariant, Modal, ModalSize, DropdownMenu};
//! use crate::components::ui::form::{ToggleSwitch, ToggleSwitchSize};
//! use crate::components::ui::feedback::{Toast, ToastType};
//! ```

pub mod card;
pub mod dialog;
pub mod dropdown;
pub mod feedback;
pub mod form;
pub mod skeleton;

// Re-exports for convenient access
pub use card::{Card, CardVariant};
pub use dialog::{ConfirmDialog, Modal, ModalBody, ModalFooter, ModalHeader, ModalSize};
pub use dropdown::{DropdownMenu, DropdownMenuDivider, DropdownMenuItem};
pub use feedback::{
    InlineLoading, InlineToast, Loading, LoadingOverlay, LoadingSize, SaveStatusIndicator, Toast,
    ToastType,
};
pub use form::{
    Input, InputSize, InputType, LabeledInput, LabeledToggle, OptionForm, Textarea, ToggleSwitch,
    ToggleSwitchSize,
};
pub use skeleton::{
    Skeleton, SkeletonAvatar, SkeletonBadge, SkeletonCard, SkeletonGraph, SkeletonStat,
    SkeletonText,
};
