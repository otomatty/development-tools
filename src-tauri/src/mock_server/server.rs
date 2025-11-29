//! Mock Server HTTP server implementation
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   └─ src-tauri/src/mock_server/mod.rs
//! Dependencies:
//!   ├─ src-tauri/src/mock_server/types.rs
//!   └─ src-tauri/src/mock_server/repository.rs

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, Method, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use tokio::sync::{broadcast, oneshot, RwLock};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};

use super::types::{AccessLogEntry, CorsMode, DirectoryMapping, MockServerConfig};

/// Shared state for the mock server
#[derive(Clone)]
pub struct ServerState {
    pub mappings: Arc<RwLock<Vec<DirectoryMapping>>>,
    pub log_sender: broadcast::Sender<AccessLogEntry>,
}

/// Mock Server instance
pub struct MockServer {
    shutdown_tx: Option<oneshot::Sender<()>>,
    port: u16,
    log_sender: broadcast::Sender<AccessLogEntry>,
}

impl MockServer {
    /// Create a new mock server
    pub fn new() -> Self {
        let (log_sender, _) = broadcast::channel(1000);
        Self {
            shutdown_tx: None,
            port: 0, // Will be set when server starts
            log_sender,
        }
    }

    /// Get a receiver for access logs
    pub fn subscribe_logs(&self) -> broadcast::Receiver<AccessLogEntry> {
        self.log_sender.subscribe()
    }

    /// Start the mock server
    pub async fn start(
        &mut self,
        config: MockServerConfig,
        mappings: Vec<DirectoryMapping>,
    ) -> Result<(), String> {
        if self.shutdown_tx.is_some() {
            return Err("Server is already running".to_string());
        }

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        self.port = config.port;

        let state = ServerState {
            mappings: Arc::new(RwLock::new(mappings.clone())),
            log_sender: self.log_sender.clone(),
        };

        // Build CORS layer
        let cors = build_cors_layer(&config);

        // Build router with mappings
        let router = build_router(state.clone(), &mappings, config.show_directory_listing)
            .layer(cors)
            .layer(TraceLayer::new_for_http());

        let addr = SocketAddr::from(([127, 0, 0, 1], config.port));

        // Spawn the server
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| format!("Failed to bind to port {}: {}", config.port, e))?;

        // Get actual port (important when port 0 is used for dynamic allocation)
        let actual_port = listener.local_addr()
            .map(|addr| addr.port())
            .unwrap_or(config.port);
        self.port = actual_port;

        tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .ok();
        });

        self.shutdown_tx = Some(shutdown_tx);
        Ok(())
    }

    /// Stop the mock server
    pub async fn stop(&mut self) -> Result<(), String> {
        if let Some(tx) = self.shutdown_tx.take() {
            tx.send(()).map_err(|_| "Failed to send shutdown signal")?;
        }
        Ok(())
    }

    /// Check if server is running
    pub fn is_running(&self) -> bool {
        self.shutdown_tx.is_some()
    }

    /// Get current port
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Update mappings on a running server
    /// 
    /// Note: Currently, updating mappings at runtime is not supported.
    /// The server must be restarted to apply new mappings.
    pub async fn update_mappings(&self, _mappings: Vec<DirectoryMapping>) -> Result<(), String> {
        // 実行時のマッピング更新は現在サポートされていません
        // 新しいマッピングを適用するにはサーバーの再起動が必要です
        Err("Updating mappings at runtime is not supported. Please restart the server to apply new mappings.".to_string())
    }
}

