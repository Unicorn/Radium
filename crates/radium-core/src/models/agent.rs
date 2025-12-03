//! Agent data structures for Radium Core.
//!
//! This module defines the core data structures for agents, including
//! the Agent struct itself, its configuration, runtime state, and
//! conversion utilities for working with gRPC protocol definitions.

use chrono::{DateTime, Utc};
use radium_abstraction::ModelParameters;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::proto_convert;
use crate::proto;

/// Runtime state of an agent.
///
/// Tracks the current execution state of an agent, allowing the system
/// to monitor and manage agent lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    /// Agent is ready but not currently executing.
    Idle,
    /// Agent is currently executing a task.
    Running,
    /// Agent execution has been paused.
    Paused,
    /// Agent encountered an error during execution.
    Error(String),
    /// Agent execution completed successfully.
    Completed,
}

impl Default for AgentState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Configuration for an agent's execution.
///
/// Contains all settings needed to configure how an agent executes,
/// including model selection, generation parameters, and execution limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// ID of the AI model to use for this agent.
    pub model_id: String,
    /// Optional model generation parameters (temperature, max_tokens, etc.).
    pub model_parameters: Option<ModelParameters>,
    /// Maximum number of execution iterations before stopping.
    pub max_iterations: Option<u32>,
    /// Execution timeout in seconds.
    pub timeout_seconds: Option<u64>,
}

impl AgentConfig {
    /// Creates a new `AgentConfig` with the specified model ID.
    ///
    /// # Arguments
    /// * `model_id` - The ID of the model to use
    ///
    /// # Returns
    /// A new `AgentConfig` with default values for optional fields.
    pub fn new(model_id: String) -> Self {
        Self { model_id, model_parameters: None, max_iterations: None, timeout_seconds: None }
    }

