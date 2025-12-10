//! Radium CLI - Command-line interface for the Radium orchestration platform
//!
//! This CLI provides a `rad` command for interacting with Radium's agent
//! orchestration system and workflow execution engine.

mod commands;
mod config;
mod validation;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, shells};
use colored::Colorize;
use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use commands::{
    agents, auth, budget, capability, checkpoint, clean, clipboard, code, context, cost, craft, doctor, engines, extension, hooks, init, learning, models, monitor, plan, playbook, policy, privacy, requirement, run,
    sandbox, secret, session, stats, status, step, theme, validate,
    // All commands enabled!
    templates, complete, autonomous, vibecheck, chat, mcp, custom, braingrid,
};
use commands::requirement::RequirementCommand;

/// Radium CLI - Next-generation agentic orchestration tool
///
/// Radium (rad) is a high-performance Rust-based agent orchestration platform
/// for creating, managing, and deploying autonomous agents.
#[derive(Parser, Debug)]
#[command(
    name = "rad",
    author,
    version,
    about = "Radium - Next-generation agentic orchestration",
    long_about = "Radium (rad) is a high-performance agent orchestration platform written in Rust.\nProvides a robust and extensible framework for autonomous agents with excellent performance and reliability."
)]
struct Args {
    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info", global = true)]
    log_level: String,

    /// Workspace directory (overrides RADIUM_WORKSPACE)
    #[arg(short = 'w', long, global = true)]
    workspace: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Initialize a new workspace
    ///
    /// Creates a new Radium workspace in the current directory or specified path.
    /// Sets up the directory structure and default configuration.
    Init {
        /// Target path (optional, defaults to current directory)
        path: Option<String>,

        /// Use default values without prompting
        #[arg(long)]
        use_defaults: bool,

        /// Create a starter GEMINI.md context file
        #[arg(long)]
        with_context: bool,

        /// Sandbox type to configure (docker, podman, seatbelt, none)
        #[arg(long)]
        sandbox: Option<String>,

        /// Network mode for sandbox (open, closed, proxied)
        #[arg(long)]
        sandbox_network: Option<String>,
    },

    /// Generate a plan from a specification file
    ///
    /// Analyzes a specification markdown file and generates a structured plan
    /// with iterations, tasks, and dependencies.
    Plan {
        /// Path to specification file or direct content
        input: Option<String>,

        /// Override auto-generated requirement ID (e.g., REQ-123)
        #[arg(long)]
        id: Option<String>,

        /// Customize folder name suffix
        #[arg(long)]
        name: Option<String>,
    },

    /// Execute a generated plan
    ///
    /// Runs the plan through its iterations and tasks, coordinating agents
    /// to complete the implementation.
    Craft {
        /// Plan identifier (REQ-XXX or folder name)
        plan_identifier: Option<String>,

        /// Execute specific iteration only (e.g., I1)
        #[arg(long)]
        iteration: Option<String>,

        /// Execute specific task only (e.g., I1.T1)
        #[arg(long)]
        task: Option<String>,

        /// Resume from last checkpoint
        #[arg(long)]
        resume: bool,

        /// Show what would be executed without running
        #[arg(long)]
        dry_run: bool,

        /// Output results as JSON
        #[arg(long)]
        json: bool,

        /// Enable continuous execution mode (YOLO mode) - runs until all tasks complete
        #[arg(long)]
        yolo: bool,

        /// Engine to use for execution (e.g., "claude", "openai", "gemini")
        #[arg(long)]
        engine: Option<String>,

        /// Model tier override (smart, eco, auto)
        #[arg(long)]
        model_tier: Option<String>,
    },

    /// Complete a requirement from source to execution
    ///
    /// Automatically detects source type (file, Jira ticket, Braingrid REQ),
    /// fetches content, generates a plan, and executes it without user intervention.
    Complete {
        /// Source (file path, Jira ticket ID, or Braingrid REQ ID)
        source: String,
    },

    /// Execute a Braingrid requirement autonomously
    ///
    /// Fetches requirement tree, triggers task breakdown if needed,
    /// executes each task autonomously with real-time status updates,
    /// and sets requirement to REVIEW when complete.
    #[command(subcommand)]
    Requirement(RequirementCommand),

