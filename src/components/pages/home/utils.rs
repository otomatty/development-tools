//! Home Page Utility Functions
//!
//! Helper functions for the home page.
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   └─ src/components/pages/home/mod.rs
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

// Re-export utility functions from features
pub use crate::components::features::gamification::{
    handle_sync_result_notifications, load_user_data,
};
