//! Comprehensive integration tests for the `rad sandbox` command.

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
fn test_sandbox_list() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("sandbox")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("none").or(predicate::str::contains("docker")));
}

#[test]
fn test_sandbox_list_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("sandbox")
        .arg("list")
        .arg("--json")
        .assert()
        .success();
}

#[test]
fn test_sandbox_test() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("sandbox")
        .arg("test")
        .assert()
        .success();
}

#[test]
fn test_sandbox_test_specific_type() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("sandbox")
        .arg("test")
        .arg("none")
        .assert()
        .success();
}

#[test]
fn test_sandbox_test_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("sandbox")
        .arg("test")
        .arg("--json")
        .assert()
        .success();
}

#[test]
fn test_sandbox_config() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("sandbox")
        .arg("config")
        .assert()
        .success();
}

#[test]
fn test_sandbox_config_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("sandbox")
        .arg("config")
        .arg("--json")
        .assert()
        .success();
}

#[test]
fn test_sandbox_doctor() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("sandbox")
        .arg("doctor")
        .assert()
        .success();
}

#[test]
fn test_sandbox_doctor_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("sandbox")
        .arg("doctor")
        .arg("--json")
        .assert()
        .success();
}

#[test]
fn test_sandbox_command_structure() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("sandbox")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("sandbox"));
}

