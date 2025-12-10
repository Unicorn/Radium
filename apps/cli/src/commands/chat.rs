//! Interactive chat mode for conversational agent interaction.
//!
//! Provides a REPL-style interface for multi-turn conversations with agents,
//! maintaining session history and context across interactions.

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use colored::*;
use radium_core::{
    analytics::{ReportFormatter, SessionAnalytics, SessionReport, SessionStorage},
    context::{ContextFileLoader, HistoryManager},
    monitoring::MonitoringService,
    mcp::{McpIntegration, SlashCommandRegistry},
    Workspace,
    code_blocks::CodeBlockStore,
};
use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::step;

/// Execute the chat command
///
/// # Arguments
/// * `agent_id` - The agent to chat with
/// * `session_name` - Optional session name (defaults to timestamp-based)
/// * `resume` - Whether to resume an existing session
pub async fn execute(agent_id: String, session_name: Option<String>, resume: bool, stream: bool, show_metadata: bool, json: bool, safety_behavior: Option<String>) -> Result<()> {
    // Get workspace
    let workspace =
        Workspace::discover().context("Failed to load workspace. Run 'rad init' first.")?;

    // Initialize history manager
    let history_dir = workspace.root().join(".radium/_internals/history");
    std::fs::create_dir_all(&history_dir)?;
    let mut history = HistoryManager::new(&history_dir)?;

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

    // Check if session exists when resuming
    if resume {
        let interactions = history.get_interactions(Some(&session_id));
        if interactions.is_empty() {
            return Err(anyhow!("Session '{}' not found", session_id));
        }
    }

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
        print_history(&history, &session_id)?;
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
                print_history(&history, &session_id)?;
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

        // Get conversation context from history
        let history_context = history.get_summary(Some(&session_id));

        // Build prompt with context files and history
        // History takes precedence (comes after context files)
        let full_prompt = if !context_files.is_empty() && !history_context.is_empty() {
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

        // Use the step command's execution logic
        let prompt_vec = vec![full_prompt];
        match step::execute(
            agent_id.clone(),
            prompt_vec,
            None, // model
            None, // engine
            None, // reasoning
            None, // model_tier
            Some(session_id.clone()), // session_id
            stream, // stream
            show_metadata, // show_metadata
            json, // json
            safety_behavior.clone(), // safety_behavior
            Vec::new(), // image
            Vec::new(), // audio
            Vec::new(), // video
            Vec::new(), // file
            false, // auto_upload
            None, // response_format
            None, // response_schema
        )
        .await
        {
            Ok(_) => {
                // Complete agent in monitoring
                if let Some(monitoring) = monitoring.as_ref() {
                    
                    let _ = monitoring.complete_agent(&tracked_agent_id, 0);
                }
                
                // Record interaction in history
                history.add_interaction(
                    Some(&session_id),
                    input.to_string(),
                    "chat".to_string(),
                    "Response logged".to_string(),
                    None, // metadata - would need to extract from step::execute response
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

    println!("{} {}", "Agent:  ".yellow().bold(), agent_id);
    println!("{} {}", "Session:".yellow().bold(), session_id);

    if resume {
        println!("{} {}", "Mode:   ".yellow().bold(), "Resuming previous conversation");
    }

    println!();
    let mcp_count = slash_registry.get_all_commands().len();
    if mcp_count > 0 {
        println!(
            "{} {} {}",
            "Commands:".green().bold(),
            "/help /history /clear /save /quit",
            format!("({} MCP commands available)", mcp_count).cyan()
        );
    } else {
        println!("{} {}", "Commands:".green().bold(), "/help /history /clear /save /quit");
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

/// Print conversation history
fn print_history(history: &HistoryManager, session_id: &str) -> Result<()> {
    let interactions = history.get_interactions(Some(session_id));

    if interactions.is_empty() {
        println!("\n{}", "No conversation history yet.".yellow());
        return Ok(());
    }

    println!();
    println!("{}", "═══ Conversation History ═══".cyan().bold());
    println!();

    for (i, interaction) in interactions.iter().enumerate() {
        println!(
            "{} {} {}",
            format!("[{}]", i + 1).blue().bold(),
            "You:".green().bold(),
            interaction.goal
        );

        let timestamp = interaction.timestamp.format("%H:%M:%S");
        println!("    ({})", timestamp.to_string().white());
    }
    println!();

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

/// List available chat sessions
pub async fn list_sessions() -> Result<()> {
    let workspace =
        Workspace::discover().context("Failed to load workspace. Run 'rad init' first.")?;

    let history_dir = workspace.root().join(".radium/_internals/history");
    let history_file = history_dir.join("history.json");

    if !history_file.exists() {
        println!("No chat sessions found.");
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

    // Read history file to get all sessions
    let content = std::fs::read_to_string(&history_file)?;
    let sessions: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(sessions_obj) = sessions.as_object() {
        if sessions_obj.is_empty() {
            println!("No chat sessions found.");
            return Ok(());
        }

        for (session_id, interactions) in sessions_obj {
            if let Some(arr) = interactions.as_array() {
                println!(
                    "  {} {} {}",
                    "•".green().bold(),
                    session_id.white().bold(),
                    format!("({} messages)", arr.len()).yellow()
                );

                if let Some(last) = arr.last() {
                    if let Some(timestamp) = last.get("timestamp") {
                        if let Some(ts_str) = timestamp.as_str() {
                            println!("    Last: {}", ts_str);
                        }
                    }
                }
            }
        }
    }

    println!();
    println!(
        "{} Resume a session with: rad chat <agent-id> --session <name> --resume",
        "Tip:".yellow().bold()
    );
    println!();

    Ok(())
}
