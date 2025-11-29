//! Skeleton UI components
//!
//! This module provides reusable skeleton loading components
//! that can be used across the application to show loading states.

mod base;
mod variants;

pub use base::Skeleton;
pub use variants::{
    SkeletonAvatar,
    SkeletonCard,
    SkeletonStat,
    SkeletonBadge,
    SkeletonGraph,
};

// Re-export SkeletonText for future use
#[allow(unused)]
pub use variants::SkeletonText;
