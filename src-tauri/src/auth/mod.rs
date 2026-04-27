//! Authentication module
//!
//! This module handles GitHub Device Flow authentication, token management,
//! and secure token storage.

pub mod crypto;
pub mod oauth;
pub mod session;
pub mod token;

pub use oauth::{
    AuthToken, DeviceCodeResponse, DeviceFlow, DeviceFlowConfig, DeviceTokenStatus, OAuthError,
};
pub use session::{
    classify_unauthorized, handle_unauthorized, map_github_result, reasons, AuthExpiredEvent,
    AUTH_EXPIRED_EVENT,
};
pub use token::{AuthState, TokenManager, UserInfo};
