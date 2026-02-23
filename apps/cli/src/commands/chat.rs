//! Interactive chat mode for conversational agent interaction.
//!
//! Provides a REPL-style interface for multi-turn conversations with agents,
//! maintaining session history and context across interactions.

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use colored::*;
use radium_abstraction::{ChatMessage, MessageContent, ToolConfig, ToolUseMode};
use radium_core::{
    analytics::{ReportFormatter, SessionAnalytics, SessionReport, SessionStorage},
    auth::{CredentialStore, ProviderType},
    context::{ContextFileLoader, HistoryManager},
    monitoring::MonitoringService,
    mcp::{McpIntegration, SlashCommandRegistry},
    session::{Session, SessionStorage as ChatSessionStorage},
    session::state::Message as SessionMessage,
    Workspace,
    code_blocks::CodeBlockStore,
};
use radium_orchestrator::orchestration::tool_builder::build_standard_tools;
use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::tool_execution;
use crate::conversation_context::ConversationContext;

/// Execute the chat command
///
/// # Arguments
/// * `agent_id` - The agent to chat with
/// * `session_name` - Optional session name (defaults to timestamp-based)
/// * `resume` - Whether to resume an existing session
pub async fn execute(agent_id: String, session_name: Option<String>, resume: bool, _stream: bool, _show_metadata: bool, _json: bool, _safety_behavior: Option<String>) -> Result<()> {
    // Get workspace
    let workspace =
        Workspace::discover().context("Failed to load workspace. Run 'rad init' first.")?;

    // Validate agent exists up-front to avoid entering the interactive loop for invalid IDs.
    // This is important for non-interactive callers (tests/CI) and for good UX.
    {
        use radium_core::AgentDiscovery;
        let discovery = AgentDiscovery::new();
        let agents = discovery.discover_all().context("Failed to discover agents")?;
        if !agents.contains_key(&agent_id) {
            return Err(anyhow!("Agent '{}' not found", agent_id));
        }
    }

    // Initialize persistent session storage
    let chat_session_storage = ChatSessionStorage::new(workspace.root())
        .context("Failed to initialize session storage")?;

    // Initialize history manager
    let history_dir = workspace.root().join(".radium/_internals/history");
    std::fs::create_dir_all(&history_dir)?;
    let mut history = HistoryManager::new(&history_dir)?;

    // Initialize conversation context tracker
    let mut conversation_context = ConversationContext::new();
    let mut turn_number = 0;

    // Determine session ID (use same ID for both chat history and analytics)
    let session_id = if resume {
        if let Some(name) = session_name {
            name
        } else {
            return Err(anyhow!("--resume requires a session name"));
        }
    } else {
        session_name
            .unwrap_or_else(|| format!("{}_{}", agent_id, Utc::now().format("%Y%m%d_%H%M%S")))
    };
    
    // Generate analytics session ID (UUID-based for analytics tracking)
    let analytics_session_id = Uuid::new_v4().to_string();
    let session_start_time = Utc::now();
    let mut executed_agent_ids = Vec::new();
    
    // Open monitoring service for agent tracking
    let monitoring_path = workspace.radium_dir().join("monitoring.db");
    let monitoring = MonitoringService::open(&monitoring_path).ok();

    // Load or create persistent session for cross-restart message persistence
    let mut persistent_session: Option<Session> = if resume {
        match chat_session_storage.load_session(&session_id) {
            Ok(session) => {
                let count = session.messages.len();
                if count > 0 {
                    println!("Loaded {} messages from previous session", count);
                }
                Some(session)
            }
            Err(_) => {
                // Fall back to HistoryManager check for old-style sessions
                let interactions = history.get_interactions(Some(&session_id));
                if interactions.is_empty() {
                    return Err(anyhow!("Session '{}' not found", session_id));
                }
                None
            }
        }
    } else {
        let new_session = Session::new(
            session_id.clone(),
            Some(agent_id.clone()),
            Some(workspace.root().to_string_lossy().to_string()),
        );
        let _ = chat_session_storage.save_session_metadata(&new_session);
        Some(new_session)
    };

    // Load context files once at session start
    let workspace_root = workspace.root().to_path_buf();
    let loader = ContextFileLoader::new(&workspace_root);
    let current_dir = std::env::current_dir().unwrap_or_else(|_| workspace_root.clone());
    let context_files = loader.load_hierarchical(&current_dir).unwrap_or_default();

    // Initialize MCP integration and load slash commands
    let mcp_integration = Arc::new(Mutex::new(McpIntegration::new()));
    let mut slash_registry = SlashCommandRegistry::new();
    if mcp_integration.lock().await.initialize(&workspace).await.is_ok() {
        // Discover MCP prompts and register them as slash commands
        let integration = mcp_integration.lock().await;
        let prompts = integration.get_all_prompts().await;
        for (server_name, prompt) in prompts {
            slash_registry.register_prompt_with_server(server_name, prompt);
        }
    }

    // Print welcome banner
    print_banner(&agent_id, &session_id, resume, &slash_registry)?;

    // Show conversation history if resuming
    if resume {
        print_history(
            persistent_session.as_ref().map(|s| s.messages.as_slice()),
            &history,
            &session_id,
        )?;
    }

    // Main chat loop
    loop {
        // Print prompt
        print!("\n{} ", ">".green().bold());
        io::stdout().flush()?;

        // Read user input
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        // Handle empty input
        if input.is_empty() {
            continue;
        }

        // Handle special commands
        match input {
            "/quit" | "/exit" | "/q" => {
                println!("\nGoodbye! Session saved as '{}'", session_id);
                
                // Generate and display session report
                if !executed_agent_ids.is_empty() {
                    let monitoring_path = workspace.radium_dir().join("monitoring.db");
                    if let Ok(monitoring) = MonitoringService::open(monitoring_path) {
                        let analytics = SessionAnalytics::new(monitoring);
                        let session_end_time = Some(Utc::now());
                        
                        if let Ok(metrics) = analytics.generate_session_metrics_with_workspace(
                            &analytics_session_id,
                            &executed_agent_ids,
                            session_start_time,
                            session_end_time,
                            Some(workspace.root()),
                        ) {
                            let report = SessionReport::new(metrics);
                            let storage = SessionStorage::new(workspace.root()).ok();
                            if let Some(ref storage) = storage {
                                let _ = storage.save_report(&report);
                                
                                // Get code block count for session
                                let block_count = CodeBlockStore::new(workspace.root(), session_id.clone())
                                    .ok()
                                    .and_then(|store| store.list_blocks(None).ok())
                                    .map(|blocks| blocks.len())
                                    .unwrap_or(0);
                                
                                display_session_summary(&report, block_count);
                            }
                        }
                    }
                }
                
                break;
            }
            "/help" | "/h" => {
                print_help(&slash_registry)?;
                continue;
            }
            "/mcp-commands" | "/mcp-help" => {
                print_mcp_commands(&slash_registry)?;
                continue;
            }
            "/history" => {
                print_history(
                    persistent_session.as_ref().map(|s| s.messages.as_slice()),
                    &history,
                    &session_id,
                )?;
                continue;
            }
            "/clear" => {
                history.clear_session(Some(&session_id))?;
                println!("Conversation history cleared.");
                continue;
            }
            "/save" => {
                println!("Session automatically saved as '{}'", session_id);
                
                // Generate and display session report on auto-save
                if !executed_agent_ids.is_empty() {
                    let monitoring_path = workspace.radium_dir().join("monitoring.db");
                    if let Ok(monitoring) = MonitoringService::open(monitoring_path) {
                        let analytics = SessionAnalytics::new(monitoring);
                        let session_end_time = Some(Utc::now());
                        
                        if let Ok(metrics) = analytics.generate_session_metrics_with_workspace(
                            &analytics_session_id,
                            &executed_agent_ids,
                            session_start_time,
                            session_end_time,
                            Some(workspace.root()),
                        ) {
                            let report = SessionReport::new(metrics);
                            let storage = SessionStorage::new(workspace.root()).ok();
                            if let Some(ref storage) = storage {
                                let _ = storage.save_report(&report);
                            }
                        }
                    }
                }
                continue;
            }
            _ => {
                // Check if it's an MCP slash command
                if input.starts_with('/') && !input.starts_with("//") {
                    if let Some(prompt) = slash_registry.get_command(input) {
                        // Execute MCP prompt
                        match execute_mcp_prompt(&mcp_integration, prompt, input, &slash_registry)
                            .await
                        {
                            Ok(result) => {
                                println!("\n{}", result);
                                continue;
                            }
                            Err(e) => {
                                eprintln!("\n{}: {}", "MCP Error".red().bold(), e);
                                continue;
                            }
                        }
                    } else {
                        // Try to discover and load MCP prompts if not found
                        if load_mcp_prompts(&mcp_integration, &mut slash_registry, &workspace)
                            .await
                            .is_ok()
                        {
                            if let Some(prompt) = slash_registry.get_command(input) {
                                match execute_mcp_prompt(
                                    &mcp_integration,
                                    prompt,
                                    input,
                                    &slash_registry,
                                )
                                .await
                                {
                                    Ok(result) => {
                                        println!("\n{}", result);
                                        continue;
                                    }
                                    Err(e) => {
                                        eprintln!("\n{}: {}", "MCP Error".red().bold(), e);
                                        continue;
                                    }
                                }
                            } else {
                                eprintln!(
                                    "\n{}: Unknown command '{}'. Use /help for available commands.",
                                    "Error".red().bold(),
                                    input
                                );
                                continue;
                            }
                        } else {
                            eprintln!(
                                "\n{}: Unknown command '{}'. Use /help for available commands.",
                                "Error".red().bold(),
                                input
                            );
                            continue;
                        }
                    }
                }
            }
        }

        // Get conversation context — prefer full SessionStorage history over HistoryManager summary
        let history_context = if let Some(ref session) = persistent_session {
            if !session.messages.is_empty() {
                format_session_history(&session.messages)
            } else {
                history.get_summary(Some(&session_id))
            }
        } else {
            history.get_summary(Some(&session_id))
        };

        // Build prompt with context files and history
        // History takes precedence (comes after context files)
        let _full_prompt = if !context_files.is_empty() && !history_context.is_empty() {
            format!(
                "# Context Files\n\n{}\n\n---\n\n# Conversation History\n\n{}\n\n---\n\nCurrent Request: {}",
                context_files, history_context, input
            )
        } else if !context_files.is_empty() {
            format!("# Context Files\n\n{}\n\n---\n\nCurrent Request: {}", context_files, input)
        } else if !history_context.is_empty() {
            format!("{}\n\nCurrent Request: {}", history_context, input)
        } else {
            input.to_string()
        };

        // Register agent in monitoring for session tracking
        let tracked_agent_id = format!("{}-{}", analytics_session_id, agent_id);
        if let Some(monitoring) = monitoring.as_ref() {
            use radium_core::monitoring::{AgentRecord, AgentStatus};
            let mut agent_record = AgentRecord::new(tracked_agent_id.clone(), agent_id.clone());
            agent_record.plan_id = Some(analytics_session_id.clone());
            if monitoring.register_agent(&agent_record).is_ok() {
                let _ = monitoring.update_status(&tracked_agent_id, AgentStatus::Running);
                executed_agent_ids.push(tracked_agent_id.clone());
            }
        }

        // Increment turn number
        turn_number += 1;

        // Execute chat turn with tool support
        let context_summary = conversation_context.to_system_context();
        match execute_chat_turn_with_tools(
            &workspace,
            &agent_id,
            input,
            &history_context,
            &context_files,
            &context_summary,
        )
        .await
        {
            Ok(response) => {
                // Print the response
                println!("\n{}", response);

                // Update conversation context (simple pattern matching for Phase 1)
                conversation_context.update_from_turn(
                    turn_number,
                    input,
                    &response,
                    &[], // Tool calls not tracked yet - will be added in future enhancement
                );

                // Complete agent in monitoring
                if let Some(monitoring) = monitoring.as_ref() {
                    let _ = monitoring.complete_agent(&tracked_agent_id, 0);
                }

                // Persist messages to SessionStorage
                if let Some(ref mut session) = persistent_session {
                    let user_msg = SessionMessage {
                        id: Uuid::new_v4().to_string(),
                        role: "user".to_string(),
                        content: input.to_string(),
                        timestamp: Utc::now(),
                    };
                    let assistant_msg = SessionMessage {
                        id: Uuid::new_v4().to_string(),
                        role: "assistant".to_string(),
                        content: response.clone(),
                        timestamp: Utc::now(),
                    };
                    let _ = chat_session_storage.append_message(&session_id, &user_msg);
                    let _ = chat_session_storage.append_message(&session_id, &assistant_msg);
                    session.messages.push(user_msg);
                    session.messages.push(assistant_msg);
                    session.touch();
                    let _ = chat_session_storage.save_session_metadata(session);
                }

                // Record interaction in history
                history.add_interaction(
                    Some(&session_id),
                    input.to_string(),
                    "chat".to_string(),
                    response,
                )?;
            }
            Err(e) => {
                // Mark agent as failed
                if let Some(monitoring) = monitoring.as_ref() {
                    let _ = monitoring.fail_agent(&tracked_agent_id, &e.to_string());
                }
                eprintln!("\n{}: {}", "Error".red().bold(), e);
            }
        }
    }

    Ok(())
}

