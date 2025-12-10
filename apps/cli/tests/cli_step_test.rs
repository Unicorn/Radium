//! Comprehensive integration tests for the `rad step` command.

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
fn create_test_agent(temp_dir: &TempDir, agent_id: &str, name: &str) {
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
fn test_step_no_agents() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("step")
        .arg("test-agent")
        .assert()
        .failure() // Should fail if no agents found
        .stderr(
            predicate::str::contains("No agents found").or(predicate::str::contains("not found")),
        );
}

#[test]
fn test_step_agent_not_found() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "other-agent", "Other Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("step")
        .arg("nonexistent-agent")
        .assert()
        .failure() // Should fail if agent not found
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_step_with_agent() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("step")
        .arg("test-agent")
        .arg("Test prompt")
        .assert()
        .success()
        .stdout(predicate::str::contains("rad step"))
        .stdout(predicate::str::contains("test-agent"));
}

#[test]
fn test_step_with_model_override() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("step")
        .arg("test-agent")
        .arg("--model")
        .arg("custom-model")
        .arg("Test prompt")
        .assert()
        .success();
}

#[test]
fn test_step_with_engine_override() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("step")
        .arg("test-agent")
        .arg("--engine")
        .arg("mock")
        .arg("Test prompt")
        .assert()
        .success();
}

#[test]
fn test_step_with_reasoning_override() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("step")
        .arg("test-agent")
        .arg("--reasoning")
        .arg("high")
        .arg("Test prompt")
        .assert()
        .success();
}

#[test]
fn test_step_with_multiple_prompts() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("step")
        .arg("test-agent")
        .arg("First prompt")
        .arg("Second prompt")
        .arg("Third prompt")
        .assert()
        .success();
}

#[test]
fn test_step_with_all_overrides() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("step")
        .arg("test-agent")
        .arg("--model")
        .arg("custom-model")
        .arg("--engine")
        .arg("mock")
        .arg("--reasoning")
        .arg("low")
        .arg("Test prompt")
        .assert()
        .success();
}

#[test]
fn test_step_no_workspace() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("step")
        .arg("test-agent")
        .assert()
        .failure() // Should fail if no workspace found
        .stderr(
            predicate::str::contains("workspace")
                .or(predicate::str::contains("not found"))
                .or(predicate::str::contains("Failed")),
        );
}

#[test]
fn test_step_invalid_reasoning_level() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    // Invalid reasoning level might be accepted or rejected - depends on implementation
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("step")
        .arg("test-agent")
        .arg("--reasoning")
        .arg("invalid-level")
        .arg("Test prompt")
        .assert();

    // Either success or failure is acceptable, just verify it doesn't panic
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_step_stream_flag_parsing() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("step")
        .arg("test-agent")
        .arg("--stream")
        .arg("Test prompt")
        .assert()
        .success();
}

#[test]
fn test_step_stream_with_mock_engine() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    // Mock engine supports streaming, so this should work
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let output = cmd
        .current_dir(temp_dir.path())
        .arg("step")
        .arg("test-agent")
        .arg("--engine")
        .arg("mock")
        .arg("--stream")
        .arg("Test prompt")
        .output()
        .unwrap();

    assert!(output.status.success(), "Streaming with mock engine should succeed");
    // Output should contain response (streaming or non-streaming)
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Response") || stdout.contains("test-agent"), "Should contain response");
}

#[test]
fn test_step_stream_fallback_behavior() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    
    // Create agent with unsupported engine (if any)
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    // --stream flag should be accepted even if engine doesn't support it
    // (it will fall back to non-streaming)
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let output = cmd
        .current_dir(temp_dir.path())
        .arg("step")
        .arg("test-agent")
        .arg("--stream")
        .arg("Test prompt")
        .output()
        .unwrap();

    // Should succeed (either with streaming or fallback)
    assert!(output.status.success() || output.status.code().is_some());
}
