//! Tests for MCP CLI commands

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to initialize a workspace for testing
fn init_workspace(temp_dir: &TempDir) {
    let temp_path = temp_dir.path().to_str().unwrap();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("init").arg("--use-defaults").arg(temp_path).assert().success();
}

/// Helper to create MCP server configuration
fn create_mcp_config(temp_dir: &TempDir, content: &str) {
    let radium_dir = temp_dir.path().join(".radium");
    fs::create_dir_all(&radium_dir).unwrap();
    fs::write(radium_dir.join("mcp-servers.toml"), content).unwrap();
}

#[test]
fn test_mcp_list_no_config() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("mcp")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No MCP servers configured"));
}

#[test]
fn test_mcp_list_with_config() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let config = r#"
[[servers]]
name = "test-server"
transport = "stdio"
command = "mcp-server"
args = ["--config", "config.json"]
"#;

    create_mcp_config(&temp_dir, config);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("mcp")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-server"))
        .stdout(predicate::str::contains("Stdio"));
}

#[test]
fn test_mcp_list_multiple_servers() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let config = r#"
[[servers]]
name = "stdio-server"
transport = "stdio"
command = "mcp-server"

[[servers]]
name = "http-server"
transport = "http"
url = "https://api.example.com/mcp"
"#;

    create_mcp_config(&temp_dir, config);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("mcp")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("stdio-server"))
        .stdout(predicate::str::contains("http-server"));
}

#[test]
fn test_mcp_tools_no_servers() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("mcp")
        .arg("tools")
        .assert()
        .success()
        .stdout(predicate::str::contains("No tools available"));
}

#[test]
fn test_mcp_prompts_no_servers() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("mcp")
        .arg("prompts")
        .assert()
        .success()
        .stdout(predicate::str::contains("No MCP prompts available"));
}

#[test]
fn test_mcp_test_no_servers() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("mcp")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("No servers connected"));
}

#[test]
fn test_mcp_test_specific_server() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let config = r#"
[[servers]]
name = "test-server"
transport = "stdio"
command = "nonexistent-command"
"#;

    create_mcp_config(&temp_dir, config);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("mcp")
        .arg("test")
        .arg("test-server")
        .assert()
        .success();
    // Should show connection failure (command doesn't exist)
}

#[test]
fn test_mcp_auth_status_no_tokens() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("mcp")
        .arg("auth")
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("No OAuth tokens found"));
}

