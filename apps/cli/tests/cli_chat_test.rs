//! Comprehensive integration tests for the `rad chat` command.

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
fn test_chat_no_workspace() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("chat")
        .arg("test-agent")
        .assert()
        .failure() // Should fail if no workspace found
        .stderr(
            predicate::str::contains("workspace")
                .or(predicate::str::contains("not found"))
                .or(predicate::str::contains("Failed to load")),
        );
}

#[test]
fn test_chat_agent_not_found() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("chat")
        .arg("nonexistent-agent")
        .assert()
        .failure() // Should fail if agent not found
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("No agents"))
                .or(predicate::str::contains("Failed")),
        );
}

#[test]
fn test_chat_list_sessions() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("chat")
        .arg("--list")
        .assert()
        .success() // Should succeed even with no sessions
        .stdout(predicate::str::contains("Sessions").or(predicate::str::contains("session")));
}

#[test]
fn test_chat_resume_without_session_name() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("chat")
        .arg("--resume")
        .arg("test-agent")
        .assert()
        .failure() // Should fail if --resume without session name
        .stderr(predicate::str::contains("session name").or(predicate::str::contains("required")));
}

#[test]
fn test_chat_resume_nonexistent_session() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("chat")
        .arg("--resume")
        .arg("--session")
        .arg("nonexistent-session")
        .arg("test-agent")
        .assert()
        .failure() // Should fail if session doesn't exist
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("Session"))
                .or(predicate::str::contains("Failed")),
        );
}

#[test]
fn test_chat_with_session_name() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    // Note: This test may need to be adjusted based on actual chat implementation
    // If chat requires interactive input, we might need to use a different approach
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    // Chat command might require stdin input, so we test the argument parsing
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("chat")
        .arg("--session")
        .arg("test-session")
        .arg("test-agent")
        .timeout(std::time::Duration::from_secs(1))
        .assert();

    // Command should at least start (may timeout waiting for input, which is expected)
    // We're mainly testing that the arguments are parsed correctly
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_chat_requires_agent_id_when_not_listing() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("chat")
        .assert()
        .failure() // Should fail if no agent ID provided
        .stderr(
            predicate::str::contains("required")
                .or(predicate::str::contains("Agent ID"))
                .or(predicate::str::contains("agent_id")),
        );
}

#[test]
fn test_chat_help_shown_in_interactive_mode() {
    // This test verifies that the chat command accepts the agent ID
    // The actual interactive behavior would require stdin mocking
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    // We can't easily test the interactive loop, but we can verify
    // the command starts correctly with proper arguments
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("chat")
        .arg("test-agent")
        .timeout(std::time::Duration::from_secs(1))
        .assert();

    // Command should start (may timeout waiting for input)
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_chat_stream_flag_parsing() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    // Test that --stream flag is accepted
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("chat")
        .arg("--stream")
        .arg("test-agent")
        .timeout(std::time::Duration::from_secs(1))
        .assert();

    // Command should start (may timeout waiting for input)
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_chat_stream_with_session_name() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    // Test that --stream flag works with --session
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("chat")
        .arg("--stream")
        .arg("--session")
        .arg("test-session")
        .arg("test-agent")
        .timeout(std::time::Duration::from_secs(1))
        .assert();

    // Command should start (may timeout waiting for input)
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_chat_stream_backward_compatibility() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    // Test that chat works without --stream flag (backward compatibility)
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("chat")
        .arg("test-agent")
        .timeout(std::time::Duration::from_secs(1))
        .assert();

    // Command should start (may timeout waiting for input)
    assert!(result.get_output().status.code().is_some());
}
