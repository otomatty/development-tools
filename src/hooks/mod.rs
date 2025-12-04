//! Hooks module
//!
//! アプリケーション全体で使用するカスタムフックを提供します。
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   - src/main.rs
//!   - src/components/ (various components)
//! Dependencies:
//!   - src/contexts/

pub mod use_animation;
pub mod use_network_status;
pub mod use_toast;

// Re-exports
pub use use_animation::{use_animation_context, use_animation_context_or_default, use_is_animated};
pub use use_network_status::{try_use_network_status, use_is_online, use_network_status};
pub use use_toast::{use_toast, ToastMessage, ToastType as ToastVariant, UseToast};
