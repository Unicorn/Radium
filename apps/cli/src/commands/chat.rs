//! Interactive chat mode for conversational agent interaction.
//!
//! Provides a REPL-style interface for multi-turn conversations with agents,
//! maintaining session history and context across interactions.

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use colored::*;
use radium_core::Workspace;
use radium_core::context::{ContextFileLoader, HistoryManager};
use std::io::{self, Write};

use super::step;

/// Execute the chat command
///
/// # Arguments
/// * `agent_id` - The agent to chat with
/// * `session_name` - Optional session name (defaults to timestamp-based)
/// * `resume` - Whether to resume an existing session
pub async fn execute(agent_id: String, session_name: Option<String>, resume: bool) -> Result<()> {
    // Get workspace
    let workspace =
        Workspace::discover().context("Failed to load workspace. Run 'rad init' first.")?;

    // Initialize history manager
    let history_dir = workspace.root().join(".radium/_internals/history");
    std::fs::create_dir_all(&history_dir)?;
    let mut history = HistoryManager::new(&history_dir)?;

    // Determine session ID
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

    // Print welcome banner
    print_banner(&agent_id, &session_id, resume)?;

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
                break;
            }
            "/help" | "/h" => {
                print_help()?;
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
                continue;
            }
            _ => {}
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

        // Use the step command's execution logic
        let prompt_vec = vec![full_prompt];
        match step::execute(
            agent_id.clone(),
            prompt_vec,
            None, // model
            None, // engine
            None, // reasoning
        )
        .await
        {
            Ok(_) => {
                // Record interaction in history
                history.add_interaction(
                    Some(&session_id),
                    input.to_string(),
                    "chat".to_string(),
                    "Response logged".to_string(),
                )?;
            }
            Err(e) => {
                eprintln!("\n{}: {}", "Error".red().bold(), e);
            }
        }
    }

    Ok(())
}

/// Print welcome banner
fn print_banner(agent_id: &str, session_id: &str, resume: bool) -> Result<()> {
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
    println!("{} {}", "Commands:".green().bold(), "/help /history /clear /save /quit");
    println!();

    Ok(())
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
fn print_help() -> Result<()> {
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

    println!();
    println!(
        "{} Your conversation is automatically saved after each message.",
        "Tip:".yellow().bold()
    );
    println!();

    Ok(())
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
