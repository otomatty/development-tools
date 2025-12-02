//! Dialog Components
//!
//! Provides modal and dialog components for overlays and confirmations.

mod confirm_dialog;
mod modal;

pub use confirm_dialog::ConfirmDialog;
pub use modal::{Modal, ModalBody, ModalFooter, ModalHeader, ModalSize};
