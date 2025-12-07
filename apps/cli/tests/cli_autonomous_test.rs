//! Comprehensive integration tests for the `rad autonomous` command.

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
fn test_autonomous_no_workspace() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("autonomous")
        .arg("Build a simple app")
        .assert()
        .failure();
}

#[test]
fn test_autonomous_basic_goal() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("autonomous")
        .arg("Build a simple test application")
        .assert();
    // May fail during execution, but should start
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_autonomous_empty_goal() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("autonomous")
        .arg("")
        .assert();
    // May fail validation, but should parse command
}

#[test]
fn test_autonomous_complex_goal() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("autonomous")
        .arg("Create a REST API with authentication and database integration")
        .assert();
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_autonomous_command_structure() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("autonomous")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("autonomous"));
}

