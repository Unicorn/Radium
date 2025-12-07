//! Comprehensive integration tests for the `rad status` command.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Helper to initialize a workspace for testing
fn init_workspace(temp_dir: &TempDir) {
    let temp_path = temp_dir.path().to_str().unwrap();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("init").arg("--use-defaults").arg(temp_path).assert().success();
}

#[test]
fn test_status_no_workspace() {
    // Run status outside of a workspace
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();

    cmd.current_dir(temp_dir.path())
        .arg("status")
        .assert()
        .success() // It exits with 0 even if no workspace is found
        .stdout(predicate::str::contains("workspace not found"));
}

#[test]
fn test_status_in_workspace() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Radium Status"))
        .stdout(predicate::str::contains("Valid: ✓"));
}

#[test]
fn test_status_json_output() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let assert = cmd.current_dir(temp_dir.path()).arg("status").arg("--json").assert().success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify it's valid JSON
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Status JSON output should be valid JSON");

    // Verify JSON structure
    assert!(json.is_object(), "Status JSON should be an object");
    // The exact structure depends on implementation, but should have workspace info
}

#[test]
fn test_status_shows_workspace_path() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains(temp_dir.path().to_str().unwrap()));
}

#[test]
fn test_status_shows_workspace_validity() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Valid: ✓").or(predicate::str::contains("Valid: true")));
}

#[test]
fn test_status_in_subdirectory_of_workspace() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Create a subdirectory
    let subdir = temp_dir.path().join("subdir");
    std::fs::create_dir_all(&subdir).unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(&subdir)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Radium Status"));
}

#[test]
fn test_status_shows_agents_section() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Agents:"));
}

#[test]
fn test_status_shows_models_section() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Models:"));
}

#[test]
fn test_status_shows_authentication_section() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Authentication:"));
}

#[test]
fn test_status_json_has_workspace_field() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let assert = cmd.current_dir(temp_dir.path()).arg("status").arg("--json").assert().success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert!(json.get("workspace").is_some(), "JSON should have workspace field");
}

#[test]
fn test_status_json_has_agents_field() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let assert = cmd.current_dir(temp_dir.path()).arg("status").arg("--json").assert().success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert!(json.get("agents").is_some(), "JSON should have agents field");
}

#[test]
fn test_status_json_has_models_field() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let assert = cmd.current_dir(temp_dir.path()).arg("status").arg("--json").assert().success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert!(json.get("models").is_some(), "JSON should have models field");
}

#[test]
fn test_status_json_has_auth_field() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let assert = cmd.current_dir(temp_dir.path()).arg("status").arg("--json").assert().success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert!(json.get("auth").is_some(), "JSON should have auth field");
}

#[test]
fn test_status_with_agents_present() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Create a test agent
    let agents_dir = temp_dir.path().join("agents");
    std::fs::create_dir_all(&agents_dir).unwrap();
    let config_content = r#"[agent]
id = "test-agent"
name = "Test Agent"
description = "A test agent"
prompt_path = "prompts/test.md"
engine = "mock"
model = "test-model"
reasoning_effort = "medium"
category = "test"
"#;
    std::fs::write(agents_dir.join("test-agent.toml"), config_content).unwrap();

    // Create prompt file
    let prompts_dir = temp_dir.path().join("prompts");
    std::fs::create_dir_all(&prompts_dir).unwrap();
    std::fs::write(prompts_dir.join("test.md"), "# Test Agent\n\nTest prompt.").unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Total:").or(predicate::str::contains("agents")));
}
