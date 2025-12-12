//! Tool inventory and introspection commands.
//!
//! Exposes the unified tool set (file/project/git/code/terminal + agents) for debugging and UX.

use anyhow::Result;
use colored::Colorize;
use radium_core::Workspace;
use radium_orchestrator::orchestration::{
    agent_tools::AgentToolRegistry,
    code_analysis_tool,
    file_tools::{self, WorkspaceRootProvider as FileWorkspaceRootProvider},
    git_extended_tools,
    project_scan_tool,
    terminal_tool::{self, WorkspaceRootProvider as TerminalWorkspaceRootProvider},
    tool::Tool,
    tool_builder::{NoOpSandboxManager, SimpleWorkspaceRootProvider},
    tool_registry::{ToolCategory, UnifiedToolRegistry},
};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

fn parse_category(category: Option<String>) -> ToolCategory {
    match category.as_deref() {
        Some("file") => ToolCategory::File,
        Some("terminal") => ToolCategory::Terminal,
        Some("agent") => ToolCategory::Agent,
        Some("mcp") => ToolCategory::MCP,
        Some("all") | None => ToolCategory::All,
        Some(_) => ToolCategory::All,
    }
}

fn build_registry(workspace_root: PathBuf) -> Result<UnifiedToolRegistry> {
    let mut registry = UnifiedToolRegistry::new();

    let workspace_provider: Arc<dyn FileWorkspaceRootProvider> = Arc::new(SimpleWorkspaceRootProvider {
        root: workspace_root.clone(),
    });

    // Treat project/git/code tools as part of "File" category for now.
    let mut file_like_tools: Vec<Tool> = Vec::new();
    file_like_tools.extend(file_tools::create_file_operation_tools(workspace_provider.clone()));
    file_like_tools.extend(project_scan_tool::create_project_analysis_tools(workspace_provider.clone()));
    file_like_tools.extend(git_extended_tools::create_git_extended_tools(workspace_provider.clone()));
    file_like_tools.push(code_analysis_tool::create_code_analysis_tool(workspace_provider));
    registry.add_file_tools(file_like_tools);

    let terminal_workspace_provider: Arc<dyn TerminalWorkspaceRootProvider> = Arc::new(SimpleWorkspaceRootProvider {
        root: workspace_root.clone(),
    });
    let terminal_tool = terminal_tool::create_terminal_command_tool(
        terminal_workspace_provider,
        Some(Arc::new(NoOpSandboxManager)),
        None,
    );
    registry.add_terminal_tools(vec![terminal_tool]);

    let mut agent_registry = AgentToolRegistry::new();
    agent_registry.load_agents()?;
    registry.add_agent_tools(agent_registry.get_tools().to_vec());

    Ok(registry)
}

pub async fn list(category: Option<String>, json_output: bool) -> Result<()> {
    let workspace_root = Workspace::discover()
        .map(|w| w.root().to_path_buf())
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let registry = build_registry(workspace_root)?;
    let category = parse_category(category);
    let tools = registry.filter_by_category(category);

    if json_output {
        let mut items: Vec<serde_json::Value> = Vec::with_capacity(tools.len());
        for t in &tools {
            items.push(json!({
                "id": t.id,
                "name": t.name,
                "description": t.description,
                "parameters": serde_json::to_value(&t.parameters)?,
                "required": t.parameters.required,
            }));
        }

        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "category": format!("{:?}", category).to_lowercase(),
                "count": items.len(),
                "tools": items,
            }))?
        );
        return Ok(());
    }

    println!("{}", "rad tools list".bold().cyan());
    println!();
    println!(
        "{} {}",
        "Category:".bold(),
        format!("{:?}", category).to_lowercase().cyan()
    );
    println!("{}", format!("Total tools: {}", tools.len()).dimmed());
    println!();

    for tool in tools {
        println!("{} {}", tool.name.bold(), format!("({})", tool.id).dimmed());
        println!("  {}", tool.description);
        if !tool.parameters.required.is_empty() {
            println!(
                "  {} {}",
                "required:".dimmed(),
                tool.parameters.required.join(", ").cyan()
            );
        }
        println!();
    }

    Ok(())
}

