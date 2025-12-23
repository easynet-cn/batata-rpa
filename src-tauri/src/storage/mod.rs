pub mod config;
pub mod entities;

use sea_orm::{Database, DatabaseConnection, DbErr, ConnectionTrait, Statement, DatabaseBackend};
use std::sync::Arc;
use tokio::sync::RwLock;

pub use config::{AppConfig, DatabaseConfig, DatabaseType};
pub use entities::*;

/// Global database connection
static DB_CONNECTION: std::sync::OnceLock<Arc<RwLock<Option<DatabaseConnection>>>> = std::sync::OnceLock::new();

/// Global configuration
static APP_CONFIG: std::sync::OnceLock<Arc<RwLock<AppConfig>>> = std::sync::OnceLock::new();

/// Initialize database connection and create tables
pub async fn init_database() -> Result<(), DbErr> {
    // Load configuration
    let config = AppConfig::load();
    let db_url = config.database.get_connection_url();

    log::info!("Initializing database with type: {:?}", config.database.db_type);
    log::info!("Connection URL: {}", mask_password(&db_url));

    let db = Database::connect(&db_url).await?;

    // Create tables based on database type
    create_tables(&db, &config.database.db_type).await?;

    // Store connection globally
    let connection = DB_CONNECTION.get_or_init(|| Arc::new(RwLock::new(None)));
    let mut conn = connection.write().await;
    *conn = Some(db);

    // Store config globally
    let app_config = APP_CONFIG.get_or_init(|| Arc::new(RwLock::new(AppConfig::default())));
    let mut cfg = app_config.write().await;
    *cfg = config;

    log::info!("Database initialized successfully");
    Ok(())
}

/// Initialize database with custom configuration
pub async fn init_database_with_config(config: DatabaseConfig) -> Result<(), DbErr> {
    let db_url = config.get_connection_url();

    log::info!("Initializing database with type: {:?}", config.db_type);
    log::info!("Connection URL: {}", mask_password(&db_url));

    let db = Database::connect(&db_url).await?;

    // Create tables based on database type
    create_tables(&db, &config.db_type).await?;

    // Store connection globally
    let connection = DB_CONNECTION.get_or_init(|| Arc::new(RwLock::new(None)));
    let mut conn = connection.write().await;
    *conn = Some(db);

    // Update config
    let app_config = APP_CONFIG.get_or_init(|| Arc::new(RwLock::new(AppConfig::default())));
    let mut cfg = app_config.write().await;
    cfg.database = config;

    // Save configuration
    if let Err(e) = cfg.save() {
        log::warn!("Failed to save configuration: {}", e);
    }

    log::info!("Database initialized successfully");
    Ok(())
}

/// Mask password in connection URL for logging
fn mask_password(url: &str) -> String {
    // Simple masking: replace password between : and @ with ***
    if let Some(at_pos) = url.find('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            let prefix = &url[..colon_pos + 1];
            let suffix = &url[at_pos..];
            return format!("{}***{}", prefix, suffix);
        }
    }
    url.to_string()
}

/// Get database connection
pub async fn get_connection() -> Result<DatabaseConnection, DbErr> {
    let connection = DB_CONNECTION.get_or_init(|| Arc::new(RwLock::new(None)));
    let conn = connection.read().await;

    match conn.as_ref() {
        Some(db) => Ok(db.clone()),
        None => Err(DbErr::Custom("Database not initialized".to_string())),
    }
}