    /// Braingrid operations
    ///
    /// Read, update, and manage Braingrid requirements and tasks.
    #[command(subcommand)]
    Braingrid(BraingridCommand),

    /// Run agent(s) with enhanced syntax
    ///
    /// Execute agents directly with support for parallel (&) and sequential (&&)
    /// execution, file input injection, and context management.
    Run {
        /// Agent script (e.g., "agent-id 'prompt'" or "agent1 & agent2")
        script: String,

        /// Model to use (overrides agent config)
        #[arg(long)]
        model: Option<String>,

        /// Working directory
        #[arg(short = 'd', long)]
        dir: Option<String>,

        /// Model tier override (smart, eco, auto)
        #[arg(long)]
        model_tier: Option<String>,

        /// Show metadata in human-readable format
        #[arg(long)]
        show_metadata: bool,

        /// Output complete response as JSON with nested metadata
        #[arg(long)]
        json: bool,

        /// Override safety behavior (return-partial, error, log)
        #[arg(long)]
        safety_behavior: Option<String>,
    },

    /// Execute a single workflow step
    ///
    /// Runs a specific agent from the agent configuration with optional
    /// prompt override.
    Step {
        /// Agent ID from config/main.agents.toml
        id: String,

        /// Additional prompt to append
        prompt: Vec<String>,

        /// Model to use (overrides agent config)
        #[arg(long)]
        model: Option<String>,

        /// Engine to use (overrides agent config)
        #[arg(long)]
        engine: Option<String>,

        /// Reasoning effort level (low, medium, high)
        #[arg(long)]
        reasoning: Option<String>,

        /// Model tier override (smart, eco, auto)
        #[arg(long)]
        model_tier: Option<String>,

        /// Stream output in real-time
        #[arg(long)]
        stream: bool,

        /// Show metadata in human-readable format
        #[arg(long)]
        show_metadata: bool,

        /// Output complete response as JSON with nested metadata
        #[arg(long)]
        json: bool,

        /// Override safety behavior (return-partial, error, log)
        #[arg(long)]
        safety_behavior: Option<String>,

        /// Path to image file(s) to include in the prompt
        #[arg(long, value_name = "PATH")]
        image: Vec<PathBuf>,

        /// Path to audio file(s) to include in the prompt
        #[arg(long, value_name = "PATH")]
        audio: Vec<PathBuf>,

        /// Path to video file(s) to include in the prompt
        #[arg(long, value_name = "PATH")]
        video: Vec<PathBuf>,

        /// Path to document/file(s) to include in the prompt
        #[arg(long, value_name = "PATH")]
        file: Vec<PathBuf>,

        /// Force File API upload regardless of file size
        #[arg(long)]
        auto_upload: bool,
    },

    /// Interactive chat mode with an agent
    ///
    /// Start a conversational session with an agent, maintaining context
    /// across multiple interactions. Sessions are automatically saved.
    Chat {
        /// Agent ID to chat with (not required when using --list)
        agent_id: Option<String>,

        /// Session name (defaults to timestamp)
        #[arg(long)]
        session: Option<String>,

        /// Resume an existing session
        #[arg(long)]
        resume: bool,

        /// Stream responses in real-time
        #[arg(long)]
        stream: bool,

        /// List available sessions
        #[arg(long)]
        list: bool,

        /// Show metadata in human-readable format
        #[arg(long)]
        show_metadata: bool,

        /// Output complete response as JSON with nested metadata
        #[arg(long)]
        json: bool,

        /// Override safety behavior (return-partial, error, log)
        #[arg(long)]
        safety_behavior: Option<String>,
    },

    /// Clipboard mode for universal editor support
    ///
    /// Bidirectional clipboard operations for sending code to Radium
    /// and receiving processed results, supporting any editor via copy/paste.
    Clipboard {
        #[command(subcommand)]
        action: commands::clipboard::ClipboardCommand,
    },

    /// Code block management
    ///
    /// List, copy, save, and append code blocks extracted from agent responses.
    Code {
        #[command(subcommand)]
        cmd: commands::code::CodeCommand,
    },

