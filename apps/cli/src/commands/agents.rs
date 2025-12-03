//! Agents command implementation.
//!
//! Provides commands for discovering, searching, and managing agents.

use super::AgentsCommand;
use colored::Colorize;
use radium_core::agents::discovery::AgentDiscovery;
use radium_core::agents::config::AgentConfig;
use serde_json::json;
use tabled::{Table, Tabled, settings::Style};
use std::collections::HashMap;

/// Execute the agents command.
pub async fn execute(command: AgentsCommand) -> anyhow::Result<()> {
    match command {
        AgentsCommand::List { json, verbose } => list_agents(json, verbose).await,
        AgentsCommand::Search { query, json } => search_agents(&query, json).await,
        AgentsCommand::Info { id, json } => show_agent_info(&id, json).await,
        AgentsCommand::Validate { verbose } => validate_agents(verbose).await,
    }
}

/// List all available agents.
async fn list_agents(json_output: bool, verbose: bool) -> anyhow::Result<()> {
    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all()?;

    if agents.is_empty() {
        if !json_output {
            println!("{}", "No agents found.".yellow());
            println!();
            println!("Try creating agents in:");
            println!("  • ./agents/ (project-local)");
            println!("  • ~/.radium/agents/ (user-level)");
        }
        return Ok(());
    }

    if json_output {
        let agent_list: Vec<_> = agents
            .iter()
            .map(|(id, config)| {
                json!({
                    "id": id,
                    "name": config.name,
                    "description": config.description,
                    "category": config.category,
                    "engine": config.engine,
                    "model": config.model,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&agent_list)?);
    } else {
        println!();
        println!("{}", format!("📦 Found {} agents", agents.len()).bold().green());
        println!();

        if verbose {
            display_agents_detailed(&agents);
        } else {
            display_agents_table(&agents);
        }
    }

    Ok(())
}

/// Search for agents by query.
async fn search_agents(query: &str, json_output: bool) -> anyhow::Result<()> {
    let discovery = AgentDiscovery::new();
    let all_agents = discovery.discover_all()?;

    let query_lower = query.to_lowercase();
    let matches: HashMap<String, AgentConfig> = all_agents
        .into_iter()
        .filter(|(id, config)| {
            id.to_lowercase().contains(&query_lower)
                || config.name.to_lowercase().contains(&query_lower)
                || config.description.to_lowercase().contains(&query_lower)
                || config
                    .category
                    .as_ref()
                    .map(|c| c.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
        })
        .collect();

    if matches.is_empty() {
        if !json_output {
            println!("{}", format!("No agents found matching '{}'", query).yellow());
        }
        return Ok(());
    }

    if json_output {
        let results: Vec<_> = matches
            .iter()
            .map(|(id, config)| {
                json!({
                    "id": id,
                    "name": config.name,
                    "description": config.description,
                    "category": config.category,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        println!();
        println!(
            "{}",
            format!("🔍 Found {} matching agents for '{}'", matches.len(), query)
                .bold()
                .green()
        );
        println!();
        display_agents_table(&matches);
    }

    Ok(())
}

/// Show detailed information about a specific agent.
async fn show_agent_info(id: &str, json_output: bool) -> anyhow::Result<()> {
    let discovery = AgentDiscovery::new();
    let agent = discovery.find_by_id(id)?
        .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", id))?;

    if json_output {
        let info = json!({
            "id": agent.id,
            "name": agent.name,
            "description": agent.description,
            "category": agent.category,
            "prompt_path": agent.prompt_path,
            "engine": agent.engine,
            "model": agent.model,
            "reasoning_effort": agent.reasoning_effort,
            "file_path": agent.file_path,
        });
        println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
        println!();
        println!("{}", format!("📋 Agent: {}", agent.name).bold().cyan());
        println!();
        println!("{}", "Details:".bold());
        println!("  ID:          {}", agent.id.green());
        println!("  Name:        {}", agent.name);
        if !agent.description.is_empty() {
            println!("  Description: {}", agent.description);
        }
        if let Some(category) = &agent.category {
            println!("  Category:    {}", category.yellow());
        }
        println!();
        println!("{}", "Configuration:".bold());
        println!("  Prompt Path: {}", agent.prompt_path.display());
        if let Some(engine) = &agent.engine {
            println!("  Engine:      {}", engine.cyan());
        }
        if let Some(model) = &agent.model {
            println!("  Model:       {}", model.cyan());
        }
        if let Some(effort) = &agent.reasoning_effort {
            println!("  Reasoning:   {:?}", effort);
        }
        println!();
        if let Some(path) = &agent.file_path {
            println!("{}", "Source:".bold());
            println!("  {}", path.display().to_string().dimmed());
        }
        println!();
    }

    Ok(())
}

/// Validate all agent configurations.
async fn validate_agents(verbose: bool) -> anyhow::Result<()> {
    let discovery = AgentDiscovery::new();
    let agents = discovery.discover_all()?;

    let mut valid_count = 0;
    let mut errors = Vec::new();

    for (id, config) in &agents {
        // Basic validation
        let mut agent_errors = Vec::new();

        if config.name.is_empty() {
            agent_errors.push("Name is empty");
        }

        if config.prompt_path.as_os_str().is_empty() {
            agent_errors.push("Prompt path is empty");
        }

        // Check if prompt file exists (if path is set)
        if !config.prompt_path.as_os_str().is_empty() {
            if !config.prompt_path.exists() && !config.prompt_path.is_absolute() {
                // Try relative to config directory
                if let Some(config_dir) = config
                    .file_path
                    .as_ref()
                    .and_then(|p| p.parent())
                {
                    let full_path = config_dir.join(&config.prompt_path);
                    if !full_path.exists() {
                        agent_errors.push("Prompt file not found");
                    }
                }
            }
        }

        if agent_errors.is_empty() {
            valid_count += 1;
        } else {
            errors.push((id.clone(), agent_errors));
        }
    }

    println!();
    if errors.is_empty() {
        println!(
            "{}",
            format!("✅ All {} agents validated successfully", agents.len())
                .bold()
                .green()
        );
    } else {
        println!(
            "{}",
            format!("⚠️  Validation: {} valid, {} with errors", valid_count, errors.len())
                .bold()
                .yellow()
        );
        println!();

        if verbose {
            for (id, agent_errors) in &errors {
                println!("{}", format!("  {} {}:", "❌".red(), id.red()));
                for error in agent_errors {
                    println!("     • {}", error);
                }
            }
        } else {
            println!("Run with {} for details", "--verbose".cyan());
        }
    }
    println!();

    Ok(())
}

/// Display agents in a compact table format.
fn display_agents_table(agents: &HashMap<String, AgentConfig>) {
    #[derive(Tabled)]
    struct AgentRow {
        #[tabled(rename = "ID")]
        id: String,
        #[tabled(rename = "Name")]
        name: String,
        #[tabled(rename = "Category")]
        category: String,
        #[tabled(rename = "Engine")]
        engine: String,
        #[tabled(rename = "Model")]
        model: String,
    }

    let mut rows: Vec<AgentRow> = agents
        .iter()
        .map(|(id, config)| AgentRow {
            id: id.clone(),
            name: config.name.clone(),
            category: config.category.clone().unwrap_or_else(|| "-".to_string()),
            engine: config.engine.clone().unwrap_or_else(|| "-".to_string()),
            model: config.model.clone().unwrap_or_else(|| "-".to_string()),
        })
        .collect();

    // Sort by ID
    rows.sort_by(|a, b| a.id.cmp(&b.id));

    let table = Table::new(rows)
        .with(Style::rounded())
        .to_string();

    println!("{}", table);
    println!();
}

/// Display agents in detailed format.
fn display_agents_detailed(agents: &HashMap<String, AgentConfig>) {
    let mut agent_list: Vec<_> = agents.iter().collect();
    agent_list.sort_by_key(|(id, _)| id.as_str());

    for (id, config) in agent_list {
        println!("{}", format!("  {} {}", "●".green(), id.bold()));
        println!("    Name:     {}", config.name);
        if !config.description.is_empty() {
            println!("    Desc:     {}", config.description);
        }
        if let Some(category) = &config.category {
            println!("    Category: {}", category.yellow());
        }
        if let Some(engine) = &config.engine {
            println!("    Engine:   {}", engine.cyan());
        }
        if let Some(model) = &config.model {
            println!("    Model:    {}", model.cyan());
        }
        println!();
    }
}
