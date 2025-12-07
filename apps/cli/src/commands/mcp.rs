//! MCP (Model Context Protocol) commands.

use clap::Subcommand;
use radium_core::mcp::{McpConfigManager, McpIntegration, OAuthTokenManager};
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
    /// OAuth authentication commands
    Auth {
        #[clap(subcommand)]
        subcommand: AuthSubcommand,
    },
    /// List available MCP prompts (slash commands)
    Prompts,
}

/// OAuth authentication subcommands.
#[derive(Subcommand, Debug)]
pub enum AuthSubcommand {
    /// Show OAuth token status for configured servers
    Status {
        /// Server name (optional, shows all if not specified)
        server: Option<String>,
    },
}

/// Execute MCP command.
pub async fn execute_mcp_command(command: McpCommand) -> anyhow::Result<()> {
    let workspace = Workspace::discover()?;
    let config_path = McpConfigManager::default_config_path(workspace.root());
    let config_path_display = config_path.clone();
    let mut config_manager = McpConfigManager::new(config_path);

    match command {
        McpCommand::List => {
            config_manager.load()?;
            let servers = config_manager.get_servers();

            if servers.is_empty() {
                println!("No MCP servers configured.");
                println!("\nTo configure a server, create a file at:");
                println!("  {}", config_path_display.display());
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
        McpCommand::Auth { subcommand } => {
            match subcommand {
                AuthSubcommand::Status { server } => {
                    let storage_dir = OAuthTokenManager::default_storage_dir();
                    let mut token_manager = OAuthTokenManager::new(storage_dir);
                    token_manager.load_tokens()?;

                    config_manager.load()?;
                    let servers = config_manager.get_servers();

                    if let Some(server_name) = server {
                        // Show status for specific server
                        if let Some(server_config) = servers.iter().find(|s| s.name == server_name) {
                            if let Some(token) = token_manager.get_token(&server_name) {
                                let expired = token_manager.is_token_expired(&server_name);
                                println!("OAuth token status for server '{}':", server_name);
                                println!("  Status: {}", if expired { "Expired" } else { "Valid" });
                                println!("  Token type: {}", token.token_type);
                                if let Some(ref expires_at) = token.expires_at {
                                    use std::time::{SystemTime, UNIX_EPOCH};
                                    let now = SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .unwrap()
                                        .as_secs();
                                    if *expires_at > now {
                                        let remaining = expires_at - now;
                                        let hours = remaining / 3600;
                                        let minutes = (remaining % 3600) / 60;
                                        println!("  Expires in: {}h {}m", hours, minutes);
                                    } else {
                                        println!("  Expired: {} seconds ago", now - expires_at);
                                    }
                                } else {
                                    println!("  Expiration: Not set");
                                }
                                if token.refresh_token.is_some() {
                                    println!("  Refresh token: Available");
                                } else {
                                    println!("  Refresh token: Not available");
                                }
                            } else {
                                println!("No OAuth token found for server '{}'", server_name);
                                if server_config.auth.is_some() {
                                    println!("  Note: Server has auth configured but no token stored.");
                                    println!("  Token will be obtained on first connection.");
                                } else {
                                    println!("  Note: Server does not have OAuth authentication configured.");
                                }
                            }
                        } else {
                            println!("Server '{}' not found in configuration.", server_name);
                        }
                    } else {
                        // Show status for all servers
                        let mut has_tokens = false;
                        for server_config in servers {
                            if let Some(token) = token_manager.get_token(&server_config.name) {
                                has_tokens = true;
                                let expired = token_manager.is_token_expired(&server_config.name);
                                println!("{}: {}", server_config.name, if expired { "Expired" } else { "Valid" });
                            } else if server_config.auth.is_some() {
                                has_tokens = true;
                                println!("{}: No token (auth configured)", server_config.name);
                            }
                        }
                        if !has_tokens {
                            println!("No OAuth tokens found for any configured servers.");
                        }
                    }
                }
            }
        }
        McpCommand::Prompts => {
            let integration = McpIntegration::new();
            integration.initialize(&workspace).await?;

            let slash_registry = integration.slash_registry();
            let registry = slash_registry.lock().await;
            let commands = registry.get_all_commands();

            if commands.is_empty() {
                println!("No MCP prompts available.");
                println!("\nMCP prompts are automatically registered as slash commands when MCP servers are connected.");
                return Ok(());
            }

            println!("Available MCP slash commands:");
            println!();
            for (cmd_name, prompt) in commands {
                let server_name = registry
                    .get_server_for_command(cmd_name)
                    .map(|s| s.as_str())
                    .unwrap_or("unknown");
                println!("  {} (from server: {})", cmd_name, server_name);
                if let Some(desc) = &prompt.description {
                    println!("    {}", desc);
                }
                if let Some(args) = &prompt.arguments {
                    if !args.is_empty() {
                        println!("    Arguments:");
                        for arg in args {
                            let required = if arg.required { "required" } else { "optional" };
                            let arg_desc = arg.description.as_deref().unwrap_or("No description");
                            println!("      {} ({}) - {}", arg.name, required, arg_desc);
                        }
                    }
                }
                println!();
            }
        }
    }

    Ok(())
}