    /// Show status of workspace, engines, and authentication
    ///
    /// Displays workspace information, available engines/models,
    /// and authentication status.
    Status {
        /// Output status as JSON
        #[arg(long)]
        json: bool,
    },

    /// Request metacognitive oversight (vibe check)
    ///
    /// Triggers a manual vibe check to get metacognitive feedback
    /// on your current approach, plan, or implementation.
    Vibecheck {
        /// Workflow phase (planning, implementation, review)
        #[arg(long)]
        phase: Option<String>,

        /// Goal or objective being pursued
        #[arg(long)]
        goal: Option<String>,

        /// Current plan or approach
        #[arg(long)]
        plan: Option<String>,

        /// Progress made so far
        #[arg(long)]
        progress: Option<String>,

        /// Task context or recent actions
        #[arg(long)]
        task_context: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Clean workspace artifacts
    ///
    /// Removes temporary files, logs, cached prompts, and execution artifacts
    /// while preserving the workspace structure.
    Clean {
        /// Show detailed output
        #[arg(short, long)]
        verbose: bool,

        /// Target workspace directory
        #[arg(short = 'd', long)]
        dir: Option<String>,
    },

    /// Manage workflow templates
    ///
    /// List, select, and configure workflow templates for plan execution.
    #[command(subcommand)]
    Templates(TemplatesCommand),

    /// Authentication management
    #[command(subcommand)]
    Auth(AuthCommand),

    /// Secret management
    ///
    /// Manage encrypted secrets for secure credential storage.
    #[command(subcommand)]
    Secret(commands::SecretCommand),

    /// Agent management
    #[command(subcommand)]
    Agents(AgentsCommand),

    /// Engine management
    ///
    /// List, inspect, and manage AI engine providers.
    #[command(subcommand)]
    Engines(commands::EnginesCommand),

    /// Manage AI model configurations
    #[command(subcommand)]
    Models(commands::types::ModelsCommand),

    /// Monitor agent execution and telemetry
    ///
    /// View agent status, execution history, and cost tracking.
    #[command(subcommand)]
    Monitor(monitor::MonitorCommand),

    /// Session statistics and analytics
    ///
    /// View comprehensive session reports with metrics, token tracking,
    /// and cost transparency.
    #[command(subcommand)]
    Stats(stats::StatsCommand),

    /// Session management
    ///
    /// Search, export, delete, and view session information.
    #[command(subcommand)]
    Session(session::SessionCommand),

    /// Theme management
    ///
    /// List, set, and preview available themes for the TUI.
    #[command(subcommand)]
    Theme(theme::ThemeCommand),

    /// Manage checkpoints for agent work snapshots
    ///
    /// List and restore git-based checkpoints created during workflow execution.
    #[command(subcommand)]
    Checkpoint(checkpoint::CheckpointCommand),

    /// Validate environment and configuration
    ///
    /// Checks workspace setup, environment files, port availability,
    /// and workspace structure to help diagnose configuration issues.
    Doctor {
        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },

    /// MCP (Model Context Protocol) server management
    ///
    /// List, test, and manage MCP servers and their tools.
    #[command(subcommand)]
    Mcp(mcp::McpCommand),

    /// Extension management
    ///
    /// Install, uninstall, and manage extension packages that bundle
    /// prompts, MCP servers, and custom commands.
    #[command(subcommand)]
    Extension(ExtensionCommand),

    /// Hook management
    ///
    /// List, enable, disable, and manage execution hooks for customizing
    /// agent behavior at various points in the execution flow.
    #[command(subcommand)]
    Hooks(hooks::HooksCommand),

    /// Policy management
    ///
    /// Manage tool execution policies for controlling agent behavior.
    /// Configure allow/deny/ask rules for tool execution with priority-based matching.
    #[command(subcommand)]
    Policy(policy::PolicyCommand),

    /// Capability management
    ///
    /// Manage runtime capability elevation for agents.
    /// Request, grant, revoke, and view capability elevations.
    #[command(subcommand)]
    Capability(capability::CapabilityCommand),

    /// Constitution management
    ///
    /// Manage session-based constitution rules for per-session constraints.
    /// Constitution rules are automatically cleaned up after 1 hour of inactivity.
    #[command(subcommand)]
    Constitution(commands::ConstitutionCommand),

    /// Context file management
    ///
    /// List, show, and validate context files (GEMINI.md) in the workspace.
    /// Context files provide persistent instructions to agents.
    #[command(subcommand)]
    Context(commands::ContextCommand),

    /// Learning system management
    ///
    /// Manage the learning system, including viewing mistakes, adding skills,
    /// tagging skills, and viewing the skillbook.
    #[command(subcommand)]
    Learning(commands::learning::LearningCommand),

    /// Playbook management
    ///
    /// Manage organizational playbooks for embedding knowledge, SOPs, and procedures
    /// into agent behavior. Playbooks are automatically loaded into agent context.
    #[command(subcommand)]
    Playbook(commands::playbook::PlaybookCommand),

    /// Custom command management
    ///
    /// List, execute, create, and validate custom commands defined in TOML files.
    /// Custom commands support shell injection, file injection, and argument substitution.
    #[command(subcommand)]
    Custom(commands::CustomCommand),

    /// Sandbox management
    ///
    /// List, test, and manage sandbox environments for safe agent execution.
    /// Supports Docker, Podman, and macOS Seatbelt sandboxing.
    #[command(subcommand)]
    Sandbox(sandbox::SandboxCommand),

    /// Validate source accessibility
    ///
    /// Tests whether source URIs (file://, http://, jira://, braingrid://)
    /// are accessible before using them in workflows or plans.
    Validate {
        /// Source URIs to validate (e.g., file://./spec.md, jira://PROJ-123)
        sources: Vec<String>,

        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },

    /// Autonomous execution from high-level goals
    ///
    /// Decomposes a high-level goal into an executable workflow and executes it
    /// autonomously with automatic failure recovery, agent reassignment, and learning.
    Autonomous {
        /// High-level goal description
        goal: String,
    },

    /// Budget management for agent execution costs
    ///
    /// Set, view, and manage budget limits for AI model costs.
    #[command(subcommand)]
    Budget(commands::BudgetCommand),

    /// Cost reporting and analytics
    ///
    /// View cost reports with tier breakdown, savings analysis, and
    /// filtering by date range, plan, or workflow.
    #[command(subcommand)]
    Cost(commands::cost::CostCommand),

    /// Privacy and sensitive data management
    ///
    /// Check files for sensitive data and test custom privacy patterns.
    #[command(subcommand)]
    Privacy(privacy::PrivacyCommand),
}

// Command types are now in commands::types module
use commands::{AgentsCommand, AuthCommand, ExtensionCommand, TemplatesCommand, BraingridCommand, CacheCommand};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Handle completion generation
    if let Ok(shell) = std::env::var("RADIUM_GENERATE_COMPLETIONS") {
        let mut cmd = Args::command();
        match shell.as_str() {
            "bash" => generate(shells::Bash, &mut cmd, "rad", &mut std::io::stdout()),
            "zsh" => generate(shells::Zsh, &mut cmd, "rad", &mut std::io::stdout()),
            "fish" => generate(shells::Fish, &mut cmd, "rad", &mut std::io::stdout()),
            "powershell" => generate(shells::PowerShell, &mut cmd, "rad", &mut std::io::stdout()),
            "elvish" => generate(shells::Elvish, &mut cmd, "rad", &mut std::io::stdout()),
            _ => {
                eprintln!("Unknown shell: {}. Supported: bash, zsh, fish, powershell, elvish", shell);
                std::process::exit(1);
            }
        };
        return Ok(());
    }