impl Default for MockServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Build CORS layer based on configuration
fn build_cors_layer(config: &MockServerConfig) -> CorsLayer {
    match config.cors_mode {
        CorsMode::Simple => CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
            .max_age(std::time::Duration::from_secs(config.cors_max_age as u64)),
        CorsMode::Advanced => {
            let mut cors = CorsLayer::new();

            // Set allowed origins
            if let Some(origins) = &config.cors_origins {
                if origins.iter().any(|o| o == "*") {
                    cors = cors.allow_origin(Any);
                } else {
                    let origins: Vec<_> = origins
                        .iter()
                        .filter_map(|o| o.parse().ok())
                        .collect();
                    cors = cors.allow_origin(origins);
                }
            } else {
                cors = cors.allow_origin(Any);
            }

            // Set allowed methods
            if let Some(methods) = &config.cors_methods {
                let methods: Vec<Method> = methods
                    .iter()
                    .filter_map(|m| m.parse().ok())
                    .collect();
                cors = cors.allow_methods(methods);
            } else {
                cors = cors.allow_methods(Any);
            }

            // Set allowed headers
            if let Some(headers) = &config.cors_headers {
                let headers: Vec<_> = headers
                    .iter()
                    .filter_map(|h| h.parse().ok())
                    .collect();
                cors = cors.allow_headers(headers);
            } else {
                cors = cors.allow_headers(Any);
            }

            cors.max_age(std::time::Duration::from_secs(config.cors_max_age as u64))
        }
    }
}

/// Build router with file serving for each mapping
fn build_router(
    state: ServerState,
    mappings: &[DirectoryMapping],
    _show_directory_listing: bool,
) -> Router {
    let mut router = Router::new();

    // Add health check endpoint
    router = router.route("/health", get(health_check));

    // Add file serving for each mapping
    for mapping in mappings.iter().filter(|m| m.enabled) {
        let path = PathBuf::from(&mapping.local_path);
        if path.exists() && path.is_dir() {
            let serve_dir = ServeDir::new(&mapping.local_path)
                .precompressed_gzip()
                .precompressed_br();

            let virtual_path = mapping.virtual_path.trim_start_matches('/');
            let route_path = format!("/{}{{*path}}", virtual_path);

            let state_clone = state.clone();
            let virtual_path_clone = mapping.virtual_path.clone();

            router = router.route(
                &route_path,
                get(move |Path(path): Path<String>, request: Request<Body>| {
                    serve_file(
                        state_clone.clone(),
                        virtual_path_clone.clone(),
                        path,
                        request,
                        serve_dir.clone(),
                    )
                }),
            );

            // Also serve root of virtual path
            let state_clone = state.clone();
            let virtual_path_clone = mapping.virtual_path.clone();
            let serve_dir_clone = ServeDir::new(&mapping.local_path)
                .precompressed_gzip()
                .precompressed_br();

            router = router.route(
                &format!("/{}", virtual_path),
                get(move |request: Request<Body>| {
                    serve_file(
                        state_clone.clone(),
                        virtual_path_clone.clone(),
                        String::new(),
                        request,
                        serve_dir_clone.clone(),
                    )
                }),
            );
        }
    }

    // Fallback for 404
    router = router.fallback(not_found);

    router.with_state(state)
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

/// Serve a file and log the access
async fn serve_file(
    state: ServerState,
    virtual_path: String,
    path: String,
    request: Request<Body>,
    serve_dir: ServeDir,
) -> Response {
    let start = Instant::now();
    let method = request.method().to_string();
    let full_path = if path.is_empty() {
        virtual_path.clone()
    } else {
        format!("{}/{}", virtual_path, path)
    };

    // Create a new request for ServeDir
    let uri = if path.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", path)
    };

    let (parts, body) = request.into_parts();
    let mut new_request = Request::builder()
        .method(parts.method)
        .uri(&uri);

    for (key, value) in parts.headers.iter() {
        new_request = new_request.header(key, value);
    }

    let new_request = new_request.body(body).expect("Failed to build request");

    // Serve the file
    use tower::ServiceExt;
    let response = serve_dir.oneshot(new_request).await;

    let elapsed = start.elapsed();

    match response {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let size = resp
                .headers()
                .get(header::CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok());

            // Log the access
            let log_entry = AccessLogEntry {
                timestamp: chrono::Utc::now().to_rfc3339(),
                method,
                path: full_path,
                status_code: status,
                response_size: size,
                response_time_ms: elapsed.as_millis() as u64,
            };
            let _ = state.log_sender.send(log_entry);

            resp.into_response()
        }
        Err(_) => {
            // Log 500 error
            let log_entry = AccessLogEntry {
                timestamp: chrono::Utc::now().to_rfc3339(),
                method,
                path: full_path,
                status_code: 500,
                response_size: None,
                response_time_ms: elapsed.as_millis() as u64,
            };
            let _ = state.log_sender.send(log_entry);

            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// 404 handler
async fn not_found(State(state): State<ServerState>, request: Request<Body>) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    let log_entry = AccessLogEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        method,
        path,
        status_code: 404,
        response_size: None,
        response_time_ms: 0,
    };
    let _ = state.log_sender.send(log_entry);

    StatusCode::NOT_FOUND.into_response()
}