    /// Validates the agent configuration.
    ///
    /// # Returns
    /// `Ok(())` if the configuration is valid, or an `AgentError` if invalid.
    ///
    /// # Errors
    /// * `AgentError::InvalidConfig` - If the configuration is invalid
    pub fn validate(&self) -> Result<(), AgentError> {
        if self.model_id.is_empty() {
            return Err(AgentError::InvalidConfig("model_id cannot be empty".to_string()));
        }

        if let Some(max_iterations) = self.max_iterations {
            if max_iterations == 0 {
                return Err(AgentError::InvalidConfig(
                    "max_iterations must be greater than 0".to_string(),
                ));
            }
        }

        if let Some(timeout) = self.timeout_seconds {
            if timeout == 0 {
                return Err(AgentError::InvalidConfig(
                    "timeout_seconds must be greater than 0".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Core agent data structure.
///
/// Represents an agent in the Radium system, including its identity,
/// configuration, current state, and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Unique identifier for the agent.
    pub id: String,
    /// Human-readable name for the agent.
    pub name: String,
    /// Description of the agent's purpose and capabilities.
    pub description: String,
    /// Configuration for agent execution.
    pub config: AgentConfig,
    /// Current runtime state of the agent.
    pub state: AgentState,
    /// Timestamp when the agent was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the agent was last updated.
    pub updated_at: DateTime<Utc>,
}

impl Agent {
    /// Creates a new agent with the specified properties.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the agent
    /// * `name` - Human-readable name
    /// * `description` - Description of the agent
    /// * `config` - Agent configuration
    ///
    /// # Returns
    /// A new `Agent` with `Idle` state and current timestamps.
    pub fn new(id: String, name: String, description: String, config: AgentConfig) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description,
            config,
            state: AgentState::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Validates the agent data.
    ///
    /// # Returns
    /// `Ok(())` if the agent is valid, or an `AgentError` if invalid.
    ///
    /// # Errors
    /// * `AgentError::InvalidAgent` - If the agent data is invalid
    /// * `AgentError::InvalidConfig` - If the configuration is invalid
    pub fn validate(&self) -> Result<(), AgentError> {
        if self.id.is_empty() {
            return Err(AgentError::InvalidAgent("id cannot be empty".to_string()));
        }

        if self.name.is_empty() {
            return Err(AgentError::InvalidAgent("name cannot be empty".to_string()));
        }

        self.config.validate()?;

        Ok(())
    }

    /// Updates the agent's state and sets the updated_at timestamp.
    ///
    /// # Arguments
    /// * `state` - The new state for the agent
    pub fn set_state(&mut self, state: AgentState) {
        self.state = state;
        self.updated_at = Utc::now();
    }
}

/// Builder for creating `Agent` instances.
///
/// Provides a fluent interface for constructing agents with optional fields.
#[derive(Debug, Default)]
pub struct AgentBuilder {
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    config: Option<AgentConfig>,
    state: Option<AgentState>,
}

impl AgentBuilder {
    /// Creates a new `AgentBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the agent ID.
    #[must_use]
    pub fn id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the agent name.
    #[must_use]
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets the agent description.
    #[must_use]
    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets the agent configuration.
    #[must_use]
    pub fn config(mut self, config: AgentConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Sets the initial agent state.
    #[must_use]
    pub fn state(mut self, state: AgentState) -> Self {
        self.state = Some(state);
        self
    }

    /// Builds the `Agent` from the builder.
    ///
    /// # Returns
    /// `Ok(Agent)` if all required fields are set, or an `AgentError` if validation fails.
    ///
    /// # Errors
    /// * `AgentError::InvalidAgent` - If required fields are missing or invalid
    pub fn build(self) -> Result<Agent, AgentError> {
        let id = self.id.ok_or_else(|| AgentError::InvalidAgent("id is required".to_string()))?;
        let name =
            self.name.ok_or_else(|| AgentError::InvalidAgent("name is required".to_string()))?;
        let description = self
            .description
            .ok_or_else(|| AgentError::InvalidAgent("description is required".to_string()))?;
        let config = self
            .config
            .ok_or_else(|| AgentError::InvalidAgent("config is required".to_string()))?;

        let now = Utc::now();
        let agent = Agent {
            id,
            name,
            description,
            config,
            state: self.state.unwrap_or_default(),
            created_at: now,
            updated_at: now,
        };

        agent.validate()?;
        Ok(agent)
    }
}

/// Errors that can occur when working with agents.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AgentError {
    /// Invalid agent data.
    #[error("Invalid agent: {0}")]
    InvalidAgent(String),

    /// Invalid agent configuration.
    #[error("Invalid config: {0}")]
    InvalidConfig(String),

    /// Error during proto conversion.
    #[error("Proto conversion error: {0}")]
    ProtoConversion(String),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(String),
}

impl From<serde_json::Error> for AgentError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}

// Conversion from proto::Agent to Agent
impl TryFrom<proto::Agent> for Agent {
    type Error = AgentError;

    fn try_from(proto_agent: proto::Agent) -> Result<Self, Self::Error> {
        let config = proto_convert::json_from_str(&proto_agent.config_json)?;
        let state = proto_convert::json_from_str(&proto_agent.state)?;
        let created_at =
            proto_convert::parse_rfc3339_timestamp(&proto_agent.created_at, "created_at")
                .map_err(|e| AgentError::ProtoConversion(e))?;
        let updated_at =
            proto_convert::parse_rfc3339_timestamp(&proto_agent.updated_at, "updated_at")
                .map_err(|e| AgentError::ProtoConversion(e))?;

        Ok(Agent {
            id: proto_agent.id,
            name: proto_agent.name,
            description: proto_agent.description,
            config,
            state,
            created_at,
            updated_at,
        })
    }
}

// Conversion from Agent to proto::Agent
impl From<Agent> for proto::Agent {
    fn from(agent: Agent) -> Self {
        let config_json = proto_convert::json_to_string(&agent.config, "{}");
        let state = proto_convert::json_to_string(&agent.state, "");

        proto::Agent {
            id: agent.id,
            name: agent.name,
            description: agent.description,
            config_json,
            state,
            created_at: proto_convert::format_rfc3339_timestamp(&agent.created_at),
            updated_at: proto_convert::format_rfc3339_timestamp(&agent.updated_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_state_default() {
        let state = AgentState::default();
        assert_eq!(state, AgentState::Idle);
    }

    #[test]
    fn test_agent_config_new() {
        let config = AgentConfig::new("test-model".to_string());
        assert_eq!(config.model_id, "test-model");
        assert!(config.model_parameters.is_none());
        assert!(config.max_iterations.is_none());
        assert!(config.timeout_seconds.is_none());
    }

    #[test]
    fn test_agent_config_validate_success() {
        let config = AgentConfig::new("test-model".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_agent_config_validate_empty_model_id() {
        let config = AgentConfig::new("".to_string());
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_agent_config_validate_zero_max_iterations() {
        let mut config = AgentConfig::new("test-model".to_string());
        config.max_iterations = Some(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_agent_config_validate_zero_timeout() {
        let mut config = AgentConfig::new("test-model".to_string());
        config.timeout_seconds = Some(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_agent_new() {
        let config = AgentConfig::new("test-model".to_string());
        let agent = Agent::new(
            "test-id".to_string(),
            "Test Agent".to_string(),
            "A test agent".to_string(),
            config,
        );

        assert_eq!(agent.id, "test-id");
        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.description, "A test agent");
        assert_eq!(agent.state, AgentState::Idle);
    }

    #[test]
    fn test_agent_validate_success() {
        let config = AgentConfig::new("test-model".to_string());
        let agent = Agent::new(
            "test-id".to_string(),
            "Test Agent".to_string(),
            "A test agent".to_string(),
            config,
        );
        assert!(agent.validate().is_ok());
    }

    #[test]
    fn test_agent_validate_empty_id() {
        let config = AgentConfig::new("test-model".to_string());
        let agent = Agent::new(
            "".to_string(),
            "Test Agent".to_string(),
            "A test agent".to_string(),
            config,
        );
        assert!(agent.validate().is_err());
    }

    #[test]
    fn test_agent_validate_empty_name() {
        let config = AgentConfig::new("test-model".to_string());
        let agent =
            Agent::new("test-id".to_string(), "".to_string(), "A test agent".to_string(), config);
        assert!(agent.validate().is_err());
    }

    #[test]
    fn test_agent_set_state() {
        let config = AgentConfig::new("test-model".to_string());
        let mut agent = Agent::new(
            "test-id".to_string(),
            "Test Agent".to_string(),
            "A test agent".to_string(),
            config,
        );

        let initial_updated_at = agent.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        agent.set_state(AgentState::Running);

        assert_eq!(agent.state, AgentState::Running);
        assert!(agent.updated_at > initial_updated_at);
    }

    #[test]
    fn test_proto_agent_to_agent() {
        let config = AgentConfig::new("test-model".to_string());
        let config_json = serde_json::to_string(&config).unwrap();
        let state = serde_json::to_string(&AgentState::Idle).unwrap();

        let proto_agent = proto::Agent {
            id: "test-id".to_string(),
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            config_json,
            state,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        };

        let agent = Agent::try_from(proto_agent).unwrap();
        assert_eq!(agent.id, "test-id");
        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.description, "A test agent");
        assert_eq!(agent.config.model_id, "test-model");
    }

    #[test]
    fn test_proto_agent_to_agent_missing_id() {
        let config = AgentConfig::new("test-model".to_string());
        let config_json = serde_json::to_string(&config).unwrap();
        let state = serde_json::to_string(&AgentState::Idle).unwrap();

        let proto_agent = proto::Agent {
            id: "".to_string(),
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            config_json,
            state,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        };

        // Agent with empty ID should still parse from proto, but validation would fail
        let agent = Agent::try_from(proto_agent).unwrap();
        assert!(agent.validate().is_err());
    }

    #[test]
    fn test_proto_agent_to_agent_missing_config() {
        let state = serde_json::to_string(&AgentState::Idle).unwrap();

        let proto_agent = proto::Agent {
            id: "test-id".to_string(),
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            config_json: "".to_string(),
            state,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        };

        assert!(Agent::try_from(proto_agent).is_err());
    }

    #[test]
    fn test_agent_to_proto_agent() {
        let config = AgentConfig::new("test-model".to_string());
        let agent = Agent::new(
            "test-id".to_string(),
            "Test Agent".to_string(),
            "A test agent".to_string(),
            config,
        );

        let proto_agent = proto::Agent::from(agent);
        assert_eq!(proto_agent.id, "test-id");
        assert_eq!(proto_agent.name, "Test Agent");
        assert_eq!(proto_agent.description, "A test agent");
        assert!(!proto_agent.config_json.is_empty());
    }

    #[test]
    fn test_agent_builder_minimal() {
        let agent = AgentBuilder::new()
            .id("test-agent".to_string())
            .name("Test Agent".to_string())
            .description("A test agent".to_string())
            .config(AgentConfig::new("mock-model".to_string()))
            .build()
            .expect("Should build agent successfully");

        assert_eq!(agent.id, "test-agent");
        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.description, "A test agent");
        assert_eq!(agent.config.model_id, "mock-model");
        assert_eq!(agent.state, AgentState::Idle);
    }

    #[test]
    fn test_agent_builder_with_state() {
        let agent = AgentBuilder::new()
            .id("test-agent".to_string())
            .name("Test Agent".to_string())
            .description("A test agent".to_string())
            .config(AgentConfig::new("mock-model".to_string()))
            .state(AgentState::Running)
            .build()
            .expect("Should build agent successfully");

        assert_eq!(agent.state, AgentState::Running);
    }

    #[test]
    fn test_agent_builder_missing_id() {
        let result = AgentBuilder::new()
            .name("Test Agent".to_string())
            .description("A test agent".to_string())
            .config(AgentConfig::new("mock-model".to_string()))
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("id is required"));
    }

    #[test]
    fn test_agent_builder_missing_name() {
        let result = AgentBuilder::new()
            .id("test-agent".to_string())
            .description("A test agent".to_string())
            .config(AgentConfig::new("mock-model".to_string()))
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name is required"));
    }

    #[test]
    fn test_agent_builder_missing_description() {
        let result = AgentBuilder::new()
            .id("test-agent".to_string())
            .name("Test Agent".to_string())
            .config(AgentConfig::new("mock-model".to_string()))
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("description is required"));
    }

    #[test]
    fn test_agent_builder_missing_config() {
        let result = AgentBuilder::new()
            .id("test-agent".to_string())
            .name("Test Agent".to_string())
            .description("A test agent".to_string())
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("config is required"));
    }

    #[test]
    fn test_agent_builder_validation() {
        let result = AgentBuilder::new()
            .id("".to_string()) // Empty ID should fail validation
            .name("Test Agent".to_string())
            .description("A test agent".to_string())
            .config(AgentConfig::new("mock-model".to_string()))
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_agent_proto_round_trip() {
        let config = AgentConfig::new("test-model".to_string());
        let original_agent = Agent::new(
            "test-id".to_string(),
            "Test Agent".to_string(),
            "A test agent".to_string(),
            config,
        );

        let proto_agent = proto::Agent::from(original_agent.clone());
        let converted_agent = Agent::try_from(proto_agent).unwrap();

        assert_eq!(original_agent.id, converted_agent.id);
        assert_eq!(original_agent.name, converted_agent.name);
        assert_eq!(original_agent.description, converted_agent.description);
        assert_eq!(original_agent.config.model_id, converted_agent.config.model_id);
    }
}