    let args = Args::parse();

    // Initialize tracing
    let level = match args.log_level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber =
        FmtSubscriber::builder().with_max_level(level).without_time().with_target(false).finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Load configuration
    let cli_config = config::load_config();
    
    // Apply config to environment (only if not already set)
    // SAFETY: We're in single-threaded main() before any async/spawning.
    // Environment variables are set here before any threads are created or
    // async operations begin. This is safe because:
    // 1. We're in main() before tokio runtime initialization
    // 2. No other threads exist at this point
    // 3. All environment access happens synchronously in this block
    unsafe {
        config::apply_config_to_env(&cli_config);
    }

    // Set workspace if provided (CLI arg takes precedence)
    // SAFETY: Same as above - single-threaded execution before async operations
    if let Some(workspace) = args.workspace {
        unsafe { std::env::set_var("RADIUM_WORKSPACE", workspace) };
    } else if let Some(ref workspace) = cli_config.workspace {
        // Use workspace from config if not provided via CLI
        unsafe { std::env::set_var("RADIUM_WORKSPACE", workspace) };
    }

    // Check for resumable executions on startup (before command execution)
    // Skip for init command and other commands that shouldn't be interrupted
    if let Some(ref cmd) = args.command {
        if !matches!(cmd, Command::Init { .. } | Command::Braingrid(_)) {
            check_and_prompt_resume().await?;
        }
    }