/// Print welcome banner
fn print_banner(
    agent_id: &str,
    session_id: &str,
    resume: bool,
    slash_registry: &SlashCommandRegistry,
) -> Result<()> {
    println!();
    println!("{}", "╔═══════════════════════════════════════════╗".cyan().bold());
    println!(
        "{}{}{}",
        "║  ".cyan().bold(),
        "Radium Interactive Chat".white().bold(),
        "                 ║".cyan().bold()
    );
    println!("{}", "╚═══════════════════════════════════════════╝".cyan().bold());
    println!();

    println!("  {} {}", "Agent:  ".yellow().bold(), agent_id);
    println!("  {} {}", "Session:".yellow().bold(), session_id);

    if resume {
        println!("  {} {}", "Mode:   ".yellow().bold(), "Resuming previous conversation");
    }

    println!();
    let mcp_count = slash_registry.get_all_commands().len();
    if mcp_count > 0 {
        println!(
            "  {} {} {}",
            "Commands:".green().bold(),
            "/help /history /clear /save /quit",
            format!("(+{} MCP)", mcp_count).cyan().dimmed()
        );
    } else {
        println!("  {} {}", "Commands:".green().bold(), "/help /history /clear /save /quit");
    }
    println!();

    Ok(())
}

/// Display session summary at end of execution.
fn display_session_summary(report: &SessionReport, block_count: usize) {
    println!();
    println!("{}", "─".repeat(60).dimmed());
    println!("{}", "Session Summary".bold().cyan());
    println!("{}", "─".repeat(60).dimmed());
    
    let formatter = ReportFormatter;
    let summary = formatter.format(report);
    
    // Print a condensed version (first few lines)
    for line in summary.lines().take(15) {
        println!("{}", line);
    }
    
    // Display code block count
    if block_count > 0 {
        println!("  {} {} code blocks extracted", "📋".cyan(), block_count);
    }
    
    println!();
    println!("  {} Full report: {}", "💡".cyan(), format!("rad stats session {}", report.metrics.session_id).dimmed());
    println!("{}", "─".repeat(60).dimmed());
    println!();
}

