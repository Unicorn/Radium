//! Sandbox command implementation.
//!
//! Provides commands for managing and testing sandbox environments.

use clap::{Args, Subcommand};
use colored::Colorize;
use radium_core::sandbox::{
    NetworkMode, Sandbox, SandboxConfig, SandboxFactory, SandboxProfile, SandboxType,
};
use serde_json::json;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Subcommand, Debug, Clone)]
pub enum SandboxCommand {
    /// List all available sandbox types and their availability status
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Test a specific sandbox type with a simple command
    Test {
        /// Sandbox type to test (docker, podman, seatbelt, none)
        sandbox_type: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show current sandbox configuration
    Config {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Check sandbox prerequisites and configuration
    Doctor {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Set default sandbox configuration for workspace
    Set {
        /// Sandbox type (docker, podman, seatbelt, none)
        sandbox_type: String,

        /// Network mode (open, closed, proxied)
        #[arg(long)]
        network: Option<String>,

        /// Container image (for docker/podman)
        #[arg(long)]
        image: Option<String>,

        /// Working directory inside sandbox
        #[arg(long)]
        working_dir: Option<String>,

        /// Volume mounts (host:container format, can be specified multiple times)
        #[arg(long)]
        volumes: Vec<String>,
    },
}

/// Execute the sandbox command.
pub async fn execute(command: SandboxCommand) -> anyhow::Result<()> {
    match command {
        SandboxCommand::List { json } => list_sandboxes(json).await,
        SandboxCommand::Test { sandbox_type, json } => test_sandbox(sandbox_type.as_deref(), json).await,
        SandboxCommand::Config { json } => show_config(json).await,
        SandboxCommand::Doctor { json } => doctor_check(json).await,
        SandboxCommand::Set {
            sandbox_type,
            network,
            image,
            working_dir,
            volumes,
        } => set_sandbox(sandbox_type, network, image, working_dir, volumes).await,
    }
}

/// List all sandbox types and their availability.
async fn list_sandboxes(json_output: bool) -> anyhow::Result<()> {
    let mut sandboxes = Vec::new();

    // Check NoSandbox (always available)
    sandboxes.push(SandboxInfo {
        name: "none".to_string(),
        available: true,
        notes: Some("Direct execution without sandboxing".to_string()),
    });

    // Check Docker
    let docker_available = std::process::Command::new("docker")
        .arg("--version")
        .output()
        .is_ok();
    sandboxes.push(SandboxInfo {
        name: "docker".to_string(),
        available: docker_available,
        notes: if docker_available {
            get_docker_version()
        } else {
            Some("Docker not installed or not in PATH".to_string())
        },
    });

    // Check Podman
    let podman_available = std::process::Command::new("podman")
        .arg("--version")
        .output()
        .is_ok();
    sandboxes.push(SandboxInfo {
        name: "podman".to_string(),
        available: podman_available,
        notes: if podman_available {
            get_podman_version()
        } else {
            Some("Podman not installed or not in PATH".to_string())
        },
    });

    // Check Seatbelt (macOS only)
    #[cfg(target_os = "macos")]
    {
        let seatbelt_available = std::process::Command::new("which")
            .arg("sandbox-exec")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        sandboxes.push(SandboxInfo {
            name: "seatbelt".to_string(),
            available: seatbelt_available,
            notes: if seatbelt_available {
                Some("macOS Seatbelt sandboxing available".to_string())
            } else {
                Some("sandbox-exec not found (macOS only)".to_string())
            },
        });
    }

    #[cfg(not(target_os = "macos"))]
    {
        sandboxes.push(SandboxInfo {
            name: "seatbelt".to_string(),
            available: false,
            notes: Some("macOS only - not available on this platform".to_string()),
        });
    }

    if json_output {
        let json_data: Vec<_> = sandboxes
            .iter()
            .map(|s| {
                json!({
                    "name": s.name,
                    "available": s.available,
                    "notes": s.notes,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_data)?);
    } else {
        println!();
        println!("{}", "Available Sandbox Types".bold().cyan());
        println!();

        for sandbox in &sandboxes {
            let status = if sandbox.available {
                "✓ Available".green()
            } else {
                "✗ Not Available".red()
            };
            println!("  {}: {}", sandbox.name.bold(), status);
            if let Some(ref notes) = sandbox.notes {
                println!("    {}", notes.dimmed());
            }
        }
        println!();
    }

    Ok(())
}

/// Test a specific sandbox type.
async fn test_sandbox(sandbox_type: Option<&str>, json_output: bool) -> anyhow::Result<()> {
    let sandbox_type = match sandbox_type {
        Some("docker") => SandboxType::Docker,
        Some("podman") => SandboxType::Podman,
        Some("seatbelt") => SandboxType::Seatbelt,
        Some("none") => SandboxType::None,
        Some(unknown) => {
            return Err(anyhow::anyhow!(
                "Unknown sandbox type: {}. Valid types: docker, podman, seatbelt, none",
                unknown
            ));
        }
        None => SandboxType::None, // Default to NoSandbox
    };

    let config = match sandbox_type {
        SandboxType::Docker | SandboxType::Podman => {
            SandboxConfig::new(sandbox_type.clone()).with_image("alpine:latest".to_string())
        }
        _ => SandboxConfig::new(sandbox_type.clone()),
    };

    let start = Instant::now();

    match SandboxFactory::create(&config) {
        Ok(mut sandbox) => {
            // Initialize
            if let Err(e) = sandbox.initialize().await {
                let error_msg = format!("Failed to initialize sandbox: {}", e);
                if json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "success": false,
                            "error": error_msg,
                            "sandbox_type": sandbox_type.to_string(),
                        }))?
                    );
                } else {
                    println!("{}", format!("✗ {}", error_msg).red());
                }
                return Ok(());
            }

            // Execute test command
            let test_result = sandbox
                .execute("echo", &["Sandbox test successful".to_string()], None)
                .await;

            // Cleanup
            let _ = sandbox.cleanup().await;

            let duration = start.elapsed();

            match test_result {
                Ok(output) => {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        if json_output {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&json!({
                                    "success": true,
                                    "sandbox_type": sandbox_type.to_string(),
                                    "duration_ms": duration.as_millis(),
                                    "output": stdout.trim(),
                                }))?
                            );
                        } else {
                            println!("{}", format!("✓ Sandbox test successful").green());
                            println!("  Type: {}", sandbox_type.to_string().bold());
                            println!("  Duration: {}ms", duration.as_millis());
                            println!("  Output: {}", stdout.trim().dimmed());
                        }
                    } else {
                        let error_msg = format!(
                            "Command failed with exit code: {:?}",
                            output.status.code()
                        );
                        if json_output {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&json!({
                                    "success": false,
                                    "error": error_msg,
                                    "sandbox_type": sandbox_type.to_string(),
                                    "duration_ms": duration.as_millis(),
                                }))?
                            );
                        } else {
                            println!("{}", format!("✗ {}", error_msg).red());
                        }
                    }
                }
                Err(e) => {
                    let error_msg = format!("Failed to execute test command: {}", e);
                    if json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&json!({
                                "success": false,
                                "error": error_msg,
                                "sandbox_type": sandbox_type.to_string(),
                                "duration_ms": duration.as_millis(),
                            }))?
                        );
                    } else {
                        println!("{}", format!("✗ {}", error_msg).red());
                    }
                }
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to create sandbox: {}", e);
            if json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "success": false,
                        "error": error_msg,
                        "sandbox_type": sandbox_type.to_string(),
                    }))?
                );
            } else {
                println!("{}", format!("✗ {}", error_msg).red());
            }
        }
    }

    Ok(())
}

