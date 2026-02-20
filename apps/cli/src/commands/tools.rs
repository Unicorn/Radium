//! Tools introspection command for listing available tools.

use anyhow::{Context, Result};
use colored::*;
use radium_core::Workspace;
use radium_orchestrator::orchestration::tool_builder::build_standard_tools;
use radium_orchestrator::orchestration::tool::{Tool, ToolParameters};
use serde_json::json;

/// Execute the tools list command
pub async fn list(category: Option<String>, json: bool) -> Result<()> {
    // Get workspace
    let workspace = Workspace::discover()
        .context("Failed to load workspace. Run 'rad init' first.")?;

    let workspace_root = workspace.root().to_path_buf();

    // Build standard tools (same as used by orchestration)
    let tools = build_standard_tools(workspace_root, None);

    // Filter by category if specified
    let category_filter = category.clone();
    let filtered_tools: Vec<&Tool> = if let Some(cat) = category {
        tools
            .iter()
            .filter(|t| {
                // Simple category matching - could be enhanced with explicit categories
                let name_lower = t.name.to_lowercase();
                let cat_lower = cat.to_lowercase();
                name_lower.contains(&cat_lower)
                    || t.description.to_lowercase().contains(&cat_lower)
            })
            .collect()
    } else {
        tools.iter().collect()
    };

    if json {
        // JSON output
        let tools_json: Vec<serde_json::Value> = filtered_tools
            .iter()
            .map(|tool| {
                json!({
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": serialize_parameters(&tool.parameters),
                    "category": infer_category(&tool.name),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&tools_json)?);
    } else {
        // Human-readable output
        println!("{}", "Available Tools".bold().cyan());
        println!("{}", "─".repeat(80).dimmed());
        println!();

        if filtered_tools.is_empty() {
            println!("{}", "No tools found".yellow());
            if category_filter.is_some() {
                println!("{}", format!("Try removing the --category filter").dimmed());
            }
            return Ok(());
        }

        let tool_count = filtered_tools.len();
        for tool in filtered_tools {
            println!("{}", format!("{}", tool.name).bold().green());
            println!("  {}", tool.description.dimmed());
            
            // Show parameters
            if !tool.parameters.properties.is_empty() {
                println!("  {}", "Parameters:".dimmed());
                for (name, prop) in &tool.parameters.properties {
                    let required = if tool.parameters.required.contains(name) {
                        "required"
                    } else {
                        "optional"
                    };
                    println!(
                        "    {} ({}) - {}",
                        name.cyan(),
                        prop.property_type.dimmed(),
                        prop.description.dimmed()
                    );
                    println!("      {}", format!("{}", required).dimmed());
                }
            }

            // Show inferred category
            let category = infer_category(&tool.name);
            println!("  {}", format!("Category: {}", category).dimmed());
            println!();
        }

        println!(
            "{}",
            format!("Total: {} tool(s)", tool_count).dimmed()
        );
    }

    Ok(())
}

/// Serialize tool parameters to JSON
fn serialize_parameters(params: &ToolParameters) -> serde_json::Value {
    let properties: serde_json::Map<String, serde_json::Value> = params
        .properties
        .iter()
        .map(|(name, prop)| {
            (
                name.clone(),
                json!({
                    "type": prop.property_type,
                    "description": prop.description,
                    "required": params.required.contains(name),
                }),
            )
        })
        .collect();

    json!({
        "type": "object",
        "properties": properties,
    })
}

/// Infer tool category from name
fn infer_category(name: &str) -> &str {
    let name_lower = name.to_lowercase();
    if name_lower.contains("file") || name_lower.contains("read") || name_lower.contains("write") {
        "file_operations"
    } else if name_lower.contains("git") || name_lower.contains("blame") || name_lower.contains("show") {
        "git"
    } else if name_lower.contains("project") || name_lower.contains("scan") {
        "project_analysis"
    } else if name_lower.contains("code") || name_lower.contains("analysis") || name_lower.contains("ast") {
        "code_analysis"
    } else if name_lower.contains("terminal") || name_lower.contains("command") || name_lower.contains("run") {
        "terminal"
    } else if name_lower.contains("agent") {
        "agents"
    } else {
        "other"
    }
}