/// Print conversation history.
///
/// Prefers showing full message pairs from `session_messages` (SessionStorage) when available;
/// falls back to HistoryManager interactions otherwise.
fn print_history(
    session_messages: Option<&[SessionMessage]>,
    history: &HistoryManager,
    session_id: &str,
) -> Result<()> {
    // Prefer rich SessionStorage messages (full user + assistant content)
    if let Some(msgs) = session_messages {
        if !msgs.is_empty() {
            println!();
            println!("{}", "═══ Conversation History ═══".cyan().bold());
            println!();

            let mut turn = 0;
            let mut i = 0;
            while i < msgs.len() {
                let msg = &msgs[i];
                if msg.role == "user" {
                    turn += 1;
                    let ts = msg.timestamp.format("%H:%M:%S");
                    println!(
                        "  {} {} {}",
                        format!("[{}]", turn).blue().bold(),
                        "You:".green().bold(),
                        msg.content
                    );

                    // Show the paired assistant response if present
                    if i + 1 < msgs.len() && msgs[i + 1].role == "assistant" {
                        let preview = if msgs[i + 1].content.len() > 120 {
                            format!("{}…", &msgs[i + 1].content[..120])
                        } else {
                            msgs[i + 1].content.clone()
                        };
                        println!("       {} {}", "Agent:".yellow().bold(), preview);
                        i += 2;
                    } else {
                        i += 1;
                    }
                    println!("       {}", ts.to_string().dimmed());
                    println!();
                } else {
                    i += 1;
                }
            }
            return Ok(());
        }
    }

    // Fallback: HistoryManager (user prompts only, no assistant content)
    let interactions = history.get_interactions(Some(session_id));
    if interactions.is_empty() {
        println!("\n{}", "No conversation history yet.".yellow());
        return Ok(());
    }

    println!();
    println!("{}", "═══ Conversation History ═══".cyan().bold());
    println!();

    for (i, interaction) in interactions.iter().enumerate() {
        let ts = interaction.timestamp.format("%H:%M:%S");
        println!(
            "  {} {} {}",
            format!("[{}]", i + 1).blue().bold(),
            "You:".green().bold(),
            interaction.goal
        );
        println!("       {}", ts.to_string().dimmed());
        println!();
    }

    Ok(())
}