/// Get files in a directory (for file browser)
pub fn list_directory(path: &str) -> Result<Vec<super::types::FileInfo>, String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err("Directory does not exist".to_string());
    }
    if !path.is_dir() {
        return Err("Path is not a directory".to_string());
    }

    let mut files = Vec::new();
    let entries = std::fs::read_dir(&path).map_err(|e| e.to_string())?;

    for entry_result in entries {
        let entry = match entry_result {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("Failed to read directory entry: {}", e);
                continue;
            }
        };
        let file_path = entry.path();
        let metadata = entry.metadata().ok();

        let name = entry.file_name().to_string_lossy().to_string();
        let is_directory = file_path.is_dir();
        let size = metadata.as_ref().and_then(|m| {
            if m.is_file() {
                Some(m.len())
            } else {
                None
            }
        });

        let mime_type = if is_directory {
            None
        } else {
            mime_guess::from_path(&file_path)
                .first()
                .map(|m| m.to_string())
        };

        files.push(super::types::FileInfo {
            name,
            path: file_path.to_string_lossy().to_string(),
            is_directory,
            size,
            mime_type,
        });
    }

    // Sort: directories first, then by name
    files.sort_by(|a, b| {
        match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_config() -> MockServerConfig {
        MockServerConfig {
            id: 1,
            port: 0, // Let OS assign port
            cors_mode: CorsMode::Simple,
            cors_origins: None,
            cors_methods: None,
            cors_headers: None,
            cors_max_age: 86400,
            show_directory_listing: false,
        }
    }

    #[test]
    fn test_mock_server_new() {
        let server = MockServer::new();
        assert_eq!(server.port, 0);
        assert!(server.shutdown_tx.is_none());
    }

    #[test]
    fn test_list_directory_success() {
        // Create a temp directory with some files
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let base_path = temp_dir.path();

        // Create test files and directories
        fs::create_dir(base_path.join("subdir")).expect("Should create subdir");
        File::create(base_path.join("file1.txt"))
            .expect("Should create file1")
            .write_all(b"hello")
            .expect("Should write");
        File::create(base_path.join("file2.html"))
            .expect("Should create file2")
            .write_all(b"<html></html>")
            .expect("Should write");

        let result = list_directory(base_path.to_str().unwrap());
        assert!(result.is_ok());

        let files = result.unwrap();
        assert_eq!(files.len(), 3);

        // First should be directory
        assert!(files[0].is_directory);
        assert_eq!(files[0].name, "subdir");

        // Files should be sorted alphabetically
        let file_names: Vec<&str> = files.iter().filter(|f| !f.is_directory).map(|f| f.name.as_str()).collect();
        assert!(file_names.contains(&"file1.txt"));
        assert!(file_names.contains(&"file2.html"));
    }

    #[test]
    fn test_list_directory_not_found() {
        let result = list_directory("/nonexistent/path/12345");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_directory_file_not_dir() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let file_path = temp_dir.path().join("test.txt");
        File::create(&file_path).expect("Should create file");

        let result = list_directory(file_path.to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_list_directory_mime_types() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let base_path = temp_dir.path();

        // Create files with different extensions
        File::create(base_path.join("script.js")).expect("Should create");
        File::create(base_path.join("styles.css")).expect("Should create");
        File::create(base_path.join("data.json")).expect("Should create");
        File::create(base_path.join("image.png")).expect("Should create");

        let result = list_directory(base_path.to_str().unwrap());
        let files = result.expect("Should list files");

        for file in &files {
            assert!(file.mime_type.is_some(), "File {} should have mime type", file.name);
        }

        // Check specific mime types
        let js_file = files.iter().find(|f| f.name == "script.js").unwrap();
        assert!(js_file.mime_type.as_ref().unwrap().contains("javascript"));

        let css_file = files.iter().find(|f| f.name == "styles.css").unwrap();
        assert!(css_file.mime_type.as_ref().unwrap().contains("css"));

        let json_file = files.iter().find(|f| f.name == "data.json").unwrap();
        assert!(json_file.mime_type.as_ref().unwrap().contains("json"));
    }

    #[test]
    fn test_list_directory_sorting() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let base_path = temp_dir.path();

        // Create files and directories with various names
        fs::create_dir(base_path.join("zebra_dir")).expect("Should create");
        fs::create_dir(base_path.join("alpha_dir")).expect("Should create");
        File::create(base_path.join("beta.txt")).expect("Should create");
        File::create(base_path.join("gamma.txt")).expect("Should create");
        File::create(base_path.join("Alpha.txt")).expect("Should create");

        let result = list_directory(base_path.to_str().unwrap());
        let files = result.expect("Should list files");

        // First two should be directories (sorted by name)
        assert!(files[0].is_directory);
        assert_eq!(files[0].name, "alpha_dir");
        assert!(files[1].is_directory);
        assert_eq!(files[1].name, "zebra_dir");

        // Rest should be files (case-insensitive sorted)
        assert!(!files[2].is_directory);
        assert_eq!(files[2].name.to_lowercase(), "alpha.txt");
    }

    #[tokio::test]
    async fn test_mock_server_start_stop() {
        let mut server = MockServer::new();
        let config = MockServerConfig {
            port: 0, // Let OS assign available port
            ..create_test_config()
        };
        let mappings = vec![];

        // Start server
        let result = server.start(config, mappings).await;
        assert!(result.is_ok(), "Server should start: {:?}", result);

        let port = server.port;
        assert!(port > 0, "Port should be assigned");

        // Verify server is running by making a request
        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://127.0.0.1:{}/", port))
            .send()
            .await;
        
        // Should get a response (404 is expected since no mappings)
        assert!(response.is_ok());

        // Stop server
        let stop_result = server.stop().await;
        assert!(stop_result.is_ok());

        // Give server time to shutdown
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Verify server is stopped (connection should fail)
        let response = client
            .get(format!("http://127.0.0.1:{}/", port))
            .send()
            .await;
        
        assert!(response.is_err(), "Connection should fail after stop");
    }

    #[tokio::test]
    async fn test_mock_server_serve_static_file() {
        // Create temp directory with test file
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let file_path = temp_dir.path().join("test.txt");
        let mut file = File::create(&file_path).expect("Should create file");
        file.write_all(b"Hello, Mock Server!").expect("Should write");

        let mut server = MockServer::new();
        let config = MockServerConfig {
            port: 0,
            ..create_test_config()
        };
        let mappings = vec![DirectoryMapping {
            id: 1,
            virtual_path: "/files".to_string(),
            local_path: temp_dir.path().to_string_lossy().to_string(),
            enabled: true,
        }];

        // Start server
        server.start(config, mappings).await.expect("Should start");
        let port = server.port;

        // Request the file
        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://127.0.0.1:{}/files/test.txt", port))
            .send()
            .await
            .expect("Should get response");

        assert_eq!(response.status(), 200);
        let body = response.text().await.expect("Should get body");
        assert_eq!(body, "Hello, Mock Server!");

        server.stop().await.expect("Should stop");
    }

    #[tokio::test]
    async fn test_mock_server_404_for_nonexistent() {
        let temp_dir = TempDir::new().expect("Should create temp dir");

        let mut server = MockServer::new();
        let config = MockServerConfig {
            port: 0,
            ..create_test_config()
        };
        let mappings = vec![DirectoryMapping {
            id: 1,
            virtual_path: "/files".to_string(),
            local_path: temp_dir.path().to_string_lossy().to_string(),
            enabled: true,
        }];

        server.start(config, mappings).await.expect("Should start");
        let port = server.port;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://127.0.0.1:{}/files/nonexistent.txt", port))
            .send()
            .await
            .expect("Should get response");

        assert_eq!(response.status(), 404);

        server.stop().await.expect("Should stop");
    }

    #[tokio::test]
    async fn test_mock_server_unmapped_path() {
        let temp_dir = TempDir::new().expect("Should create temp dir");

        let mut server = MockServer::new();
        let config = MockServerConfig {
            port: 0,
            ..create_test_config()
        };
        let mappings = vec![DirectoryMapping {
            id: 1,
            virtual_path: "/mapped".to_string(),
            local_path: temp_dir.path().to_string_lossy().to_string(),
            enabled: true,
        }];

        server.start(config, mappings).await.expect("Should start");
        let port = server.port;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://127.0.0.1:{}/unmapped/file.txt", port))
            .send()
            .await
            .expect("Should get response");

        assert_eq!(response.status(), 404);

        server.stop().await.expect("Should stop");
    }

    #[tokio::test]
    async fn test_mock_server_cors_simple() {
        let temp_dir = TempDir::new().expect("Should create temp dir");

        let mut server = MockServer::new();
        let config = MockServerConfig {
            port: 0,
            cors_mode: CorsMode::Simple,
            ..create_test_config()
        };
        let mappings = vec![DirectoryMapping {
            id: 1,
            virtual_path: "/".to_string(),
            local_path: temp_dir.path().to_string_lossy().to_string(),
            enabled: true,
        }];

        server.start(config, mappings).await.expect("Should start");
        let port = server.port;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://127.0.0.1:{}/", port))
            .header("Origin", "http://example.com")
            .send()
            .await
            .expect("Should get response");

        // CORS headers should be present
        let headers = response.headers();
        assert!(
            headers.get("access-control-allow-origin").is_some(),
            "Should have CORS origin header"
        );

        server.stop().await.expect("Should stop");
    }

    #[tokio::test]
    async fn test_mock_server_disabled_mapping() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let file_path = temp_dir.path().join("test.txt");
        File::create(&file_path).expect("Should create file");

        let mut server = MockServer::new();
        let config = MockServerConfig {
            port: 0,
            ..create_test_config()
        };
        let mappings = vec![DirectoryMapping {
            id: 1,
            virtual_path: "/files".to_string(),
            local_path: temp_dir.path().to_string_lossy().to_string(),
            enabled: false, // Disabled!
        }];

        server.start(config, mappings).await.expect("Should start");
        let port = server.port;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://127.0.0.1:{}/files/test.txt", port))
            .send()
            .await
            .expect("Should get response");

        // Should return 404 because mapping is disabled
        assert_eq!(response.status(), 404);

        server.stop().await.expect("Should stop");
    }

    #[tokio::test]
    async fn test_mock_server_log_subscription() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let file_path = temp_dir.path().join("test.txt");
        let mut file = File::create(&file_path).expect("Should create file");
        file.write_all(b"test content").expect("Should write");

        let mut server = MockServer::new();
        let config = MockServerConfig {
            port: 0,
            ..create_test_config()
        };
        let mappings = vec![DirectoryMapping {
            id: 1,
            virtual_path: "/".to_string(),
            local_path: temp_dir.path().to_string_lossy().to_string(),
            enabled: true,
        }];

        server.start(config, mappings).await.expect("Should start");
        let port = server.port;

        // Subscribe to logs
        let mut log_receiver = server.subscribe_logs();

        // Make a request
        let client = reqwest::Client::new();
        client
            .get(format!("http://127.0.0.1:{}/test.txt", port))
            .send()
            .await
            .expect("Should get response");

        // Wait for log entry
        let log_result = tokio::time::timeout(
            tokio::time::Duration::from_secs(1),
            log_receiver.recv(),
        )
        .await;

        assert!(log_result.is_ok(), "Should receive log within timeout");
        let log_entry = log_result.unwrap().expect("Should have log entry");
        
        assert_eq!(log_entry.method, "GET");
        assert!(log_entry.path.contains("test.txt"));
        assert_eq!(log_entry.status_code, 200);

        server.stop().await.expect("Should stop");
    }

    #[tokio::test]
    async fn test_mock_server_multiple_mappings() {
        let temp_dir1 = TempDir::new().expect("Should create temp dir 1");
        let temp_dir2 = TempDir::new().expect("Should create temp dir 2");

        // Create different files in each directory
        let mut file1 = File::create(temp_dir1.path().join("file1.txt")).expect("Create file1");
        file1.write_all(b"Content from dir 1").expect("Write");
        
        let mut file2 = File::create(temp_dir2.path().join("file2.txt")).expect("Create file2");
        file2.write_all(b"Content from dir 2").expect("Write");

        let mut server = MockServer::new();
        let config = MockServerConfig {
            port: 0,
            ..create_test_config()
        };
        let mappings = vec![
            DirectoryMapping {
                id: 1,
                virtual_path: "/dir1".to_string(),
                local_path: temp_dir1.path().to_string_lossy().to_string(),
                enabled: true,
            },
            DirectoryMapping {
                id: 2,
                virtual_path: "/dir2".to_string(),
                local_path: temp_dir2.path().to_string_lossy().to_string(),
                enabled: true,
            },
        ];

        server.start(config, mappings).await.expect("Should start");
        let port = server.port;

        let client = reqwest::Client::new();

        // Request from first mapping
        let response1 = client
            .get(format!("http://127.0.0.1:{}/dir1/file1.txt", port))
            .send()
            .await
            .expect("Should get response");
        assert_eq!(response1.status(), 200);
        assert_eq!(response1.text().await.unwrap(), "Content from dir 1");

        // Request from second mapping
        let response2 = client
            .get(format!("http://127.0.0.1:{}/dir2/file2.txt", port))
            .send()
            .await
            .expect("Should get response");
        assert_eq!(response2.status(), 200);
        assert_eq!(response2.text().await.unwrap(), "Content from dir 2");

        server.stop().await.expect("Should stop");
    }

    #[tokio::test]
    async fn test_mock_server_content_type_detection() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        
        // Create files with different types
        File::create(temp_dir.path().join("page.html"))
            .expect("Create")
            .write_all(b"<html></html>")
            .expect("Write");
        File::create(temp_dir.path().join("data.json"))
            .expect("Create")
            .write_all(b"{}")
            .expect("Write");

        let mut server = MockServer::new();
        let config = MockServerConfig {
            port: 0,
            ..create_test_config()
        };
        let mappings = vec![DirectoryMapping {
            id: 1,
            virtual_path: "/".to_string(),
            local_path: temp_dir.path().to_string_lossy().to_string(),
            enabled: true,
        }];

        server.start(config, mappings).await.expect("Should start");
        let port = server.port;

        let client = reqwest::Client::new();

        // Check HTML content type
        let response = client
            .get(format!("http://127.0.0.1:{}/page.html", port))
            .send()
            .await
            .expect("Should get response");
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(content_type.contains("text/html"), "HTML should have text/html content type");

        // Check JSON content type
        let response = client
            .get(format!("http://127.0.0.1:{}/data.json", port))
            .send()
            .await
            .expect("Should get response");
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(content_type.contains("json"), "JSON should have application/json content type");

        server.stop().await.expect("Should stop");
    }
}
