//! MCP proxy server commands.

use anyhow::Context;
use clap::Subcommand;
use colored::Colorize;
use radium_core::mcp::proxy::{
    DefaultSecurityLayer, DefaultToolCatalog, DefaultToolRouter, HealthChecker, ProxyConfig,
    ProxyConfigManager, ProxyServer, ProxyTransport, SecurityConfig, UpstreamPool,
};
use radium_core::mcp::proxy::types::{ConflictStrategy, SecurityLayer as SecurityLayerTrait};
use radium_core::workspace::Workspace;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;

/// MCP proxy command options.
#[derive(Subcommand, Debug)]
pub enum McpProxyCommand {
    /// Initialize proxy configuration
    Init,
    /// Start the proxy server
    Start,
    /// Stop the proxy server
    Stop,
    /// Check proxy server status
    Status,
}

/// Execute MCP proxy command.
pub async fn execute_mcp_proxy_command(command: McpProxyCommand) -> anyhow::Result<()> {
    let workspace = Workspace::discover()?;
    let workspace_root = workspace.root();

    match command {
        McpProxyCommand::Init => {
            init_proxy_config(workspace_root).await?;
        }
        McpProxyCommand::Start => {
            start_proxy_server(workspace_root).await?;
        }
        McpProxyCommand::Stop => {
            stop_proxy_server(workspace_root).await?;
        }
        McpProxyCommand::Status => {
            check_proxy_status(workspace_root).await?;
        }
    }

    Ok(())
}

/// Initialize proxy configuration file.
async fn init_proxy_config(workspace_root: &std::path::Path) -> anyhow::Result<()> {
    let config_path = ProxyConfigManager::default_config_path(workspace_root);
    let config_dir = config_path.parent().unwrap();

    // Create .radium directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(config_dir)
            .with_context(|| format!("Failed to create directory: {}", config_dir.display()))?;
    }

    // Generate default config
    let default_config = ProxyConfigManager::generate_default();

    // Save config
    let manager = ProxyConfigManager::new(config_path.clone());
    manager.save(&default_config)
        .with_context(|| format!("Failed to save config to: {}", config_path.display()))?;

    println!("{} Proxy configuration initialized!", "✓".green());
    println!();
    println!("Configuration file: {}", config_path.display());
    println!();
    println!("Next steps:");
    println!("  1. Edit {} to configure upstream servers", config_path.display());
    println!("  2. Start the proxy: {}", "rad mcp proxy start".cyan());
    println!();
    println!("Example configuration:");
    println!("  [mcp.proxy]");
    println!("  enable = true");
    println!("  port = 3000");
    println!("  transport = \"sse\"");
    println!();
    println!("  [[mcp.proxy.upstreams]]");
    println!("  name = \"my-upstream\"");
    println!("  transport = \"http\"");
    println!("  url = \"http://localhost:8080/mcp\"");
    println!("  priority = 1");

    Ok(())
}

/// Start the proxy server.
async fn start_proxy_server(workspace_root: &std::path::Path) -> anyhow::Result<()> {
    let config_path = ProxyConfigManager::default_config_path(workspace_root);
    let pid_path = workspace_root.join(".radium").join("mcp-proxy.pid");

    // Check if already running
    if pid_path.exists() {
        if let Ok(pid_str) = fs::read_to_string(&pid_path) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                // Check if process is still running
                if process_exists(pid) {
                    println!("{} Proxy server is already running (PID: {})", "⚠".yellow(), pid);
                    println!("Use '{}' to stop it first.", "rad mcp proxy stop".cyan());
                    return Ok(());
                }
            }
        }
        // Stale PID file, remove it
        let _ = fs::remove_file(&pid_path);
    }

    // Load configuration
    if !config_path.exists() {
        println!("{} Proxy configuration not found at: {}", "✗".red(), config_path.display());
        println!();
        println!("Run '{}' to create a default configuration.", "rad mcp proxy init".cyan());
        return Ok(());
    }

    let manager = ProxyConfigManager::new(config_path.clone());
    let config = manager.load()
        .with_context(|| format!("Failed to load config from: {}", config_path.display()))?;

    if !config.enable {
        println!("{} Proxy is disabled in configuration.", "⚠".yellow());
        println!("Set 'enable = true' in {} to start the proxy.", config_path.display());
        return Ok(());
    }

    println!("{} Starting MCP proxy server...", "→".cyan());
    println!("  Port: {}", config.port);
    println!("  Transport: {:?}", config.transport);
    println!("  Upstreams: {}", config.upstreams.len());

    // Initialize components
    let pool = Arc::new(UpstreamPool::new());
    
    // Add upstreams to pool
    for upstream_config in &config.upstreams {
        println!("  Connecting to upstream: {}...", upstream_config.server.name);
        if let Err(e) = pool.add_upstream(upstream_config.clone()).await {
            println!("    {} Failed to connect: {}", "✗".red(), e);
            // Continue with other upstreams
        } else {
            println!("    {} Connected", "✓".green());
        }
    }

    // Build upstream priorities map for catalog
    let mut priorities = HashMap::new();
    for upstream in &config.upstreams {
        priorities.insert(upstream.server.name.clone(), upstream.priority);
    }

    // Create router
    let router: Arc<dyn radium_core::mcp::proxy::types::ToolRouter> =
        Arc::new(DefaultToolRouter::new(Arc::clone(&pool)));

    // Create catalog
    let catalog_impl = DefaultToolCatalog::new(ConflictStrategy::AutoPrefix, priorities.clone());
    
    // Rebuild catalog from upstreams
    if let Err(e) = catalog_impl.rebuild_catalog(&pool).await {
        println!("{} Warning: Failed to rebuild catalog: {}", "⚠".yellow(), e);
    }

    let catalog: Arc<dyn radium_core::mcp::proxy::types::ToolCatalog> =
        Arc::new(catalog_impl);

    // Create security layer
    let security: Arc<dyn SecurityLayerTrait> = Arc::new(
        DefaultSecurityLayer::new(config.security.clone())
            .context("Failed to create security layer")?,
    );

    // Start health checker
    let health_checker = HealthChecker::new(Arc::clone(&pool));
    for upstream in &config.upstreams {
        health_checker
            .start_health_check(upstream.server.name.clone(), upstream.health_check_interval)
            .await;
    }

    // Create and start proxy server
    let mut proxy_server = ProxyServer::new(config.clone(), router, catalog, security);
    
    proxy_server.start().await
        .with_context(|| format!("Failed to start proxy server on port {}", config.port))?;

    // Write PID file
    let pid = process::id();
    fs::write(&pid_path, pid.to_string())
        .with_context(|| format!("Failed to write PID file: {}", pid_path.display()))?;

    println!();
    println!("{} Proxy server started!", "✓".green());
    println!("  PID: {}", pid);
    println!("  Listening on port: {}", config.port);
    println!("  PID file: {}", pid_path.display());
    println!();
    println!("Stop the server with: {}", "rad mcp proxy stop".cyan());

    // Run until interrupted
    tokio::signal::ctrl_c().await
        .context("Failed to listen for Ctrl+C")?;

    println!();
    println!("{} Shutting down proxy server...", "→".cyan());

    // Stop server
    proxy_server.stop().await
        .context("Failed to stop proxy server gracefully")?;

    // Stop health checker
    health_checker.stop_all().await;

    // Remove PID file
    let _ = fs::remove_file(&pid_path);

    println!("{} Proxy server stopped.", "✓".green());

    Ok(())
}

