//! Mock Server repository for database operations
//!
//! DEPENDENCY MAP:
//! Parents (Files that import this module):
//!   ├─ src-tauri/src/mock_server/mod.rs
//!   └─ src-tauri/src/commands/mock_server.rs
//! Dependencies:
//!   ├─ src-tauri/src/mock_server/types.rs
//!   └─ src-tauri/src/database/connection.rs

use sqlx::{Pool, Row, Sqlite};

use super::types::{
    CorsMode, CreateMappingRequest, DirectoryMapping, MockServerConfig, UpdateConfigRequest,
    UpdateMappingRequest,
};
use crate::database::{DatabaseError, DbResult};

/// Get Mock Server configuration
pub async fn get_config(pool: &Pool<Sqlite>) -> DbResult<MockServerConfig> {
    let row = sqlx::query(
        r#"
        SELECT id, port, cors_mode, cors_origins, cors_methods, cors_headers, 
               cors_max_age, show_directory_listing
        FROM mock_server_config
        WHERE id = 1
        "#,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| DatabaseError::Query(e.to_string()))?;

    match row {
        Some(row) => {
            let cors_origins: Option<String> = row.get("cors_origins");
            let cors_methods: Option<String> = row.get("cors_methods");
            let cors_headers: Option<String> = row.get("cors_headers");

            Ok(MockServerConfig {
                id: row.get("id"),
                port: row.get::<i64, _>("port") as u16,
                cors_mode: CorsMode::from(row.get::<String, _>("cors_mode")),
                cors_origins: cors_origins
                    .and_then(|s| serde_json::from_str(&s).ok()),
                cors_methods: cors_methods
                    .and_then(|s| serde_json::from_str(&s).ok()),
                cors_headers: cors_headers
                    .and_then(|s| serde_json::from_str(&s).ok()),
                cors_max_age: row.get("cors_max_age"),
                show_directory_listing: row.get::<i64, _>("show_directory_listing") != 0,
            })
        }
        None => {
            // Insert default config and return it
            let config = MockServerConfig::default();
            sqlx::query(
                r#"
                INSERT INTO mock_server_config (id, port, cors_mode, cors_max_age, show_directory_listing)
                VALUES (1, ?, ?, ?, ?)
                "#,
            )
            .bind(config.port as i64)
            .bind(String::from(config.cors_mode.clone()))
            .bind(config.cors_max_age)
            .bind(config.show_directory_listing as i64)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

            Ok(config)
        }
    }
}

/// Update Mock Server configuration
pub async fn update_config(
    pool: &Pool<Sqlite>,
    request: UpdateConfigRequest,
) -> DbResult<MockServerConfig> {
    let current = get_config(pool).await?;

    let port = request.port.unwrap_or(current.port);
    let cors_mode = request.cors_mode.unwrap_or(current.cors_mode);
    let cors_origins = request.cors_origins.or(current.cors_origins);
    let cors_methods = request.cors_methods.or(current.cors_methods);
    let cors_headers = request.cors_headers.or(current.cors_headers);
    let cors_max_age = request.cors_max_age.unwrap_or(current.cors_max_age);
    let show_directory_listing = request
        .show_directory_listing
        .unwrap_or(current.show_directory_listing);

    let cors_origins_json = cors_origins
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_default());
    let cors_methods_json = cors_methods
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_default());
    let cors_headers_json = cors_headers
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_default());

    sqlx::query(
        r#"
        UPDATE mock_server_config
        SET port = ?, cors_mode = ?, cors_origins = ?, cors_methods = ?, 
            cors_headers = ?, cors_max_age = ?, show_directory_listing = ?,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = 1
        "#,
    )
    .bind(port as i64)
    .bind(String::from(cors_mode.clone()))
    .bind(cors_origins_json)
    .bind(cors_methods_json)
    .bind(cors_headers_json)
    .bind(cors_max_age)
    .bind(show_directory_listing as i64)
    .execute(pool)
    .await
    .map_err(|e| DatabaseError::Query(e.to_string()))?;

    Ok(MockServerConfig {
        id: 1,
        port,
        cors_mode,
        cors_origins,
        cors_methods,
        cors_headers,
        cors_max_age,
        show_directory_listing,
    })
}

/// Get all directory mappings
pub async fn get_mappings(pool: &Pool<Sqlite>) -> DbResult<Vec<DirectoryMapping>> {
    let rows = sqlx::query(
        r#"
        SELECT id, virtual_path, local_path, enabled
        FROM mock_server_mappings
        ORDER BY virtual_path
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| DatabaseError::Query(e.to_string()))?;

    let mappings = rows
        .into_iter()
        .map(|row| DirectoryMapping {
            id: row.get("id"),
            virtual_path: row.get("virtual_path"),
            local_path: row.get("local_path"),
            enabled: row.get::<i64, _>("enabled") != 0,
        })
        .collect();

    Ok(mappings)
}

/// Get enabled directory mappings
pub async fn get_enabled_mappings(pool: &Pool<Sqlite>) -> DbResult<Vec<DirectoryMapping>> {
    let rows = sqlx::query(
        r#"
        SELECT id, virtual_path, local_path, enabled
        FROM mock_server_mappings
        WHERE enabled = 1
        ORDER BY virtual_path
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| DatabaseError::Query(e.to_string()))?;

    let mappings = rows
        .into_iter()
        .map(|row| DirectoryMapping {
            id: row.get("id"),
            virtual_path: row.get("virtual_path"),
            local_path: row.get("local_path"),
            enabled: true,
        })
        .collect();

    Ok(mappings)
}

