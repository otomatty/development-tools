//! Feedback Components
//!
//! User feedback components like toasts, notifications, and loading states.
//!
//! ## Components
//!
//! - `Toast` - Temporary notification overlay (fixed position)
//! - `InlineToast` - Inline notification within panels
//! - `SaveStatusIndicator` - Auto-save status indicator
//! - `Loading` - Loading spinner/indicator
//! - `InlineLoading` - Inline loading indicator
//! - `LoadingOverlay` - Full page loading overlay
//!
//! ## Usage
//!
//! ```rust
//! use crate::components::ui::feedback::{Toast, ToastType, Loading, LoadingSize};
//!
//! let (visible, set_visible) = signal(false);
//! let message = Signal::derive(|| "Saved!".to_string());
//!
//! view! {
//!     <Toast
//!         visible=visible
//!         message=message
//!         toast_type=ToastType::Success
//!     />
//!     <Loading size=LoadingSize::Medium text="読み込み中..." />
//! }
//! ```
//!
//! ## Related Documentation
//!
//! - Spec: `./feedback.spec.md`
//! - Parent Issue: #115

mod loading;
mod toast;

pub use loading::{InlineLoading, Loading, LoadingOverlay, LoadingSize};
pub use toast::{InlineToast, SaveStatusIndicator, Toast, ToastType};
