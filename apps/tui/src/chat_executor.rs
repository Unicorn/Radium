//! Chat execution module for TUI.
//!
//! Handles local agent execution and history management for chat functionality.

use anyhow::{Context, Result};
use radium_core::auth::{CredentialStore, ProviderType};
use radium_core::context::HistoryManager;
use radium_core::{AgentDiscovery, PromptContext, PromptTemplate, Workspace};
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

    let agent =
        agents.get(agent_id).ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", agent_id))?;

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

    // Load API key from CredentialStore
    let api_key = if let Ok(store) = CredentialStore::new() {
        let provider = match engine {
            "gemini" => ProviderType::Gemini,
            "openai" => ProviderType::OpenAI,
            _ => ProviderType::Gemini, // default
        };
        store.get(provider).ok()
    } else {
        None
    };

    // Execute model
    let result = match if let Some(key) = api_key {
        ModelFactory::create_with_api_key(engine, model.to_string(), key)
    } else {
        ModelFactory::create_from_str(engine, model.to_string())
    } {
        Ok(model_instance) => match model_instance.generate_text(&rendered, None).await {
            Ok(response) => {
                ChatExecutionResult { response: response.content, success: true, error: None }
            }
            Err(e) => {
                let error_msg = format_model_error(&e, engine);
                ChatExecutionResult {
                    response: String::new(),
                    success: false,
                    error: Some(error_msg),
                }
            }
        },
        Err(e) => {
            let error_msg = format_creation_error(&e, engine);
            ChatExecutionResult { response: String::new(), success: false, error: Some(error_msg) }
        }
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
///
/// Search order (precedence from highest to lowest):
/// 1. Absolute path (if provided)
/// 2. Relative to current directory
/// 3. Relative to workspace root
/// 4. Relative to home directory (.radium/)
/// 5. Extension prompt directories (project-level, then user-level)
fn load_prompt(prompt_path: &PathBuf) -> Result<String> {
    use radium_core::extensions::integration::get_extension_prompt_dirs;

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

    // Try extension prompt directories (lowest precedence)
    // Extract just the filename from the path to search in extension directories
    if let Some(file_name) = prompt_path.file_name() {
        if let Ok(extension_dirs) = get_extension_prompt_dirs() {
            for ext_dir in extension_dirs {
                let ext_prompt_path = ext_dir.join(file_name);
                if ext_prompt_path.exists() {
                    return Ok(fs::read_to_string(ext_prompt_path)?);
                }
            }
        }
    }

    anyhow::bail!("Prompt file not found: {}", prompt_path.display())
}

/// Get list of available agents.
pub fn get_available_agents() -> Result<Vec<(String, String)>> {
    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all()?;

    Ok(agents.into_iter().map(|(id, config)| (id, config.name)).collect())
}

/// Format model creation errors with helpful guidance.
fn format_creation_error(error: &radium_abstraction::ModelError, engine: &str) -> String {
    let error_str = error.to_string();

    // Check for authentication errors
    if error_str.contains("API_KEY") || error_str.contains("environment variable not set") {
        let provider = engine.to_uppercase();
        return format!(
            "⚠️  Authentication Required\n\n\
            No {} API key found. You need to authenticate before chatting.\n\n\
            Quick fix:\n\
            rad auth login {}\n\n\
            Or set environment variable:\n\
            export {}_API_KEY='your-key-here'\n\n\
            Press 'a' to authenticate, or restart after setting up auth.",
            provider, engine, provider
        );
    }

    // Check for unsupported provider
    if error_str.contains("Unsupported Model Provider") {
        return format!(
            "⚠️  Unsupported Provider\n\n\
            The '{}' provider is not supported or not configured.\n\n\
            Supported providers:\n\
            • gemini (Google Gemini)\n\
            • openai (OpenAI GPT)\n\n\
            Try:\n\
            rad auth login gemini\n\
            rad auth login openai",
            engine
        );
    }

    // Generic error
    format!(
        "❌ Model Creation Failed\n\n\
        {}\n\n\
        This could be due to:\n\
        • Missing or invalid API key\n\
        • Network connectivity issues\n\
        • Unsupported model configuration\n\n\
        Try: rad auth status",
        error_str
    )
}

/// Format model execution errors with helpful guidance.
fn format_model_error(error: &radium_abstraction::ModelError, engine: &str) -> String {
    let error_str = error.to_string();

    // Check for rate limiting
    if error_str.contains("429") || error_str.contains("rate limit") {
        return format!(
            "⏳ Rate Limit Exceeded\n\n\
            You've hit the API rate limit for {}.\n\n\
            Please wait a moment and try again.",
            engine
        );
    }

    // Check for invalid API key
    if error_str.contains("401") || error_str.contains("403") || error_str.contains("unauthorized")
    {
        return format!(
            "🔑 Authentication Failed\n\n\
            Your {} API key appears to be invalid.\n\n\
            Update your credentials:\n\
            rad auth login {}",
            engine, engine
        );
    }

    // Check for network errors
    if error_str.contains("network")
        || error_str.contains("connection")
        || error_str.contains("timeout")
    {
        return format!(
            "🌐 Network Error\n\n\
            Failed to connect to {} API.\n\n\
            Please check your internet connection and try again.",
            engine
        );
    }

    // Generic execution error
    format!(
        "❌ Model Execution Failed\n\n\
        {}\n\n\
        The agent encountered an error while processing your message.",
        error_str
    )
}
