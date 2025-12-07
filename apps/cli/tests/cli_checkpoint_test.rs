//! Comprehensive integration tests for the `rad checkpoint` command.

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
fn test_checkpoint_list_no_workspace() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("checkpoint")
        .arg("list")
        .assert()
        .failure();
}

#[test]
fn test_checkpoint_list_no_git() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("checkpoint")
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("git repository").or(predicate::str::contains("git")));
}

#[test]
fn test_checkpoint_list_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Initialize git repo
    std::process::Command::new("git")
        .arg("init")
        .current_dir(temp_dir.path())
        .output()
        .ok();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("checkpoint")
        .arg("list")
        .arg("--json")
        .assert()
        .success();
}

#[test]
fn test_checkpoint_restore_no_workspace() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("checkpoint")
        .arg("restore")
        .arg("test-checkpoint")
        .assert()
        .failure();
}

#[test]
fn test_checkpoint_restore_no_git() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("checkpoint")
        .arg("restore")
        .arg("test-checkpoint")
        .assert()
        .failure();
}

#[test]
fn test_checkpoint_restore_not_found() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Initialize git repo
    std::process::Command::new("git")
        .arg("init")
        .current_dir(temp_dir.path())
        .output()
        .ok();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("checkpoint")
        .arg("restore")
        .arg("nonexistent-checkpoint")
        .assert()
        .failure();
}

#[test]
fn test_checkpoint_command_structure() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("checkpoint")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("checkpoint"));
}

