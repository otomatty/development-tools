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
//! - `skeleton/` - Loading skeleton components
//!
//! ## Usage
//!
//! ```rust
//! use crate::components::ui::{Card, CardVariant, Modal, DropdownMenu};
//! ```

pub mod card;
pub mod dialog;
pub mod dropdown;
pub mod skeleton;

// Re-exports for convenient access
pub use card::{Card, CardVariant};
pub use dialog::{ConfirmDialog, Modal};
pub use dropdown::{DropdownMenu, DropdownMenuDivider, DropdownMenuItem};
pub use skeleton::{Skeleton, SkeletonAvatar, SkeletonCard, SkeletonText};
