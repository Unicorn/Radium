//! Integration tests for chat session persistence.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn init_workspace(temp_dir: &TempDir) {
    let temp_path = temp_dir.path().to_str().unwrap();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("init").arg("--use-defaults").arg(temp_path).assert().success();
}

fn create_mock_agent(temp_dir: &TempDir) {
    let agents_dir = temp_dir.path().join("agents");
    fs::create_dir_all(&agents_dir).unwrap();

    let prompts_dir = temp_dir.path().join("agents").join("mock-agent").join("prompts");
    fs::create_dir_all(&prompts_dir).unwrap();
    fs::write(prompts_dir.join("mock-agent.md"), "You are a mock test agent.").unwrap();

    let config = r#"[agent]
id = "mock-agent"
name = "Mock Agent"
description = "A mock agent for testing"
prompt_path = "agents/mock-agent/prompts/mock-agent.md"
engine = "mock"
model = "mock-model"
reasoning_effort = "medium"
category = "test"
"#;
    fs::write(agents_dir.join("mock-agent.toml"), config).unwrap();
}

fn sessions_dir(temp_dir: &TempDir) -> std::path::PathBuf {
    temp_dir.path().join(".radium/_internals/sessions")
}

#[test]
fn test_chat_creates_session_directory() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_mock_agent(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("chat")
        .arg("mock-agent")
        .arg("--session")
        .arg("test-session-dir")
        .write_stdin("/quit\n")
        .assert()
        .success();

    let session_json = sessions_dir(&temp_dir).join("test-session-dir").join("session.json");
    assert!(
        session_json.exists(),
        "session.json should be created at: {}",
        session_json.display()
    );
}

#[test]
fn test_chat_resume_shows_message_count() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_mock_agent(&temp_dir);

    // Pre-create session files
    let session_dir = sessions_dir(&temp_dir).join("resume-test");
    fs::create_dir_all(&session_dir).unwrap();

    let session_json = r#"{
  "id": "resume-test",
  "created_at": "2024-01-01T00:00:00Z",
  "last_active": "2024-01-01T00:00:01Z",
  "state": "ACTIVE",
  "agent_id": "mock-agent",
  "workspace_root": null,
  "name": null,
  "messages": [],
  "tool_calls": [],
  "approvals": [],
  "artifacts": [],
  "metadata": {}
}"#;
    fs::write(session_dir.join("session.json"), session_json).unwrap();

    // Write 2 messages to messages.jsonl
    let messages_jsonl = "{\"id\":\"msg1\",\"content\":\"hello from before\",\"role\":\"user\",\"timestamp\":\"2024-01-01T00:00:00Z\"}\n{\"id\":\"msg2\",\"content\":\"Mock response from before\",\"role\":\"assistant\",\"timestamp\":\"2024-01-01T00:00:01Z\"}\n";
    fs::write(session_dir.join("messages.jsonl"), messages_jsonl).unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let assert = cmd
        .current_dir(temp_dir.path())
        .arg("chat")
        .arg("mock-agent")
        .arg("--session")
        .arg("resume-test")
        .arg("--resume")
        .write_stdin("/quit\n")
        .assert()
        .success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Loaded") && (stdout.contains("2") || stdout.contains("messages")),
        "stdout should say 'Loaded N messages from previous session', got: {stdout}"
    );
}

#[test]
fn test_chat_appends_messages_to_jsonl() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_mock_agent(&temp_dir);

    // Run one chat turn then quit
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("chat")
        .arg("mock-agent")
        .arg("--session")
        .arg("append-test")
        .write_stdin("hello from test\n/quit\n")
        .assert()
        .success();

    let messages_path = sessions_dir(&temp_dir).join("append-test").join("messages.jsonl");
    assert!(
        messages_path.exists(),
        "messages.jsonl should exist after a chat turn"
    );

    let content = fs::read_to_string(&messages_path).unwrap();
    let line_count = content.lines().count();
    assert_eq!(
        line_count, 2,
        "messages.jsonl should have 2 lines (1 user + 1 assistant), got {} lines",
        line_count
    );

    // Verify roles
    assert!(
        content.contains("\"user\""),
        "messages.jsonl should contain user role"
    );
    assert!(
        content.contains("\"assistant\""),
        "messages.jsonl should contain assistant role"
    );
}
