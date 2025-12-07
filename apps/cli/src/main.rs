//! Radium CLI - Command-line interface for the Radium orchestration platform
//!
//! This CLI provides a `rad` command for interacting with Radium's agent
//! orchestration system and workflow execution engine.

mod commands;

use clap::{CommandFactory, Parser, Subcommand};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use commands::{
    agents, auth, chat, checkpoint, clean, craft, doctor, extension, init, mcp, monitor, plan, run,
    stats, status, step, templates,
};

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
    },

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

        /// List available sessions
        #[arg(long)]
        list: bool,
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

    /// Agent management
    #[command(subcommand)]
    Agents(AgentsCommand),

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
}

// Command types are now in commands::types module
use commands::{AgentsCommand, AuthCommand, ExtensionCommand, TemplatesCommand};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    // Set workspace if provided
    if let Some(workspace) = args.workspace {
        // TODO: Audit that the environment access only happens in single-threaded code.
        unsafe { std::env::set_var("RADIUM_WORKSPACE", workspace) };
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
        Command::Init { path, use_defaults } => {
            init::execute(path, use_defaults).await?;
        }
        Command::Plan { input, id, name } => {
            plan::execute(input, id, name).await?;
        }
        Command::Craft { plan_identifier, iteration, task, resume, dry_run, json } => {
            craft::execute(plan_identifier, iteration, task, resume, dry_run, json).await?;
        }
        Command::Run { script, model, dir } => {
            run::execute(script, model, dir).await?;
        }
        Command::Step { id, prompt, model, engine, reasoning } => {
            step::execute(id, prompt, model, engine, reasoning).await?;
        }
        Command::Chat { agent_id, session, resume, list } => {
            if list {
                chat::list_sessions().await?;
            } else if let Some(id) = agent_id {
                chat::execute(id, session, resume).await?;
            } else {
                anyhow::bail!("Agent ID is required when not using --list");
            }
        }
        Command::Status { json } => {
            status::execute(json).await?;
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
        Command::Agents(cmd) => {
            agents::execute(cmd).await?;
        }
        Command::Monitor(cmd) => {
            monitor::execute(cmd).await?;
        }
        Command::Stats(cmd) => {
            stats::execute(cmd).await?;
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
    }

    Ok(())
}
