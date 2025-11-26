//! Authentication module
//!
//! This module handles GitHub OAuth authentication, token management,
//! and secure token storage.

pub mod crypto;
pub mod oauth;
pub mod token;

pub use crypto::Crypto;
pub use oauth::{OAuthConfig, OAuthFlow};
pub use token::{AuthState, TokenManager, UserInfo};

