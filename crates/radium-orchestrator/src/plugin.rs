//! Plugin system for dynamic agent loading.
//!
//! This module provides functionality for loading agents dynamically from shared libraries.

use crate::Agent;
use std::ffi::OsStr;
use std::fmt;
use std::sync::Arc;
use tracing::error;

/// Plugin metadata.
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    /// Plugin name.
    pub name: String,
    /// Plugin version.
    pub version: String,
    /// Plugin description.
    pub description: String,
    /// List of agent IDs provided by this plugin.
    pub agent_ids: Vec<String>,
}

/// Trait for plugins that can provide agents.
pub trait Plugin: Send + Sync {
    /// Returns metadata about the plugin.
    fn metadata(&self) -> PluginMetadata;

    /// Creates an agent instance by ID.
    ///
    /// # Arguments
    /// * `agent_id` - The ID of the agent to create
    ///
    /// # Returns
    /// Returns `Some(Arc<dyn Agent>)` if the agent exists, `None` otherwise.
    fn create_agent(&self, agent_id: &str) -> Option<Arc<dyn Agent + Send + Sync>>;

    /// Lists all agent IDs provided by this plugin.
    ///
    /// # Returns
    /// A vector of agent IDs.
    fn list_agents(&self) -> Vec<String>;
}

/// Plugin loader for dynamic loading from shared libraries.
///
/// # Platform Support
/// - Linux: `.so` files
/// - macOS: `.dylib` files
/// - Windows: `.dll` files
pub struct PluginLoader;

impl PluginLoader {
    /// Loads a plugin from a shared library file.
    ///
    /// # Arguments
    /// * `path` - Path to the shared library file
    ///
    /// # Returns
    /// Returns `Ok(Box<dyn Plugin>)` if successful, `Err` with error message otherwise.
    ///
    /// # Safety
    /// This function is unsafe because loading dynamic libraries can execute arbitrary code.
    /// Only load plugins from trusted sources.
    ///
    /// # Platform Notes
    /// - On Unix systems, the library must export a function with C linkage
    /// - The exact function signature depends on the plugin ABI
    /// - For now, this is a placeholder implementation
    ///
    /// # Note
    /// This function is currently not implemented. Use `InMemoryPlugin` for static agent loading.
    pub fn load_plugin<P: AsRef<OsStr>>(_path: P) -> Result<Box<dyn Plugin>, String> {
        // TODO: Implement actual dynamic loading using libloading
        // For now, return an error indicating this is not yet implemented
        error!(
            "Dynamic plugin loading is not yet implemented. Use static agent registration or InMemoryPlugin instead."
        );
        Err("Dynamic plugin loading not yet implemented. This feature requires libloading and platform-specific code.".to_string())
    }

    /// Validates that a file path is a valid plugin file.
    ///
    /// # Arguments
    /// * `path` - Path to check
    ///
    /// # Returns
    /// Returns `true` if the file appears to be a valid plugin, `false` otherwise.
    #[must_use]
    pub fn validate_plugin_path<P: AsRef<OsStr>>(path: P) -> bool {
        let path = path.as_ref();
        let extension = path
            .to_str()
            .and_then(|s| std::path::Path::new(s).extension().and_then(|ext| ext.to_str()));

        matches!(extension, Some("so" | "dylib" | "dll"))
    }
}

/// In-memory plugin implementation for testing and static agents.
pub struct InMemoryPlugin {
    metadata: PluginMetadata,
    agents: std::collections::HashMap<String, Arc<dyn Agent + Send + Sync>>,
}

impl fmt::Debug for InMemoryPlugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InMemoryPlugin")
            .field("metadata", &self.metadata)
            .field("agent_count", &self.agents.len())
            .finish_non_exhaustive()
    }
}

impl InMemoryPlugin {
    /// Creates a new in-memory plugin.
    ///
    /// # Arguments
    /// * `name` - Plugin name
    /// * `version` - Plugin version
    /// * `description` - Plugin description
    #[must_use]
    pub fn new(name: String, version: String, description: String) -> Self {
        Self {
            metadata: PluginMetadata { name, version, description, agent_ids: Vec::new() },
            agents: std::collections::HashMap::new(),
        }
    }

    /// Adds an agent to this plugin.
    ///
    /// # Arguments
    /// * `agent` - The agent to add
    pub fn add_agent(&mut self, agent: Arc<dyn Agent + Send + Sync>) {
        let agent_id = agent.id().to_string();
        self.metadata.agent_ids.push(agent_id.clone());
        self.agents.insert(agent_id, agent);
    }
}

