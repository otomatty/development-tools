//! Authentication module
//!
//! This module handles GitHub Device Flow authentication, token management,
//! and secure token storage.

pub mod crypto;
pub mod oauth;
pub mod token;

pub use oauth::{
    AuthToken, DeviceCodeResponse, DeviceFlow, DeviceFlowConfig, DeviceTokenStatus, OAuthError,
};
pub use token::{AuthState, TokenManager, UserInfo};