    // If no command provided, show help
    let command = if let Some(cmd) = args.command {
        cmd
    } else {
        Args::command().print_help()?;
        return Ok(());
    };

    // Execute command
    match command {
        Command::Init { path, use_defaults, with_context, sandbox, sandbox_network } => {
            init::execute(path, use_defaults, with_context, sandbox, sandbox_network).await?;
        }
        Command::Plan { input, id, name } => {
            plan::execute(input, id, name).await?;
        }
        Command::Craft { plan_identifier, iteration, task, resume, dry_run, json, yolo, engine, model_tier } => {
            craft::execute(plan_identifier, iteration, task, resume, dry_run, json, yolo, engine, model_tier).await?;
        }
        Command::Complete { source } => {
            complete::execute(source).await?;
        }
        Command::Requirement(cmd) => {
            requirement::execute_command(cmd).await?;
        }
        Command::Braingrid(cmd) => {
            match cmd {
                BraingridCommand::Read { req_id, project } => {
                    braingrid::read(req_id, project).await?;
                }
                BraingridCommand::Tasks { req_id, project } => {
                    braingrid::tasks(req_id, project).await?;
                }
                BraingridCommand::UpdateTask { task_id, req_id, status, project } => {
                    braingrid::update_task(task_id, req_id, status, project).await?;
                }
                BraingridCommand::UpdateReq { req_id, status, project } => {
                    braingrid::update_req(req_id, status, project).await?;
                }
                BraingridCommand::Breakdown { req_id, project } => {
                    braingrid::breakdown(req_id, project).await?;
                }
                BraingridCommand::Cache(cache_cmd) => {
                    match cache_cmd {
                        CacheCommand::Clear { project } => {
                            braingrid::cache_clear(project).await?;
                        }
                        CacheCommand::Stats { project } => {
                            braingrid::cache_stats(project).await?;
                        }
                    }
                }
            }
        }
        Command::Run { script, model, dir, model_tier, show_metadata, json, safety_behavior } => {
            run::execute(script, model, dir, model_tier, show_metadata, json, safety_behavior).await?;
        }
        Command::Step { id, prompt, model, engine, reasoning, model_tier, stream, show_metadata, json, safety_behavior, image, audio, video, file, auto_upload } => {
            step::execute(id, prompt, model, engine, reasoning, model_tier, None, stream, show_metadata, json, safety_behavior, image, audio, video, file, auto_upload).await?;
        }
        Command::Chat { agent_id, session, resume, list, stream, show_metadata, json, safety_behavior } => {
            if list {
                chat::list_sessions().await?;
            } else if let Some(id) = agent_id {
                chat::execute(id, session, resume, stream, show_metadata, json, safety_behavior).await?;
            } else {
                anyhow::bail!("Agent ID is required when not using --list");
            }
        }
        Command::Clipboard { action } => {
            match action {
                commands::clipboard::ClipboardCommand::Send => {
                    clipboard::send().await?;
                }
                commands::clipboard::ClipboardCommand::Receive => {
                    clipboard::receive().await?;
                }
            }
        }
        Command::Code { cmd } => {
            code::execute(cmd).await?;
        }
        Command::Status { json } => {
            status::execute(json).await?;
        }
        Command::Vibecheck { phase, goal, plan, progress, task_context, json } => {
            vibecheck::execute(phase, goal, plan, progress, task_context, json).await?;
        }
        Command::Clean { verbose, dir } => {
            clean::execute(verbose, dir).await?;
        }
        Command::Templates(cmd) => {
            templates::execute(cmd).await?;
        }
        Command::Auth(cmd) => {
            auth::execute(cmd).await?;
        }
        Command::Secret(cmd) => {
            secret::execute(cmd).await?;
        }
        Command::Agents(cmd) => {
            agents::execute(cmd).await?;
        }
        Command::Engines(cmd) => {
            engines::execute(cmd).await?;
        }
        Command::Models(cmd) => {
            models::execute(cmd).await?;
        }
        Command::Monitor(cmd) => {
            monitor::execute(cmd).await?;
        }
        Command::Stats(cmd) => {
            stats::execute(cmd).await?;
        }
        Command::Session(cmd) => {
            session::execute(cmd).await?;
        }
        Command::Theme(cmd) => {
            theme::execute(cmd).await?;
        }
        Command::Checkpoint(cmd) => {
            checkpoint::execute(cmd).await?;
        }
        Command::Doctor { json } => {
            doctor::execute(json).await?;
        }
        Command::Mcp(cmd) => {
            mcp::execute_mcp_command(cmd).await?;
        }
        Command::Extension(cmd) => {
            extension::execute(cmd).await?;
        }
        Command::Hooks(cmd) => {
            hooks::execute_hooks_command(cmd).await?;
        }
        Command::Policy(cmd) => {
            policy::execute_policy_command(cmd).await?;
        }
        Command::Capability(cmd) => {
            capability::execute_capability_command(cmd).await?;
        }
        Command::Constitution(cmd) => {
            commands::constitution::execute_constitution_command(cmd).await?;
        }
        Command::Context(cmd) => {
            context::execute(cmd).await?;
        }
        Command::Learning(cmd) => {
            learning::execute(cmd).await?;
        }
        Command::Playbook(cmd) => {
            playbook::execute_playbook_command(cmd).await?;
        }
        Command::Custom(cmd) => {
            custom::execute(cmd).await?;
        }
        Command::Sandbox(cmd) => {
            sandbox::execute(cmd).await?;
        }
        Command::Validate { sources, json } => {
            validate::execute(sources, json).await?;
        }
        Command::Autonomous { goal } => {
            autonomous::execute(goal).await?;
        }
        Command::Budget(cmd) => {
            budget::execute(cmd).await?;
        }
        Command::Cost(cmd) => {
            cost::execute(cmd).await?;
        }
        Command::Privacy(cmd) => {
            privacy::execute(cmd).await?;
        }
    }

