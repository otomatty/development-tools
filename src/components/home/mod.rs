//! Home page components
//!
//! This module contains skeleton and utility components for the home page.
//! The main HomePage component has been moved to pages/home_page.rs.
//!
//! DEPENDENCY MAP:
//! Children:
//!   └─ skeleton.rs - Home page skeleton loader
//! Related Documentation:
//!   └─ Issue: https://github.com/otomatty/development-tools/issues/117

pub mod skeleton;

// Re-export skeleton for use in pages/home_page.rs
pub use skeleton::HomeSkeleton;