/// Print help text
fn print_help(slash_registry: &SlashCommandRegistry) -> Result<()> {
    println!();
    println!("{}", "╔═══════════════════════════════════════════╗".cyan().bold());
    println!(
        "{}{}{}",
        "║  ".cyan().bold(),
        "Chat Commands".white().bold(),
        "                            ║".cyan().bold()
    );
    println!("{}", "╚═══════════════════════════════════════════╝".cyan().bold());
    println!();

    let commands = vec![
        ("/help, /h", "Show this help message"),
        ("/history", "Display conversation history"),
        ("/clear", "Clear conversation history for this session"),
        ("/save", "Confirm session is saved (auto-saves on each message)"),
        ("/quit, /exit, /q", "Exit chat and save session"),
    ];

    for (cmd, desc) in commands {
        println!("  {} - {}", cmd.green().bold(), desc);
    }

    let mcp_commands = slash_registry.get_all_commands();
    if !mcp_commands.is_empty() {
        println!();
        println!("  {} - List available MCP slash commands", "/mcp-commands".green().bold());
    }

    println!();
    println!(
        "{} Your conversation is automatically saved after each message.",
        "Tip:".yellow().bold()
    );
    println!();

    Ok(())
}

/// Print MCP commands
fn print_mcp_commands(slash_registry: &SlashCommandRegistry) -> Result<()> {
    let commands = slash_registry.get_all_commands();

    if commands.is_empty() {
        println!("\n{}", "No MCP commands available.".yellow());
        println!("{} Configure MCP servers to enable slash commands.", "Tip:".yellow().bold());
        return Ok(());
    }

    println!();
    println!("{}", "╔═══════════════════════════════════════════╗".cyan().bold());
    println!(
        "{}{}{}",
        "║  ".cyan().bold(),
        "MCP Slash Commands".white().bold(),
        "                        ║".cyan().bold()
    );
    println!("{}", "╚═══════════════════════════════════════════╝".cyan().bold());
    println!();

    for (cmd_name, prompt) in commands {
        let desc = prompt
            .description
            .as_ref()
            .map(|d| d.as_str())
            .unwrap_or("No description");
        println!("  {} - {}", cmd_name.green().bold(), desc);

        // Show arguments if available
        if let Some(args) = &prompt.arguments {
            if !args.is_empty() {
                for arg in args {
                    let required = if arg.required { "required" } else { "optional" };
                    let arg_desc = arg
                        .description
                        .as_ref()
                        .map(|d| d.as_str())
                        .unwrap_or("");
                    println!("      {} {}: {}", arg.name.cyan(), required.yellow(), arg_desc);
                }
            }
        }
    }

    println!();

    Ok(())
}