/// Show current sandbox configuration.
async fn show_config(json_output: bool) -> anyhow::Result<()> {
    use radium_core::workspace::Workspace;
    use std::fs;
    use toml;

    // Try to load from workspace config
    let config = if let Ok(workspace) = Workspace::discover() {
        let config_path = workspace.radium_dir().join("config.toml");
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(workspace_config) = toml::from_str::<toml::Value>(&content) {
                    if let Some(sandbox_table) = workspace_config.get("sandbox") {
                        // Deserialize sandbox config from TOML table
                    if let Ok(sandbox_config) = toml::from_str::<SandboxConfig>(
                        &toml::to_string(sandbox_table)?
                    ) {
                        sandbox_config
                    } else {
                        SandboxConfig::default()
                    }
                    } else {
                        SandboxConfig::default()
                    }
                } else {
                    SandboxConfig::default()
                }
            } else {
                SandboxConfig::default()
            }
        } else {
            SandboxConfig::default()
        }
    } else {
        SandboxConfig::default()
    };

    if json_output {
        let mut env_map = HashMap::new();
        for (k, v) in &config.env {
            env_map.insert(k.clone(), v.clone());
        }

        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "sandbox_type": config.sandbox_type.to_string(),
                "network": format!("{:?}", config.network),
                "profile": match &config.profile {
                    SandboxProfile::Permissive => "permissive",
                    SandboxProfile::Restrictive => "restrictive",
                    SandboxProfile::Custom(path) => return Err(anyhow::anyhow!("Custom profile path: {}", path)),
                },
                "image": config.image,
                "working_dir": config.working_dir,
                "volumes": config.volumes,
                "env": env_map,
                "custom_flags": config.custom_flags,
            }))?
        );
    } else {
        println!();
        println!("{}", "Sandbox Configuration".bold().cyan());
        println!();
        println!("  Type: {}", config.sandbox_type.to_string().bold());
        println!("  Network: {:?}", config.network);
        println!(
            "  Profile: {}",
            match &config.profile {
                SandboxProfile::Permissive => "permissive",
                SandboxProfile::Restrictive => "restrictive",
                SandboxProfile::Custom(path) => path.as_str(),
            }
        );
        if let Some(ref image) = config.image {
            println!("  Image: {}", image);
        }
        if let Some(ref working_dir) = config.working_dir {
            println!("  Working Directory: {}", working_dir);
        }
        if !config.volumes.is_empty() {
            println!("  Volumes: {}", config.volumes.join(", "));
        }
        if !config.env.is_empty() {
            println!("  Environment Variables: {} set", config.env.len());
        }
        if !config.custom_flags.is_empty() {
            println!("  Custom Flags: {}", config.custom_flags.join(" "));
        }
        println!();
        if config.sandbox_type == SandboxType::None && config.network == NetworkMode::Open {
            println!("{}", "Note: Using default configuration (NoSandbox). Use 'rad sandbox set' to configure.".dimmed());
        }
        println!();
    }

    Ok(())
}

