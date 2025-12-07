//! Step command implementation.
//!
//! Executes a single agent from configuration.

use anyhow::{Context, bail};
use colored::Colorize;
use radium_core::{
    context::ContextFileLoader, AgentDiscovery, PromptContext, PromptTemplate, Workspace,
};
use radium_models::ModelFactory;
use std::fs;

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

    // Execute agent (simulated)
    println!();
    println!("{}", "Executing agent...".bold());
    println!();

    execute_agent_stub(&agent.id, &rendered, selected_engine, selected_model).await?;

    println!();
    println!("{}", "Agent execution completed!".green().bold());
    println!();

    Ok(())
}

/// Load prompt from file.
fn load_prompt(prompt_path: &std::path::Path) -> anyhow::Result<String> {
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
