//! Settings and database configuration commands

use crate::storage::{
    DatabaseConfig, DatabaseType,
    get_database_config, update_database_config, test_database_connection,
    init_database_with_config,
};
use serde::{Deserialize, Serialize};

/// Database configuration for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseConfigDto {
    /// Database type: "sqlite", "mysql", or "postgres"
    pub db_type: String,
    /// Host for MySQL/PostgreSQL
    pub host: String,
    /// Port for MySQL/PostgreSQL
    pub port: u16,
    /// Database name
    pub database: String,
    /// Username for MySQL/PostgreSQL
    pub username: String,
    /// Password for MySQL/PostgreSQL
    pub password: String,
    /// Max connections in pool
    pub max_connections: u32,
    /// Connection timeout in seconds
    pub connect_timeout: u64,
}

impl From<DatabaseConfig> for DatabaseConfigDto {
    fn from(config: DatabaseConfig) -> Self {
        Self {
            db_type: config.db_type.as_str().to_string(),
            host: config.host,
            port: config.port,
            database: config.database,
            username: config.username,
            password: config.password,
            max_connections: config.max_connections,
            connect_timeout: config.connect_timeout,
        }
    }
}

impl From<DatabaseConfigDto> for DatabaseConfig {
    fn from(dto: DatabaseConfigDto) -> Self {
        let db_type = match dto.db_type.to_lowercase().as_str() {
            "mysql" => DatabaseType::Mysql,
            "postgres" | "postgresql" => DatabaseType::Postgres,
            _ => DatabaseType::Sqlite,
        };

        DatabaseConfig {
            db_type,
            host: dto.host,
            port: dto.port,
            database: dto.database,
            username: dto.username,
            password: dto.password,
            options: String::new(),
            max_connections: dto.max_connections,
            min_connections: 1,
            connect_timeout: dto.connect_timeout,
        }
    }
}

/// Get current database configuration
#[tauri::command]
pub async fn get_db_config() -> Result<DatabaseConfigDto, String> {
    let config = get_database_config().await;
    Ok(config.into())
}

/// Update database configuration
#[tauri::command]
pub async fn set_db_config(config: DatabaseConfigDto) -> Result<(), String> {
    let db_config: DatabaseConfig = config.into();
    update_database_config(db_config).await
}

/// Test database connection
#[tauri::command]
pub async fn test_db_connection(config: DatabaseConfigDto) -> Result<String, String> {
    let db_config: DatabaseConfig = config.into();

    test_database_connection(&db_config)
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    Ok("Connection successful".to_string())
}

/// Apply database configuration and reconnect
#[tauri::command]
pub async fn apply_db_config(config: DatabaseConfigDto) -> Result<String, String> {
    let db_config: DatabaseConfig = config.into();

    // First test the connection
    test_database_connection(&db_config)
        .await
        .map_err(|e| format!("Connection test failed: {}", e))?;

    // If test succeeds, initialize with the new config
    init_database_with_config(db_config)
        .await
        .map_err(|e| format!("Failed to initialize database: {}", e))?;

    Ok("Database configuration applied successfully".to_string())
}

/// Get supported database types
#[tauri::command]
pub fn get_supported_db_types() -> Vec<DatabaseTypeInfo> {
    vec![
        DatabaseTypeInfo {
            value: "sqlite".to_string(),
            label: "SQLite".to_string(),
            description: "Local file-based database, no server required".to_string(),
            default_port: 0,
        },
        DatabaseTypeInfo {
            value: "mysql".to_string(),
            label: "MySQL".to_string(),
            description: "Popular open-source relational database".to_string(),
            default_port: 3306,
        },
        DatabaseTypeInfo {
            value: "postgres".to_string(),
            label: "PostgreSQL".to_string(),
            description: "Advanced open-source relational database".to_string(),
            default_port: 5432,
        },
    ]
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseTypeInfo {
    pub value: String,
    pub label: String,
    pub description: String,
    pub default_port: u16,
}
