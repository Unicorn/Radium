//! Step command implementation.
//!
//! Executes a single agent from configuration.

use anyhow::{Context, bail};
use chrono::Utc;
use colored::Colorize;
use radium_core::{
    analytics::{ReportFormatter, SessionAnalytics, SessionReport, SessionStorage},
    context::ContextFileLoader, AgentDiscovery, monitoring::MonitoringService, PromptContext,
    PromptTemplate, Workspace,
};
use radium_models::ModelFactory;
use std::fs;
use uuid::Uuid;

/// Execute the step command.
///
/// Executes a single workflow step (agent from configuration).
pub async fn execute(
    id: String,
    prompt: Vec<String>,
    model: Option<String>,
    engine: Option<String>,
    reasoning: Option<String>,
) -> anyhow::Result<()> {
    println!("{}", "rad step".bold().cyan());
    println!();
    
    // Generate session ID for tracking
    let session_id = Uuid::new_v4().to_string();
    let session_start_time = Utc::now();

    // Discover workspace (optional for step command)
    let workspace = Workspace::discover().ok();
    let workspace_root = workspace
        .as_ref()
        .map(|w| w.root().to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));

    // Load context files if available
    let loader = ContextFileLoader::new(&workspace_root);
    let current_dir = std::env::current_dir().unwrap_or_else(|_| workspace_root.clone());
    let context_files = loader.load_hierarchical(&current_dir).unwrap_or_default();

    // Discover all available agents
    println!("  {}", "Discovering agents...".dimmed());
    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all().context("Failed to discover agents")?;

    if agents.is_empty() {
        bail!("No agents found. Place agent configs in ./agents/ or ~/.radium/agents/");
    }

    println!("  {} Found {} agents", "✓".green(), agents.len());
    println!();

    // Find the requested agent
    let agent = agents.get(&id).ok_or_else(|| anyhow::anyhow!("Agent not found: {}", id))?;

    // Display agent information
    println!("{}", "Agent Information:".bold());
    println!("  ID: {}", agent.id.cyan());
    println!("  Name: {}", agent.name);
    println!("  Description: {}", agent.description.dimmed());
    println!("  Prompt: {}", agent.prompt_path.display().to_string().dimmed());

    // Display model/engine info (with overrides if provided)
    let selected_engine = engine.as_deref().unwrap_or(agent.engine.as_deref().unwrap_or("none"));
    let selected_model = model.as_deref().unwrap_or(agent.model.as_deref().unwrap_or("default"));
    let selected_reasoning =
        reasoning.as_deref().unwrap_or_else(|| match agent.reasoning_effort.unwrap_or_default() {
            radium_core::ReasoningEffort::Low => "low",
            radium_core::ReasoningEffort::Medium => "medium",
            radium_core::ReasoningEffort::High => "high",
        });

    println!();
    println!("{}", "Execution Configuration:".bold());
    println!("  Engine: {}", selected_engine.cyan());
    println!("  Model: {}", selected_model.cyan());
    println!("  Reasoning: {}", selected_reasoning.cyan());

    // Load and render prompt
    println!();
    println!("  {}", "Loading prompt template...".dimmed());

    let prompt_content = load_prompt(&agent.prompt_path)?;
    let user_input = if prompt.is_empty() {
        String::from("No additional input provided")
    } else {
        prompt.join(" ")
    };

    println!("  {} Loaded {} bytes", "✓".green(), prompt_content.len());

    if !user_input.is_empty() && user_input != "No additional input provided" {
        println!();
        println!("{}", "User Input:".bold());
        println!("  {}", user_input.dimmed());
    }

    println!();
    println!("{}", "Rendering prompt template...".bold());

    let mut context = PromptContext::new();
    context.set("user_input", user_input.clone());

    // Inject context files if available
    if !context_files.is_empty() {
        context.set("context_files", context_files.clone());
        let context_file_paths = loader.get_context_file_paths(&current_dir);
        if !context_file_paths.is_empty() {
            println!("  {} Loaded context from {} file(s)", "✓".green(), context_file_paths.len());
        }
    }

    let template = PromptTemplate::from_string(prompt_content);
    let rendered = template.render(&context)?;

    println!("  {} Rendered {} bytes", "✓".green(), rendered.len());

    // Display prompt preview
    println!();
    println!("{}", "Prompt Preview:".bold().dimmed());
    println!("{}", "─".repeat(60).dimmed());
    let preview = if rendered.len() > 500 {
        format!("{}...\n\n[truncated {} bytes]", &rendered[..500], rendered.len() - 500)
    } else {
        rendered.clone()
    };
    println!("{}", preview.dimmed());
    println!("{}", "─".repeat(60).dimmed());

    // Register agent in monitoring for session tracking
    let tracked_agent_id = format!("{}-{}", session_id, agent.id);
    let monitoring_path = workspace
        .as_ref()
        .map(|w| w.radium_dir().join("monitoring.db"))
        .unwrap_or_else(|| std::path::PathBuf::from(".radium/monitoring.db"));
    let monitoring = MonitoringService::open(&monitoring_path).ok();
    
    if let Some(monitoring) = monitoring.as_ref() {
        use radium_core::monitoring::{AgentRecord, AgentStatus};
        let mut agent_record = AgentRecord::new(tracked_agent_id.clone(), agent.id.clone());
        agent_record.plan_id = Some(session_id.clone());
        if monitoring.register_agent(&agent_record).is_ok() {
            let _ = monitoring.update_status(&tracked_agent_id, AgentStatus::Running);
        }
    }

    // Execute agent (simulated)
    println!();
    println!("{}", "Executing agent...".bold());
    println!();

    let execution_result = execute_agent_stub(&agent.id, &rendered, selected_engine, selected_model).await;

    // Complete agent in monitoring
    if let Some(monitoring) = monitoring.as_ref() {
        use radium_core::monitoring::AgentStatus;
        match execution_result {
            Ok(_) => {
                let _ = monitoring.complete_agent(&tracked_agent_id, 0);
            }
            Err(ref e) => {
                let _ = monitoring.fail_agent(&tracked_agent_id, &e.to_string());
            }
        }
    }

    execution_result?;

    // Generate and display session report
    let session_end_time = Some(Utc::now());
    if let Some(monitoring) = monitoring {
        let analytics = SessionAnalytics::new(monitoring);
        let agent_ids = vec![tracked_agent_id];
        
        if let Ok(metrics) = analytics.generate_session_metrics_with_workspace(
            &session_id,
            &agent_ids,
            session_start_time,
            session_end_time,
            workspace.as_ref().map(|w| w.root()),
        ) {
            let report = SessionReport::new(metrics);
            if let Some(workspace) = workspace.as_ref() {
                let storage = SessionStorage::new(workspace.root()).ok();
                if let Some(ref storage) = storage {
                    let _ = storage.save_report(&report);
                    display_session_summary(&report);
                }
            }
        }
    }

    println!();
    println!("{}", "Agent execution completed!".green().bold());
    println!();

    Ok(())
}

