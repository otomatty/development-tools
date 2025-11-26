//! Database module for SQLite operations
//!
//! This module provides database connection management, migrations,
//! and CRUD operations for the gamification system.

pub mod connection;
pub mod migrations;
pub mod models;
pub mod repository;

pub use connection::{Database, DatabaseError};
pub use models::*;