/// Create a new directory mapping
pub async fn create_mapping(
    pool: &Pool<Sqlite>,
    request: CreateMappingRequest,
) -> DbResult<DirectoryMapping> {
    // Ensure virtual_path starts with /
    let virtual_path = if request.virtual_path.starts_with('/') {
        request.virtual_path
    } else {
        format!("/{}", request.virtual_path)
    };

    let result = sqlx::query(
        r#"
        INSERT INTO mock_server_mappings (virtual_path, local_path, enabled)
        VALUES (?, ?, 1)
        "#,
    )
    .bind(&virtual_path)
    .bind(&request.local_path)
    .execute(pool)
    .await
    .map_err(|e| DatabaseError::Query(e.to_string()))?;

    Ok(DirectoryMapping {
        id: result.last_insert_rowid(),
        virtual_path,
        local_path: request.local_path,
        enabled: true,
    })
}

/// Update a directory mapping
pub async fn update_mapping(
    pool: &Pool<Sqlite>,
    request: UpdateMappingRequest,
) -> DbResult<DirectoryMapping> {
    // Get current mapping
    let current = sqlx::query(
        r#"
        SELECT id, virtual_path, local_path, enabled
        FROM mock_server_mappings
        WHERE id = ?
        "#,
    )
    .bind(request.id)
    .fetch_optional(pool)
    .await
    .map_err(|e| DatabaseError::Query(e.to_string()))?
    .ok_or_else(|| DatabaseError::NotFound(format!("Mapping {} not found", request.id)))?;

    let virtual_path = request
        .virtual_path
        .map(|p| {
            if p.starts_with('/') {
                p
            } else {
                format!("/{}", p)
            }
        })
        .unwrap_or_else(|| current.get("virtual_path"));
    let local_path = request
        .local_path
        .unwrap_or_else(|| current.get("local_path"));
    let enabled = request
        .enabled
        .unwrap_or_else(|| current.get::<i64, _>("enabled") != 0);

    sqlx::query(
        r#"
        UPDATE mock_server_mappings
        SET virtual_path = ?, local_path = ?, enabled = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(&virtual_path)
    .bind(&local_path)
    .bind(enabled as i64)
    .bind(request.id)
    .execute(pool)
    .await
    .map_err(|e| DatabaseError::Query(e.to_string()))?;

    Ok(DirectoryMapping {
        id: request.id,
        virtual_path,
        local_path,
        enabled,
    })
}

/// Delete a directory mapping
pub async fn delete_mapping(pool: &Pool<Sqlite>, id: i64) -> DbResult<()> {
    let result = sqlx::query("DELETE FROM mock_server_mappings WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(DatabaseError::NotFound(format!("Mapping {} not found", id)));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn create_test_pool() -> Pool<Sqlite> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create test pool");

        // Run migrations
        crate::database::migrations::run_migrations(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    #[tokio::test]
    async fn test_get_default_config() {
        let pool = create_test_pool().await;
        let config = get_config(&pool).await.expect("Should get config");

        assert_eq!(config.port, 9876);
        assert_eq!(config.cors_mode, CorsMode::Simple);
    }

    #[tokio::test]
    async fn test_update_config() {
        let pool = create_test_pool().await;

        let update = UpdateConfigRequest {
            port: Some(8080),
            cors_mode: Some(CorsMode::Advanced),
            cors_origins: Some(vec!["http://localhost:3000".to_string()]),
            cors_methods: None,
            cors_headers: None,
            cors_max_age: None,
            show_directory_listing: Some(true),
        };

        let config = update_config(&pool, update)
            .await
            .expect("Should update config");

        assert_eq!(config.port, 8080);
        assert_eq!(config.cors_mode, CorsMode::Advanced);
        assert!(config.show_directory_listing);
    }

    #[tokio::test]
    async fn test_mapping_crud() {
        let pool = create_test_pool().await;

        // Create
        let mapping = create_mapping(
            &pool,
            CreateMappingRequest {
                virtual_path: "/images".to_string(),
                local_path: "/tmp/images".to_string(),
            },
        )
        .await
        .expect("Should create mapping");

        assert_eq!(mapping.virtual_path, "/images");
        assert!(mapping.enabled);

        // Read
        let mappings = get_mappings(&pool).await.expect("Should get mappings");
        assert_eq!(mappings.len(), 1);

        // Update
        let updated = update_mapping(
            &pool,
            UpdateMappingRequest {
                id: mapping.id,
                virtual_path: Some("/pics".to_string()),
                local_path: None,
                enabled: Some(false),
            },
        )
        .await
        .expect("Should update mapping");

        assert_eq!(updated.virtual_path, "/pics");
        assert!(!updated.enabled);

        // Delete
        delete_mapping(&pool, mapping.id)
            .await
            .expect("Should delete mapping");

        let mappings = get_mappings(&pool).await.expect("Should get mappings");
        assert!(mappings.is_empty());
    }
}
