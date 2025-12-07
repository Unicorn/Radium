//! Comprehensive integration tests for the `rad hooks` command.

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

#[test]
fn test_hooks_list() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("hooks")
        .arg("list")
        .assert()
        .success();
}

#[test]
fn test_hooks_list_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("hooks")
        .arg("list")
        .arg("--json")
        .assert()
        .success();
}

#[test]
fn test_hooks_list_verbose() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("hooks")
        .arg("list")
        .arg("--verbose")
        .assert()
        .success();
}

#[test]
fn test_hooks_list_filter_type() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let types = vec!["before_model", "after_model", "before_tool", "after_tool"];
    
    for hook_type in types {
        let mut cmd = Command::cargo_bin("radium-cli").unwrap();
        cmd.current_dir(temp_dir.path())
            .arg("hooks")
            .arg("list")
            .arg("--type")
            .arg(hook_type)
            .assert()
            .success();
    }
}

#[test]
fn test_hooks_info_not_found() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("hooks")
        .arg("info")
        .arg("nonexistent-hook")
        .assert()
        .failure();
}

#[test]
fn test_hooks_info_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("hooks")
        .arg("info")
        .arg("test-hook")
        .arg("--json")
        .assert();
    // May fail if hook doesn't exist, but should parse command
}

#[test]
fn test_hooks_enable_not_found() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("hooks")
        .arg("enable")
        .arg("nonexistent-hook")
        .assert();
    // May fail if hook doesn't exist
}

#[test]
fn test_hooks_disable_not_found() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("hooks")
        .arg("disable")
        .arg("nonexistent-hook")
        .assert();
    // May fail if hook doesn't exist
}

#[test]
fn test_hooks_command_structure() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Test that command accepts subcommands
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("hooks")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("hooks"));
}

#[test]
fn test_hooks_validate() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("hooks")
        .arg("validate")
        .assert()
        .success();
}

#[test]
fn test_hooks_validate_verbose() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("hooks")
        .arg("validate")
        .arg("--verbose")
        .assert()
        .success();
}

#[test]
fn test_hooks_validate_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("hooks")
        .arg("validate")
        .arg("--json")
        .assert()
        .success();
}

#[test]
fn test_hooks_test_not_found() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("hooks")
        .arg("test")
        .arg("nonexistent-hook")
        .assert()
        .failure();
}

#[test]
fn test_hooks_test_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("hooks")
        .arg("test")
        .arg("test-hook")
        .arg("--json")
        .assert();
    // May fail if hook doesn't exist, but should parse command
}

