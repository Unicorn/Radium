//! MCP (Model Context Protocol) commands.

use anyhow::Context;
use clap::Subcommand;
use colored::Colorize;
use inquire::{Select, Text, Confirm};
use radium_core::mcp::{McpAuthConfig, McpConfigManager, McpIntegration, McpServerConfig, OAuthTokenManager, TransportType};
use radium_core::workspace::Workspace;
use std::collections::HashMap;

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
    /// Interactive setup wizard for configuring MCP servers
    Setup,
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
        McpCommand::Setup => {
            setup_wizard(&workspace, &mut config_manager).await?;
        }
    }

    Ok(())
}

/// Interactive setup wizard for configuring MCP servers.
async fn setup_wizard(
    workspace: &Workspace,
    config_manager: &mut McpConfigManager,
) -> anyhow::Result<()> {
    println!("{}", "MCP Server Setup Wizard".bold().cyan());
    println!();
    println!("This wizard will guide you through configuring an MCP server.");
    println!();

    // Load existing config
    config_manager.load()?;
    let existing_servers: Vec<String> = config_manager.get_servers()
        .iter()
        .map(|s| s.name.clone())
        .collect();

    // Step 1: Server name
    let server_name = Text::new("Server name:")
        .with_help_message("A unique identifier for this MCP server")
        .with_validator(|name: &str| {
            if name.is_empty() {
                return Ok(inquire::validator::Validation::Invalid(
                    "Server name cannot be empty".into(),
                ));
            }
            if existing_servers.contains(&name.to_string()) {
                return Ok(inquire::validator::Validation::Invalid(
                    format!("A server named '{}' already exists", name).into(),
                ));
            }
            Ok(inquire::validator::Validation::Valid)
        })
        .prompt()
        .context("Failed to read server name")?;

    // Step 2: Transport type
    let transport_options = vec![
        "stdio - Local command-line server",
        "sse - Server-Sent Events (SSE) endpoint",
        "http - HTTP-based server",
    ];

    let transport_choice = Select::new("Transport type:", transport_options)
        .with_help_message("Choose how to connect to the MCP server")
        .prompt()
        .context("Failed to select transport type")?;

    let transport = if transport_choice.starts_with("stdio") {
        TransportType::Stdio
    } else if transport_choice.starts_with("sse") {
        TransportType::Sse
    } else if transport_choice.starts_with("http") {
        TransportType::Http
    } else {
        unreachable!()
    };

    // Step 3: Transport-specific configuration
    let (command, args, url) = match transport {
        TransportType::Stdio => {
            let cmd = Text::new("Command:")
                .with_help_message("The executable command to run (e.g., 'mcp-server' or '/usr/local/bin/mcp-server')")
                .prompt()
                .context("Failed to read command")?;

            let args_input = Text::new("Arguments (space-separated, optional):")
                .with_help_message("Command-line arguments for the server (leave empty if none)")
                .with_default("")
                .prompt()
                .context("Failed to read arguments")?;

            let args: Option<Vec<String>> = if args_input.trim().is_empty() {
                None
            } else {
                Some(args_input.split_whitespace().map(|s| s.to_string()).collect())
            };

            (Some(cmd), args, None)
        }
        TransportType::Sse | TransportType::Http => {
            let transport_name = if transport == TransportType::Sse { "SSE" } else { "HTTP" };
            let help_message = format!("The {} endpoint URL (e.g., 'http://localhost:8080/mcp')", transport_name);
            let url_input = Text::new("Server URL:")
                .with_help_message(&help_message)
                .prompt()
                .context("Failed to read URL")?;

            (None, None, Some(url_input))
        }
    };

    // Step 4: OAuth authentication (optional)
    let use_oauth = Confirm::new("Configure OAuth authentication?")
        .with_default(false)
        .with_help_message("OAuth is required for some remote MCP servers")
        .prompt()
        .context("Failed to read OAuth choice")?;

    let auth = if use_oauth {
        println!();
        println!("{}", "OAuth Configuration".bold().yellow());
        println!("You'll need to provide OAuth configuration details from your MCP server provider.");
        println!();

        let token_url = Text::new("Token URL:")
            .with_help_message("OAuth token endpoint URL (e.g., 'https://api.example.com/oauth/token')")
            .prompt()
            .context("Failed to read token URL")?;

        let client_id = Text::new("Client ID:")
            .with_help_message("OAuth client ID")
            .prompt()
            .context("Failed to read client ID")?;

        let client_secret = Text::new("Client Secret:")
            .with_help_message("OAuth client secret")
            .prompt()
            .context("Failed to read client secret")?;

        let mut auth_params = HashMap::new();
        auth_params.insert("token_url".to_string(), token_url);
        auth_params.insert("client_id".to_string(), client_id);
        auth_params.insert("client_secret".to_string(), client_secret);

        Some(McpAuthConfig {
            auth_type: "oauth".to_string(),
            params: auth_params,
        })
    } else {
        None
    };

    // Step 5: Review and confirm
    println!();
    println!("{}", "Configuration Summary".bold().green());
    println!("  Server name: {}", server_name.cyan());
    println!("  Transport: {:?}", transport);
    if let Some(ref cmd) = command {
        println!("  Command: {}", cmd.cyan());
        if let Some(ref args) = args {
            println!("  Arguments: {}", args.join(" ").cyan());
        }
    }
    if let Some(ref url) = url {
        println!("  URL: {}", url.cyan());
    }
    if auth.is_some() {
        println!("  Authentication: {}", "OAuth configured".green());
    }
    println!();

    let confirm = Confirm::new("Save this configuration?")
        .with_default(true)
        .prompt()
        .context("Failed to read confirmation")?;

    if !confirm {
        println!("{}", "Setup cancelled.".yellow());
        return Ok(());
    }

    // Create server config
    let server_config = McpServerConfig {
        name: server_name.clone(),
        transport,
        command,
        args,
        url,
        auth,
    };

    // Add to config manager
    config_manager.get_servers_mut().push(server_config);

    // Save configuration
    config_manager.save()
        .context("Failed to save MCP server configuration")?;

    println!();
    println!("{} Configuration saved successfully!", "✓".green());
    println!();
    println!("Next steps:");
    println!("  1. Test the connection: {}", format!("rad mcp test {}", server_name).cyan());
    println!("  2. List available tools: {}", format!("rad mcp tools {}", server_name).cyan());
    println!("  3. View all servers: {}", "rad mcp list".cyan());

    Ok(())
}

// Helper method to get mutable servers (we need to add this to McpConfigManager)
// Actually, let me check if we can just push directly or if we need a method
