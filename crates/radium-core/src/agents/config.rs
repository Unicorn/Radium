//! Agent configuration file format.
//!
//! Defines the TOML configuration format for agents.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

// Note: We use a type alias to avoid circular dependencies.
// The actual types are in crate::workflow::behaviors, but we'll
// define them here as optional TOML fields that can be deserialized.
// For now, we'll use a simplified representation that can be
// converted to the full types when needed.

/// Simplified loop behavior configuration for TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLoopBehavior {
    /// Number of steps to go back when looping.
    pub steps: usize,
    /// Maximum number of iterations before stopping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_iterations: Option<usize>,
    /// List of step IDs to skip during loop.
    #[serde(default)]
    pub skip: Vec<String>,
}

/// Simplified trigger behavior configuration for TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTriggerBehavior {
    /// Default agent ID to trigger (can be overridden in behavior.json).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_agent_id: Option<String>,
}

/// Agent configuration errors.
#[derive(Debug, Error)]
pub enum AgentConfigError {
    /// Invalid configuration.
    #[error("invalid configuration: {0}")]
    Invalid(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML deserialization error.
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}

/// Result type for agent configuration operations.
pub type Result<T> = std::result::Result<T, AgentConfigError>;

/// Reasoning effort levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ReasoningEffort {
    /// Minimal reasoning effort.
    Low,

    /// Moderate reasoning effort.
    #[default]
    Medium,

    /// Maximum reasoning effort.
    High,
}

impl std::fmt::Display for ReasoningEffort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
        }
    }
}

/// Agent configuration file (TOML format).
///
/// This is the structure of an agent configuration file, typically stored at
/// `agents/<category>/<agent-id>.toml`.
///
/// # Example TOML
///
/// ```toml
/// [agent]
/// id = "arch-agent"
/// name = "Architecture Agent"
/// description = "Defines system architecture and technical design decisions"
/// prompt_path = "prompts/agents/my-agents/arch-agent.md"
/// engine = "gemini"
/// model = "gemini-2.0-flash-exp"
/// reasoning_effort = "medium"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfigFile {
    /// Agent configuration.
    pub agent: AgentConfig,
}

impl AgentConfigFile {
    /// Load agent configuration from a TOML file.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or parsed.
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let config: Self = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Save agent configuration to a TOML file.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be written.
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let content =
            toml::to_string_pretty(self).map_err(|e| AgentConfigError::Invalid(e.to_string()))?;
        std::fs::write(path.as_ref(), content)?;
        Ok(())
    }

    /// Validate configuration.
    fn validate(&self) -> Result<()> {
        if self.agent.id.is_empty() {
            return Err(AgentConfigError::Invalid("agent ID cannot be empty".to_string()));
        }

        if self.agent.name.is_empty() {
            return Err(AgentConfigError::Invalid("agent name cannot be empty".to_string()));
        }

        if self.agent.prompt_path.as_os_str().is_empty() {
            return Err(AgentConfigError::Invalid("prompt path cannot be empty".to_string()));
        }

        Ok(())
    }
}

/// Agent configuration section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Unique agent identifier (e.g., "arch-agent", "plan-agent").
    pub id: String,

    /// Human-readable agent name (e.g., "Architecture Agent").
    pub name: String,

    /// Agent description.
    pub description: String,

    /// Path to the prompt template file (markdown).
    ///
    /// Can be absolute or relative to the workspace root.
    pub prompt_path: PathBuf,

    /// Optional mirror path for RAD-agents.
    ///
    /// This is used when agents are mirrored from another location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror_path: Option<PathBuf>,

    /// Default engine for this agent (optional).
    ///
    /// Examples: "gemini", "openai", "claude", "codex"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,

    /// Default model for this agent (optional).
    ///
    /// Examples: "gemini-2.0-flash-exp", "gpt-4", "claude-3-opus-20240229"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Default reasoning effort level (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,

    /// Optional loop behavior configuration.
    ///
    /// When set, this agent can request looping back to previous steps
    /// during workflow execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loop_behavior: Option<AgentLoopBehavior>,

    /// Optional trigger behavior configuration.
    ///
    /// When set, this agent can dynamically trigger other agents
    /// during workflow execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_behavior: Option<AgentTriggerBehavior>,

    /// Agent category (e.g., "my-agents", "rad-agents/design").
    ///
    /// This is typically derived from the file path, not stored in the TOML.
    #[serde(skip)]
    pub category: Option<String>,

    /// File path where this config was loaded from.
    ///
    /// This is not stored in the TOML, but set during loading.
    #[serde(skip)]
    pub file_path: Option<PathBuf>,
}

