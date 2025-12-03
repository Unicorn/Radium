//! Agent discovery from configuration files.
//!
//! Scans agent directories for TOML configuration files and loads them.

use crate::agents::config::{AgentConfig, AgentConfigFile};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Agent discovery errors.
#[derive(Debug, Error)]
pub enum DiscoveryError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(#[from] crate::agents::config::AgentConfigError),

    /// Agent not found.
    #[error("agent not found: {0}")]
    NotFound(String),
}

/// Result type for discovery operations.
pub type Result<T> = std::result::Result<T, DiscoveryError>;

/// Agent discovery options.
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct DiscoveryOptions {
    /// Agent directories to search.
    ///
    /// If empty, uses default directories.
    pub search_paths: Vec<PathBuf>,

    /// Filter by sub-agent IDs (for template filtering).
    ///
    /// If set, only agents with IDs in this list will be discovered.
    pub sub_agent_filter: Option<Vec<String>>,
}


/// Agent discovery service.
///
/// Scans directories for agent configuration files (*.toml) and loads them.
pub struct AgentDiscovery {
    options: DiscoveryOptions,
}

impl AgentDiscovery {
    /// Create a new agent discovery service with default options.
    pub fn new() -> Self {
        Self { options: DiscoveryOptions::default() }
    }

    /// Create a new agent discovery service with custom options.
    pub fn with_options(options: DiscoveryOptions) -> Self {
        Self { options }
    }

    /// Discover all agents in the default or configured directories.
    ///
    /// Default search order:
    /// 1. `./agents/` (project-local agents)
    /// 2. `~/.radium/agents/` (user agents)
    /// 3. Built-in agents (if applicable)
    ///
    /// # Errors
    ///
    /// Returns error if agents cannot be discovered or loaded.
    pub fn discover_all(&self) -> Result<HashMap<String, AgentConfig>> {
        let mut agents = HashMap::new();

        let search_paths = if self.options.search_paths.is_empty() {
            Self::default_search_paths()
        } else {
            self.options.search_paths.clone()
        };

        for search_path in search_paths {
            if !search_path.exists() {
                continue;
            }

            self.discover_in_directory(&search_path, &mut agents)?;
        }

        Ok(agents)
    }

    /// Discover agents in a specific directory.
    fn discover_in_directory(
        &self,
        dir: &Path,
        agents: &mut HashMap<String, AgentConfig>,
    ) -> Result<()> {
        self.scan_directory(dir, dir, agents)?;
        Ok(())
    }

