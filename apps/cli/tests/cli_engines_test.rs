//! Comprehensive integration tests for the `rad engines` command.

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
fn test_engines_list() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("engines")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("mock").or(predicate::str::contains("Available Engines")));
}

#[test]
fn test_engines_list_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("engines")
        .arg("list")
        .arg("--json")
        .assert()
        .success();
}

#[test]
fn test_engines_show() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("engines")
        .arg("show")
        .arg("mock")
        .assert()
        .success();
}

#[test]
fn test_engines_show_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("engines")
        .arg("show")
        .arg("mock")
        .arg("--json")
        .assert()
        .success();
}

#[test]
fn test_engines_show_invalid() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("engines")
        .arg("show")
        .arg("nonexistent-engine")
        .assert()
        .failure();
}

#[test]
fn test_engines_status() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("engines")
        .arg("status")
        .assert()
        .success();
}

#[test]
fn test_engines_status_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("engines")
        .arg("status")
        .arg("--json")
        .assert()
        .success();
}

#[test]
fn test_engines_set_default() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("engines")
        .arg("set-default")
        .arg("mock")
        .assert()
        .success();
}

#[test]
fn test_engines_set_default_invalid() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("engines")
        .arg("set-default")
        .arg("nonexistent-engine")
        .assert()
        .failure();
}

#[test]
fn test_engines_command_structure() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("engines")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("engines"));
}