/// Load MCP prompts into the slash command registry
async fn load_mcp_prompts(
    mcp_integration: &Arc<Mutex<McpIntegration>>,
    registry: &mut SlashCommandRegistry,
    workspace: &Workspace,
) -> Result<()> {
    let integration = mcp_integration.lock().await;

    // Re-initialize if needed
    if integration.connected_server_count().await == 0 {
        integration.initialize(workspace).await?;
    }

    // Get all prompts from all servers
    let prompts = integration.get_all_prompts().await;
    for (server_name, prompt) in prompts {
        registry.register_prompt_with_server(server_name, prompt);
    }

    Ok(())
}

/// Execute an MCP prompt
async fn execute_mcp_prompt(
    mcp_integration: &Arc<Mutex<McpIntegration>>,
    prompt: &radium_core::mcp::McpPrompt,
    input: &str,
    slash_registry: &SlashCommandRegistry,
) -> Result<String> {
    // Parse arguments from input
    // Format: /command_name arg1 arg2 ...
    let parts: Vec<&str> = input.split_whitespace().collect();
    let args = if parts.len() > 1 {
        // Simple argument parsing - for now, just pass as key-value pairs
        // In a more complete implementation, we'd parse based on prompt.arguments
        let mut arg_map = serde_json::Map::new();
        for (i, part) in parts.iter().skip(1).enumerate() {
            if let Some(arg_def) = prompt.arguments.as_ref().and_then(|args| args.get(i)) {
                arg_map.insert(arg_def.name.clone(), serde_json::Value::String(part.to_string()));
            } else {
                // Fallback: use index as key
                arg_map.insert(format!("arg{}", i), serde_json::Value::String(part.to_string()));
            }
        }
        Some(serde_json::Value::Object(arg_map))
    } else {
        None
    };

    // Get server name from registry
    let server_name = slash_registry
        .get_server_for_command(input.split_whitespace().next().unwrap_or(""))
        .ok_or_else(|| anyhow!("Could not find server for command: {}", input))?;

    // Execute the prompt
    let integration = mcp_integration.lock().await;
    let result = integration
        .execute_prompt(server_name, &prompt.name, args)
        .await
        .map_err(|e| anyhow!("Failed to execute MCP prompt: {}", e))?;

    // Format the result for display
    // The result is a JSON value, extract text content if available
    if let Some(messages) = result.get("messages").and_then(|m| m.as_array()) {
        let mut output_parts = Vec::new();
        for message in messages {
            if let Some(content) = message.get("content").and_then(|c| c.as_array()) {
                for item in content {
                    if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                        output_parts.push(text.to_string());
                    }
                }
            } else if let Some(text) = message.get("content").and_then(|c| c.as_str()) {
                output_parts.push(text.to_string());
            }
        }
        if !output_parts.is_empty() {
            return Ok(output_parts.join("\n"));
        }
    }

    // Fallback: return JSON representation
    Ok(serde_json::to_string_pretty(&result)
        .unwrap_or_else(|_| format!("Prompt executed: {}", prompt.name)))
}

