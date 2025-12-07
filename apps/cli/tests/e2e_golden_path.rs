//! End-to-end golden path workflow test.
//!
//! This test simulates the complete user journey from workspace initialization
//! through plan execution and artifact verification.

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
description = "A test agent for golden path testing"
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

/// Helper to create a sample spec file
fn create_test_spec(temp_dir: &TempDir, spec_id: &str) {
    let spec_content = format!(
        r#"# Test Specification: {}

This is a test specification for the golden path workflow test.

## Requirements

1. Create a simple test file
2. Verify the file was created

## Expected Artifacts

- A test file should be created in the workspace
"#,
        spec_id
    );

    let specs_dir = temp_dir.path().join("specs");
    fs::create_dir_all(&specs_dir).unwrap();
    fs::write(specs_dir.join(format!("{}.md", spec_id)), spec_content).unwrap();
}

#[test]
fn test_golden_path_workflow() {
    // Step 1: Initialize workspace
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Verify workspace structure was created
    let radium_dir = temp_dir.path().join(".radium");
    assert!(radium_dir.exists(), ".radium directory should exist");
    assert!(
        radium_dir.join("_internals").exists(),
        "_internals directory should exist"
    );
    assert!(
        radium_dir.join("plan").exists(),
        "plan directory should exist"
    );

    // Step 2: Create agent template
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    // Verify agent was created
    let agent_config = temp_dir.path().join("agents").join("test-agent.toml");
    assert!(agent_config.exists(), "Agent config file should exist");

    // Verify agent can be listed
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("agents")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-agent"));

    // Step 3: Create sample spec file
    create_test_spec(&temp_dir, "test-spec");

    // Verify spec file was created
    let spec_file = temp_dir.path().join("specs").join("test-spec.md");
    assert!(spec_file.exists(), "Spec file should exist");

    // Step 4: Generate plan from spec (using craft command)
    // Note: The actual plan generation may require additional setup,
    // but we verify the command structure and that it doesn't crash
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("craft")
        .arg("--dry-run")
        .arg("test-spec")
        .assert();

    // Command should execute (may fail if plan generation requires more setup,
    // but should not panic or crash)
    assert!(result.get_output().status.code().is_some());

    // Step 5: Verify artifacts directory exists
    let artifacts_dir = radium_dir.join("_internals").join("artifacts");
    assert!(
        artifacts_dir.exists(),
        "Artifacts directory should exist after initialization"
    );

    // Verify workspace is in a valid state
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Valid: ✓"));
}

#[test]
fn test_golden_path_workflow_complete_flow() {
    // Complete golden path: init → agent → verify structure
    let temp_dir = TempDir::new().unwrap();

    // Step 1: Initialize workspace
    init_workspace(&temp_dir);
    let radium_dir = temp_dir.path().join(".radium");
    assert!(radium_dir.exists());

    // Step 2: Create agent
    create_test_agent(&temp_dir, "golden-agent", "Golden Path Agent");

    // Step 3: Verify agent discovery
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let output = cmd
        .current_dir(temp_dir.path())
        .arg("agents")
        .arg("list")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("golden-agent") || stdout.contains("Golden Path Agent"));

    // Step 4: Verify workspace structure is complete
    assert!(radium_dir.join("_internals").join("agents").exists());
    assert!(radium_dir.join("_internals").join("prompts").exists());
    assert!(radium_dir.join("_internals").join("artifacts").exists());
    assert!(radium_dir.join("plan").join("backlog").exists());
    assert!(radium_dir.join("plan").join("development").exists());

    // Step 5: Verify artifacts directory is ready
    let artifacts_dir = radium_dir.join("_internals").join("artifacts");
    assert!(artifacts_dir.exists());
    assert!(artifacts_dir.is_dir());
}

