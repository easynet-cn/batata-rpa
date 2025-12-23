//! Database configuration module
//! Supports SQLite, MySQL, and PostgreSQL

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Database type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    #[default]
    Sqlite,
    Mysql,
    Postgres,
}

impl DatabaseType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseType::Sqlite => "sqlite",
            DatabaseType::Mysql => "mysql",
            DatabaseType::Postgres => "postgres",
        }
    }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database type: sqlite, mysql, or postgres
    #[serde(rename = "type")]
    pub db_type: DatabaseType,

    /// Host for MySQL/PostgreSQL (ignored for SQLite)
    #[serde(default = "default_host")]
    pub host: String,

    /// Port for MySQL/PostgreSQL (ignored for SQLite)
    #[serde(default = "default_port")]
    pub port: u16,

    /// Database name (or file path for SQLite)
    #[serde(default = "default_database")]
    pub database: String,

    /// Username for MySQL/PostgreSQL (ignored for SQLite)
    #[serde(default)]
    pub username: String,

    /// Password for MySQL/PostgreSQL (ignored for SQLite)
    #[serde(default)]
    pub password: String,

    /// Additional connection options
    #[serde(default)]
    pub options: String,

    /// Max connections in pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Min connections in pool
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    /// Connection timeout in seconds
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout: u64,
}

fn default_host() -> String {
    "localhost".to_string()
}

fn default_port() -> u16 {
    3306 // MySQL default, will be adjusted based on db_type
}

fn default_database() -> String {
    "batata_rpa".to_string()
}

fn default_max_connections() -> u32 {
    10
}

fn default_min_connections() -> u32 {
    1
}

fn default_connect_timeout() -> u64 {
    30
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            db_type: DatabaseType::Sqlite,
            host: default_host(),
            port: 0, // Will use file path for SQLite
            database: default_database(),
            username: String::new(),
            password: String::new(),
            options: String::new(),
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            connect_timeout: default_connect_timeout(),
        }
    }
}

impl DatabaseConfig {
    /// Create default SQLite configuration
    pub fn sqlite() -> Self {
        Self {
            db_type: DatabaseType::Sqlite,
            ..Default::default()
        }
    }

    /// Create MySQL configuration
    pub fn mysql(host: &str, port: u16, database: &str, username: &str, password: &str) -> Self {
        Self {
            db_type: DatabaseType::Mysql,
            host: host.to_string(),
            port,
            database: database.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            ..Default::default()
        }
    }

    /// Create PostgreSQL configuration
    pub fn postgres(host: &str, port: u16, database: &str, username: &str, password: &str) -> Self {
        Self {
            db_type: DatabaseType::Postgres,
            host: host.to_string(),
            port,
            database: database.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            ..Default::default()
        }
    }

    /// Get the connection URL for SeaORM
    pub fn get_connection_url(&self) -> String {
        match self.db_type {
            DatabaseType::Sqlite => {
                let db_path = self.get_sqlite_path();
                format!("sqlite:{}?mode=rwc", db_path.display())
            }
            DatabaseType::Mysql => {
                let mut url = format!(
                    "mysql://{}:{}@{}:{}/{}",
                    self.username, self.password, self.host, self.port, self.database
                );
                if !self.options.is_empty() {
                    url.push('?');
                    url.push_str(&self.options);
                }
                url
            }
            DatabaseType::Postgres => {
                let mut url = format!(
                    "postgres://{}:{}@{}:{}/{}",
                    self.username, self.password, self.host, self.port, self.database
                );
                if !self.options.is_empty() {
                    url.push('?');
                    url.push_str(&self.options);
                }
                url
            }
        }
    }

    /// Get SQLite database file path
    fn get_sqlite_path(&self) -> PathBuf {
        // If database is an absolute path, use it directly
        if PathBuf::from(&self.database).is_absolute() {
            return PathBuf::from(&self.database);
        }

        // Otherwise, use the app data directory
        if let Some(proj_dirs) = ProjectDirs::from("com", "batata", "rpa") {
            let data_dir = proj_dirs.data_dir();
            fs::create_dir_all(data_dir).ok();
            data_dir.join(format!("{}.db", self.database))
        } else {
            PathBuf::from(format!("{}.db", self.database))
        }
    }

    /// Get the default port for the database type
    pub fn default_port_for_type(db_type: &DatabaseType) -> u16 {
        match db_type {
            DatabaseType::Sqlite => 0,
            DatabaseType::Mysql => 3306,
            DatabaseType::Postgres => 5432,
        }
    }
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub database: DatabaseConfig,
}

impl AppConfig {
    /// Load configuration from file
    pub fn load() -> Self {
        let config_path = Self::get_config_path();

        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => {
                        log::info!("Loaded configuration from: {}", config_path.display());
                        return config;
                    }
                    Err(e) => {
                        log::warn!("Failed to parse config file: {}. Using defaults.", e);
                    }
                },
                Err(e) => {
                    log::warn!("Failed to read config file: {}. Using defaults.", e);
                }
            }
        }

        // Return default configuration
        let default = Self::default();

        // Save default config for user reference
        if let Err(e) = default.save() {
            log::warn!("Failed to save default config: {}", e);
        }

        default
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::get_config_path();

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&config_path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        log::info!("Configuration saved to: {}", config_path.display());
        Ok(())
    }

    /// Get the configuration file path
    fn get_config_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "batata", "rpa") {
            let config_dir = proj_dirs.config_dir();
            fs::create_dir_all(config_dir).ok();
            config_dir.join("config.toml")
        } else {
            PathBuf::from("batata-rpa.toml")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_url() {
        let config = DatabaseConfig::sqlite();
        let url = config.get_connection_url();
        assert!(url.starts_with("sqlite:"));
        assert!(url.contains("mode=rwc"));
    }

    #[test]
    fn test_mysql_url() {
        let config = DatabaseConfig::mysql("localhost", 3306, "test_db", "user", "pass");
        let url = config.get_connection_url();
        assert_eq!(url, "mysql://user:pass@localhost:3306/test_db");
    }

    #[test]
    fn test_postgres_url() {
        let config = DatabaseConfig::postgres("localhost", 5432, "test_db", "user", "pass");
        let url = config.get_connection_url();
        assert_eq!(url, "postgres://user:pass@localhost:5432/test_db");
    }
}