/// Execute a single chat turn with tool support
///
/// This function enables tool calling in conversational mode,
/// allowing the model to autonomously call tools as needed.
async fn execute_chat_turn_with_tools(
    workspace: &Workspace,
    agent_id: &str,
    user_input: &str,
    history_context: &str,
    context_files: &str,
    conversation_context: &str,
) -> Result<String> {
    // Load agent configuration using AgentDiscovery
    use radium_core::AgentDiscovery;
    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all()
        .context("Failed to discover agents")?;
    let agent = agents.get(agent_id)
        .with_context(|| format!("Agent '{}' not found", agent_id))?;

    // Get engine and model from agent config
    let engine_id = agent.engine.as_deref().unwrap_or("gemini");
    let model_id = agent.model.as_deref().unwrap_or("gemini-2.0-flash-exp");

    // Get API key from credential store
    let credential_store = Arc::new(
        CredentialStore::new().unwrap_or_else(|_| {
            let temp_path = std::env::temp_dir().join("radium_credentials.json");
            CredentialStore::with_path(temp_path)
        })
    );

    let provider_type = match engine_id {
        "claude" | "anthropic" => Some(ProviderType::Claude),
        "gemini" => Some(ProviderType::Gemini),
        "openai" => Some(ProviderType::OpenAI),
        "mock" => None,
        _ => return Err(anyhow!("Unsupported engine: {}", engine_id)),
    };

    let api_key = if let Some(provider) = provider_type {
        credential_store
            .get(provider)
            .map_err(|e| anyhow!("Failed to get API key for {}: {}", engine_id, e))?
    } else {
        String::new() // Mock doesn't need API key
    };

    // Create Model instance
    let model = tool_execution::create_model(engine_id, model_id, api_key)?;

    // Build tools
    let workspace_root = workspace.root().to_path_buf();
    let orchestration_tools = build_standard_tools(workspace_root.clone(), None);
    let abstraction_tools = tool_execution::convert_tools(&orchestration_tools);

    // Build tool config
    let tool_config = ToolConfig {
        mode: ToolUseMode::Auto,
        allowed_function_names: None, // Allow all tools
    };

    // Load system prompt from agent's prompt file
    let system_prompt = if agent.prompt_path.exists() {
        std::fs::read_to_string(&agent.prompt_path)
            .unwrap_or_else(|_| format!("You are {}. {}", agent.name, agent.description))
    } else {
        format!("You are {}. {}", agent.name, agent.description)
    };

    // Combine context and history
    let combined_context = if !context_files.is_empty() && !history_context.is_empty() {
        format!(
            "# Context Files\n\n{}\n\n---\n\n# Conversation History\n\n{}\n\n---",
            context_files, history_context
        )
    } else if !context_files.is_empty() {
        format!("# Context Files\n\n{}\n\n---", context_files)
    } else if !history_context.is_empty() {
        format!("# Conversation History\n\n{}\n\n---", history_context)
    } else {
        String::new()
    };

    // Build full system prompt with conversation context
    let full_system_prompt = if !combined_context.is_empty() && !conversation_context.is_empty() {
        format!("{}\n\n{}\n\n{}", system_prompt, combined_context, conversation_context)
    } else if !combined_context.is_empty() {
        format!("{}\n\n{}", system_prompt, combined_context)
    } else if !conversation_context.is_empty() {
        format!("{}\n\n{}", system_prompt, conversation_context)
    } else {
        system_prompt.to_string()
    };

    // Create initial messages
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: MessageContent::Text(full_system_prompt),
        },
        ChatMessage {
            role: "user".to_string(),
            content: MessageContent::Text(user_input.to_string()),
        },
    ];

    // Execute with tools loop
    let response = tool_execution::execute_with_tools_loop(
        model.as_ref(),
        messages,
        &abstraction_tools,
        &tool_config,
        &orchestration_tools,
        &workspace_root,
    ).await?;

    Ok(response.content)
}

