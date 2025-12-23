//! Plugin loader for loading Lua plugin files

use super::{PluginDef, PluginMetadata, PluginNodeDef, PluginError, PluginResult, PluginRegistry};
use mlua::{Lua, Value, Table};
use std::path::Path;
use std::sync::Arc;

/// Loads plugins from Lua files
pub struct PluginLoader {
    registry: Arc<PluginRegistry>,
}

impl PluginLoader {
    pub fn new(registry: Arc<PluginRegistry>) -> Self {
        Self { registry }
    }

    /// Load a plugin from a Lua file
    pub async fn load_file(&self, path: &Path) -> PluginResult<PluginDef> {
        let source = std::fs::read_to_string(path)?;
        let source_path = path.to_string_lossy().to_string();

        let plugin = self.parse_plugin(&source, &source_path)?;

        // Register the plugin
        self.registry.register(plugin.clone()).await?;

        Ok(plugin)
    }

    /// Load all plugins from a directory
    pub async fn load_directory(&self, dir: &Path) -> PluginResult<Vec<PluginDef>> {
        let mut plugins = Vec::new();

        if !dir.exists() {
            log::warn!("Plugin directory does not exist: {:?}", dir);
            return Ok(plugins);
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "lua").unwrap_or(false) {
                match self.load_file(&path).await {
                    Ok(plugin) => {
                        log::info!("Loaded plugin: {} from {:?}", plugin.metadata.name, path);
                        plugins.push(plugin);
                    }
                    Err(e) => {
                        log::error!("Failed to load plugin from {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(plugins)
    }

    /// Parse a Lua plugin source into a PluginDef
    fn parse_plugin(&self, source: &str, source_path: &str) -> PluginResult<PluginDef> {
        let lua = Lua::new();

        // Execute the plugin source
        let result: Value = lua
            .load(source)
            .eval()
            .map_err(|e| PluginError::LoadError(format!("Failed to evaluate plugin: {}", e)))?;

        // Extract plugin table
        let plugin_table = match result {
            Value::Table(t) => t,
            _ => {
                return Err(PluginError::InvalidFormat(
                    "Plugin must return a table".to_string(),
                ))
            }
        };

        // Parse metadata
        let metadata = self.parse_metadata(&plugin_table)?;

        // Parse nodes
        let nodes = self.parse_nodes(&plugin_table)?;

        Ok(PluginDef {
            metadata,
            nodes,
            source_path: source_path.to_string(),
        })
    }

    /// Parse plugin metadata from table
    fn parse_metadata(&self, table: &Table) -> PluginResult<PluginMetadata> {
        let name: String = table
            .get("name")
            .map_err(|_| PluginError::InvalidFormat("Plugin must have a 'name' field".to_string()))?;

        let version: String = table.get("version").unwrap_or_else(|_| "1.0.0".to_string());

        let description: Option<String> = table.get("description").ok();
        let author: Option<String> = table.get("author").ok();

        Ok(PluginMetadata {
            name,
            version,
            description,
            author,
        })
    }

    /// Parse node definitions from table
    fn parse_nodes(&self, table: &Table) -> PluginResult<Vec<PluginNodeDef>> {
        let nodes_table: Table = table
            .get("nodes")
            .map_err(|_| PluginError::InvalidFormat("Plugin must have a 'nodes' field".to_string()))?;

        let mut nodes = Vec::new();

        for pair in nodes_table.pairs::<i32, Table>() {
            let (_, node_table) = pair.map_err(|e| PluginError::InvalidFormat(e.to_string()))?;

            let node_type: String = node_table.get("type").map_err(|_| {
                PluginError::InvalidFormat("Node must have a 'type' field".to_string())
            })?;

            let label: String = node_table
                .get("label")
                .unwrap_or_else(|_| node_type.clone());

            let category: String = node_table
                .get("category")
                .unwrap_or_else(|_| "action".to_string());

            let icon: Option<String> = node_table.get("icon").ok();
            let color: Option<String> = node_table.get("color").ok();

            // Verify that execute function exists
            if !node_table.contains_key("execute").unwrap_or(false) {
                return Err(PluginError::InvalidFormat(format!(
                    "Node '{}' must have an 'execute' function",
                    node_type
                )));
            }

            nodes.push(PluginNodeDef {
                node_type,
                label,
                icon,
                category,
                color,
            });
        }

        if nodes.is_empty() {
            return Err(PluginError::InvalidFormat(
                "Plugin must define at least one node".to_string(),
            ));
        }

        Ok(nodes)
    }

    /// Get the plugin registry
    pub fn registry(&self) -> Arc<PluginRegistry> {
        self.registry.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plugin() {
        let source = r#"
            return {
                name = "test_plugin",
                version = "1.0.0",
                description = "A test plugin",
                nodes = {
                    {
                        type = "testNode",
                        label = "测试节点",
                        category = "action",
                        execute = function(ctx) return true end
                    }
                }
            }
        "#;

        let registry = Arc::new(PluginRegistry::new());
        let loader = PluginLoader::new(registry);
        let plugin = loader.parse_plugin(source, "test.lua").unwrap();

        assert_eq!(plugin.metadata.name, "test_plugin");
        assert_eq!(plugin.metadata.version, "1.0.0");
        assert_eq!(plugin.nodes.len(), 1);
        assert_eq!(plugin.nodes[0].node_type, "testNode");
        assert_eq!(plugin.nodes[0].label, "测试节点");
    }
}
