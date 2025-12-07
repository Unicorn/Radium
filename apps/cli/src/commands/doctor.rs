//! Doctor command implementation.
//!
//! Validates environment, configuration, and workspace setup.

use colored::Colorize;
use radium_core::{Workspace, engines::{EngineRegistry, HealthStatus}, engines::providers::{ClaudeEngine, GeminiEngine, MockEngine, OpenAIEngine}};
use serde_json::json;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;

/// Execute the doctor command.
///
/// Validates environment, configuration, and workspace setup.
pub async fn execute(json_output: bool) -> anyhow::Result<()> {
    if json_output { execute_json().await } else { execute_human().await }
}

async fn execute_human() -> anyhow::Result<()> {
    println!("{}", "Radium Doctor - Environment Validation".bold().cyan());
    println!();

    let mut all_ok = true;

    // Check workspace
    println!("{}", "Workspace:".bold());
    match Workspace::discover() {
        Ok(workspace) => {
            println!("  Status: {}", "✓ Found".green());
            println!("  Location: {}", workspace.root().display().to_string().dimmed());

            if workspace.is_empty()? {
                println!("  Plans: {}", "0 (empty)".yellow());
            } else {
                let plans = workspace.discover_plans()?;
                println!("  Plans: {}", format!("{}", plans.len()).green());
            }
        }
        Err(e) => {
            println!("  Status: {}", format!("✗ Not found - {}", e).red());
            println!();
            println!("  {}", "Fix:".yellow());
            println!("    rad init");
            println!();
            all_ok = false;
        }
    }
    println!();

    // Check environment files
    println!("{}", "Environment:".bold());
    let env_files = detect_env_files();
    if env_files.cwd_env.is_some() || env_files.home_env.is_some() {
        println!("  Status: {}", "✓ Environment files found".green());
        if let Some(ref path) = env_files.cwd_env {
            println!("  CWD .env: {}", path.display().to_string().dimmed());
        }
        if let Some(ref path) = env_files.home_env {
            println!("  Home .env: {}", path.display().to_string().dimmed());
        }
    } else {
        println!("  Status: {}", "⚠ No .env files found".yellow());
        println!("  {}", "Note: API keys may be configured elsewhere".dimmed());
    }
    println!();

    // Check port availability (for future HTTP server)
    println!("{}", "Network:".bold());
    let default_port = 50051; // gRPC default
    match check_port(default_port) {
        PortStatus::Free => {
            println!("  Port {}: {}", default_port, "✓ Available".green());
        }
        PortStatus::InUse => {
            println!("  Port {}: {}", default_port, "⚠ In use".yellow());
            println!("  {}", "Note: Radium server may already be running".dimmed());
        }
        PortStatus::Unknown => {
            println!("  Port {}: {}", default_port, "? Unknown status".dimmed());
        }
    }
    println!();

    // Check workspace structure
    println!("{}", "Workspace Structure:".bold());
    if let Ok(workspace) = Workspace::discover() {
        let root = workspace.root();
        let required_dirs = vec![".radium", ".radium/plan", ".radium/memory"];

        let mut structure_ok = true;
        for dir in &required_dirs {
            let path = root.join(dir);
            if path.exists() {
                println!("  {}: {}", dir, "✓".green());
            } else {
                println!("  {}: {}", dir, "✗ Missing".red());
                structure_ok = false;
            }
        }

        if !structure_ok {
            println!();
            println!("  {}", "Fix:".yellow());
            println!("    rad init");
            println!();
            all_ok = false;
        }
    }
    println!();

    // Check engine health
    println!("{}", "Engine Health:".bold());
    let config_path = Workspace::discover()
        .ok()
        .map(|w| w.radium_dir().join("config.toml"));
    let registry = if let Some(ref path) = config_path {
        EngineRegistry::with_config_path(path)
    } else {
        EngineRegistry::new()
    };
    
    // Register all available engines
    let _ = registry.register(Arc::new(MockEngine::new()));
    let _ = registry.register(Arc::new(ClaudeEngine::new()));
    let _ = registry.register(Arc::new(OpenAIEngine::new()));
    let _ = registry.register(Arc::new(GeminiEngine::new()));
    
    let health_results = registry.check_health(5).await;
    let mut engine_issues = false;
    
    for health in &health_results {
        match &health.status {
            HealthStatus::Healthy => {
                println!("  {} {}: {}", "✓".green(), health.engine_id.cyan(), "Healthy".green());
            }
            HealthStatus::Warning(msg) => {
                println!("  {} {}: {}", "⚠".yellow(), health.engine_id.cyan(), format!("Warning - {}", msg).yellow());
                engine_issues = true;
            }
            HealthStatus::Failed(msg) => {
                println!("  {} {}: {}", "✗".red(), health.engine_id.cyan(), format!("Failed - {}", msg).red());
                engine_issues = true;
            }
        }
    }
    
    if engine_issues {
        all_ok = false;
    }
    println!();

    // Summary
    if all_ok {
        println!("{}", "✓ All checks passed!".green().bold());
    } else {
        println!("{}", "⚠ Some issues found - see above".yellow().bold());
    }

    Ok(())
}

async fn execute_json() -> anyhow::Result<()> {
    let mut results = json!({
        "workspace": {},
        "environment": {},
        "network": {},
        "structure": {}
    });

    // Workspace check
    match Workspace::discover() {
        Ok(workspace) => {
            results["workspace"] = json!({
                "status": "ok",
                "location": workspace.root().display().to_string(),
                "plans": workspace.discover_plans()?.len()
            });
        }
        Err(e) => {
            results["workspace"] = json!({
                "status": "error",
                "error": e.to_string()
            });
        }
    }

    // Environment check
    let env_files = detect_env_files();
    results["environment"] = json!({
        "cwd_env": env_files.cwd_env.map(|p| p.display().to_string()),
        "home_env": env_files.home_env.map(|p| p.display().to_string())
    });

    // Port check
    let port_status = check_port(50051);
    results["network"] = json!({
        "port": 50051,
        "status": format!("{:?}", port_status)
    });

    println!("{}", serde_json::to_string_pretty(&results)?);
    Ok(())
}

/// Detected environment files.
struct EnvFiles {
    /// .env file in current working directory.
    cwd_env: Option<PathBuf>,
    /// .env file in home directory.
    home_env: Option<PathBuf>,
}

/// Detects environment files in common locations.
fn detect_env_files() -> EnvFiles {
    let cwd_env =
        std::env::current_dir().ok().map(|dir| dir.join(".env")).filter(|path| path.exists());

    let home_env = dirs::home_dir().map(|dir| dir.join(".env")).filter(|path| path.exists());

    EnvFiles { cwd_env, home_env }
}

/// Port status.
#[derive(Debug)]
enum PortStatus {
    /// Port is free and available.
    Free,
    /// Port is in use.
    InUse,
    /// Status unknown.
    Unknown,
}

/// Checks if a port is available.
fn check_port(port: u16) -> PortStatus {
    match TcpListener::bind(format!("127.0.0.1:{}", port)) {
        Ok(_) => PortStatus::Free,
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => PortStatus::InUse,
        Err(_) => PortStatus::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_env_files() {
        let env_files = detect_env_files();
        // Just verify it doesn't panic
        let _ = env_files.cwd_env;
        let _ = env_files.home_env;
    }

    #[test]
    fn test_check_port() {
        // Check a high port that's unlikely to be in use
        let status = check_port(65535);
        // Just verify it returns a status
        match status {
            PortStatus::Free | PortStatus::InUse | PortStatus::Unknown => {}
        }
    }
}
