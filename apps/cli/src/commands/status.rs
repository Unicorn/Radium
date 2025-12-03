//! Status command implementation.

use colored::Colorize;
use radium_core::{AgentDiscovery, Workspace};
use serde_json::json;

/// Execute the status command.
///
/// Shows workspace status, available agents, models, and authentication.
pub async fn execute(json_output: bool) -> anyhow::Result<()> {
    if json_output { execute_json().await } else { execute_human().await }
}

async fn execute_human() -> anyhow::Result<()> {
    println!("{}", "Radium Status".bold().cyan());
    println!();

    // Workspace status
    println!("{}", "Workspace:".bold());
    match Workspace::discover() {
        Ok(workspace) => {
            println!("  Location: {}", workspace.root().display().to_string().green());
            println!("  Valid: {}", "✓".green());

            // Check if empty
            if workspace.is_empty()? {
                println!("  Plans: {}", "0 (empty)".yellow());
            } else {
                let plans = workspace.discover_plans()?;
                println!("  Plans: {}", format!("{}", plans.len()).green());
            }
        }
        Err(e) => {
            println!("  Status: {}", format!("Not found - {}", e).red());
            println!();
            println!("  {}", "Create a workspace with:".yellow());
            println!("    rad plan <spec.md>");
            println!();
        }
    }
    println!();

    // Agent discovery
    println!("{}", "Agents:".bold());
    let discovery = AgentDiscovery::new();
    match discovery.discover_all() {
        Ok(agents) => {
            if agents.is_empty() {
                println!("  {}", "No agents found".yellow());
                println!("  {}", "Place agent configs in ./agents/ or ~/.radium/agents/".dimmed());
            } else {
                println!("  Total: {}", format!("{}", agents.len()).green());
                println!();
                println!("  {}:", "Categories".dimmed());

                // Group by category
                let mut by_category: std::collections::HashMap<String, Vec<&str>> =
                    std::collections::HashMap::new();
                for (id, config) in &agents {
                    let category = config.category.as_deref().unwrap_or("uncategorized");
                    by_category.entry(category.to_string()).or_default().push(id.as_str());
                }

                for (category, agent_ids) in by_category.iter() {
                    println!(
                        "    {}: {} agents",
                        category.cyan(),
                        agent_ids.len().to_string().dimmed()
                    );
                }
            }
        }
        Err(e) => {
            println!("  {}", format!("Discovery failed - {}", e).red());
        }
    }
    println!();

    // Models (stub for now)
    println!("{}", "Models:".bold());
    println!("  {}", "Available:".dimmed());
    println!("    • Gemini: gemini-2.0-flash-exp");
    println!("    • OpenAI: gpt-4, gpt-3.5-turbo");
    println!();

    // Authentication (stub for now)
    println!("{}", "Authentication:".bold());
    println!("  {}", "Status:".dimmed());
    println!("    • Gemini: {}", "Not configured".yellow());
    println!("    • OpenAI: {}", "Not configured".yellow());
    println!();
    println!("  Use {} to configure authentication", "rad auth login".cyan());

    Ok(())
}

async fn execute_json() -> anyhow::Result<()> {
    let mut status = json!({
        "workspace": null,
        "agents": {},
        "models": [],
        "authentication": {}
    });

    // Workspace status
    if let Ok(workspace) = Workspace::discover() {
        let plans = workspace.discover_plans().unwrap_or_default();
        status["workspace"] = json!({
            "location": workspace.root().display().to_string(),
            "valid": true,
            "plan_count": plans.len(),
        });
    }

    // Agent discovery
    if let Ok(agents) = AgentDiscovery::new().discover_all() {
        let mut by_category: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for (id, config) in agents {
            let category = config.category.unwrap_or_else(|| "uncategorized".to_string());
            by_category.entry(category).or_default().push(id);
        }
        status["agents"] = json!(by_category);
    }

    // Models
    status["models"] = json!([
        { "provider": "gemini", "model": "gemini-2.0-flash-exp" },
        { "provider": "openai", "model": "gpt-4" },
        { "provider": "openai", "model": "gpt-3.5-turbo" }
    ]);

    // Authentication (stub)
    status["authentication"] = json!({
        "gemini": "not_configured",
        "openai": "not_configured"
    });

    println!("{}", serde_json::to_string_pretty(&status)?);

    Ok(())
}
