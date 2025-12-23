//! Plugin system for extending RPA node handlers
//!
//! This module provides a Lua-based plugin system that allows users to create
//! custom node handlers without modifying the core application.
//!
//! # Example Plugin (Lua)
//!
//! ```lua
//! -- plugins/my_plugin.lua
//! return {
//!     name = "my_plugin",
//!     version = "1.0.0",
//!     nodes = {
//!         {
//!             type = "myCustomNode",
//!             label = "我的自定义节点",
//!             category = "action",
//!             execute = function(context)
//!                 local value = context:get_data("value")
//!                 context:log("info", "Processing: " .. tostring(value))
//!                 context:set_variable("result", value * 2)
//!                 return true
//!             end
//!         }
//!     }
//! }
//! ```

mod loader;
mod registry;
mod lua_executor;
mod context;

pub use loader::PluginLoader;
pub use registry::{PluginRegistry, PluginNodeHandler};
pub use lua_executor::{LuaPluginExecutor, execute_plugin_node};
pub use context::PluginContext;

use serde::{Deserialize, Serialize};

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
}

/// Plugin node definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginNodeDef {
    pub node_type: String,
    pub label: String,
    pub icon: Option<String>,
    pub category: String,
    pub color: Option<String>,
}

/// Plugin definition loaded from Lua
#[derive(Debug, Clone)]
pub struct PluginDef {
    pub metadata: PluginMetadata,
    pub nodes: Vec<PluginNodeDef>,
    pub source_path: String,
}

/// Result type for plugin operations
pub type PluginResult<T> = Result<T, PluginError>;

/// Plugin error types
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Failed to load plugin: {0}")]
    LoadError(String),

    #[error("Plugin execution error: {0}")]
    ExecutionError(String),

    #[error("Invalid plugin format: {0}")]
    InvalidFormat(String),

    #[error("Node handler not found: {0}")]
    HandlerNotFound(String),

    #[error("Lua error: {0}")]
    LuaError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl From<mlua::Error> for PluginError {
    fn from(err: mlua::Error) -> Self {
        PluginError::LuaError(err.to_string())
    }
}
