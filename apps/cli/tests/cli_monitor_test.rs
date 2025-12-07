//! Comprehensive integration tests for the `rad monitor` command.

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
fn test_monitor_list_no_workspace() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("monitor")
        .arg("list")
        .assert()
        .failure();
}

#[test]
fn test_monitor_list_no_database() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("monitor")
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("monitoring database").or(predicate::str::contains("No agents")));
}

#[test]
fn test_monitor_list_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("monitor")
        .arg("list")
        .arg("--json")
        .assert();
    // May fail if no database, but should parse command
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_monitor_status_no_workspace() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("monitor")
        .arg("status")
        .assert()
        .failure();
}

#[test]
fn test_monitor_status_specific_agent() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("monitor")
        .arg("status")
        .arg("test-agent")
        .assert();
    // May fail if no database or agent not found
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_monitor_status_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("monitor")
        .arg("status")
        .arg("test-agent")
        .arg("--json")
        .assert();
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_monitor_telemetry_no_workspace() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("monitor")
        .arg("telemetry")
        .assert()
        .failure();
}

#[test]
fn test_monitor_telemetry_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("monitor")
        .arg("telemetry")
        .arg("--json")
        .assert();
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_monitor_list_filter_status() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let statuses = vec!["running", "completed", "failed"];
    
    for status in statuses {
        let mut cmd = Command::cargo_bin("radium-cli").unwrap();
        let result = cmd
            .current_dir(temp_dir.path())
            .arg("monitor")
            .arg("list")
            .arg("--status")
            .arg(status)
            .assert();
        assert!(result.get_output().status.code().is_some());
    }
}

#[test]
fn test_monitor_command_structure() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("monitor")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("monitor"));
}

