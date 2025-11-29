//! Mock Server-related types

use serde::{Deserialize, Serialize};

/// Mock Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MockServerConfig {
    pub id: i64,
    pub port: u16,
    pub cors_mode: CorsMode,
    pub cors_origins: Option<Vec<String>>,
    pub cors_methods: Option<Vec<String>>,
    pub cors_headers: Option<Vec<String>>,
    pub cors_max_age: i64,
    pub show_directory_listing: bool,
}

impl Default for MockServerConfig {
    fn default() -> Self {
        Self {
            id: 1,
            port: 9876,
            cors_mode: CorsMode::Simple,
            cors_origins: None,
            cors_methods: None,
            cors_headers: None,
            cors_max_age: 86400,
            show_directory_listing: false,
        }
    }
}

/// CORS mode for the Mock Server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum CorsMode {
    #[default]
    Simple,
    Advanced,
}

/// Directory mapping for the Mock Server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryMapping {
    pub id: i64,
    pub virtual_path: String,
    pub local_path: String,
    pub enabled: bool,
}

/// Request to create a new directory mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMappingRequest {
    pub virtual_path: String,
    pub local_path: String,
}

/// Request to update a directory mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMappingRequest {
    pub id: i64,
    pub virtual_path: Option<String>,
    pub local_path: Option<String>,
    pub enabled: Option<bool>,
}

/// Request to update Mock Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConfigRequest {
    pub port: Option<u16>,
    pub cors_mode: Option<CorsMode>,
    pub cors_origins: Option<Vec<String>>,
    pub cors_methods: Option<Vec<String>>,
    pub cors_headers: Option<Vec<String>>,
    pub cors_max_age: Option<i64>,
    pub show_directory_listing: Option<bool>,
}

/// Mock Server status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ServerStatus {
    Running,
    #[default]
    Stopped,
}

/// Mock Server state information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MockServerState {
    pub status: ServerStatus,
    pub port: u16,
    pub url: String,
    pub mappings_count: usize,
}

impl Default for MockServerState {
    fn default() -> Self {
        Self {
            status: ServerStatus::Stopped,
            port: 9876,
            url: "http://localhost:9876".to_string(),
            mappings_count: 0,
        }
    }
}

/// Access log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessLogEntry {
    pub timestamp: String,
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub response_size: Option<u64>,
    pub response_time_ms: u64,
}

/// File information for file browser
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size: Option<u64>,
    pub mime_type: Option<String>,
}
