// Agent Tool Registry - Converts agents to tools for orchestration
//
// This module loads agent configurations and converts them into tool definitions
// that orchestrators can invoke.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::tool::{Tool, ToolArguments, ToolHandler, ToolParameters, ToolResult};
use crate::error::Result;

/// Agent metadata from JSON config files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    /// Agent unique identifier
    pub id: String,
    /// Agent display name
    pub name: String,
    /// Agent description
    pub description: String,
    /// System prompt
    pub system_prompt: String,
    /// Model to use
    pub model: String,
    /// Engine (gemini, claude, openai)
    pub engine: String,
    /// Agent category
    #[serde(default)]
    pub category: String,
    /// Display color
    #[serde(default)]
    pub color: String,
}

/// Registry for converting agents to tools
pub struct AgentToolRegistry {
    /// Loaded agent metadata
    agents: HashMap<String, AgentMetadata>,
    /// Cached tools
    tools: Vec<Tool>,
    /// Optional orchestrator for agent execution
    orchestrator: Option<Arc<crate::Orchestrator>>,
}

impl AgentToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self { agents: HashMap::new(), tools: Vec::new(), orchestrator: None }
    }

    /// Create a new registry with an orchestrator for agent execution
    pub fn with_orchestrator(orchestrator: Arc<crate::Orchestrator>) -> Self {
        Self { agents: HashMap::new(), tools: Vec::new(), orchestrator: Some(orchestrator) }
    }

    /// Set the orchestrator for agent execution
    pub fn set_orchestrator(&mut self, orchestrator: Arc<crate::Orchestrator>) {
        self.orchestrator = Some(orchestrator);
        // Rebuild tools with the new orchestrator
        self.build_tools();
    }

    /// Load agents from default directories
    ///
    /// Searches:
    /// 1. `./agents/` (project-local agents)
    /// 2. `~/.radium/agents/` (user agents)
    ///
    /// # Errors
    ///
    /// Returns error if agent files cannot be read or parsed
    pub fn load_agents(&mut self) -> Result<()> {
        let search_paths = Self::default_search_paths();

        for path in search_paths {
            if path.exists() {
                self.load_agents_from_directory(&path)?;
            }
        }

        // Build tools after loading all agents
        self.build_tools();

        Ok(())
    }

    /// Load agents from a specific directory
    fn load_agents_from_directory(&mut self, dir: &Path) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "json" {
                        if let Ok(agent) = Self::load_agent_json(&path) {
                            self.agents.insert(agent.id.clone(), agent);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a single agent from JSON file
    fn load_agent_json(path: &Path) -> Result<AgentMetadata> {
        let content = fs::read_to_string(path)?;
        let agent: AgentMetadata = serde_json::from_str(&content)?;
        Ok(agent)
    }

    /// Get default search paths for agent directories
    fn default_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // 1. Project-local agents
        if let Ok(cwd) = std::env::current_dir() {
            paths.push(cwd.join("agents"));
        }

        // 2. User agents in home directory
        #[allow(clippy::disallowed_methods)]
        if let Ok(home) = std::env::var("HOME") {
            paths.push(PathBuf::from(home).join(".radium/agents"));
        }

        // 3. Workspace agents if RADIUM_WORKSPACE is set
        #[allow(clippy::disallowed_methods)]
        if let Ok(workspace) = std::env::var("RADIUM_WORKSPACE") {
            let workspace_path = PathBuf::from(workspace);
            paths.push(workspace_path.join("agents"));
            paths.push(workspace_path.join(".radium/agents"));
        }

        paths
    }

    /// Build tools from loaded agents
    fn build_tools(&mut self) {
        self.tools.clear();

        for (id, agent) in &self.agents {
            let tool = self.agent_to_tool(id, agent);
            self.tools.push(tool);
        }
    }

    /// Convert an agent metadata to a tool definition
    fn agent_to_tool(&self, id: &str, agent: &AgentMetadata) -> Tool {
        let parameters = ToolParameters::new()
            .add_property("task", "string", "The task for the agent to perform", true)
            .add_property("context", "string", "Additional context for the task (optional)", false);

        let handler = Arc::new(AgentToolHandler {
            agent_id: id.to_string(),
            agent_name: agent.name.clone(),
            orchestrator: self.orchestrator.clone(),
        });

        Tool::new(
            format!("agent_{}", id),
            agent.name.replace(' ', "_").to_lowercase(),
            format!("{}: {}", agent.name, agent.description),
            parameters,
            handler,
        )
    }

    /// Get all tools
    pub fn get_tools(&self) -> &[Tool] {
        &self.tools
    }

    /// Get agent by ID
    pub fn get_agent(&self, id: &str) -> Option<&AgentMetadata> {
        self.agents.get(id)
    }

    /// Refresh - reload agents and rebuild tools
    pub fn refresh(&mut self) -> Result<()> {
        self.agents.clear();
        self.load_agents()
    }

    /// Get number of loaded agents
    pub fn count(&self) -> usize {
        self.agents.len()
    }
}

impl Default for AgentToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Handler for agent tool execution
struct AgentToolHandler {
    agent_id: String,
    agent_name: String,
    orchestrator: Option<Arc<crate::Orchestrator>>,
}