/// Create database tables based on database type
async fn create_tables(db: &DatabaseConnection, db_type: &DatabaseType) -> Result<(), DbErr> {
    let backend = db.get_database_backend();

    // Create workflows table
    let workflows_sql = match db_type {
        DatabaseType::Sqlite => r#"
            CREATE TABLE IF NOT EXISTS workflows (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                nodes TEXT NOT NULL,
                edges TEXT NOT NULL,
                variables TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
        "#.to_string(),
        DatabaseType::Mysql => r#"
            CREATE TABLE IF NOT EXISTS workflows (
                id VARCHAR(36) PRIMARY KEY NOT NULL,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                nodes LONGTEXT NOT NULL,
                edges LONGTEXT NOT NULL,
                variables LONGTEXT NOT NULL,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
                INDEX idx_workflows_name (name),
                INDEX idx_workflows_updated (updated_at)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#.to_string(),
        DatabaseType::Postgres => r#"
            CREATE TABLE IF NOT EXISTS workflows (
                id VARCHAR(36) PRIMARY KEY NOT NULL,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                nodes JSONB NOT NULL,
                edges JSONB NOT NULL,
                variables JSONB NOT NULL,
                created_at TIMESTAMP NOT NULL,
                updated_at TIMESTAMP NOT NULL
            )
        "#.to_string(),
    };
    db.execute(Statement::from_string(backend, workflows_sql)).await?;

    // Create indexes for PostgreSQL (separate statements)
    if matches!(db_type, DatabaseType::Postgres) {
        let _ = db.execute(Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_workflows_name ON workflows(name)".to_string(),
        )).await;
        let _ = db.execute(Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_workflows_updated ON workflows(updated_at)".to_string(),
        )).await;
    }

    // Create element_libraries table
    let element_libraries_sql = match db_type {
        DatabaseType::Sqlite => r#"
            CREATE TABLE IF NOT EXISTS element_libraries (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                elements TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
        "#.to_string(),
        DatabaseType::Mysql => r#"
            CREATE TABLE IF NOT EXISTS element_libraries (
                id VARCHAR(36) PRIMARY KEY NOT NULL,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                elements LONGTEXT NOT NULL,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
                INDEX idx_element_libraries_name (name)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#.to_string(),
        DatabaseType::Postgres => r#"
            CREATE TABLE IF NOT EXISTS element_libraries (
                id VARCHAR(36) PRIMARY KEY NOT NULL,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                elements JSONB NOT NULL,
                created_at TIMESTAMP NOT NULL,
                updated_at TIMESTAMP NOT NULL
            )
        "#.to_string(),
    };
    db.execute(Statement::from_string(backend, element_libraries_sql)).await?;

    // Create indexes for PostgreSQL element_libraries
    if matches!(db_type, DatabaseType::Postgres) {
        let _ = db.execute(Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_element_libraries_name ON element_libraries(name)".to_string(),
        )).await;
    }

    // Create settings table
    let settings_sql = match db_type {
        DatabaseType::Sqlite => r#"
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY NOT NULL,
                value TEXT NOT NULL
            )
        "#.to_string(),
        DatabaseType::Mysql => r#"
            CREATE TABLE IF NOT EXISTS settings (
                `key` VARCHAR(255) PRIMARY KEY NOT NULL,
                value LONGTEXT NOT NULL
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#.to_string(),
        DatabaseType::Postgres => r#"
            CREATE TABLE IF NOT EXISTS settings (
                key VARCHAR(255) PRIMARY KEY NOT NULL,
                value JSONB NOT NULL
            )
        "#.to_string(),
    };
    db.execute(Statement::from_string(backend, settings_sql)).await?;

    // Create execution_logs table for tracking workflow executions
    let execution_logs_sql = match db_type {
        DatabaseType::Sqlite => r#"
            CREATE TABLE IF NOT EXISTS execution_logs (
                id TEXT PRIMARY KEY NOT NULL,
                workflow_id TEXT NOT NULL,
                status TEXT NOT NULL,
                started_at TEXT NOT NULL,
                finished_at TEXT,
                error_message TEXT,
                log_data TEXT
            )
        "#.to_string(),
        DatabaseType::Mysql => r#"
            CREATE TABLE IF NOT EXISTS execution_logs (
                id VARCHAR(36) PRIMARY KEY NOT NULL,
                workflow_id VARCHAR(36) NOT NULL,
                status VARCHAR(50) NOT NULL,
                started_at DATETIME NOT NULL,
                finished_at DATETIME,
                error_message TEXT,
                log_data LONGTEXT,
                INDEX idx_execution_logs_workflow (workflow_id),
                INDEX idx_execution_logs_status (status),
                INDEX idx_execution_logs_started (started_at)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#.to_string(),
        DatabaseType::Postgres => r#"
            CREATE TABLE IF NOT EXISTS execution_logs (
                id VARCHAR(36) PRIMARY KEY NOT NULL,
                workflow_id VARCHAR(36) NOT NULL,
                status VARCHAR(50) NOT NULL,
                started_at TIMESTAMP NOT NULL,
                finished_at TIMESTAMP,
                error_message TEXT,
                log_data JSONB
            )
        "#.to_string(),
    };
    db.execute(Statement::from_string(backend, execution_logs_sql)).await?;

    // Create indexes for PostgreSQL execution_logs
    if matches!(db_type, DatabaseType::Postgres) {
        let _ = db.execute(Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_execution_logs_workflow ON execution_logs(workflow_id)".to_string(),
        )).await;
        let _ = db.execute(Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_execution_logs_status ON execution_logs(status)".to_string(),
        )).await;
        let _ = db.execute(Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_execution_logs_started ON execution_logs(started_at)".to_string(),
        )).await;
    }

    log::info!("Database tables created successfully");
    Ok(())
}

/// Get current database configuration
pub async fn get_database_config() -> DatabaseConfig {
    let app_config = APP_CONFIG.get_or_init(|| Arc::new(RwLock::new(AppConfig::default())));
    let cfg = app_config.read().await;
    cfg.database.clone()
}

/// Update database configuration (requires restart)
pub async fn update_database_config(config: DatabaseConfig) -> Result<(), String> {
    let app_config = APP_CONFIG.get_or_init(|| Arc::new(RwLock::new(AppConfig::default())));
    let mut cfg = app_config.write().await;
    cfg.database = config;
    cfg.save()
}

/// Test database connection without changing current connection
pub async fn test_database_connection(config: &DatabaseConfig) -> Result<(), DbErr> {
    let db_url = config.get_connection_url();
    log::info!("Testing database connection: {}", mask_password(&db_url));

    let db = Database::connect(&db_url).await?;

    // Try a simple query to verify connection
    let backend = db.get_database_backend();
    let test_query = match backend {
        DatabaseBackend::Sqlite => "SELECT 1",
        DatabaseBackend::MySql => "SELECT 1",
        DatabaseBackend::Postgres => "SELECT 1",
    };

    db.execute(Statement::from_string(backend, test_query.to_string())).await?;

    log::info!("Database connection test successful");
    Ok(())
}

/// Close database connection
pub async fn close_database() {
    let connection = DB_CONNECTION.get_or_init(|| Arc::new(RwLock::new(None)));
    let mut conn = connection.write().await;
    *conn = None;
    log::info!("Database connection closed");
}
