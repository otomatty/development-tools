//! GitHub API module
//!
//! This module provides a client for interacting with the GitHub API
//! to fetch user data, contributions, and activity metrics.

pub mod client;
pub mod issues;
pub mod notifications;
pub mod types;

pub use client::GitHubClient;
pub use issues::{generate_actions_template, IssuesClient};
pub use notifications::{
    build_html_url as build_notification_html_url, GitHubNotification, NotificationsClient,
    NotificationsResponse,
};
pub use types::*;