    Ok(())
}

/// Checks for resumable executions and prompts user to resume.
async fn check_and_prompt_resume() -> anyhow::Result<()> {
    use radium_core::workspace::Workspace;
    use radium_core::workflow::StatePersistence;
    
    // Try to discover workspace (may not exist yet)
    let workspace = match Workspace::discover() {
        Ok(ws) => ws,
        Err(_) => return Ok(()), // No workspace, nothing to resume
    };
    
    let state_persistence = StatePersistence::new(workspace.root());
    let resumable = state_persistence.list_resumable()
        .map_err(|e| anyhow::anyhow!("Failed to list resumable executions: {}", e))?;
    
    if resumable.is_empty() {
        return Ok(()); // No resumable executions
    }
    
    // Load state for each resumable requirement
    let mut resumable_info = Vec::new();
    for req_id in &resumable {
        if let Ok(Some(state)) = state_persistence.load_state(req_id) {
            resumable_info.push((req_id.clone(), state));
        }
    }
    
    if resumable_info.is_empty() {
        return Ok(());
    }
    
    // Display resumable executions
    println!("\n{} Found {} interrupted execution(s):", "ℹ".cyan(), resumable_info.len());
    println!();
    for (idx, (req_id, state)) in resumable_info.iter().enumerate() {
        let completed = state.completed_tasks.len();
        let total = completed + state.next_tasks.len();
        let last_checkpoint = state.last_checkpoint_at.format("%Y-%m-%d %H:%M:%S UTC");
        println!("  {}. {} - {} ({} of {} tasks completed, last checkpoint: {})",
            idx + 1, req_id, state.requirement_title, completed, total, last_checkpoint);
    }
    println!();
    println!("{} To resume, run: rad requirement resume <req-id>", "ℹ".cyan());
    println!();
    
    Ok(())
}