/// Stop the proxy server.
async fn stop_proxy_server(workspace_root: &std::path::Path) -> anyhow::Result<()> {
    let pid_path = workspace_root.join(".radium").join("mcp-proxy.pid");

    if !pid_path.exists() {
        println!("{} Proxy server is not running (no PID file found)", "✗".red());
        return Ok(());
    }

    let pid_str = fs::read_to_string(&pid_path)
        .with_context(|| format!("Failed to read PID file: {}", pid_path.display()))?;

    let pid: u32 = pid_str.trim().parse()
        .with_context(|| format!("Invalid PID in file: {}", pid_path.display()))?;

    if !process_exists(pid) {
        println!("{} Proxy server process (PID {}) not found", "✗".red(), pid);
        println!("Removing stale PID file...");
        fs::remove_file(&pid_path)
            .with_context(|| format!("Failed to remove PID file: {}", pid_path.display()))?;
        return Ok(());
    }

    println!("{} Stopping proxy server (PID: {})...", "→".cyan(), pid);

    // Send SIGTERM
    #[cfg(unix)]
    {
        use std::process::Command;
        Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .output()
            .with_context(|| format!("Failed to send SIGTERM to process {}", pid))?;
    }

    #[cfg(windows)]
    {
        // On Windows, use taskkill
        use std::process::Command;
        Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .output()
            .with_context(|| format!("Failed to stop process {}", pid))?;
    }

    // Wait a bit for graceful shutdown
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Remove PID file
    fs::remove_file(&pid_path)
        .with_context(|| format!("Failed to remove PID file: {}", pid_path.display()))?;

    println!("{} Proxy server stopped.", "✓".green());

    Ok(())
}

/// Check proxy server status.
async fn check_proxy_status(workspace_root: &std::path::Path) -> anyhow::Result<()> {
    let pid_path = workspace_root.join(".radium").join("mcp-proxy.pid");
    let config_path = ProxyConfigManager::default_config_path(workspace_root);

    if !pid_path.exists() {
        println!("{} Proxy server is not running", "✗".red());
        return Ok(());
    }

    let pid_str = fs::read_to_string(&pid_path)
        .with_context(|| format!("Failed to read PID file: {}", pid_path.display()))?;

    let pid: u32 = pid_str.trim().parse()
        .with_context(|| format!("Invalid PID in file: {}", pid_path.display()))?;

    if !process_exists(pid) {
        println!("{} Proxy server process (PID {}) not found", "✗".red(), pid);
        println!("PID file may be stale.");
        return Ok(());
    }

    // Load config to show port
    let mut port = None;
    if config_path.exists() {
        if let Ok(manager) = ProxyConfigManager::new(config_path).load() {
            port = Some(manager.port);
        }
    }

    println!("{} Proxy server is running", "✓".green());
    println!("  PID: {}", pid);
    if let Some(p) = port {
        println!("  Port: {}", p);
    }

    Ok(())
}

/// Check if a process exists.
fn process_exists(pid: u32) -> bool {
    #[cfg(unix)]
    {
        use std::process::Command;
        let output = Command::new("ps")
            .arg("-p")
            .arg(pid.to_string())
            .output();
        output.map(|o| o.status.success()).unwrap_or(false)
    }

    #[cfg(windows)]
    {
        use std::process::Command;
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid)])
            .output();
        output
            .map(|o| {
                let stdout = String::from_utf8_lossy(&o.stdout);
                stdout.contains(&pid.to_string())
            })
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_init_proxy_config() {
        let temp_dir = TempDir::new().unwrap();
        init_proxy_config(temp_dir.path()).await.unwrap();
        
        let config_path = ProxyConfigManager::default_config_path(temp_dir.path());
        assert!(config_path.exists());
    }
}
