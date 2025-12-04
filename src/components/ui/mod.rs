//! UI Components Module
//!
//! This module contains reusable UI primitives that form the building blocks
//! of the application's user interface.
//!
//! ## Structure
//!
//! - `accordion/` - Collapsible accordion components
//! - `alert/` - Alert and banner components
//! - `badge/` - Status and label badge components
//! - `button/` - Button components with multiple variants
//! - `card/` - Card container components
//! - `dialog/` - Modal and dialog components
//! - `display/` - Avatar, progress bar, and display components
//! - `dropdown/` - Dropdown menu components
//! - `feedback/` - Toast, notification, and loading components
//! - `form/` - Form input components (toggle, input, etc.)
//! - `layout/` - Page header, empty state, and layout components
//! - `skeleton/` - Loading skeleton components
//!
//! ## Usage
//!
//! ```rust
//! use crate::components::ui::{Button, ButtonVariant, Badge, BadgeVariant};
//! use crate::components::ui::{Card, CardVariant, Modal, ModalSize, DropdownMenu};
//! use crate::components::ui::{PageHeader, EmptyState, Alert, AlertVariant};
//! use crate::components::ui::{Avatar, AvatarSize, ProgressBar};
//! use crate::components::ui::form::{ToggleSwitch, ToggleSwitchSize};
//! use crate::components::ui::feedback::{Toast, ToastType};
//! ```

pub mod accordion;
pub mod alert;
pub mod badge;
pub mod button;
pub mod card;
pub mod dialog;
pub mod display;
pub mod dropdown;
pub mod feedback;
pub mod form;
pub mod layout;
pub mod skeleton;

// Re-exports for convenient access
// Accordion
pub use accordion::{Accordion, AccordionItem, AccordionSection};
// Alert
pub use alert::{Alert, AlertVariant, Banner};
// Badge
pub use badge::{Badge, BadgeSize, BadgeVariant, DynamicBadge, Status, StatusBadge};
// Button
pub use button::{Button, ButtonSize, ButtonVariant, IconButton};
// Card
pub use card::{Card, CardVariant};
// Dialog
pub use dialog::{ConfirmDialog, Modal, ModalBody, ModalFooter, ModalHeader, ModalSize};
// Display
pub use display::{Avatar, AvatarSize, ProgressBar, ProgressBarVariant};
// Dropdown
pub use dropdown::{DropdownMenu, DropdownMenuDivider, DropdownMenuItem};
// Feedback
pub use feedback::{
    InlineLoading, InlineToast, Loading, LoadingOverlay, LoadingSize, SaveStatusIndicator, Toast,
    ToastType,
};
// Form
pub use form::{
    Input, InputSize, InputType, LabeledInput, LabeledToggle, OptionForm, Textarea, ToggleSwitch,
    ToggleSwitchSize,
};
// Layout
pub use layout::{EmptyState, PageHeader, PageHeaderAction};
// Skeleton
pub use skeleton::{
    Skeleton, SkeletonAvatar, SkeletonBadge, SkeletonCard, SkeletonGraph, SkeletonStat,
    SkeletonText,
};
