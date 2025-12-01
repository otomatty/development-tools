//! Mock Server type definitions
//!
//! DEPENDENCY MAP:
//! Parents (Files that import these types):
//!   ├─ src-tauri/src/mock_server/mod.rs
//!   ├─ src-tauri/src/mock_server/server.rs
//!   ├─ src-tauri/src/mock_server/repository.rs
//!   └─ src-tauri/src/commands/mock_server.rs

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CorsMode {
    Simple,
    Advanced,
}

impl From<String> for CorsMode {
    fn from(s: String) -> Self {
        match s.as_str() {
            "advanced" => CorsMode::Advanced,
            _ => CorsMode::Simple,
        }
    }
}

impl From<CorsMode> for String {
    fn from(mode: CorsMode) -> Self {
        match mode {
            CorsMode::Simple => "simple".to_string(),
            CorsMode::Advanced => "advanced".to_string(),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_server_config_default() {
        let config = MockServerConfig::default();

        assert_eq!(config.id, 1);
        assert_eq!(config.port, 9876);
        assert_eq!(config.cors_mode, CorsMode::Simple);
        assert!(config.cors_origins.is_none());
        assert!(config.cors_methods.is_none());
        assert!(config.cors_headers.is_none());
        assert_eq!(config.cors_max_age, 86400);
        assert!(!config.show_directory_listing);
    }

    #[test]
    fn test_cors_mode_from_string() {
        assert_eq!(CorsMode::from("simple".to_string()), CorsMode::Simple);
        assert_eq!(CorsMode::from("advanced".to_string()), CorsMode::Advanced);
        assert_eq!(CorsMode::from("unknown".to_string()), CorsMode::Simple);
        assert_eq!(CorsMode::from("".to_string()), CorsMode::Simple);
    }

    #[test]
    fn test_cors_mode_to_string() {
        assert_eq!(String::from(CorsMode::Simple), "simple");
        assert_eq!(String::from(CorsMode::Advanced), "advanced");
    }

    #[test]
    fn test_mock_server_config_serialization() {
        let config = MockServerConfig {
            id: 1,
            port: 8080,
            cors_mode: CorsMode::Advanced,
            cors_origins: Some(vec!["http://localhost:3000".to_string()]),
            cors_methods: Some(vec!["GET".to_string(), "POST".to_string()]),
            cors_headers: Some(vec!["Content-Type".to_string()]),
            cors_max_age: 3600,
            show_directory_listing: true,
        };

        let json = serde_json::to_string(&config).expect("Should serialize");
        assert!(json.contains("\"port\":8080"));
        assert!(json.contains("\"corsMode\":\"advanced\""));
        assert!(json.contains("\"showDirectoryListing\":true"));

        let deserialized: MockServerConfig =
            serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.port, 8080);
        assert_eq!(deserialized.cors_mode, CorsMode::Advanced);
        assert!(deserialized.show_directory_listing);
    }

    #[test]
    fn test_directory_mapping_serialization() {
        let mapping = DirectoryMapping {
            id: 1,
            virtual_path: "/api".to_string(),
            local_path: "/tmp/api".to_string(),
            enabled: true,
        };

        let json = serde_json::to_string(&mapping).expect("Should serialize");
        assert!(json.contains("\"virtualPath\":\"/api\""));
        assert!(json.contains("\"localPath\":\"/tmp/api\""));
        assert!(json.contains("\"enabled\":true"));

        let deserialized: DirectoryMapping =
            serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.virtual_path, "/api");
        assert_eq!(deserialized.local_path, "/tmp/api");
        assert!(deserialized.enabled);
    }

    #[test]
    fn test_create_mapping_request() {
        let request = CreateMappingRequest {
            virtual_path: "/static".to_string(),
            local_path: "/var/www/static".to_string(),
        };

        let json = serde_json::to_string(&request).expect("Should serialize");
        let deserialized: CreateMappingRequest =
            serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.virtual_path, "/static");
        assert_eq!(deserialized.local_path, "/var/www/static");
    }

    #[test]
    fn test_update_mapping_request_partial() {
        let request = UpdateMappingRequest {
            id: 1,
            virtual_path: Some("/new-path".to_string()),
            local_path: None,
            enabled: Some(false),
        };

        let json = serde_json::to_string(&request).expect("Should serialize");
        let deserialized: UpdateMappingRequest =
            serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.id, 1);
        assert_eq!(deserialized.virtual_path, Some("/new-path".to_string()));
        assert!(deserialized.local_path.is_none());
        assert_eq!(deserialized.enabled, Some(false));
    }

    #[test]
    fn test_update_config_request() {
        let request = UpdateConfigRequest {
            port: Some(3000),
            cors_mode: Some(CorsMode::Advanced),
            cors_origins: Some(vec!["*".to_string()]),
            cors_methods: None,
            cors_headers: None,
            cors_max_age: Some(7200),
            show_directory_listing: Some(true),
        };

        let json = serde_json::to_string(&request).expect("Should serialize");
        let deserialized: UpdateConfigRequest =
            serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.port, Some(3000));
        assert_eq!(deserialized.cors_mode, Some(CorsMode::Advanced));
        assert!(deserialized.cors_methods.is_none());
    }

    #[test]
    fn test_server_status() {
        assert_eq!(
            serde_json::to_string(&ServerStatus::Running).unwrap(),
            "\"running\""
        );
        assert_eq!(
            serde_json::to_string(&ServerStatus::Stopped).unwrap(),
            "\"stopped\""
        );
    }

    #[test]
    fn test_mock_server_state() {
        let state = MockServerState {
            status: ServerStatus::Running,
            port: 8080,
            url: "http://localhost:8080".to_string(),
            mappings_count: 3,
        };

        let json = serde_json::to_string(&state).expect("Should serialize");
        assert!(json.contains("\"status\":\"running\""));
        assert!(json.contains("\"port\":8080"));
        assert!(json.contains("\"mappingsCount\":3"));
    }

    #[test]
    fn test_access_log_entry() {
        let entry = AccessLogEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            method: "GET".to_string(),
            path: "/api/users".to_string(),
            status_code: 200,
            response_size: Some(1234),
            response_time_ms: 45,
        };

        let json = serde_json::to_string(&entry).expect("Should serialize");
        assert!(json.contains("\"method\":\"GET\""));
        assert!(json.contains("\"statusCode\":200"));
        assert!(json.contains("\"responseSize\":1234"));
        assert!(json.contains("\"responseTimeMs\":45"));
    }

    #[test]
    fn test_file_info() {
        let file = FileInfo {
            name: "index.html".to_string(),
            path: "/var/www/index.html".to_string(),
            is_directory: false,
            size: Some(2048),
            mime_type: Some("text/html".to_string()),
        };

        let json = serde_json::to_string(&file).expect("Should serialize");
        assert!(json.contains("\"name\":\"index.html\""));
        assert!(json.contains("\"isDirectory\":false"));
        assert!(json.contains("\"mimeType\":\"text/html\""));
    }

    #[test]
    fn test_file_info_directory() {
        let dir = FileInfo {
            name: "images".to_string(),
            path: "/var/www/images".to_string(),
            is_directory: true,
            size: None,
            mime_type: None,
        };

        let json = serde_json::to_string(&dir).expect("Should serialize");
        assert!(json.contains("\"isDirectory\":true"));
        assert!(json.contains("\"size\":null"));
        assert!(json.contains("\"mimeType\":null"));
    }
}