/// Check sandbox prerequisites and configuration.
async fn doctor_check(json_output: bool) -> anyhow::Result<()> {
    if json_output {
        let mut checks = Vec::new();

        // Check Docker
        let docker_available = std::process::Command::new("docker")
            .arg("--version")
            .output()
            .is_ok();
        checks.push(json!({
            "name": "Docker",
            "status": if docker_available { "available" } else { "not_available" },
            "version": if docker_available { get_docker_version().unwrap_or_default() } else { String::new() },
        }));

        // Check Podman
        let podman_available = std::process::Command::new("podman")
            .arg("--version")
            .output()
            .is_ok();
        checks.push(json!({
            "name": "Podman",
            "status": if podman_available { "available" } else { "not_available" },
            "version": if podman_available { get_podman_version().unwrap_or_default() } else { String::new() },
        }));

        // Check Seatbelt
        #[cfg(target_os = "macos")]
        {
            let seatbelt_available = std::process::Command::new("which")
                .arg("sandbox-exec")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            checks.push(json!({
                "name": "Seatbelt",
                "status": if seatbelt_available { "available" } else { "not_available" },
                "platform": "macOS",
            }));
        }

        #[cfg(not(target_os = "macos"))]
        {
            checks.push(json!({
                "name": "Seatbelt",
                "status": "not_available",
                "platform": "macOS only",
            }));
        }

        println!("{}", serde_json::to_string_pretty(&json!({ "checks": checks }))?);
    } else {
        println!();
        println!("{}", "Sandbox Doctor - Prerequisites Check".bold().cyan());
        println!();

        // Check Docker
        println!("{}", "Docker:".bold());
        match std::process::Command::new("docker").arg("--version").output() {
            Ok(output) => {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    println!("  Status: {}", "✓ Available".green());
                    println!("  Version: {}", version.dimmed());
                } else {
                    println!("  Status: {}", "✗ Not Available".red());
                }
            }
            Err(_) => {
                println!("  Status: {}", "✗ Not Installed".red());
                println!("  {}", "Install: https://docs.docker.com/get-docker/".dimmed());
            }
        }
        println!();

        // Check Podman
        println!("{}", "Podman:".bold());
        match std::process::Command::new("podman").arg("--version").output() {
            Ok(output) => {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    println!("  Status: {}", "✓ Available".green());
                    println!("  Version: {}", version.dimmed());
                } else {
                    println!("  Status: {}", "✗ Not Available".red());
                }
            }
            Err(_) => {
                println!("  Status: {}", "✗ Not Installed".red());
                println!("  {}", "Install: https://podman.io/getting-started/installation".dimmed());
            }
        }
        println!();

        // Check Seatbelt
        println!("{}", "Seatbelt (macOS):".bold());
        #[cfg(target_os = "macos")]
        {
            match std::process::Command::new("which").arg("sandbox-exec").output() {
                Ok(output) => {
                    if output.status.success() {
                        println!("  Status: {}", "✓ Available".green());
                        println!("  {}", "macOS native sandboxing".dimmed());
                    } else {
                        println!("  Status: {}", "✗ Not Available".red());
                    }
                }
                Err(_) => {
                    println!("  Status: {}", "✗ Not Available".red());
                }
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            println!("  Status: {}", "✗ Not Available (macOS only)".yellow());
        }
        println!();
    }

    Ok(())
}

