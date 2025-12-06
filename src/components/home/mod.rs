//! Home page components
//!
//! This module contains skeleton and utility components for the home page.
//! The main HomePage component has been moved to pages/home_page.rs.
//!
//! DEPENDENCY MAP:
//! Children:
//!   └─ skeleton.rs - Home page skeleton loader
//! Related Documentation:
//!   ├─ Issue #117: Home page gamification
//!   └─ Phase 3 Performance: https://github.com/otomatty/development-tools/issues/126

pub mod skeleton;

// Re-export skeletons for use in dashboard_content.rs and other components
pub use skeleton::{
    BadgeGridSkeleton, ChallengeCardSkeleton, ContributionGraphSkeleton, HomeSkeleton,
    ProfileCardSkeleton, StatsDisplaySkeleton,
};