/// Format loaded session messages into a history context string for the LLM.
fn format_session_history(messages: &[SessionMessage]) -> String {
    if messages.is_empty() {
        return String::new();
    }
    let mut out = String::from("Conversation History:\n");
    for msg in messages {
        out.push_str(&format!("{}: {}\n", msg.role, msg.content));
    }
    out
}

/// Format a UTC timestamp as a human-readable relative time (e.g. "2h ago", "3d ago").
fn format_relative_time(dt: chrono::DateTime<Utc>) -> String {
    let secs = Utc::now().signed_duration_since(dt).num_seconds();
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
    }
}

/// List available chat sessions
pub async fn list_sessions() -> Result<()> {
    let workspace =
        Workspace::discover().context("Failed to load workspace. Run 'rad init' first.")?;

    let chat_session_storage = ChatSessionStorage::new(workspace.root())
        .context("Failed to initialize session storage")?;

    let ids = chat_session_storage.list_session_ids()?;

    if ids.is_empty() {
        println!("No chat sessions found.");
        println!(
            "\n  {} Start a session with: {}",
            "Tip:".yellow().bold(),
            "rad chat <agent-id>".cyan()
        );
        return Ok(());
    }

    println!();
    println!("{}", "╔═══════════════════════════════════════════╗".cyan().bold());
    println!(
        "{}{}{}",
        "║  ".cyan().bold(),
        "Chat Sessions".white().bold(),
        "                            ║".cyan().bold()
    );
    println!("{}", "╚═══════════════════════════════════════════╝".cyan().bold());
    println!();

    // Sort sessions by last_active descending (most recent first)
    let mut sessions: Vec<_> = ids
        .iter()
        .filter_map(|id| {
            chat_session_storage.load_session(id).ok().map(|s| (id.clone(), s))
        })
        .collect();
    sessions.sort_by(|(_, a), (_, b)| b.last_active.cmp(&a.last_active));

    for (id, session) in &sessions {
        let msg_count = session.messages.iter().filter(|m| m.role == "user").count();
        let agent = session.agent_id.as_deref().unwrap_or("unknown");
        let age = format_relative_time(session.last_active);

        println!(
            "  {} {}  {}  {}",
            "•".green().bold(),
            id.white().bold(),
            format!("agent:{}", agent).cyan(),
            age.dimmed()
        );
        println!(
            "      {}",
            format!("{} turn{}", msg_count, if msg_count == 1 { "" } else { "s" }).yellow()
        );
        println!();
    }

    println!(
        "  {} Resume a session: {}",
        "Tip:".yellow().bold(),
        "rad chat <agent-id> --session <name> --resume".cyan()
    );
    println!();

    Ok(())
}
