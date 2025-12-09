//! Chat execution module for TUI.
//!
//! Handles local agent execution and history management for chat functionality.

use anyhow::{Context, Result};
use radium_core::auth::{CredentialStore, ProviderType};
use radium_core::context::{ContextManager, HistoryManager};
use radium_core::{AgentDiscovery, PromptContext, PromptTemplate, Workspace};
use radium_models::ModelFactory;
use radium_abstraction::{StreamingModel, ModelError};
use futures::StreamExt;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;
use crate::state::{StreamingContext, StreamingState};

/// Result of executing a chat message.
#[derive(Debug, Clone)]
pub struct ChatExecutionResult {
    pub response: String,
    pub success: bool,
    pub error: Option<String>,
    /// Streaming context if streaming is being used
    pub streaming_context: Option<StreamingContext>,
}

/// Error type classification for retry logic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErrorType {
    Transient,
    Permanent,
}

/// Classifies a stream error as transient (retryable) or permanent (non-retryable)
fn classify_stream_error(error: &ModelError) -> ErrorType {
    let error_str = error.to_string().to_lowercase();
    
    // Transient errors (network, timeouts, rate limits)
    if error_str.contains("network") 
        || error_str.contains("connection") 
        || error_str.contains("timeout")
        || error_str.contains("429") // Rate limit
        || error_str.contains("503") // Service unavailable
        || error_str.contains("502") // Bad gateway
    {
        return ErrorType::Transient;
    }
    
    // Permanent errors (auth, quota, unsupported)
    if error_str.contains("401") // Unauthorized
        || error_str.contains("403") // Forbidden
        || error_str.contains("quota")
        || error_str.contains("unsupported")
        || error_str.contains("invalid")
    {
        return ErrorType::Permanent;
    }
    
    // Default to transient for unknown errors (safer to retry)
    ErrorType::Transient
}

