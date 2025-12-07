//! MCP (Model Context Protocol) commands.

use clap::Subcommand;
use radium_core::mcp::{McpConfigManager, McpIntegration};
use radium_core::workspace::Workspace;
use std::path::PathBuf;

/// MCP command options.
#[derive(Subcommand, Debug)]
pub enum McpCommand {
    /// List configured MCP servers
    List,
    /// List tools from MCP servers
    Tools {
        /// Server name (optional, lists all if not specified)
        server: Option<String>,
    },
    /// Test connection to MCP servers
    Test {
        /// Server name (optional, tests all if not specified)
        server: Option<String>,
    },
}

/// Execute MCP command.
pub async fn execute_mcp_command(command: McpCommand) -> anyhow::Result<()> {
    let workspace = Workspace::discover()?;
    let config_path = McpConfigManager::default_config_path(workspace.root());
    let mut config_manager = McpConfigManager::new(config_path);

    match command {
        McpCommand::List => {
            config_manager.load()?;
            let servers = config_manager.get_servers();

            if servers.is_empty() {
                println!("No MCP servers configured.");
                println!("\nTo configure a server, create a file at:");
                println!("  {}", config_path.display());
                println!("\nExample configuration:");
                println!("  [[servers]]");
                println!("  name = \"my-server\"");
                println!("  transport = \"stdio\"");
                println!("  command = \"mcp-server\"");
                println!("  args = [\"--config\", \"config.json\"]");
                return Ok(());
            }

            println!("Configured MCP servers:");
            println!();
            for server in servers {
                println!("  {} ({:?})", server.name, server.transport);
                if let Some(ref command) = server.command {
                    println!("    Command: {}", command);
                }
                if let Some(ref url) = server.url {
                    println!("    URL: {}", url);
                }
            }
        }
        McpCommand::Tools { server } => {
            let integration = McpIntegration::new();
            integration.initialize(&workspace).await?;

            let all_tools = integration.get_all_tools().await;

            if all_tools.is_empty() {
                println!("No tools available from MCP servers.");
                return Ok(());
            }

            if let Some(server_name) = server {
                if let Some((_, tools)) = all_tools.iter().find(|(s, _)| s == &server_name) {
                    println!("Tools from server '{}':", server_name);
                    for tool in tools {
                        println!("  {}", tool);
                    }
                } else {
                    println!("Server '{}' not found or has no tools.", server_name);
                }
            } else {
                println!("Available MCP tools:");
                println!();
                for (server_name, tools) in &all_tools {
                    println!("  {}:", server_name);
                    for tool in tools {
                        println!("    {}", tool);
                    }
                }
            }
        }
        McpCommand::Test { server } => {
            let integration = McpIntegration::new();
            
            if let Some(server_name) = server {
                match integration.initialize(&workspace).await {
                    Ok(()) => {
                        if integration.is_server_connected(&server_name).await {
                            println!("✓ Server '{}' is connected.", server_name);
                        } else {
                            println!("✗ Server '{}' is not connected.", server_name);
                        }
                    }
                    Err(e) => {
                        println!("✗ Failed to initialize MCP integration: {}", e);
                    }
                }
            } else {
                match integration.initialize(&workspace).await {
                    Ok(()) => {
                        let count = integration.connected_server_count().await;
                        if count > 0 {
                            println!("✓ {} server(s) connected.", count);
                        } else {
                            println!("✗ No servers connected.");
                        }
                    }
                    Err(e) => {
                        println!("✗ Failed to initialize MCP integration: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}

