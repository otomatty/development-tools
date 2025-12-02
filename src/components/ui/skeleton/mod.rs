//! Skeleton UI components
//!
//! This module provides reusable skeleton loading components
//! that can be used across the application to show loading states.
//!
//! DEPENDENCY MAP:
//!
//! Parents (Files that import this component):
//!   └─ src/components/ui/mod.rs
//!
//! Related Documentation:
//!   └─ GitHub Issue: #114

mod base;
mod variants;

pub use base::Skeleton;
pub use variants::{SkeletonAvatar, SkeletonBadge, SkeletonCard, SkeletonGraph, SkeletonStat};

// Re-export SkeletonText for future use
#[allow(unused)]
pub use variants::SkeletonText;
