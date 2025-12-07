//! Extension conflict detection.
//!
//! Detects conflicts between extension components and existing components
//! to prevent installation of conflicting extensions.

use crate::agents::discovery::AgentDiscovery;
use crate::commands::CommandRegistry;
use crate::extensions::manifest::ExtensionManifest;
use crate::workflow::template_discovery::TemplateDiscovery;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use thiserror::Error;

/// Conflict detection errors.
#[derive(Debug, Error)]
pub enum ConflictError {
    /// Agent ID conflict.
    #[error("agent ID conflict: {0}")]
    AgentConflict(String),

    /// Template name conflict.
    #[error("template name conflict: {0}")]
    TemplateConflict(String),

    /// Command name conflict.
    #[error("command name conflict: {0}")]
    CommandConflict(String),

    /// Dependency cycle detected.
    #[error("dependency cycle detected: {0}")]
    DependencyCycle(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for conflict detection.
pub type Result<T> = std::result::Result<T, ConflictError>;

/// Extension conflict detector.
pub struct ConflictDetector;

impl ConflictDetector {
    /// Checks for conflicts before installing an extension.
    ///
    /// # Arguments
    /// * `manifest` - Extension manifest to check
    /// * `package_path` - Path to extension package
    ///
    /// # Returns
    /// Ok(()) if no conflicts, error otherwise
    pub fn check_conflicts(manifest: &ExtensionManifest, package_path: &Path) -> Result<()> {
        // Check agent conflicts
        Self::check_agent_conflicts(manifest, package_path)?;

        // Check template conflicts
        Self::check_template_conflicts(manifest, package_path)?;

        // Check command conflicts
        Self::check_command_conflicts(manifest, package_path)?;

        Ok(())
    }

    /// Checks for agent ID conflicts.
    fn check_agent_conflicts(manifest: &ExtensionManifest, package_path: &Path) -> Result<()> {
        // Discover existing agents
        let discovery = AgentDiscovery::new();
        let existing_agents = discovery.discover_all()
            .map_err(|e| ConflictError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to discover agents: {}", e),
            )))?;

        // Check if extension has agents directory
        let agents_dir = package_path.join("agents");
        if !agents_dir.exists() {
            return Ok(()); // No agents in extension
        }

        // Scan extension agents directory for TOML files
        if let Ok(entries) = std::fs::read_dir(&agents_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    // Try to load agent config to get ID
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(config) = toml::from_str::<toml::Value>(&content) {
                            if let Some(agent) = config.get("agent") {
                                if let Some(id) = agent.get("id").and_then(|v| v.as_str()) {
                                    if existing_agents.contains_key(id) {
                                        return Err(ConflictError::AgentConflict(format!(
                                            "Agent ID '{}' already exists",
                                            id
                                        )));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Checks for template name conflicts.
    fn check_template_conflicts(manifest: &ExtensionManifest, package_path: &Path) -> Result<()> {
        // Discover existing templates
        let discovery = TemplateDiscovery::new();
        let existing_templates = discovery.discover_all()
            .map_err(|e| ConflictError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to discover templates: {}", e),
            )))?;

        // Check if extension has templates directory
        let templates_dir = package_path.join("templates");
        if !templates_dir.exists() {
            return Ok(()); // No templates in extension
        }

        // Scan extension templates directory for JSON files
        if let Ok(entries) = std::fs::read_dir(&templates_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                    // Try to load template to get name
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(template) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(name) = template.get("name").and_then(|v| v.as_str()) {
                                if existing_templates.contains_key(name) {
                                    return Err(ConflictError::TemplateConflict(format!(
                                        "Template name '{}' already exists",
                                        name
                                    )));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Checks for command name conflicts.
    fn check_command_conflicts(manifest: &ExtensionManifest, package_path: &Path) -> Result<()> {
        // Discover existing commands
        let mut registry = CommandRegistry::new();
        registry.discover()
            .map_err(|e| ConflictError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to discover commands: {}", e),
            )))?;

        // Check if extension has commands directory
        let commands_dir = package_path.join("commands");
        if !commands_dir.exists() {
            return Ok(()); // No commands in extension
        }

        // Extension commands are namespaced, so conflicts are less likely
        // But we still check for exact name matches (without namespace)
        let extension_name = &manifest.name;

        // Scan extension commands directory
        if let Ok(entries) = std::fs::read_dir(&commands_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    // Try to load command to get name
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(cmd) = toml::from_str::<toml::Value>(&content) {
                            if let Some(name) = cmd.get("name").and_then(|v| v.as_str()) {
                                let namespaced_name = format!("{}:{}", extension_name, name);
                                // Check if namespaced command already exists
                                if registry.get(&namespaced_name).is_some() {
                                    return Err(ConflictError::CommandConflict(format!(
                                        "Command '{}' already exists",
                                        namespaced_name
                                    )));
                                }
                                // Also check for non-namespaced conflict (shouldn't happen but be safe)
                                if registry.get(name).is_some() {
                                    return Err(ConflictError::CommandConflict(format!(
                                        "Command name '{}' conflicts with existing command",
                                        name
                                    )));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Detects dependency cycles in extension dependencies.
    ///
    /// # Arguments
    /// * `extension_name` - Name of extension being installed
    /// * `manifest` - Extension manifest
    /// * `all_extensions` - All installed extensions (name -> dependencies map)
    ///
    /// # Returns
    /// Ok(()) if no cycles, error otherwise
    pub fn detect_dependency_cycles(
        extension_name: &str,
        manifest: &ExtensionManifest,
        all_extensions: &HashMap<String, Vec<String>>,
    ) -> Result<()> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        // Create a combined map that includes the extension being installed
        let mut combined_extensions = all_extensions.clone();
        combined_extensions.insert(extension_name.to_string(), manifest.dependencies.clone());

        Self::dfs_cycle_detection(
            extension_name,
            &manifest.dependencies,
            &combined_extensions,
            &mut visited,
            &mut rec_stack,
        )?;

        Ok(())
    }

    /// Depth-first search for cycle detection.
    fn dfs_cycle_detection(
        current: &str,
        dependencies: &[String],
        all_extensions: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> Result<()> {
        visited.insert(current.to_string());
        rec_stack.insert(current.to_string());

        for dep in dependencies {
            if rec_stack.contains(dep) {
                // Found a back edge - cycle detected
                return Err(ConflictError::DependencyCycle(format!(
                    "Circular dependency detected involving '{}' and '{}'",
                    current, dep
                )));
            }
            
            if !visited.contains(dep) {
                if let Some(dep_deps) = all_extensions.get(dep) {
                    Self::dfs_cycle_detection(dep, dep_deps, all_extensions, visited, rec_stack)?;
                }
            }
        }

        rec_stack.remove(current);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::manifest::ExtensionComponents;
    use std::collections::HashMap;

    fn create_test_manifest(name: &str) -> ExtensionManifest {
        ExtensionManifest {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Author".to_string(),
            components: ExtensionComponents::default(),
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_detect_dependency_cycle() {
        let mut all_extensions = HashMap::new();
        all_extensions.insert("B".to_string(), vec!["A".to_string()]); // B depends on A

        let mut manifest = create_test_manifest("A");
        manifest.dependencies.push("B".to_string()); // A depends on B -> cycle!

        let result = ConflictDetector::detect_dependency_cycles("A", &manifest, &all_extensions);
        assert!(result.is_err(), "Expected cycle detection but got: {:?}", result);
        assert!(matches!(result.unwrap_err(), ConflictError::DependencyCycle(_)));
    }

    #[test]
    fn test_detect_no_dependency_cycle() {
        let mut all_extensions = HashMap::new();
        all_extensions.insert("A".to_string(), vec!["B".to_string()]);
        all_extensions.insert("B".to_string(), vec![]);

        let manifest = create_test_manifest("A");
        let result = ConflictDetector::detect_dependency_cycles("A", &manifest, &all_extensions);
        assert!(result.is_ok());
    }
}

