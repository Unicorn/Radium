//! Chat execution module for TUI.
//!
//! Handles local agent execution and history management for chat functionality.

use anyhow::{Context, Result};
use radium_core::{AgentDiscovery, PromptContext, PromptTemplate, Workspace};
use radium_core::context::HistoryManager;
use radium_models::ModelFactory;
use std::fs;
use std::path::PathBuf;

/// Result of executing a chat message.
#[derive(Debug, Clone)]
pub struct ChatExecutionResult {
    pub response: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Execute a chat message with an agent.
pub async fn execute_chat_message(
    agent_id: &str,
    message: &str,
    session_id: &str,
) -> Result<ChatExecutionResult> {
    // Discover agents
    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all().context("Failed to discover agents")?;

    let agent = agents
        .get(agent_id)
        .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", agent_id))?;

    // Load prompt template
    let prompt_content = load_prompt(&agent.prompt_path)?;

    // Render prompt with user message
    let mut context = PromptContext::new();
    context.set("user_input", message.to_string());

    let template = PromptTemplate::from_string(prompt_content);
    let rendered = template.render(&context)?;

    // Get model configuration
    let engine = agent.engine.as_deref().unwrap_or("gemini");
    let model = agent.model.as_deref().unwrap_or("gemini-2.0-flash-exp");

    // Execute model
    let result = match ModelFactory::create_from_str(engine, model.to_string()) {
        Ok(model_instance) => {
            match model_instance.generate_text(&rendered, None).await {
                Ok(response) => ChatExecutionResult {
                    response: response.content,
                    success: true,
                    error: None,
                },
                Err(e) => ChatExecutionResult {
                    response: String::new(),
                    success: false,
                    error: Some(format!("Model execution failed: {}", e)),
                },
            }
        }
        Err(e) => ChatExecutionResult {
            response: String::new(),
            success: false,
            error: Some(format!("Failed to create model: {}", e)),
        },
    };

    // Save to history if successful
    if result.success {
        if let Ok(workspace) = Workspace::discover() {
            let history_dir = workspace.root().join(".radium/_internals/history");
            let _ = std::fs::create_dir_all(&history_dir);

            if let Ok(mut history) = HistoryManager::new(&history_dir) {
                let _ = history.add_interaction(
                    Some(session_id),
                    message.to_string(),
                    "chat".to_string(),
                    result.response.clone(),
                );
            }
        }
    }

    Ok(result)
}

/// Load prompt from file.
fn load_prompt(prompt_path: &PathBuf) -> Result<String> {
    // Try as absolute path first
    if prompt_path.is_absolute() && prompt_path.exists() {
        return Ok(fs::read_to_string(prompt_path)?);
    }

    // Try relative to current directory
    if prompt_path.exists() {
        return Ok(fs::read_to_string(prompt_path)?);
    }

    // Try relative to workspace
    if let Ok(workspace) = Workspace::discover() {
        let workspace_path = workspace.root().join(prompt_path);
        if workspace_path.exists() {
            return Ok(fs::read_to_string(workspace_path)?);
        }
    }

    // Try relative to home directory
    if let Ok(home) = std::env::var("HOME") {
        let home_path = std::path::PathBuf::from(home).join(".radium").join(prompt_path);
        if home_path.exists() {
            return Ok(fs::read_to_string(home_path)?);
        }
    }

    anyhow::bail!("Prompt file not found: {}", prompt_path.display())
}

/// Get list of available agents.
pub fn get_available_agents() -> Result<Vec<(String, String)>> {
    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all()?;

    Ok(agents
        .into_iter()
        .map(|(id, config)| (id, config.name))
        .collect())
}
