//! Authentication-related types

use serde::{Deserialize, Serialize};

/// 認証状態
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AuthState {
    pub is_logged_in: bool,
    pub user: Option<UserInfo>,
}

/// ユーザー情報
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: i64,
    pub github_id: i64,
    pub username: String,
    pub avatar_url: Option<String>,
    pub created_at: Option<String>,
}

/// Device Flow開始時のレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: i64,
    pub interval: i64,
}

/// Device Flowトークンポーリングのステータス
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DeviceTokenStatus {
    /// 認証待ち - ユーザーがまだ認証を完了していない
    Pending,
    /// 認証成功 - ログイン完了
    Success { auth_state: AuthState },
    /// エラー発生
    Error { message: String },
}

/// GitHubユーザー
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    pub avatar_url: String,
    pub name: Option<String>,
    pub bio: Option<String>,
    pub public_repos: i32,
    pub followers: i32,
    pub following: i32,
    pub created_at: String,
}
