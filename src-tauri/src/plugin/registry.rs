//! Plugin registry for managing loaded plugins

use super::{PluginDef, PluginNodeDef, PluginError, PluginResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Handler function signature for plugin nodes
pub type PluginNodeHandler = Arc<dyn Fn(&str) -> PluginResult<()> + Send + Sync>;

/// Registry for managing plugins and their node handlers
pub struct PluginRegistry {
    /// Loaded plugin definitions
    plugins: RwLock<HashMap<String, PluginDef>>,

    /// Node type to plugin name mapping
    node_to_plugin: RwLock<HashMap<String, String>>,

    /// Node definitions from all plugins
    node_defs: RwLock<HashMap<String, PluginNodeDef>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            node_to_plugin: RwLock::new(HashMap::new()),
            node_defs: RwLock::new(HashMap::new()),
        }
    }

    /// Register a plugin
    pub async fn register(&self, plugin: PluginDef) -> PluginResult<()> {
        let plugin_name = plugin.metadata.name.clone();

        // Register all node types
        for node in &plugin.nodes {
            self.node_to_plugin
                .write()
                .await
                .insert(node.node_type.clone(), plugin_name.clone());

            self.node_defs
                .write()
                .await
                .insert(node.node_type.clone(), node.clone());
        }

        self.plugins
            .write()
            .await
            .insert(plugin_name.clone(), plugin);

        log::info!("Registered plugin: {}", plugin_name);
        Ok(())
    }

    /// Unregister a plugin
    pub async fn unregister(&self, plugin_name: &str) -> PluginResult<()> {
        if let Some(plugin) = self.plugins.write().await.remove(plugin_name) {
            // Remove all node types from this plugin
            for node in &plugin.nodes {
                self.node_to_plugin.write().await.remove(&node.node_type);
                self.node_defs.write().await.remove(&node.node_type);
            }
            log::info!("Unregistered plugin: {}", plugin_name);
        }
        Ok(())
    }

    /// Check if a node type is provided by a plugin
    pub async fn has_node_type(&self, node_type: &str) -> bool {
        self.node_to_plugin.read().await.contains_key(node_type)
    }

    /// Get the plugin name that provides a node type
    pub async fn get_plugin_for_node(&self, node_type: &str) -> Option<String> {
        self.node_to_plugin.read().await.get(node_type).cloned()
    }

    /// Get plugin definition
    pub async fn get_plugin(&self, name: &str) -> Option<PluginDef> {
        self.plugins.read().await.get(name).cloned()
    }

    /// Get node definition
    pub async fn get_node_def(&self, node_type: &str) -> Option<PluginNodeDef> {
        self.node_defs.read().await.get(node_type).cloned()
    }

    /// Get all registered plugins
    pub async fn list_plugins(&self) -> Vec<PluginDef> {
        self.plugins.read().await.values().cloned().collect()
    }

    /// Get all registered node definitions
    pub async fn list_node_defs(&self) -> Vec<PluginNodeDef> {
        self.node_defs.read().await.values().cloned().collect()
    }

    /// Get plugin source path for a node type
    pub async fn get_plugin_source(&self, node_type: &str) -> Option<String> {
        let plugin_name = self.get_plugin_for_node(node_type).await?;
        let plugin = self.get_plugin(&plugin_name).await?;
        Some(plugin.source_path)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
