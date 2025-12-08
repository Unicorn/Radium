//! Run command implementation.
//!
//! Executes agents with a simple script syntax.

use anyhow::{Context, bail};
use colored::Colorize;
use radium_core::{
    context::ContextFileLoader, AgentDiscovery, PromptContext, PromptTemplate, Workspace,
};
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
    model_tier: Option<String>,
) -> anyhow::Result<()> {
    println!("{}", "rad run".bold().cyan());
    println!();

    // Determine working directory
    let working_dir = if let Some(dir_path) = &dir {
        std::path::PathBuf::from(dir_path)
    } else {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    };

    // Change to working directory if specified
    if let Some(dir_path) = &dir {
        std::env::set_current_dir(dir_path)
            .context(format!("Failed to change to directory: {}", dir_path))?;
        println!("  {} Working directory: {}", "•".dimmed(), dir_path.dimmed());
    }

    // Discover workspace and load context files
    let workspace = Workspace::discover().ok();
    let workspace_root = workspace
        .as_ref()
        .map(|w| w.root().to_path_buf())
        .unwrap_or_else(|| working_dir.clone());
    let loader = ContextFileLoader::new(&workspace_root);
    let context_files = loader.load_hierarchical(&working_dir).unwrap_or_default();

    // Parse script - simple format: "agent-id prompt"
    let (agent_id, prompt_text) = parse_simple_script(&script)?;

    println!("  {} Agent: {}", "•".dimmed(), agent_id.cyan());
    println!("  {} Prompt: {}", "•".dimmed(), prompt_text.dimmed());
    println!();

    // Validate sources before execution
    {
        use crate::validation::validate_and_prompt;
        let validation_root = workspace.as_ref().map(|w| w.root().to_path_buf());
        if !validate_and_prompt(&prompt_text, validation_root).await? {
            anyhow::bail!("Source validation failed or user declined to proceed");
        }
    }
    println!();

    // Discover agents
    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all().context("Failed to discover agents")?;

    let agent =
        agents.get(&agent_id).ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

    // Load and render prompt
    let prompt_content = load_prompt(&agent.prompt_path)?;

    let template = PromptTemplate::from_string(prompt_content);

    let mut context = PromptContext::new();
    context.set("user_input", prompt_text.clone());

    // Inject context files if available
    if !context_files.is_empty() {
        context.set("context_files", context_files);
        let context_file_paths = loader.get_context_file_paths(&working_dir);
        if !context_file_paths.is_empty() {
            println!("  {} Loaded context from {} file(s)", "✓".green(), context_file_paths.len());
        }
    }

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
        bail!(
            "Invalid script format. Expected: \"agent-id prompt-text\"
Got: {}",
            script
        );
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
///
/// Search order (precedence from highest to lowest):
/// 1. Absolute path (if provided)
/// 2. Relative to current directory
/// 3. Relative to workspace root
/// 4. Relative to home directory (.radium/)
/// 5. Extension prompt directories (project-level, then user-level)
fn load_prompt(prompt_path: &std::path::Path) -> anyhow::Result<String> {
    use radium_core::{Workspace, extensions::integration::get_extension_prompt_dirs};

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

    // Try extension prompt directories (lowest precedence)
    // Extract just the filename from the path to search in extension directories
    if let Some(file_name) = prompt_path.file_name() {
        if let Ok(extension_dirs) = get_extension_prompt_dirs() {
            for ext_dir in extension_dirs {
                let ext_prompt_path = ext_dir.join(file_name);
                if ext_prompt_path.exists() {
                    return Ok(std::fs::read_to_string(ext_prompt_path)?);
                }
            }
        }
    }

    bail!("Prompt file not found: {}", prompt_path.display())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_parse_simple_script_valid() {
        let script = "test-agent Hello world";
        let (id, prompt) = parse_simple_script(script).unwrap();
        assert_eq!(id, "test-agent");
        assert_eq!(prompt, "Hello world");

        let script = "agent2   Multiple spaces    here";
        let (id, prompt) = parse_simple_script(script).unwrap();
        assert_eq!(id, "agent2");
        assert_eq!(prompt, "Multiple spaces    here");
    }

    #[test]
    fn test_parse_simple_script_invalid() {
        // Empty script
        assert!(parse_simple_script("").is_err());
        assert!(parse_simple_script("   ").is_err());

        // Missing prompt (single word)
        assert!(parse_simple_script("agent-only").is_err());
        assert!(parse_simple_script("singleword").is_err());
    }

    #[test]
    fn test_load_prompt_absolute_path() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("prompt.md");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Absolute prompt").unwrap();

        let content = load_prompt(&file_path).unwrap();
        assert_eq!(content.trim(), "Absolute prompt");
    }

    #[test]
    fn test_load_prompt_relative_path() {
        let dir = TempDir::new().unwrap();
        let file_name = "relative_prompt.md";
        let file_path = dir.path().join(file_name);
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Relative prompt").unwrap();

        // Change current dir to temp dir for relative path test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let result = load_prompt(std::path::Path::new(file_name));

        // Restore dir
        std::env::set_current_dir(original_dir).unwrap();

        let content = result.unwrap();
        assert_eq!(content.trim(), "Relative prompt");
    }

    #[test]
    fn test_load_prompt_not_found() {
        let result = load_prompt(std::path::Path::new("non_existent_file.md"));
        assert!(result.is_err());
    }
}
