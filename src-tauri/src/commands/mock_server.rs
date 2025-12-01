//! Mock Server Tauri commands
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   └─ src-tauri/src/commands/mod.rs
//! Dependencies:
//!   ├─ src-tauri/src/mock_server/mod.rs
//!   └─ src-tauri/src/database/connection.rs
//! Spec: docs/prd/mock-server.md

use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::RwLock;

use crate::database::Database;
use crate::mock_server::{
    self, AccessLogEntry, CreateMappingRequest, DirectoryMapping, FileInfo, MockServer,
    MockServerConfig, MockServerState, ServerStatus, UpdateConfigRequest, UpdateMappingRequest,
};

/// Mock Server state managed by Tauri
pub struct MockServerManager {
    server: Arc<RwLock<MockServer>>,
}

impl MockServerManager {
    pub fn new() -> Self {
        Self {
            server: Arc::new(RwLock::new(MockServer::new())),
        }
    }
}

impl Default for MockServerManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============== Server Control Commands ==============

/// Get current Mock Server state
#[tauri::command]
pub async fn get_mock_server_state(
    db: State<'_, Database>,
    manager: State<'_, MockServerManager>,
) -> Result<MockServerState, String> {
    let server = manager.server.read().await;
    let config = mock_server::get_config(db.pool())
        .await
        .map_err(|e| e.to_string())?;
    let mappings = mock_server::get_mappings(db.pool())
        .await
        .map_err(|e| e.to_string())?;

    let status = if server.is_running() {
        ServerStatus::Running
    } else {
        ServerStatus::Stopped
    };

    Ok(MockServerState {
        status,
        port: config.port,
        url: format!("http://localhost:{}", config.port),
        mappings_count: mappings.len(),
    })
}

/// Start the Mock Server
#[tauri::command]
pub async fn start_mock_server(
    app: AppHandle,
    db: State<'_, Database>,
    manager: State<'_, MockServerManager>,
) -> Result<MockServerState, String> {
    let config = mock_server::get_config(db.pool())
        .await
        .map_err(|e| e.to_string())?;
    let mappings = mock_server::get_enabled_mappings(db.pool())
        .await
        .map_err(|e| e.to_string())?;

    let mut server = manager.server.write().await;

    if server.is_running() {
        return Err("Server is already running".to_string());
    }

    // Subscribe to logs and forward to frontend
    let mut log_receiver = server.subscribe_logs();
    let app_handle = app.clone();
    tokio::spawn(async move {
        while let Ok(log) = log_receiver.recv().await {
            let _ = app_handle.emit("mock-server-log", log);
        }
    });

    server.start(config.clone(), mappings.clone()).await?;

    Ok(MockServerState {
        status: ServerStatus::Running,
        port: config.port,
        url: format!("http://localhost:{}", config.port),
        mappings_count: mappings.len(),
    })
}

/// Stop the Mock Server
#[tauri::command]
pub async fn stop_mock_server(
    db: State<'_, Database>,
    manager: State<'_, MockServerManager>,
) -> Result<MockServerState, String> {
    let config = mock_server::get_config(db.pool())
        .await
        .map_err(|e| e.to_string())?;
    let mappings = mock_server::get_mappings(db.pool())
        .await
        .map_err(|e| e.to_string())?;

    let mut server = manager.server.write().await;
    server.stop().await?;

    Ok(MockServerState {
        status: ServerStatus::Stopped,
        port: config.port,
        url: format!("http://localhost:{}", config.port),
        mappings_count: mappings.len(),
    })
}

// ============== Configuration Commands ==============

/// Get Mock Server configuration
#[tauri::command]
pub async fn get_mock_server_config(db: State<'_, Database>) -> Result<MockServerConfig, String> {
    mock_server::get_config(db.pool())
        .await
        .map_err(|e| e.to_string())
}

/// Update Mock Server configuration
#[tauri::command]
pub async fn update_mock_server_config(
    db: State<'_, Database>,
    request: UpdateConfigRequest,
) -> Result<MockServerConfig, String> {
    mock_server::update_config(db.pool(), request)
        .await
        .map_err(|e| e.to_string())
}

// ============== Mapping Commands ==============

/// Get all directory mappings
#[tauri::command]
pub async fn get_mock_server_mappings(
    db: State<'_, Database>,
) -> Result<Vec<DirectoryMapping>, String> {
    mock_server::get_mappings(db.pool())
        .await
        .map_err(|e| e.to_string())
}

/// Create a new directory mapping
#[tauri::command]
pub async fn create_mock_server_mapping(
    db: State<'_, Database>,
    request: CreateMappingRequest,
) -> Result<DirectoryMapping, String> {
    // Validate that the local path exists
    let path = std::path::Path::new(&request.local_path);
    if !path.exists() {
        return Err(format!("Directory does not exist: {}", request.local_path));
    }
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", request.local_path));
    }

    mock_server::create_mapping(db.pool(), request)
        .await
        .map_err(|e| e.to_string())
}

/// Update a directory mapping
#[tauri::command]
pub async fn update_mock_server_mapping(
    db: State<'_, Database>,
    request: UpdateMappingRequest,
) -> Result<DirectoryMapping, String> {
    // Validate local path if provided
    if let Some(ref local_path) = request.local_path {
        let path = std::path::Path::new(local_path);
        if !path.exists() {
            return Err(format!("Directory does not exist: {}", local_path));
        }
        if !path.is_dir() {
            return Err(format!("Path is not a directory: {}", local_path));
        }
    }

    mock_server::update_mapping(db.pool(), request)
        .await
        .map_err(|e| e.to_string())
}

/// Delete a directory mapping
#[tauri::command]
pub async fn delete_mock_server_mapping(db: State<'_, Database>, id: i64) -> Result<(), String> {
    mock_server::delete_mapping(db.pool(), id)
        .await
        .map_err(|e| e.to_string())
}

// ============== File Browser Commands ==============

/// List files in a directory
#[tauri::command]
pub async fn list_mock_server_directory(path: String) -> Result<Vec<FileInfo>, String> {
    mock_server::list_directory(&path)
}

/// Select a directory using native dialog
#[tauri::command]
pub async fn select_mock_server_directory(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let result = app.dialog().file().blocking_pick_folder();

    Ok(result.map(|p| p.to_string()))
}