/// Set sandbox configuration for workspace.
async fn set_sandbox(
    sandbox_type: String,
    network: Option<String>,
    image: Option<String>,
    working_dir: Option<String>,
    volumes: Vec<String>,
) -> anyhow::Result<()> {
    use radium_core::sandbox::{NetworkMode, SandboxConfig, SandboxFactory, SandboxType};
    use radium_core::workspace::Workspace;
    use std::fs;
    use toml;

    // Parse sandbox type
    let sandbox_type_enum = match sandbox_type.to_lowercase().as_str() {
        "none" => SandboxType::None,
        "docker" => SandboxType::Docker,
        "podman" => SandboxType::Podman,
        "seatbelt" => SandboxType::Seatbelt,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid sandbox type: {}. Valid types: none, docker, podman, seatbelt",
                sandbox_type
            ));
        }
    };

    // Validate sandbox availability before saving
    let test_config = SandboxConfig::new(sandbox_type_enum.clone());
    match SandboxFactory::create(&test_config) {
        Ok(_) => {
            // Sandbox is available
        }
        Err(e) => {
            if matches!(e, radium_core::sandbox::SandboxError::NotAvailable(_)) {
                println!("{}", format!("⚠ Warning: Sandbox type '{}' is not available on this system", sandbox_type).yellow());
                println!("{}", "Configuration will be saved, but sandbox will not be used until available.".dimmed());
            } else {
                return Err(anyhow::anyhow!("Failed to validate sandbox: {}", e));
            }
        }
    }

    // Parse network mode
    let network_mode = if let Some(net) = network {
        match net.to_lowercase().as_str() {
            "open" => NetworkMode::Open,
            "closed" => NetworkMode::Closed,
            "proxied" => NetworkMode::Proxied,
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid network mode: {}. Valid modes: open, closed, proxied",
                    net
                ));
            }
        }
    } else {
        NetworkMode::Open // Default
    };

    // Build sandbox config
    let mut config = SandboxConfig::new(sandbox_type_enum)
        .with_network(network_mode.clone());

    if let Some(img) = image.clone() {
        config = config.with_image(img);
    }

    if let Some(wd) = working_dir.clone() {
        config = config.with_working_dir(wd);
    }

    if !volumes.is_empty() {
        config = config.with_volumes(volumes.clone());
    }

    // Discover workspace
    let workspace = Workspace::discover()
        .map_err(|_| anyhow::anyhow!("No Radium workspace found. Run 'rad init' first."))?;

    // Load existing config or create new
    let config_path = workspace.radium_dir().join("config.toml");
    let mut workspace_config: toml::Value = if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        toml::from_str(&content)?
    } else {
        toml::Value::Table(toml::map::Map::new())
    };

    // Update sandbox section
    // Convert SandboxConfig to toml::Value via string serialization
    let sandbox_str = toml::to_string(&config)?;
    let sandbox_table: toml::Value = toml::from_str(&sandbox_str)?;
    workspace_config
        .as_table_mut()
        .ok_or_else(|| anyhow::anyhow!("Invalid config format"))?
        .insert("sandbox".to_string(), sandbox_table);

    // Write config back
    let config_str = toml::to_string_pretty(&workspace_config)?;
    fs::write(&config_path, config_str)?;

    println!();
    println!("{}", "✓ Sandbox configuration updated".green().bold());
    println!("  Type: {}", sandbox_type.bold());
    println!("  Network: {}", format!("{:?}", network_mode).bold());
    if let Some(ref img) = image {
        println!("  Image: {}", img.bold());
    }
    if let Some(ref wd) = working_dir {
        println!("  Working Directory: {}", wd.bold());
    }
    if !volumes.is_empty() {
        println!("  Volumes: {}", volumes.join(", ").bold());
    }
    println!();

    Ok(())
}

/// Get Docker version string.
fn get_docker_version() -> Option<String> {
    std::process::Command::new("docker")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
}

/// Get Podman version string.
fn get_podman_version() -> Option<String> {
    std::process::Command::new("podman")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
}

/// Sandbox information for listing.
struct SandboxInfo {
    name: String,
    available: bool,
    notes: Option<String>,
}

