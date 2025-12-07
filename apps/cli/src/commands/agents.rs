//! Agents command implementation.
//!
//! Provides commands for discovering, searching, and managing agents.

use super::AgentsCommand;
use colored::Colorize;
use radium_core::agents::config::{AgentConfig, AgentConfigFile, ReasoningEffort};
use radium_core::agents::discovery::AgentDiscovery;
use radium_core::agents::registry::{AgentRegistry, FilterCriteria, SortOrder};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tabled::{Table, Tabled, settings::Style};

/// Execute the agents command.
pub async fn execute(command: AgentsCommand) -> anyhow::Result<()> {
    match command {
        AgentsCommand::List { json, verbose } => list_agents(json, verbose).await,
        AgentsCommand::Search {
            query,
            json,
            category,
            engine,
            model,
            sort,
        } => {
            search_agents(&query, json, category.as_deref(), engine.as_deref(), model.as_deref(), sort.as_deref()).await
        }
        AgentsCommand::Info { id, json } => show_agent_info(&id, json).await,
        AgentsCommand::Validate { verbose } => validate_agents(verbose).await,
        AgentsCommand::Create {
            id,
            name,
            description,
            category,
            engine,
            model,
            reasoning,
            output,
        } => {
            create_agent(
                &id,
                &name,
                description.as_deref(),
                category.as_deref(),
                engine.as_deref(),
                model.as_deref(),
                reasoning.as_deref(),
                output.as_deref(),
            )
            .await
        }
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

/// Search for agents by query with optional filters and sorting.
async fn search_agents(
    query: &str,
    json_output: bool,
    category_filter: Option<&str>,
    engine_filter: Option<&str>,
    model_filter: Option<&str>,
    sort_option: Option<&str>,
) -> anyhow::Result<()> {
    // Create registry and discover agents
    let registry = AgentRegistry::with_discovery()?;

    // Build filter criteria
    let mut criteria = FilterCriteria::default();
    if let Some(cat) = category_filter {
        criteria.category = Some(cat.to_string());
    }
    if let Some(eng) = engine_filter {
        criteria.engine = Some(eng.to_string());
    }
    if let Some(modl) = model_filter {
        criteria.model = Some(modl.to_string());
    }

    // Apply filters if any are specified
    let mut candidates = if criteria.category.is_some()
        || criteria.engine.is_some()
        || criteria.model.is_some()
    {
        registry.filter_combined(&criteria)?
    } else {
        registry.list_all()?
    };

    // Apply text search query
    let query_lower = query.to_lowercase();
    candidates.retain(|config| {
        config.id.to_lowercase().contains(&query_lower)
            || config.name.to_lowercase().contains(&query_lower)
            || config.description.to_lowercase().contains(&query_lower)
            || config
                .category
                .as_ref()
                .map(|c| c.to_lowercase().contains(&query_lower))
                .unwrap_or(false)
    });

    // Apply sorting if specified
    if let Some(sort_str) = sort_option {
        let sort_order = match sort_str.to_lowercase().as_str() {
            "name" => SortOrder::Name,
            "category" => SortOrder::Category,
            "engine" => SortOrder::Engine,
            _ => {
                eprintln!("{} Invalid sort option: {}. Valid options: name, category, engine", "⚠️".yellow(), sort_str);
                SortOrder::Name // Default to name
            }
        };
        // Create a temporary registry with filtered agents for sorting
        let temp_registry = AgentRegistry::new();
        for agent in &candidates {
            let _ = temp_registry.register_or_replace(agent.clone());
        }
        candidates = temp_registry.sort(sort_order)?;
    }

    if candidates.is_empty() {
        if !json_output {
            println!("{}", format!("No agents found matching '{}'", query).yellow());
            if category_filter.is_some() || engine_filter.is_some() || model_filter.is_some() {
                println!("  Applied filters:");
                if let Some(cat) = category_filter {
                    println!("    Category: {}", cat);
                }
                if let Some(eng) = engine_filter {
                    println!("    Engine: {}", eng);
                }
                if let Some(modl) = model_filter {
                    println!("    Model: {}", modl);
                }
            }
        }
        return Ok(());
    }

    // Convert to HashMap for display
    let matches: HashMap<String, AgentConfig> = candidates
        .into_iter()
        .map(|config| (config.id.clone(), config))
        .collect();

    if json_output {
        let results: Vec<_> = matches
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
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        println!();
        println!(
            "{}",
            format!("🔍 Found {} matching agents for '{}'", matches.len(), query).bold().green()
        );
        if category_filter.is_some() || engine_filter.is_some() || model_filter.is_some() {
            println!("  {} Filters applied:", "•".cyan());
            if let Some(cat) = category_filter {
                println!("    Category: {}", cat.cyan());
            }
            if let Some(eng) = engine_filter {
                println!("    Engine: {}", eng.cyan());
            }
            if let Some(modl) = model_filter {
                println!("    Model: {}", modl.cyan());
            }
        }
        if let Some(sort_str) = sort_option {
            println!("  {} Sorted by: {}", "•".cyan(), sort_str.cyan());
        }
        println!();
        display_agents_table(&matches);
    }

    Ok(())
}

/// Show detailed information about a specific agent.
async fn show_agent_info(id: &str, json_output: bool) -> anyhow::Result<()> {
    let discovery = AgentDiscovery::new();
    let agent =
        discovery.find_by_id(id)?.ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", id))?;

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
        let mut agent_errors = Vec::new();

        // Reload and validate the config file to get comprehensive validation
        if let Some(config_path) = &config.file_path {
            match AgentConfigFile::load(config_path) {
                Ok(_) => {
                    // Validation passed
                }
                Err(e) => {
                    // Extract error message
                    let error_msg = e.to_string();
                    // Remove "invalid configuration: " prefix if present
                    let clean_msg = error_msg
                        .strip_prefix("invalid configuration: ")
                        .unwrap_or(&error_msg);
                    agent_errors.push(clean_msg.to_string());
                }
            }
        } else {
            // If file_path is not set, we can't reload, so do basic validation
            if config.name.is_empty() {
                agent_errors.push("Name is empty".to_string());
            }
            if config.prompt_path.as_os_str().is_empty() {
                agent_errors.push("Prompt path is empty".to_string());
            }
        }

        if agent_errors.is_empty() {
            valid_count += 1;
        } else {
            errors.push((id.clone(), config.file_path.clone(), agent_errors));
        }
    }

    println!();
    if errors.is_empty() {
        println!(
            "{}",
            format!("✅ All {} agents validated successfully", agents.len()).bold().green()
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
            for (id, file_path, agent_errors) in &errors {
                println!("{}", format!("  {} {}:", "❌".red(), id.red()));
                if let Some(path) = file_path {
                    println!("     {}", format!("File: {}", path.display()).dimmed());
                }
                for error in agent_errors {
                    println!("     • {}", error);
                }
                println!();
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

    let table = Table::new(rows).with(Style::rounded()).to_string();

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

/// Create a new agent template.
#[allow(clippy::too_many_arguments)]
async fn create_agent(
    id: &str,
    name: &str,
    description: Option<&str>,
    category: Option<&str>,
    engine: Option<&str>,
    model: Option<&str>,
    reasoning: Option<&str>,
    output_dir: Option<&str>,
) -> anyhow::Result<()> {
    // Validate agent ID
    if id.is_empty() {
        anyhow::bail!("Agent ID cannot be empty");
    }

    // Use default category if not provided
    let category = category.unwrap_or("custom");

    // Determine output directory
    let base_dir = output_dir.unwrap_or("./agents");
    let agent_dir = Path::new(base_dir).join(category);

    // Create directory structure
    fs::create_dir_all(&agent_dir)?;

    // Create prompts directory
    let prompts_dir = Path::new("./prompts/agents").join(category);
    fs::create_dir_all(&prompts_dir)?;

    // File paths
    let config_path = agent_dir.join(format!("{}.toml", id));
    let prompt_path = prompts_dir.join(format!("{}.md", id));

    // Check if agent already exists
    if config_path.exists() {
        anyhow::bail!("Agent '{}' already exists at {}", id, config_path.display());
    }

    // Parse reasoning effort
    let reasoning_effort = reasoning.and_then(|r| match r.to_lowercase().as_str() {
        "low" => Some(ReasoningEffort::Low),
        "medium" => Some(ReasoningEffort::Medium),
        "high" => Some(ReasoningEffort::High),
        _ => None,
    });

    // Build agent config
    let prompt_path_relative = PathBuf::from(format!("prompts/agents/{}/{}.md", category, id));

    let mut agent = AgentConfig::new(id, name, prompt_path_relative);
    agent.description = description.unwrap_or("").to_string();

    if let Some(eng) = engine {
        agent = agent.with_engine(eng);
    }

    if let Some(mdl) = model {
        agent = agent.with_model(mdl);
    }

    if let Some(effort) = reasoning_effort {
        agent = agent.with_reasoning_effort(effort);
    }

    // Wrap in AgentConfigFile
    let config_file = AgentConfigFile { agent };

    // Save TOML configuration
    config_file.save(&config_path)?;

    // Generate prompt template
    let prompt_template = generate_prompt_template(name, description);
    fs::write(&prompt_path, prompt_template)?;

    // Success output
    println!();
    println!("{}", "✅ Agent template created successfully!".bold().green());
    println!();
    println!("{}", "Files created:".bold());
    println!("  • Configuration: {}", config_path.display().to_string().cyan());
    println!("  • Prompt:        {}", prompt_path.display().to_string().cyan());
    println!();
    println!("{}", "Next steps:".bold());
    println!("  1. Edit the prompt file to define agent behavior");
    println!("  2. Validate: {}", format!("rad agents validate").yellow());
    println!("  3. Test: {}", format!("rad agents info {}", id).yellow());
    println!();

    Ok(())
}

/// Generate a prompt template for a new agent.
fn generate_prompt_template(name: &str, description: Option<&str>) -> String {
    let desc = description.unwrap_or("Add agent description here");

    format!(
        r#"# {name}

{desc}

## Role

Define the agent's role and primary responsibilities here.

## Capabilities

- List the agent's core capabilities
- Include what tasks it can perform
- Specify any constraints or limitations

## Input

Describe what inputs this agent expects:
- Context from previous steps
- Required parameters
- Optional configuration

## Output

Describe what this agent produces:
- Expected output format
- Key deliverables
- Success criteria

## Instructions

Provide step-by-step instructions for the agent:

1. First step - explain what to do
2. Second step - detail the process
3. Third step - clarify expectations
4. Continue as needed...

## Examples

### Example 1: [Scenario Name]

**Input:**
```
Provide sample input
```

**Expected Output:**
```
Show expected result
```

### Example 2: [Another Scenario]

**Input:**
```
Different scenario input
```

**Expected Output:**
```
Corresponding output
```

## Notes

- Add any important notes
- Include edge cases to consider
- Document best practices
"#
    )
}