/// Display session summary at end of execution.
fn display_session_summary(report: &SessionReport) {
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
    
    println!();
    println!("  {} Full report: {}", "💡".cyan(), format!("rad stats session {}", report.metrics.session_id).dimmed());
    println!("{}", "─".repeat(60).dimmed());
    println!();
}

/// Load prompt from file.
///
/// Search order (precedence from highest to lowest):
/// 1. Absolute path (if provided)
/// 2. Relative to current directory
/// 3. Relative to workspace root
/// 4. Relative to home directory (.radium/)
/// 5. Extension prompt directories (project-level, then user-level)
fn load_prompt(prompt_path: &std::path::Path) -> anyhow::Result<String> {
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

    bail!("Prompt file not found: {}", prompt_path.display())
}

/// Execute the agent with the actual model.
async fn execute_agent_stub(
    agent_id: &str,
    rendered_prompt: &str,
    engine: &str,
    model: &str,
) -> anyhow::Result<()> {
    println!("  {} Executing agent with {}...", "•".cyan(), engine);
    println!("  {} Agent: {}", "•".dimmed(), agent_id.cyan());
    println!("  {} Engine: {}", "•".dimmed(), engine.cyan());
    println!("  {} Model: {}", "•".dimmed(), model.cyan());
    println!();

    // Try to create model instance
    match ModelFactory::create_from_str(engine, model.to_string()) {
        Ok(model_instance) => {
            println!("  {} Model initialized successfully", "✓".green());
            println!("  {} Sending prompt to model...", "•".cyan());
            println!();

            // Execute the model
            match model_instance.generate_text(rendered_prompt, None).await {
                Ok(response) => {
                    println!("{}", "Response:".bold().green());
                    println!("{}", "─".repeat(60).dimmed());
                    println!("{}", response.content);
                    println!("{}", "─".repeat(60).dimmed());

                    if let Some(usage) = response.usage {
                        println!();
                        println!("{}", "Token Usage:".bold().dimmed());
                        println!("  Prompt: {} tokens", usage.prompt_tokens.to_string().dimmed());
                        println!(
                            "  Completion: {} tokens",
                            usage.completion_tokens.to_string().dimmed()
                        );
                        println!("  Total: {} tokens", usage.total_tokens.to_string().cyan());
                    }

                    Ok(())
                }
                Err(e) => {
                    println!();
                    println!("  {} {}", "✗".red(), format!("Model execution failed: {}", e).red());
                    println!();
                    println!("  {} Check your API key and model configuration", "i".yellow());
                    Err(anyhow::anyhow!("Model execution failed: {}", e))
                }
            }
        }
        Err(e) => {
            println!(
                "  {} {}",
                "!".yellow(),
                format!("Could not initialize model: {}", e).yellow()
            );
            println!();
            println!("  {} Possible reasons:", "i".cyan());
            println!(
                "    • API key not set ({}_API_KEY environment variable)",
                engine.to_uppercase()
            );
            println!("    • Invalid model configuration");
            println!("    • Network connectivity issues");
            println!();
            println!("  {} Set up API key:", "i".cyan());
            println!("    export {}_API_KEY=your-api-key-here", engine.to_uppercase());
            println!();
            println!("  {} Falling back to mock execution...", "→".dimmed());
            println!();

            // Fall back to mock model
            let mock_model = ModelFactory::create_from_str("mock", model.to_string())?;
            let response = mock_model.generate_text(rendered_prompt, None).await?;

            println!("{}", "Mock Response:".bold().yellow());
            println!("{}", "─".repeat(60).dimmed());
            println!("{}", response.content.dimmed());
            println!("{}", "─".repeat(60).dimmed());

            Ok(())
        }
    }
}