#[async_trait]
impl ToolHandler for AgentToolHandler {
    async fn execute(&self, args: &ToolArguments) -> Result<ToolResult> {
        // Extract task from arguments
        let task = args.get_string("task").ok_or_else(|| {
            crate::error::OrchestrationError::InvalidToolArguments {
                tool: self.agent_name.clone(),
                reason: "Missing required 'task' argument".to_string(),
            }
        })?;

        let context_str = args.get_string("context").unwrap_or_default();

        // Build full input with context if provided
        let input = if context_str.is_empty() {
            task.clone()
        } else {
            format!("{}\n\nContext: {}", task, context_str)
        };

        // Execute the agent if orchestrator is available
        if let Some(ref orchestrator) = self.orchestrator {
            match orchestrator.execute_agent(&self.agent_id, &input).await {
                Ok(execution_result) => {
                    // Convert AgentOutput to string
                    let output_text = match execution_result.output {
                        crate::AgentOutput::Text(text) => text,
                        crate::AgentOutput::StructuredData(data) => {
                            serde_json::to_string_pretty(&data).unwrap_or_else(|_| format!("{:?}", data))
                        }
                        crate::AgentOutput::ToolCall { name, args } => {
                            format!("Tool call: {} with args: {:?}", name, args)
                        }
                        crate::AgentOutput::Terminate => "Agent terminated".to_string(),
                    };

                    if execution_result.success {
                        let mut result = ToolResult::success(&output_text)
                            .with_metadata("agent_id", &self.agent_id)
                            .with_metadata("agent_name", &self.agent_name);

                        // Add duration if available
                        if let Some(ref telemetry) = execution_result.telemetry {
                            if let Some(model_id) = &telemetry.model_id {
                                result = result.with_metadata("model_id", model_id);
                            }
                        }

                        Ok(result)
                    } else {
                        let error_msg = execution_result.error.unwrap_or_else(|| output_text.clone());
                        Ok(ToolResult::error(format!("Agent execution failed: {}", error_msg))
                            .with_metadata("agent_id", &self.agent_id)
                            .with_metadata("agent_name", &self.agent_name))
                    }
                }
                Err(e) => Ok(ToolResult::error(format!("Agent execution error: {}", e))
                    .with_metadata("agent_id", &self.agent_id)
                    .with_metadata("agent_name", &self.agent_name)),
            }
        } else {
            // No orchestrator - return placeholder
            let output = format!(
                "[Agent: {}] Task received: {}\nContext: {}\n(No orchestrator configured - placeholder mode)",
                self.agent_name, task, context_str
            );

            Ok(ToolResult::success(output)
                .with_metadata("agent_id", &self.agent_id)
                .with_metadata("agent_name", &self.agent_name)
                .with_metadata("placeholder", "true"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_agent_json(dir: &Path, id: &str, name: &str, description: &str) {
        let agent = AgentMetadata {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            system_prompt: "Test system prompt".to_string(),
            model: "gemini-2.0-flash-exp".to_string(),
            engine: "gemini".to_string(),
            category: "test".to_string(),
            color: "blue".to_string(),
        };

        let json = serde_json::to_string_pretty(&agent).unwrap();
        fs::write(dir.join(format!("{}.json", id)), json).unwrap();
    }

    #[test]
    fn test_load_agents_from_directory() {
        let temp = TempDir::new().unwrap();

        create_test_agent_json(
            temp.path(),
            "senior-developer",
            "Senior Developer",
            "Premium implementation specialist",
        );
        create_test_agent_json(temp.path(), "architect", "Architect", "System architecture expert");

        let mut registry = AgentToolRegistry::new();
        registry.load_agents_from_directory(temp.path()).unwrap();

        assert_eq!(registry.count(), 2);
        assert!(registry.get_agent("senior-developer").is_some());
        assert!(registry.get_agent("architect").is_some());
    }

    #[test]
    fn test_build_tools() {
        let temp = TempDir::new().unwrap();

        create_test_agent_json(
            temp.path(),
            "senior-developer",
            "Senior Developer",
            "Premium implementation specialist",
        );

        let mut registry = AgentToolRegistry::new();
        registry.load_agents_from_directory(temp.path()).unwrap();
        registry.build_tools();

        let tools = registry.get_tools();
        assert_eq!(tools.len(), 1);

        let tool = &tools[0];
        assert_eq!(tool.id, "agent_senior-developer");
        assert_eq!(tool.name, "senior_developer");
        assert!(tool.description.contains("Senior Developer"));
        assert!(tool.description.contains("Premium implementation specialist"));
    }

    #[test]
    fn test_agent_to_tool() {
        let agent = AgentMetadata {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            system_prompt: "Test prompt".to_string(),
            model: "gemini-2.0-flash-exp".to_string(),
            engine: "gemini".to_string(),
            category: "test".to_string(),
            color: "blue".to_string(),
        };

        let registry = AgentToolRegistry::new();
        let tool = registry.agent_to_tool("test-agent", &agent);

        assert_eq!(tool.id, "agent_test-agent");
        assert_eq!(tool.name, "test_agent");
        assert_eq!(tool.description, "Test Agent: A test agent");
        assert_eq!(tool.parameters.properties.len(), 2);
        assert!(tool.parameters.properties.contains_key("task"));
        assert!(tool.parameters.properties.contains_key("context"));
    }

    #[test]
    fn test_refresh() {
        let temp = TempDir::new().unwrap();

        create_test_agent_json(temp.path(), "agent1", "Agent 1", "First agent");

        let mut registry = AgentToolRegistry::new();
        registry.load_agents_from_directory(temp.path()).unwrap();
        assert_eq!(registry.count(), 1);

        // Add another agent
        create_test_agent_json(temp.path(), "agent2", "Agent 2", "Second agent");

        // Refresh to pick up new agent
        registry.agents.clear();
        registry.load_agents_from_directory(temp.path()).unwrap();
        assert_eq!(registry.count(), 2);
    }
}
