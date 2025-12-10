//! Integration tests for reasoning effort precedence and propagation.

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

/// Helper to create a test agent configuration with reasoning effort
fn create_test_agent_with_reasoning(
    temp_dir: &TempDir,
    agent_id: &str,
    name: &str,
    reasoning_effort: Option<&str>,
) {
    let agents_dir = temp_dir.path().join("agents");
    fs::create_dir_all(&agents_dir).unwrap();

    // Create prompt file
    let prompts_dir = temp_dir.path().join("prompts");
    fs::create_dir_all(&prompts_dir).unwrap();
    fs::write(
        prompts_dir.join(format!("{}.md", agent_id)),
        format!("# {}\n\nYou are a test agent.\n\n## User Input\n\n{{user_input}}", name),
    )
    .unwrap();

    let reasoning_line = if let Some(effort) = reasoning_effort {
        format!("reasoning_effort = \"{}\"", effort)
    } else {
        String::new()
    };

    let config_content = format!(
        r#"[agent]
id = "{}"
name = "{}"
description = "A test agent for integration testing"
prompt_path = "prompts/{}.md"
engine = "mock"
model = "test-model"
{}
category = "test"
"#,
        agent_id, name, agent_id, reasoning_line
    );

    fs::write(agents_dir.join(format!("{}.toml", agent_id)), config_content).unwrap();
}

#[test]
fn test_reasoning_effort_cli_override() {
    // Test: CLI flag overrides agent config
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent_with_reasoning(&temp_dir, "test-agent", "Test Agent", Some("low"));

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("step")
        .arg("test-agent")
        .arg("Test prompt")
        .arg("--reasoning")
        .arg("high")
        .assert()
        .success()
        .stdout(predicate::str::contains("Reasoning: high"));
}

#[test]
fn test_reasoning_effort_from_config() {
    // Test: Agent config used when no CLI flag
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent_with_reasoning(&temp_dir, "test-agent", "Test Agent", Some("high"));

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("step")
        .arg("test-agent")
        .arg("Test prompt")
        .assert()
        .success()
        .stdout(predicate::str::contains("Reasoning: high"));
}

#[test]
fn test_reasoning_effort_default() {
    // Test: Default used when neither specified
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent_with_reasoning(&temp_dir, "test-agent", "Test Agent", None);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("step")
        .arg("test-agent")
        .arg("Test prompt")
        .assert()
        .success()
        .stdout(predicate::str::contains("Reasoning: medium")); // Default
}

#[test]
fn test_reasoning_effort_invalid_value() {
    // Test: Invalid reasoning effort value is handled gracefully
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent_with_reasoning(&temp_dir, "test-agent", "Test Agent", None);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("step")
        .arg("test-agent")
        .arg("Test prompt")
        .arg("--reasoning")
        .arg("invalid")
        .assert()
        .success(); // Should fall back to default or handle gracefully
}

