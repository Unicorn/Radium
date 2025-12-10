//! Agent discovery from configuration files.
//!
//! Scans agent directories for TOML configuration files and loads them.

use crate::agents::config::{AgentConfig, AgentConfigFile};
use crate::agents::metadata::AgentMetadata;
use crate::agents::persona::{PersonaConfig, PerformanceConfig, PerformanceProfile, RecommendedModels, SimpleModelRecommendation};
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
#[derive(Debug, Clone, Default)]
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
                        // Continue discovery even if a config file fails to load
                        if let Err(e) = self.load_agent_config(&path, root, agents) {
                            tracing::debug!(
                                path = %path.display(),
                                error = %e,
                                "Skipping invalid agent config file"
                            );
                        }
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

        // Parse YAML frontmatter from prompt file if present
        // This enhances the agent with persona metadata from the prompt
        if let Some(prompt_path) = self.resolve_prompt_path(&agent.prompt_path, path) {
            if let Ok(content) = fs::read_to_string(&prompt_path) {
                if content.trim_start().starts_with("---") {
                    if let Ok((metadata, _)) = AgentMetadata::from_markdown(&content) {
                        // Convert AgentMetadata to PersonaConfig if recommended_models exist
                        if let Some(ref recommended_models) = metadata.recommended_models {
                            let persona_config = Self::convert_metadata_to_persona(
                                recommended_models,
                                &metadata,
                            );
                            // YAML frontmatter takes precedence over TOML persona config
                            agent.persona_config = Some(persona_config);
                        }
                    }
                }
            }
        }

        // Apply sub-agent filter if set
        if let Some(filter) = &self.options.sub_agent_filter {
            if !filter.contains(&agent.id) {
                return Ok(());
            }
        }

        // Add to agents map (only if not already present to maintain precedence)
        // Earlier paths (project, user) take precedence over later paths (extensions)
        // This ensures project-local agents override extension agents
        if !agents.contains_key(&agent.id) {
            agents.insert(agent.id.clone(), agent);
        }

        Ok(())
    }

    /// Get default search paths.
    ///
    /// Search order (precedence from highest to lowest):
    /// 1. Project-local agents
    /// 2. User agents
    /// 3. Workspace agents
    /// 4. Project-level extension agents
    /// 5. User-level extension agents
    fn default_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // 1. Project-local agents
        if let Ok(cwd) = std::env::current_dir() {
            paths.push(cwd.join("agents"));
        }

        // 2. User agents in home directory
        // Allow env::var for standard HOME environment variable (path discovery)
        #[allow(clippy::disallowed_methods)]
        if let Ok(home) = std::env::var("HOME") {
            paths.push(PathBuf::from(home).join(".radium/agents"));
        }

        // 3. Workspace agents if RADIUM_WORKSPACE is set
        // Allow env::var for RADIUM_WORKSPACE (path discovery, not app config)
        #[allow(clippy::disallowed_methods)]
        if let Ok(workspace) = std::env::var("RADIUM_WORKSPACE") {
            let workspace_path = PathBuf::from(workspace);
            paths.push(workspace_path.join("agents"));
            paths.push(workspace_path.join(".radium/agents"));
        }

        // 4. Project-level extension agents (higher precedence than user extensions)
        if let Ok(cwd) = std::env::current_dir() {
            let project_extensions_dir = cwd.join(".radium").join("extensions");
            if project_extensions_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&project_extensions_dir) {
                    for entry in entries.flatten() {
                        let ext_path = entry.path();
                        if ext_path.is_dir() {
                            let agents_dir = ext_path.join("agents");
                            if agents_dir.exists() {
                                paths.push(agents_dir);
                            }
                        }
                    }
                }
            }
        }

        // 5. User-level extension agents (lowest precedence)
        #[allow(clippy::disallowed_methods)]
        if let Ok(home) = std::env::var("HOME") {
            let user_extensions_dir = PathBuf::from(home).join(".radium").join("extensions");
            if user_extensions_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&user_extensions_dir) {
                    for entry in entries.flatten() {
                        let ext_path = entry.path();
                        if ext_path.is_dir() {
                            let agents_dir = ext_path.join("agents");
                            if agents_dir.exists() {
                                paths.push(agents_dir);
                            }
                        }
                    }
                }
            }
        }

        paths
    }

    /// Resolve prompt path relative to config file location.
    fn resolve_prompt_path(&self, prompt_path: &PathBuf, config_path: &Path) -> Option<PathBuf> {
        if prompt_path.is_absolute() {
            if prompt_path.exists() {
                Some(prompt_path.clone())
            } else {
                None
            }
        } else if let Some(config_dir) = config_path.parent() {
            let resolved = config_dir.join(prompt_path);
            if resolved.exists() {
                Some(resolved)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Convert AgentMetadata::RecommendedModels to PersonaConfig.
    fn convert_metadata_to_persona(
        recommended_models: &crate::agents::metadata::RecommendedModels,
        metadata: &AgentMetadata,
    ) -> PersonaConfig {
        // Convert primary model
        let primary = SimpleModelRecommendation {
            engine: recommended_models.primary.engine.clone(),
            model: recommended_models.primary.model.clone(),
        };

        // Convert fallback model if present
        let fallback = recommended_models.fallback.as_ref().map(|f| SimpleModelRecommendation {
            engine: f.engine.clone(),
            model: f.model.clone(),
        });

        // Convert premium model if present
        let premium = recommended_models.premium.as_ref().map(|p| SimpleModelRecommendation {
            engine: p.engine.clone(),
            model: p.model.clone(),
        });

        // Determine performance profile from primary model's priority
        let performance_profile = match recommended_models.primary.priority {
            crate::agents::metadata::ModelPriority::Speed => PerformanceProfile::Speed,
            crate::agents::metadata::ModelPriority::Balanced => PerformanceProfile::Balanced,
            crate::agents::metadata::ModelPriority::Thinking => PerformanceProfile::Thinking,
            crate::agents::metadata::ModelPriority::Expert => PerformanceProfile::Expert,
        };

        // Create performance config
        let performance = PerformanceConfig {
            profile: performance_profile,
            estimated_tokens: None, // estimated_tokens not in AgentMetadata, can be added later
        };

        PersonaConfig {
            models: RecommendedModels {
                primary,
                fallback,
                premium,
            },
            performance,
        }
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

        // Create prompts directory relative to where configs are (at root level)
        // and also relative to each category directory for nested categories
        let prompts_dir_root = dir.join("prompts");
        fs::create_dir_all(&prompts_dir_root).unwrap();
        let prompt_path_root = prompts_dir_root.join("test.md");
        fs::write(&prompt_path_root, format!("# Test Agent: {}", id)).unwrap();

        // Also create in category directory for nested paths
        let prompts_dir_category = category_dir.join("prompts");
        fs::create_dir_all(&prompts_dir_category).unwrap();
        let prompt_path_category = prompts_dir_category.join("test.md");
        fs::write(&prompt_path_category, format!("# Test Agent: {}", id)).unwrap();

        let config = AgentConfigFile {
            model: None,
            safety: None,
            agent: AgentConfig::new(id, format!("{} Agent", id), PathBuf::from("prompts/test.md"))
                .with_description(format!("Test agent {}", id))
                .with_engine("gemini")
                .with_model("gemini-2.0-flash-exp")
                .with_reasoning_effort(ReasoningEffort::Medium),
            persona: None,
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
    fn test_discovery_with_malformed_toml() {
        let temp = TempDir::new().unwrap();

        // Create a valid agent
        create_test_agent_config(temp.path(), "test-agents", "valid-agent");

        // Create a malformed TOML file
        let test_agents_dir = temp.path().join("test-agents");
        fs::create_dir_all(&test_agents_dir).unwrap();
        let malformed_path = test_agents_dir.join("malformed.toml");
        fs::write(&malformed_path, "[agent]\nid = invalid syntax\n").unwrap();

        let options = DiscoveryOptions {
            search_paths: vec![temp.path().to_path_buf()],
            sub_agent_filter: None,
        };

        let discovery = AgentDiscovery::with_options(options);
        // Discovery should continue even with malformed files
        // It should log errors but not fail completely
        let result = discovery.discover_all();
        
        // Should still discover the valid agent
        if let Ok(agents) = result {
            assert!(agents.contains_key("valid-agent"), "Should discover valid agent even with malformed file");
        }
    }

    #[test]
    fn test_discovery_mixed_valid_invalid_configs() {
        let temp = TempDir::new().unwrap();

        // Create 3 valid agents
        create_test_agent_config(temp.path(), "test-agents", "agent-1");
        create_test_agent_config(temp.path(), "test-agents", "agent-2");
        create_test_agent_config(temp.path(), "test-agents", "agent-3");

        // Create 2 invalid configs (missing required fields)
        let invalid_dir = temp.path().join("test-agents");
        fs::write(invalid_dir.join("invalid-1.toml"), "[agent]\nid = \"invalid-1\"\n").unwrap();
        fs::write(invalid_dir.join("invalid-2.toml"), "[agent]\nname = \"Invalid\"\n").unwrap();

        let options = DiscoveryOptions {
            search_paths: vec![temp.path().to_path_buf()],
            sub_agent_filter: None,
        };

        let discovery = AgentDiscovery::with_options(options);
        let agents = discovery.discover_all().expect("Discovery should continue despite errors");

        // Should discover 3 valid agents
        assert_eq!(agents.len(), 3, "Should discover 3 valid agents");
        assert!(agents.contains_key("agent-1"));
        assert!(agents.contains_key("agent-2"));
        assert!(agents.contains_key("agent-3"));
        assert!(!agents.contains_key("invalid-1"));
        assert!(!agents.contains_key("invalid-2"));
    }

    #[test]
    fn test_discovery_prompt_file_resolution_edge_cases() {
        use crate::agents::config::AgentConfigFile;

        let temp = TempDir::new().unwrap();
        let agents_dir = temp.path().join("agents");
        fs::create_dir_all(&agents_dir).unwrap();

        // Create agent with relative path that should resolve
        // Create prompts relative to config directory (agents/prompts/)
        let prompts_dir = agents_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        let prompt_file = prompts_dir.join("test.md");
        fs::write(&prompt_file, "# Test").unwrap();

        // Also create at root level for workspace root resolution
        let root_prompts_dir = temp.path().join("prompts");
        fs::create_dir_all(&root_prompts_dir).unwrap();
        let root_prompt_file = root_prompts_dir.join("test.md");
        fs::write(&root_prompt_file, "# Test").unwrap();

        let config = AgentConfigFile {
            model: None,
            safety: None,
            agent: AgentConfig::new("test-agent", "Test Agent", PathBuf::from("prompts/test.md"))
                .with_description("Test")
                .with_file_path(agents_dir.join("test-agent.toml")),
            persona: None,
        };
        config.save(&agents_dir.join("test-agent.toml")).unwrap();

        let options = DiscoveryOptions {
            search_paths: vec![temp.path().to_path_buf()],
            sub_agent_filter: None,
        };

        let discovery = AgentDiscovery::with_options(options);
        let agents = discovery.discover_all().expect("Should discover agent");

        assert!(agents.contains_key("test-agent"));
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

    #[test]
    fn test_handle_malformed_agent_definitions() {
        let temp = TempDir::new().unwrap();
        let category_dir = temp.path().join("test-agents");
        fs::create_dir_all(&category_dir).unwrap();

        // Create a valid agent config
        create_test_agent_config(temp.path(), "test-agents", "valid-agent");

        // Create a malformed TOML file
        let malformed_path = category_dir.join("malformed-agent.toml");
        fs::write(&malformed_path, "this is not valid TOML content!!!").unwrap();

        // Create a file with missing required fields
        let incomplete_path = category_dir.join("incomplete-agent.toml");
        fs::write(&incomplete_path, "[agent]\nid = \"incomplete\"\n").unwrap();

        let options = DiscoveryOptions {
            search_paths: vec![temp.path().to_path_buf()],
            sub_agent_filter: None,
        };

        let discovery = AgentDiscovery::with_options(options);
        let agents = discovery.discover_all().unwrap();

        // Should only discover the valid agent, skipping malformed ones gracefully
        assert_eq!(agents.len(), 1);
        assert!(agents.contains_key("valid-agent"));
        assert!(!agents.contains_key("malformed-agent"));
        assert!(!agents.contains_key("incomplete-agent"));
    }
}