impl AgentConfig {
    /// Create a new agent configuration.
    pub fn new(id: impl Into<String>, name: impl Into<String>, prompt_path: PathBuf) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            prompt_path,
            mirror_path: None,
            engine: None,
            model: None,
            reasoning_effort: None,
            loop_behavior: None,
            trigger_behavior: None,
            category: None,
            file_path: None,
        }
    }

    /// Set the description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set the default engine.
    #[must_use]
    pub fn with_engine(mut self, engine: impl Into<String>) -> Self {
        self.engine = Some(engine.into());
        self
    }

    /// Set the default model.
    #[must_use]
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the reasoning effort level.
    #[must_use]
    pub fn with_reasoning_effort(mut self, effort: ReasoningEffort) -> Self {
        self.reasoning_effort = Some(effort);
        self
    }

    /// Set the category.
    #[must_use]
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Set the file path.
    #[must_use]
    pub fn with_file_path(mut self, path: PathBuf) -> Self {
        self.file_path = Some(path);
        self
    }

    /// Set the loop behavior configuration.
    #[must_use]
    pub fn with_loop_behavior(mut self, config: AgentLoopBehavior) -> Self {
        self.loop_behavior = Some(config);
        self
    }

    /// Set the trigger behavior configuration.
    #[must_use]
    pub fn with_trigger_behavior(mut self, config: AgentTriggerBehavior) -> Self {
        self.trigger_behavior = Some(config);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_agent_config_new() {
        let config = AgentConfig::new("test-agent", "Test Agent", PathBuf::from("prompts/test.md"));

        assert_eq!(config.id, "test-agent");
        assert_eq!(config.name, "Test Agent");
        assert_eq!(config.prompt_path, PathBuf::from("prompts/test.md"));
    }

    #[test]
    fn test_agent_config_builder() {
        let config = AgentConfig::new("test-agent", "Test Agent", PathBuf::from("prompts/test.md"))
            .with_description("A test agent")
            .with_engine("gemini")
            .with_model("gemini-2.0-flash-exp")
            .with_reasoning_effort(ReasoningEffort::High)
            .with_category("test");

        assert_eq!(config.description, "A test agent");
        assert_eq!(config.engine, Some("gemini".to_string()));
        assert_eq!(config.model, Some("gemini-2.0-flash-exp".to_string()));
        assert_eq!(config.reasoning_effort, Some(ReasoningEffort::High));
        assert_eq!(config.category, Some("test".to_string()));
    }

    #[test]
    fn test_agent_config_load() {
        let toml_content = r#"
[agent]
id = "arch-agent"
name = "Architecture Agent"
description = "Defines system architecture"
prompt_path = "prompts/arch-agent.md"
engine = "gemini"
model = "gemini-2.0-flash-exp"
reasoning_effort = "medium"
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(toml_content.as_bytes()).unwrap();
        file.flush().unwrap();

        let config = AgentConfigFile::load(file.path()).unwrap();
        assert_eq!(config.agent.id, "arch-agent");
        assert_eq!(config.agent.name, "Architecture Agent");
        assert_eq!(config.agent.engine, Some("gemini".to_string()));
        assert_eq!(config.agent.reasoning_effort, Some(ReasoningEffort::Medium));
    }

    #[test]
    fn test_agent_config_save() {
        let config = AgentConfigFile {
            agent: AgentConfig::new("test-agent", "Test Agent", PathBuf::from("prompts/test.md"))
                .with_description("A test agent")
                .with_engine("gemini"),
        };

        let temp = NamedTempFile::new().unwrap();
        config.save(temp.path()).unwrap();

        let loaded = AgentConfigFile::load(temp.path()).unwrap();
        assert_eq!(loaded.agent.id, config.agent.id);
        assert_eq!(loaded.agent.name, config.agent.name);
        assert_eq!(loaded.agent.engine, config.agent.engine);
    }

    #[test]
    fn test_agent_config_minimal() {
        let toml_content = r#"
[agent]
id = "minimal"
name = "Minimal Agent"
description = "Minimal config"
prompt_path = "prompts/minimal.md"
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(toml_content.as_bytes()).unwrap();
        file.flush().unwrap();

        let config = AgentConfigFile::load(file.path()).unwrap();
        assert_eq!(config.agent.id, "minimal");
        assert_eq!(config.agent.engine, None);
        assert_eq!(config.agent.model, None);
        assert_eq!(config.agent.reasoning_effort, None);
    }

    #[test]
    fn test_reasoning_effort_display() {
        assert_eq!(ReasoningEffort::Low.to_string(), "low");
        assert_eq!(ReasoningEffort::Medium.to_string(), "medium");
        assert_eq!(ReasoningEffort::High.to_string(), "high");
    }

    #[test]
    fn test_reasoning_effort_default() {
        assert_eq!(ReasoningEffort::default(), ReasoningEffort::Medium);
    }

    #[test]
    fn test_agent_config_with_loop_behavior() {
        let toml_content = r#"
[agent]
id = "test-agent"
name = "Test Agent"
description = "Test agent with loop behavior"
prompt_path = "prompts/test.md"

[agent.loop_behavior]
steps = 2
max_iterations = 5
skip = ["step-1", "step-3"]
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(toml_content.as_bytes()).unwrap();
        file.flush().unwrap();

        let config = AgentConfigFile::load(file.path()).unwrap();
        assert_eq!(config.agent.id, "test-agent");
        assert!(config.agent.loop_behavior.is_some());

        let loop_behavior = config.agent.loop_behavior.unwrap();
        assert_eq!(loop_behavior.steps, 2);
        assert_eq!(loop_behavior.max_iterations, Some(5));
        assert_eq!(loop_behavior.skip, vec!["step-1", "step-3"]);
    }

    #[test]
    fn test_agent_config_with_trigger_behavior() {
        let toml_content = r#"
[agent]
id = "test-agent"
name = "Test Agent"
description = "Test agent with trigger behavior"
prompt_path = "prompts/test.md"

[agent.trigger_behavior]
trigger_agent_id = "fallback-agent"
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(toml_content.as_bytes()).unwrap();
        file.flush().unwrap();

        let config = AgentConfigFile::load(file.path()).unwrap();
        assert_eq!(config.agent.id, "test-agent");
        assert!(config.agent.trigger_behavior.is_some());

        let trigger_behavior = config.agent.trigger_behavior.unwrap();
        assert_eq!(trigger_behavior.trigger_agent_id, Some("fallback-agent".to_string()));
    }

    #[test]
    fn test_agent_config_with_both_behaviors() {
        let toml_content = r#"
[agent]
id = "test-agent"
name = "Test Agent"
description = "Test agent with both behaviors"
prompt_path = "prompts/test.md"

[agent.loop_behavior]
steps = 3
max_iterations = 10

[agent.trigger_behavior]
trigger_agent_id = "helper-agent"
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(toml_content.as_bytes()).unwrap();
        file.flush().unwrap();

        let config = AgentConfigFile::load(file.path()).unwrap();
        assert!(config.agent.loop_behavior.is_some());
        assert!(config.agent.trigger_behavior.is_some());
    }
}