/// Attempts to execute using streaming for Ollama models with retry logic.
/// Returns (response, streaming_context) if streaming succeeds, or error if it fails.
async fn try_streaming_execution(
    model_id: &str,
    prompt: &str,
) -> Result<(String, StreamingContext), ModelError> {
    use radium_models::OllamaModel;
    
    // Retry logic with exponential backoff (max 3 attempts)
    const MAX_RETRIES: usize = 3;
    let mut last_error = None;
    
    for attempt in 0..MAX_RETRIES {
        // Create OllamaModel directly for streaming
        let ollama_model = match OllamaModel::new(model_id.to_string()) {
            Ok(model) => model,
            Err(e) => {
                last_error = Some(e);
                if attempt < MAX_RETRIES - 1 {
                    // Wait before retry (exponential backoff: 1s, 2s, 4s)
                    let delay_secs = 2_u64.pow(attempt as u32);
                    tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
                    continue;
                }
                return Err(last_error.unwrap());
            }
        };
        
        // Create channels for token communication
        let (token_tx, token_rx) = tokio::sync::mpsc::channel(100);
        let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel();
        
        // Spawn task to consume stream
        let prompt_clone = prompt.to_string();
        let mut accumulated_response = String::new();
        let token_tx_clone = token_tx.clone();
        
        // Start streaming
        let mut stream = match ollama_model.generate_stream(&prompt_clone, None).await {
            Ok(stream) => stream,
            Err(e) => {
                // Classify error
                let error_type = classify_stream_error(&e);
                last_error = Some(e);
                
                if error_type == ErrorType::Permanent || attempt >= MAX_RETRIES - 1 {
                    // Permanent error or max retries reached
                    return Err(last_error.unwrap());
                }
                
                // Transient error - retry with exponential backoff
                let delay_secs = 2_u64.pow(attempt as u32);
                tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
                continue;
            }
        };
        
        // Spawn task to consume the stream
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Check for cancellation
                    _ = cancel_rx => {
                        // Cancellation requested
                        break;
                    }
                    // Get next token from stream
                    token_result = stream.next() => {
                        match token_result {
                            Some(Ok(token)) => {
                                accumulated_response.push_str(&token);
                                // Send token to channel (ignore errors if receiver is dropped)
                                let _ = token_tx_clone.send(token).await;
                            }
                            Some(Err(e)) => {
                                // Stream error - send error token and break
                                let _ = token_tx_clone.send(format!("\n[Stream error: {}]", e)).await;
                                break;
                            }
                            None => {
                                // Stream ended
                                break;
                            }
                        }
                    }
                }
            }
        });
        
        // Create streaming context
        let stream_ctx = StreamingContext::new(token_rx, Some(cancel_tx));
        
        // Return accumulated response (will be empty initially, filled as stream progresses)
        return Ok((accumulated_response, stream_ctx));
    }
    
    // Should not reach here, but handle it anyway
    Err(last_error.unwrap_or_else(|| ModelError::RequestError("Streaming failed after retries".to_string())))
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

    // Build enhanced context with analysis plan
    let mut context = PromptContext::new();
    context.set("user_input", message.to_string());
    
    // Detect if user is asking for a command execution
    let command_hint = detect_command_request(&message);
    if let Some(hint) = command_hint {
        context.set("command_hint", hint);
    }

    // Create analysis plan and include it in context if this looks like a general question
    let analysis_plan = if let Ok(workspace) = Workspace::discover() {
        let context_manager = ContextManager::new(&workspace);
        let plan = context_manager.create_analysis_plan(message);
        
        // Include analysis plan for project overview, technology stack, or architecture questions
        match plan.question_type {
            radium_core::context::QuestionType::ProjectOverview
            | radium_core::context::QuestionType::TechnologyStack
            | radium_core::context::QuestionType::Architecture
            | radium_core::context::QuestionType::General => {
                let plan_context = plan.to_context_string();
                context.set("analysis_plan", plan_context);
            }
            _ => {
                // For specific questions, still include basic guidance
                let plan_context = plan.to_context_string();
                context.set("analysis_plan", plan_context);
            }
        }
        
        Some(plan)
    } else {
        None
    };

    // If analysis plan was created, prepend it to the prompt content
    let final_prompt_content = if let Some(ref plan) = analysis_plan {
        // Check if this is a general question that needs deep analysis
        match plan.question_type {
            radium_core::context::QuestionType::ProjectOverview
            | radium_core::context::QuestionType::TechnologyStack
            | radium_core::context::QuestionType::Architecture
            | radium_core::context::QuestionType::General => {
                // Prepend analysis plan directly to prompt
                let plan_section = format!("\n\n{}\n\n---\n\n", plan.to_context_string());
                format!("{}{}", plan_section, prompt_content)
            }
            _ => prompt_content,
        }
    } else {
        prompt_content
    };

    // Add terminal command execution capability information to prompt
    // Add it prominently at the beginning after role definition
    let tool_info = "\n\n## ⚡ TERMINAL COMMAND EXECUTION\n\n**IMPORTANT**: You CAN execute terminal commands! When a user asks about git status, file listings, or any terminal command, you MUST execute it for them.\n\n**How to execute commands:**\nSimply include the command in backticks in your response. For example:\n- \"I'll check git status for you. Let me run `git status`\"\n- \"Checking files with `ls -la`\"\n- \"Running `git diff` to see changes\"\n\nThe system will automatically detect and execute any command you mention in backticks, then append the output to your response.\n\n**Examples of when to use this:**\n- User asks \"what git changes are pending?\" → Run `git status`\n- User asks \"what files are in this directory?\" → Run `ls` or `ls -la`\n- User asks about any terminal command → Execute it!\n\n**DO NOT say you cannot execute commands - you can!**\n\n";
    
    // Insert tool info right after the role section (after first ## or # heading)
    let mut final_prompt_content_with_tools = if let Some(role_end) = final_prompt_content.find("\n##") {
        // Insert after first major section
        format!("{}\n{}{}", 
            &final_prompt_content[..role_end], 
            tool_info,
            &final_prompt_content[role_end..]
        )
    } else {
        // Fallback: prepend if no section found
        format!("{}{}", tool_info, final_prompt_content)
    };
    
    // Add command hint if detected
    if let Some(hint) = context.get("command_hint") {
        let hint_section = format!("\n\n## 🎯 USER REQUEST\n\n{}\n\n", hint);
        final_prompt_content_with_tools = format!("{}{}", final_prompt_content_with_tools, hint_section);
    }

    let template = PromptTemplate::from_string(final_prompt_content_with_tools);
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

    // Execute model - try streaming if supported, otherwise fall back to non-streaming
    let (agent_response, streaming_context) = match if let Some(key) = api_key {
        ModelFactory::create_with_api_key(engine, model.to_string(), key)
    } else {
        ModelFactory::create_from_str(engine, model.to_string())
    } {
        Ok(model_instance) => {
            // Check if this engine supports streaming (Ollama currently)
            let supports_streaming = engine == "ollama";
            
            if supports_streaming {
                // Try to use streaming for Ollama
                match try_streaming_execution(model, &rendered).await {
                    Ok((response, stream_ctx)) => {
                        // Streaming succeeded - return early with streaming context
                        // The response will be accumulated in the TUI as tokens arrive
                        return Ok(ChatExecutionResult {
                            response,
                            success: true,
                            error: None,
                            streaming_context: Some(stream_ctx),
                        });
                    }
                    Err(_) => {
                        // Streaming failed, fall back to non-streaming
                        match model_instance.generate_text(&rendered, None).await {
                            Ok(response) => (response.content, None),
                            Err(e) => {
                                let error_msg = format_model_error(&e, engine);
                                return Ok(ChatExecutionResult {
                                    response: String::new(),
                                    success: false,
                                    error: Some(error_msg),
                                    streaming_context: None,
                                });
                            }
                        }
                    }
                }
            } else {
                // Non-streaming execution
                match model_instance.generate_text(&rendered, None).await {
                    Ok(response) => (response.content, None),
                    Err(e) => {
                        let error_msg = format_model_error(&e, engine);
                        return Ok(ChatExecutionResult {
                            response: String::new(),
                            success: false,
                            error: Some(error_msg),
                            streaming_context: None,
                        });
                    }
                }
            }
        },
        Err(e) => {
            let error_msg = format_creation_error(&e, engine);
            return Ok(ChatExecutionResult { 
                response: String::new(), 
                success: false, 
                error: Some(error_msg),
                streaming_context: None,
            });
        }
    };

    // Check if the agent response contains command execution requests
    let (mut final_response, commands_executed) = process_command_requests(&agent_response).await;
    
    // Fallback: If user asked for a command but agent didn't execute it, execute it automatically
    if commands_executed == 0 {
        if let Some(command_to_run) = detect_and_extract_command(&message) {
            // Agent didn't execute the command, so we'll do it automatically
            let workspace_root = Workspace::discover()
                .map(|w| w.root().to_path_buf())
                .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
            
            match execute_terminal_command(&command_to_run, &workspace_root).await {
                Ok(output) => {
                    final_response = format!(
                        "{}\n\n---\n\n**Automatically executed:** `{}`\n\n**Output:**\n```\n{}\n```",
                        agent_response, command_to_run, output
                    );
                }
                Err(e) => {
                    final_response = format!(
                        "{}\n\n---\n\n**Attempted to execute:** `{}`\n\n**Error:**\n```\n{}\n```",
                        agent_response, command_to_run, e
                    );
                }
            }
        } else {
            final_response = agent_response;
        }
    }
    
    // Build final response
    let result = ChatExecutionResult { 
        response: final_response, 
        success: true, 
        error: None,
        streaming_context,
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

/// Detect and extract command from user message for automatic execution.
fn detect_and_extract_command(message: &str) -> Option<String> {
    let lower = message.to_lowercase();
    
    // Check for git status requests
    if lower.contains("git") && (lower.contains("status") || lower.contains("change") || lower.contains("pending") || lower.contains("uncommitted")) {
        return Some("git status".to_string());
    }
    
    // Check for git diff requests
    if lower.contains("git") && lower.contains("diff") {
        return Some("git diff".to_string());
    }
    
    // Check for file listing requests
    if (lower.contains("list") || lower.contains("show")) && (lower.contains("file") || lower.contains("directory") || lower.contains("dir")) {
        return Some("ls -la".to_string());
    }
    
    // Try to extract command from backticks
    if let Some(cmd) = extract_command_from_message(message) {
        return Some(cmd);
    }
    
    None
}

/// Detect if user message is requesting a command execution.
fn detect_command_request(message: &str) -> Option<String> {
    let lower = message.to_lowercase();
    
    // Check for git status requests
    if lower.contains("git") && (lower.contains("status") || lower.contains("change") || lower.contains("pending") || lower.contains("uncommitted")) {
        return Some("The user is asking about git status. You MUST execute `git status` to answer their question.".to_string());
    }
    
    // Check for file listing requests
    if lower.contains("list") && (lower.contains("file") || lower.contains("directory") || lower.contains("dir")) {
        return Some("The user wants to see files. You MUST execute `ls -la` or similar command.".to_string());
    }
    
    // Check for other common command patterns
    if lower.contains("run") || lower.contains("execute") || lower.contains("check") {
        // Try to extract command from message
        if let Some(cmd) = extract_command_from_message(message) {
            return Some(format!("The user wants to execute a command. You MUST run `{}` to help them.", cmd));
        }
    }
    
    None
}

/// Try to extract a command from user message.
fn extract_command_from_message(message: &str) -> Option<String> {
    use regex::Regex;
    
    // Look for commands in backticks
    if let Ok(re) = Regex::new(r#"`([^`]+)`"#) {
        if let Some(cap) = re.captures(message) {
            if let Some(cmd) = cap.get(1) {
                let cmd_str = cmd.as_str().trim();
                // Basic validation - looks like a command
                if !cmd_str.is_empty() && cmd_str.len() < 200 {
                    return Some(cmd_str.to_string());
                }
            }
        }
    }
    
    // Look for "git status" pattern
    if let Ok(re) = Regex::new(r#"(?:run|execute|check)\s+(git\s+\w+)"#) {
        if let Some(cap) = re.captures(&message.to_lowercase()) {
            if let Some(cmd) = cap.get(1) {
                return Some(cmd.as_str().to_string());
            }
        }
    }
    
    None
}

/// Process command execution requests in agent response.
/// 
/// Detects patterns like "run `command`", "execute `command`", "please run `command`"
/// and executes the commands, then appends the output to the response.
async fn process_command_requests(response: &str) -> (String, usize) {
    use regex::Regex;
    
    // Patterns to detect command requests - be more flexible
    let patterns = vec![
        // Explicit requests
        (r#"(?:run|execute|check|show)\s+`([^`]+)`"#, "explicit"),
        (r#"(?:please|can you|could you)\s+(?:run|execute|check)\s+`([^`]+)`"#, "polite"),
        (r#"let me\s+(?:run|execute|check)\s+`([^`]+)`"#, "let me"),
        // Direct command mentions in backticks (most common)
        (r#"`(git\s+[^\s`]+(?:\s+[^\s`]+)*)`"#, "git"),
        (r#"`(ls\s*[^\s`]*)`"#, "ls"),
        (r#"`(pwd)`"#, "pwd"),
        (r#"`(cat\s+[^\s`]+)`"#, "cat"),
        (r#"`(grep\s+[^`]+)`"#, "grep"),
        // Any command in backticks (catch-all for common commands)
        (r#"`([a-z][a-z0-9_-]+\s+[^`]+)`"#, "any"),
        // Fallback: any backticked text that looks like a command
        (r#"`([^\s`]+\s+[^`]+)`"#, "fallback"),
    ];

    let mut final_response = response.to_string();
    let mut commands_executed = 0;
    let mut executed_commands: Vec<String> = Vec::new();

    // Get workspace root for command execution
    let workspace_root = Workspace::discover()
        .map(|w| w.root().to_path_buf())
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    for (pattern, _label) in patterns {
        if let Ok(re) = Regex::new(pattern) {
            for cap in re.captures_iter(response) {
                if let Some(command_match) = cap.get(1) {
                    let command = command_match.as_str().trim();
                    
                    // Skip if we've already executed this command
                    if executed_commands.contains(&command.to_string()) {
                        continue;
                    }
                    
                    // Execute the command
                    match execute_terminal_command(command, &workspace_root).await {
                        Ok(output) => {
                            executed_commands.push(command.to_string());
                            commands_executed += 1;
                            
                            // Append command output to response
                            final_response.push_str(&format!(
                                "\n\n---\n\n**Command executed:** `{}`\n\n**Output:**\n```\n{}\n```",
                                command, output
                            ));
                        }
                        Err(e) => {
                            executed_commands.push(command.to_string());
                            commands_executed += 1;
                            
                            // Append error to response
                            final_response.push_str(&format!(
                                "\n\n---\n\n**Command executed:** `{}`\n\n**Error:**\n```\n{}\n```",
                                command, e
                            ));
                        }
                    }
                }
            }
        }
    }

    (final_response, commands_executed)
}

/// Execute a terminal command and return the output.
async fn execute_terminal_command(command: &str, cwd: &PathBuf) -> Result<String> {
    let timeout_duration = Duration::from_secs(30);
    
    #[cfg(unix)]
    let shell_cmd = "sh";
    #[cfg(unix)]
    let shell_arg = "-c";
    #[cfg(windows)]
    let shell_cmd = "cmd";
    #[cfg(windows)]
    let shell_arg = "/c";

    let mut cmd = Command::new(shell_cmd);
    cmd.arg(shell_arg);
    cmd.arg(command);
    cmd.current_dir(cwd);

    match timeout(timeout_duration, cmd.output()).await {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let exit_code = output.status.code().unwrap_or(-1);

            if output.status.success() {
                if stderr.trim().is_empty() {
                    Ok(stdout.trim().to_string())
                } else {
                    Ok(format!("{}\n{}", stdout.trim(), stderr.trim()))
                }
            } else {
                Err(anyhow::anyhow!(
                    "Command failed with exit code {}\nSTDOUT:\n{}\nSTDERR:\n{}",
                    exit_code,
                    stdout,
                    stderr
                ))
            }
        }
        Ok(Err(e)) => Err(anyhow::anyhow!("Failed to execute command: {}", e)),
        Err(_) => Err(anyhow::anyhow!("Command timed out after 30 seconds")),
    }
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