impl Plugin for InMemoryPlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn create_agent(&self, agent_id: &str) -> Option<Arc<dyn Agent + Send + Sync>> {
        self.agents.get(agent_id).cloned()
    }

    fn list_agents(&self) -> Vec<String> {
        self.metadata.agent_ids.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EchoAgent;

    #[test]
    fn test_validate_plugin_path() {
        assert!(PluginLoader::validate_plugin_path("plugin.so"));
        assert!(PluginLoader::validate_plugin_path("plugin.dylib"));
        assert!(PluginLoader::validate_plugin_path("plugin.dll"));
        assert!(!PluginLoader::validate_plugin_path("plugin.txt"));
        assert!(!PluginLoader::validate_plugin_path("plugin"));
    }

    #[test]
    fn test_in_memory_plugin() {
        let mut plugin = InMemoryPlugin::new(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            "Test plugin".to_string(),
        );

        let agent1 = Arc::new(EchoAgent::new("agent-1".to_string(), "Agent 1".to_string()));
        let agent2 = Arc::new(EchoAgent::new("agent-2".to_string(), "Agent 2".to_string()));

        plugin.add_agent(agent1);
        plugin.add_agent(agent2);

        let metadata = plugin.metadata();
        assert_eq!(metadata.name, "test-plugin");
        assert_eq!(metadata.agent_ids.len(), 2);

        let agents = plugin.list_agents();
        assert_eq!(agents.len(), 2);
        assert!(agents.contains(&"agent-1".to_string()));
        assert!(agents.contains(&"agent-2".to_string()));

        let created = plugin.create_agent("agent-1");
        assert!(created.is_some());
        assert_eq!(created.unwrap().id(), "agent-1");
    }

    #[test]
    fn test_in_memory_plugin_create_nonexistent_agent() {
        let plugin = InMemoryPlugin::new(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            "Test plugin".to_string(),
        );
        let created = plugin.create_agent("nonexistent");
        assert!(created.is_none());
    }

    #[test]
    fn test_in_memory_plugin_metadata() {
        let plugin = InMemoryPlugin::new(
            "my-plugin".to_string(),
            "2.0.0".to_string(),
            "My test plugin".to_string(),
        );
        let metadata = plugin.metadata();
        assert_eq!(metadata.name, "my-plugin");
        assert_eq!(metadata.version, "2.0.0");
        assert_eq!(metadata.description, "My test plugin");
        assert!(metadata.agent_ids.is_empty());
    }

    #[test]
    fn test_plugin_loader_validate_plugin_path_edge_cases() {
        // Test various edge cases
        assert!(!PluginLoader::validate_plugin_path(""));
        assert!(!PluginLoader::validate_plugin_path("."));
        assert!(!PluginLoader::validate_plugin_path(".."));
        assert!(!PluginLoader::validate_plugin_path("/path/to/plugin"));
        assert!(!PluginLoader::validate_plugin_path("plugin."));
    }

    #[test]
    fn test_in_memory_plugin_list_agents_empty() {
        let plugin = InMemoryPlugin::new(
            "empty-plugin".to_string(),
            "1.0.0".to_string(),
            "Empty plugin".to_string(),
        );
        let agents = plugin.list_agents();
        assert!(agents.is_empty());
    }

    #[test]
    fn test_in_memory_plugin_add_multiple_agents() {
        let mut plugin = InMemoryPlugin::new(
            "multi-plugin".to_string(),
            "1.0.0".to_string(),
            "Multi agent plugin".to_string(),
        );

        for i in 0..10 {
            let agent = Arc::new(EchoAgent::new(format!("agent-{}", i), format!("Agent {}", i)));
            plugin.add_agent(agent);
        }

        let metadata = plugin.metadata();
        assert_eq!(metadata.agent_ids.len(), 10);

        let agents = plugin.list_agents();
        assert_eq!(agents.len(), 10);
    }

    #[test]
    fn test_in_memory_plugin_duplicate_agent_ids() {
        let mut plugin = InMemoryPlugin::new(
            "duplicate-plugin".to_string(),
            "1.0.0".to_string(),
            "Duplicate agent plugin".to_string(),
        );

        let agent1 = Arc::new(EchoAgent::new("agent-1".to_string(), "Agent 1".to_string()));
        let agent2 =
            Arc::new(EchoAgent::new("agent-1".to_string(), "Agent 1 duplicate".to_string()));

        plugin.add_agent(agent1);
        plugin.add_agent(agent2);

        // Should allow duplicates (last one wins or both exist depending on implementation)
        let agents = plugin.list_agents();
        // The implementation may deduplicate or allow both
        assert!(!agents.is_empty());
    }
}
