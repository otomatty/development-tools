//! Feature Components Module
//!
//! This module contains feature-specific components grouped by business domain.
//! Each feature module manages related components for a specific functional area.
//!
//! ## Structure
//!
//! - `gamification/` - Gamification and XP system components
//! - `auth/` - Authentication related components
//! - `issues/` - Issue management and project components

pub mod auth;
pub mod gamification;
pub mod issues;
