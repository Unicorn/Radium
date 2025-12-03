//! Run command implementation.
//!
//! Executes agents with a simple script syntax.

use anyhow::{Context, bail};
use colored::Colorize;
use radium_core::{AgentDiscovery, PromptContext, PromptTemplate};
use radium_models::ModelFactory;

/// Execute the run command.
///
/// Runs agents with simple script syntax: "agent-id prompt-text"
///
/// Future enhancements will support:
/// - Parallel execution with &
/// - Sequential execution with &&
/// - File input with [input:file.md]
/// - Context tail with [tail:50]
pub async fn execute(
    script: String,
    model: Option<String>,
    dir: Option<String>,
) -> anyhow::Result<()> {
    println!("{}", "rad run".bold().cyan());
    println!();

    // Change to working directory if specified
    if let Some(working_dir) = &dir {
        std::env::set_current_dir(working_dir)
            .context(format!("Failed to change to directory: {}", working_dir))?;
        println!("  {} Working directory: {}", "•".dimmed(), working_dir.dimmed());
    }

    // Parse script - simple format: "agent-id prompt"
    let (agent_id, prompt_text) = parse_simple_script(&script)?;

    println!("  {} Agent: {}", "•".dimmed(), agent_id.cyan());
    println!("  {} Prompt: {}", "•".dimmed(), prompt_text.dimmed());
    println!();

    // Discover agents
    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all().context("Failed to discover agents")?;

    let agent =
        agents.get(&agent_id).ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

    // Load and render prompt
    let prompt_content = load_prompt(&agent.prompt_path)?;

    let template = PromptTemplate::from_str(prompt_content);

    let mut context = PromptContext::new();
    context.set("user_input", prompt_text.clone());

    let rendered = template.render(&context)?;

    // Determine model configuration
    let engine = agent.engine.as_deref().unwrap_or("mock");
    let model_id = model.clone().or_else(|| agent.model.clone()).unwrap_or_default();

    println!("{}", "Executing agent...".bold());
    println!("  {} Engine: {}", "•".dimmed(), engine.cyan());
    println!("  {} Model: {}", "•".dimmed(), model_id.cyan());
    println!();

    // Execute with model
    match ModelFactory::create_from_str(engine, model_id.clone()) {
        Ok(model_instance) => match model_instance.generate_text(&rendered, None).await {
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
                println!("{} Model execution failed: {}", "✗".red(), e.to_string().red());
                Err(anyhow::anyhow!("Model execution failed: {}", e))
            }
        },
        Err(e) => {
            println!("{} Could not initialize model: {}", "!".yellow(), e);
            println!(
                "{} Set API key: export {}_API_KEY=your-key",
                "i".cyan(),
                engine.to_uppercase()
            );
            println!("{} Falling back to mock model...", "→".dimmed());
            println!();

            // Fall back to mock model
            let mock_model = ModelFactory::create_from_str("mock", model_id)?;
            let response = mock_model.generate_text(&rendered, None).await?;

            println!("{}", "Mock Response:".bold().yellow());
            println!("{}", "─".repeat(60).dimmed());
            println!("{}", response.content.dimmed());
            println!("{}", "─".repeat(60).dimmed());

            Ok(())
        }
    }
}

/// Parse a simple script format: "agent-id prompt-text"
///
/// Examples:
/// - "test-agent Hello world"
/// - "arch-agent Design a REST API"
fn parse_simple_script(script: &str) -> anyhow::Result<(String, String)> {
    let script = script.trim();

    if script.is_empty() {
        bail!("Script cannot be empty");
    }

    // Find first space to split agent-id from prompt
    let parts: Vec<&str> = script.splitn(2, ' ').collect();

    if parts.len() < 2 {
        bail!("Invalid script format. Expected: \"agent-id prompt-text\"\nGot: {}", script);
    }

    let agent_id = parts[0].trim().to_string();
    let prompt = parts[1].trim().to_string();

    if agent_id.is_empty() {
        bail!("Agent ID cannot be empty");
    }

    if prompt.is_empty() {
        bail!("Prompt text cannot be empty");
    }

    Ok((agent_id, prompt))
}

/// Load prompt from file with multiple search paths.
fn load_prompt(prompt_path: &std::path::Path) -> anyhow::Result<String> {
    use radium_core::Workspace;

    // Try as absolute path first
    if prompt_path.is_absolute() && prompt_path.exists() {
        return Ok(std::fs::read_to_string(prompt_path)?);
    }

    // Try relative to current directory
    if prompt_path.exists() {
        return Ok(std::fs::read_to_string(prompt_path)?);
    }

    // Try relative to workspace
    if let Ok(workspace) = Workspace::discover() {
        let workspace_path = workspace.root().join(prompt_path);
        if workspace_path.exists() {
            return Ok(std::fs::read_to_string(workspace_path)?);
        }
    }

    // Try relative to home directory
    if let Ok(home) = std::env::var("HOME") {
        let home_path = std::path::PathBuf::from(home).join(".radium").join(prompt_path);
        if home_path.exists() {
            return Ok(std::fs::read_to_string(home_path)?);
        }
    }

    bail!("Prompt file not found: {}", prompt_path.display())
}
