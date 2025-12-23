//! Lua plugin executor for running plugin node handlers

use super::{PluginError, PluginResult, PluginRegistry};
use super::context::LuaContext;
use crate::engine::runtime::Runtime;
use mlua::{Lua, Table, Function, Value};
use std::sync::Arc;

/// Executes Lua plugin node handlers
pub struct LuaPluginExecutor {
    registry: Arc<PluginRegistry>,
}

impl LuaPluginExecutor {
    pub fn new(registry: Arc<PluginRegistry>) -> Self {
        Self { registry }
    }

    /// Execute a plugin node
    pub async fn execute(
        &self,
        node_type: &str,
        node_id: &str,
        node_data: serde_json::Value,
        runtime: Arc<Runtime>,
    ) -> PluginResult<()> {
        // Get plugin source path
        let source_path = self
            .registry
            .get_plugin_source(node_type)
            .await
            .ok_or_else(|| PluginError::HandlerNotFound(node_type.to_string()))?;

        // Load plugin source
        let source = std::fs::read_to_string(&source_path)?;

        // Create Lua state
        let lua = Lua::new();

        // Create context
        let context = LuaContext {
            runtime: runtime.clone(),
            node_id: node_id.to_string(),
            node_data: node_data.clone(),
        };

        // Create context userdata
        let ctx = lua
            .scope(|scope| {
                scope.create_userdata(context)
            })
            .map_err(|e| PluginError::LuaError(e.to_string()))?;

        // Load and execute the plugin
        let plugin_table: Table = lua
            .load(&source)
            .eval()
            .map_err(|e| PluginError::ExecutionError(format!("Failed to load plugin: {}", e)))?;

        // Find the node definition
        let nodes_table: Table = plugin_table
            .get("nodes")
            .map_err(|e| PluginError::ExecutionError(format!("No nodes table: {}", e)))?;

        let mut execute_fn: Option<Function> = None;

        for pair in nodes_table.pairs::<i32, Table>() {
            let (_, node_table) =
                pair.map_err(|e| PluginError::ExecutionError(e.to_string()))?;

            let type_name: String = node_table
                .get("type")
                .map_err(|e| PluginError::ExecutionError(e.to_string()))?;

            if type_name == node_type {
                execute_fn = Some(
                    node_table
                        .get("execute")
                        .map_err(|e| PluginError::ExecutionError(e.to_string()))?,
                );
                break;
            }
        }

        let execute_fn = execute_fn.ok_or_else(|| {
            PluginError::HandlerNotFound(format!("Node type '{}' not found in plugin", node_type))
        })?;

        // Call the execute function
        let result: Value = execute_fn
            .call(ctx)
            .map_err(|e| PluginError::ExecutionError(format!("Execution failed: {}", e)))?;

        // Check result
        match result {
            Value::Boolean(true) | Value::Nil => Ok(()),
            Value::Boolean(false) => Err(PluginError::ExecutionError(
                "Node execution returned false".to_string(),
            )),
            Value::String(s) => Err(PluginError::ExecutionError(
                s.to_str().map(|s| s.to_string()).unwrap_or_else(|_| "Unknown error".to_string()),
            )),
            _ => Ok(()),
        }
    }

    /// Check if a node type is handled by a plugin
    pub async fn can_handle(&self, node_type: &str) -> bool {
        self.registry.has_node_type(node_type).await
    }

    /// Get the registry
    pub fn registry(&self) -> Arc<PluginRegistry> {
        self.registry.clone()
    }
}

/// Helper to execute plugin in async context
pub async fn execute_plugin_node(
    executor: &LuaPluginExecutor,
    node_type: &str,
    node_id: &str,
    node_data: serde_json::Value,
    runtime: Arc<Runtime>,
) -> PluginResult<()> {
    // Run in blocking task to allow sync Lua execution
    let executor_clone = LuaPluginExecutor::new(executor.registry());
    let node_type = node_type.to_string();
    let node_id = node_id.to_string();

    tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async {
            executor_clone
                .execute(&node_type, &node_id, node_data, runtime)
                .await
        })
    })
    .await
    .map_err(|e| PluginError::ExecutionError(format!("Task join error: {}", e)))?
}
