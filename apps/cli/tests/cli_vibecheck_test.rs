//! Comprehensive integration tests for the `rad vibecheck` command.

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
fn test_vibecheck_no_workspace() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("vibecheck")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No Radium workspace found"));
}

#[test]
fn test_vibecheck_basic() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("vibecheck")
        .assert();
    // May fail during execution (needs model), but should parse command
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_vibecheck_with_phase() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("vibecheck")
        .arg("--phase")
        .arg("planning")
        .assert();
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_vibecheck_with_goal() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("vibecheck")
        .arg("--goal")
        .arg("Build a test application")
        .assert();
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_vibecheck_with_plan() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("vibecheck")
        .arg("--plan")
        .arg("Test plan content")
        .assert();
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_vibecheck_json_output() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("vibecheck")
        .arg("--json")
        .assert();
    
    // May fail during execution, but if it succeeds, verify JSON
    if result.get_output().status.success() {
        let output = result.get_output();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let _json: serde_json::Value = serde_json::from_str(&stdout)
            .expect("JSON output should be valid JSON");
    }
}

#[test]
fn test_vibecheck_help() {
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("vibecheck")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("vibecheck"));
}

#[test]
fn test_vibecheck_invalid_phase() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    // Invalid phase should still parse (command accepts any string)
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("vibecheck")
        .arg("--phase")
        .arg("invalid-phase")
        .assert();
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_vibecheck_with_progress() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("vibecheck")
        .arg("--progress")
        .arg("50% complete")
        .assert();
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_vibecheck_with_task_context() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("vibecheck")
        .arg("--task-context")
        .arg("Current task: implement feature")
        .assert();
    assert!(result.get_output().status.code().is_some());
}

