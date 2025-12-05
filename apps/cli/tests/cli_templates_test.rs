//! Comprehensive integration tests for the `rad templates` command.

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

/// Helper to create a test workflow template
fn create_test_template(temp_dir: &TempDir, template_name: &str) {
    let templates_dir = temp_dir.path().join("templates");
    fs::create_dir_all(&templates_dir).unwrap();

    // Use the correct WorkflowTemplate structure (matching basic-workflow.json format)
    let template_content = format!(
        r#"{{
  "name": "{}",
  "description": "A test workflow template",
  "steps": [
    {{
      "agentId": "test-agent",
      "agentName": "Test Agent",
      "type": "step",
      "executeOnce": false
    }}
  ],
  "subAgentIds": ["test-agent"]
}}
"#,
        template_name
    );

    fs::write(templates_dir.join(format!("{}.json", template_name)), template_content).unwrap();
}

#[test]
fn test_templates_list_no_templates() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("templates")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No templates found"));
}

#[test]
fn test_templates_list_with_templates() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_template(&temp_dir, "test-template");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("templates")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-template"));
}

#[test]
fn test_templates_list_json_output() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_template(&temp_dir, "test-template");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let assert = cmd
        .current_dir(temp_dir.path())
        .arg("templates")
        .arg("list")
        .arg("--json")
        .assert()
        .success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify it's valid JSON
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Templates list JSON output should be valid JSON");

    assert!(json.is_array(), "Templates list JSON should be an array");
}

#[test]
fn test_templates_list_verbose() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_template(&temp_dir, "test-template");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("templates")
        .arg("list")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-template"));
}

#[test]
fn test_templates_info() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_template(&temp_dir, "test-template");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("templates")
        .arg("info")
        .arg("test-template")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-template"));
}

#[test]
fn test_templates_info_not_found() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("templates")
        .arg("info")
        .arg("nonexistent-template")
        .assert()
        .failure(); // Should fail if template not found
}

#[test]
fn test_templates_info_json_output() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_template(&temp_dir, "test-template");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let assert = cmd
        .current_dir(temp_dir.path())
        .arg("templates")
        .arg("info")
        .arg("test-template")
        .arg("--json")
        .assert()
        .success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify it's valid JSON
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Template info JSON output should be valid JSON");

    assert!(json.is_object(), "Template info JSON should be an object");
    assert_eq!(json["name"], "test-template");
}

#[test]
fn test_templates_validate_valid() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_template(&temp_dir, "test-template");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path()).arg("templates").arg("validate").assert().success();
}

#[test]
fn test_templates_validate_verbose() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_template(&temp_dir, "test-template");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("templates")
        .arg("validate")
        .arg("--verbose")
        .assert()
        .success();
}
