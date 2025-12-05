//! Comprehensive integration tests for the `rad agents` command.

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

/// Helper to create a test agent configuration
/// Agents are discovered from ./agents/ directory (project-local)
fn create_test_agent(temp_dir: &TempDir, agent_id: &str, name: &str) {
    // Create agents in ./agents/ directory (project-local, not in .radium)
    let agents_dir = temp_dir.path().join("agents");
    fs::create_dir_all(&agents_dir).unwrap();

    let config_content = format!(
        r#"[agent]
id = "{}"
name = "{}"
description = "A test agent for integration testing"
prompt_path = "prompts/{}.md"
engine = "mock"
model = "test-model"
reasoning_effort = "medium"
category = "test"
"#,
        agent_id, name, agent_id
    );

    fs::write(agents_dir.join(format!("{}.toml", agent_id)), config_content).unwrap();
}

#[test]
fn test_agents_list_no_agents() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("agents")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No agents found"));
}

#[test]
fn test_agents_list_with_agents() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("agents")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-agent"))
        .stdout(predicate::str::contains("Test Agent"));
}

#[test]
fn test_agents_list_json_output() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let assert =
        cmd.current_dir(temp_dir.path()).arg("agents").arg("list").arg("--json").assert().success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify it's valid JSON
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Agents list JSON output should be valid JSON");

    assert!(json.is_array(), "Agents list JSON should be an array");
    assert!(json.as_array().unwrap().len() > 0, "Should have at least one agent");
}

#[test]
fn test_agents_list_verbose() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("agents")
        .arg("list")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-agent"));
}

#[test]
fn test_agents_search() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");
    create_test_agent(&temp_dir, "other-agent", "Other Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("agents")
        .arg("search")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-agent"))
        .stdout(predicate::str::contains("Test Agent"));
}

#[test]
fn test_agents_search_no_results() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("agents")
        .arg("search")
        .arg("nonexistent")
        .assert()
        .success();
    // Should not contain test-agent
}

#[test]
fn test_agents_search_json_output() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let assert = cmd
        .current_dir(temp_dir.path())
        .arg("agents")
        .arg("search")
        .arg("test")
        .arg("--json")
        .assert()
        .success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify it's valid JSON
    let _json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Agents search JSON output should be valid JSON");
}

#[test]
fn test_agents_info() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("agents")
        .arg("info")
        .arg("test-agent")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-agent"))
        .stdout(predicate::str::contains("Test Agent"));
}

#[test]
fn test_agents_info_not_found() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("agents")
        .arg("info")
        .arg("nonexistent-agent")
        .assert()
        .failure(); // Should fail if agent not found
}

#[test]
fn test_agents_info_json_output() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let assert = cmd
        .current_dir(temp_dir.path())
        .arg("agents")
        .arg("info")
        .arg("test-agent")
        .arg("--json")
        .assert()
        .success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify it's valid JSON
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Agent info JSON output should be valid JSON");

    assert!(json.is_object(), "Agent info JSON should be an object");
    assert_eq!(json["id"], "test-agent");
}

#[test]
fn test_agents_validate_valid() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path()).arg("agents").arg("validate").assert().success();
}

#[test]
fn test_agents_validate_verbose() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("agents")
        .arg("validate")
        .arg("--verbose")
        .assert()
        .success();
}