    /// Recursively scan a directory for agent configs.
    fn scan_directory(
        &self,
        root: &Path,
        current: &Path,
        agents: &mut HashMap<String, AgentConfig>,
    ) -> Result<()> {
        for entry in fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively scan subdirectories
                self.scan_directory(root, &path, agents)?;
            } else if path.is_file() {
                // Check if this is a TOML file
                if let Some(ext) = path.extension() {
                    if ext == "toml" {
                        self.load_agent_config(&path, root, agents)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Load an agent configuration file.
    fn load_agent_config(
        &self,
        path: &Path,
        root: &Path,
        agents: &mut HashMap<String, AgentConfig>,
    ) -> Result<()> {
        let config_file = AgentConfigFile::load(path)?;
        let mut agent = config_file.agent;

        // Set the file path
        agent.file_path = Some(path.to_path_buf());

        // Derive category from file path relative to root
        if let Ok(relative) = path.strip_prefix(root) {
            if let Some(parent) = relative.parent() {
                let category = parent.to_string_lossy().to_string();
                if !category.is_empty() {
                    agent.category = Some(category);
                }
            }
        }

        // Apply sub-agent filter if set
        if let Some(filter) = &self.options.sub_agent_filter {
            if !filter.contains(&agent.id) {
                return Ok(());
            }
        }

        // Add to agents map (later entries override earlier ones)
        agents.insert(agent.id.clone(), agent);

        Ok(())
    }

    /// Get default search paths.
    fn default_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // 1. Project-local agents
        if let Ok(cwd) = std::env::current_dir() {
            paths.push(cwd.join("agents"));
        }

        // 2. User agents in home directory
        if let Ok(home) = std::env::var("HOME") {
            paths.push(PathBuf::from(home).join(".radium/agents"));
        }

        // 3. Workspace agents if RADIUM_WORKSPACE is set
        if let Ok(workspace) = std::env::var("RADIUM_WORKSPACE") {
            let workspace_path = PathBuf::from(workspace);
            paths.push(workspace_path.join("agents"));
            paths.push(workspace_path.join(".radium/agents"));
        }

        paths
    }

    /// Find an agent by ID.
    ///
    /// # Errors
    ///
    /// Returns error if agent cannot be found or loaded.
    pub fn find_by_id(&self, id: &str) -> Result<Option<AgentConfig>> {
        let agents = self.discover_all()?;
        Ok(agents.get(id).cloned())
    }

    /// List all agent IDs.
    ///
    /// # Errors
    ///
    /// Returns error if agents cannot be discovered.
    pub fn list_ids(&self) -> Result<Vec<String>> {
        let agents = self.discover_all()?;
        let mut ids: Vec<String> = agents.keys().cloned().collect();
        ids.sort();
        Ok(ids)
    }
}

impl Default for AgentDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::config::ReasoningEffort;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_agent_config(dir: &Path, category: &str, id: &str) {
        let category_dir = dir.join(category);
        fs::create_dir_all(&category_dir).unwrap();

        let config = AgentConfigFile {
            agent: AgentConfig::new(id, format!("{} Agent", id), PathBuf::from("prompts/test.md"))
                .with_description(format!("Test agent {}", id))
                .with_engine("gemini")
                .with_model("gemini-2.0-flash-exp")
                .with_reasoning_effort(ReasoningEffort::Medium),
        };

        let config_path = category_dir.join(format!("{}.toml", id));
        config.save(&config_path).unwrap();
    }

    #[test]
    fn test_discover_all() {
        let temp = TempDir::new().unwrap();

        create_test_agent_config(temp.path(), "test-agents", "arch-agent");
        create_test_agent_config(temp.path(), "test-agents", "plan-agent");
        create_test_agent_config(temp.path(), "rad-agents/design", "ui-designer");

        let options = DiscoveryOptions {
            search_paths: vec![temp.path().to_path_buf()],
            sub_agent_filter: None,
        };

        let discovery = AgentDiscovery::with_options(options);
        let agents = discovery.discover_all().unwrap();

        assert_eq!(agents.len(), 3);
        assert!(agents.contains_key("arch-agent"));
        assert!(agents.contains_key("plan-agent"));
        assert!(agents.contains_key("ui-designer"));
    }

    #[test]
    fn test_agent_category() {
        let temp = TempDir::new().unwrap();
        create_test_agent_config(temp.path(), "test-agents", "arch-agent");
        create_test_agent_config(temp.path(), "rad-agents/design", "ui-designer");

        let options = DiscoveryOptions {
            search_paths: vec![temp.path().to_path_buf()],
            sub_agent_filter: None,
        };

        let discovery = AgentDiscovery::with_options(options);
        let agents = discovery.discover_all().unwrap();

        let arch_agent = agents.get("arch-agent").unwrap();
        assert_eq!(arch_agent.category, Some("test-agents".to_string()));

        let ui_designer = agents.get("ui-designer").unwrap();
        assert_eq!(ui_designer.category, Some("rad-agents/design".to_string()));
    }

    #[test]
    fn test_sub_agent_filter() {
        let temp = TempDir::new().unwrap();
        create_test_agent_config(temp.path(), "test-agents", "arch-agent");
        create_test_agent_config(temp.path(), "test-agents", "plan-agent");
        create_test_agent_config(temp.path(), "rad-agents/design", "ui-designer");

        let options = DiscoveryOptions {
            search_paths: vec![temp.path().to_path_buf()],
            sub_agent_filter: Some(vec!["arch-agent".to_string(), "ui-designer".to_string()]),
        };

        let discovery = AgentDiscovery::with_options(options);
        let agents = discovery.discover_all().unwrap();

        assert_eq!(agents.len(), 2);
        assert!(agents.contains_key("arch-agent"));
        assert!(agents.contains_key("ui-designer"));
        assert!(!agents.contains_key("plan-agent"));
    }

    #[test]
    fn test_find_by_id() {
        let temp = TempDir::new().unwrap();
        create_test_agent_config(temp.path(), "test-agents", "arch-agent");

        let options = DiscoveryOptions {
            search_paths: vec![temp.path().to_path_buf()],
            sub_agent_filter: None,
        };

        let discovery = AgentDiscovery::with_options(options);
        let agent = discovery.find_by_id("arch-agent").unwrap();

        assert!(agent.is_some());
        let agent = agent.unwrap();
        assert_eq!(agent.id, "arch-agent");
        assert_eq!(agent.category, Some("test-agents".to_string()));
    }

    #[test]
    fn test_list_ids() {
        let temp = TempDir::new().unwrap();
        create_test_agent_config(temp.path(), "test-agents", "arch-agent");
        create_test_agent_config(temp.path(), "test-agents", "plan-agent");
        create_test_agent_config(temp.path(), "rad-agents/design", "ui-designer");

        let options = DiscoveryOptions {
            search_paths: vec![temp.path().to_path_buf()],
            sub_agent_filter: None,
        };

        let discovery = AgentDiscovery::with_options(options);
        let ids = discovery.list_ids().unwrap();

        assert_eq!(ids.len(), 3);
        assert_eq!(ids, vec!["arch-agent", "plan-agent", "ui-designer"]);
    }

    #[test]
    fn test_empty_directory() {
        let temp = TempDir::new().unwrap();

        let options = DiscoveryOptions {
            search_paths: vec![temp.path().to_path_buf()],
            sub_agent_filter: None,
        };

        let discovery = AgentDiscovery::with_options(options);
        let agents = discovery.discover_all().unwrap();

        assert_eq!(agents.len(), 0);
    }

    #[test]
    fn test_nonexistent_directory() {
        let options = DiscoveryOptions {
            search_paths: vec![PathBuf::from("/nonexistent/path")],
            sub_agent_filter: None,
        };

        let discovery = AgentDiscovery::with_options(options);
        let agents = discovery.discover_all().unwrap();

        // Should not error, just return empty
        assert_eq!(agents.len(), 0);
    }
}
