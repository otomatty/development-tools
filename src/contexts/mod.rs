//! Contexts module
//!
//! グローバル状態管理のためのコンテキストを提供します。
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   - src/main.rs
//!   - src/app.rs
//! Children (Submodules):
//!   - animation_context
//!   - network_context

pub mod animation_context;
pub mod network_context;

// Re-exports
pub use animation_context::AnimationContext;
pub use network_context::{NetworkStatusContext, NetworkStatusProvider};
