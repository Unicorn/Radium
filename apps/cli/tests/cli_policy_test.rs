//! Comprehensive integration tests for the `rad policy` command.

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
fn test_policy_list() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("policy")
        .arg("list")
        .assert()
        .success();
}

#[test]
fn test_policy_list_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let assert = cmd
        .current_dir(temp_dir.path())
        .arg("policy")
        .arg("list")
        .arg("--json")
        .assert()
        .success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("JSON output should be valid JSON");
}

#[test]
fn test_policy_list_verbose() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("policy")
        .arg("list")
        .arg("--verbose")
        .assert()
        .success();
}

#[test]
fn test_policy_init() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("policy")
        .arg("init")
        .assert()
        .success();
}

#[test]
fn test_policy_init_force() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // First init
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("policy")
        .arg("init")
        .assert()
        .success();

    // Then init with force
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("policy")
        .arg("init")
        .arg("--force")
        .assert()
        .success();
}

#[test]
fn test_policy_validate() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // First create a policy file
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("policy")
        .arg("init")
        .assert()
        .success();

    // Then validate it
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("policy")
        .arg("validate")
        .assert()
        .success();
}

#[test]
fn test_policy_check() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("policy")
        .arg("check")
        .arg("test-tool")
        .arg("arg1")
        .arg("arg2")
        .assert()
        .success();
}

#[test]
fn test_policy_check_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let assert = cmd
        .current_dir(temp_dir.path())
        .arg("policy")
        .arg("check")
        .arg("--json")
        .arg("test-tool")
        .assert()
        .success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("JSON output should be valid JSON");
}

#[test]
fn test_policy_add() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("policy")
        .arg("add")
        .arg("test-rule")
        .arg("--priority")
        .arg("user")
        .arg("--action")
        .arg("allow")
        .arg("--tool-pattern")
        .arg("test-*")
        .arg("--reason")
        .arg("Test rule")
        .assert()
        .success();
}

#[test]
fn test_policy_remove() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // First add a rule
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("policy")
        .arg("add")
        .arg("test-rule")
        .arg("--priority")
        .arg("user")
        .arg("--action")
        .arg("allow")
        .arg("--tool-pattern")
        .arg("test-*")
        .assert()
        .success();

    // Then remove it
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("policy")
        .arg("remove")
        .arg("test-rule")
        .assert()
        .success();
}

#[test]
fn test_policy_conflicts() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("policy")
        .arg("conflicts")
        .assert()
        .success();
}

#[test]
fn test_policy_conflicts_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let assert = cmd
        .current_dir(temp_dir.path())
        .arg("policy")
        .arg("conflicts")
        .arg("--json")
        .assert()
        .success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("JSON output should be valid JSON");
}

#[test]
fn test_policy_help() {
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("policy")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("policy"));
}

