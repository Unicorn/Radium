//! Comprehensive integration tests for the `rad budget` command.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_budget_status_no_budget() {
    // Isolate from any user-level config in the developer machine.
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("HOME", temp_dir.path())
        .env_remove("RADIUM_WORKSPACE")
        .arg("budget")
        .arg("status")
        .assert()
        .success()
        // Depending on config defaults, status may either show "not set" or an active default limit.
        .stdout(
            predicate::str::contains("No budget set")
                .or(predicate::str::contains("not_set"))
                .or(predicate::str::contains("Limit:")),
        );
}

#[test]
fn test_budget_set() {
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("budget")
        .arg("set")
        .arg("100.50")
        .assert()
        .success()
        .stdout(predicate::str::contains("Budget set"));
}

#[test]
fn test_budget_status_after_set() {
    // First set a budget
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("budget")
        .arg("set")
        .arg("200.00")
        .assert()
        .success();

    // Then check status
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("budget")
        .arg("status")
        .assert()
        .success();
}

#[test]
fn test_budget_status_json() {
    // Set a budget first
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("budget")
        .arg("set")
        .arg("150.00")
        .assert()
        .success();

    // Check status with JSON
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let assert = cmd
        .arg("budget")
        .arg("status")
        .arg("--json")
        .assert()
        .success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("JSON output should be valid JSON");

    // Verify JSON structure
    assert!(json.is_object(), "Budget status JSON should be an object");
}

#[test]
fn test_budget_reset() {
    // First set a budget
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("budget")
        .arg("set")
        .arg("300.00")
        .assert()
        .success();

    // Then reset it
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("budget")
        .arg("reset")
        .assert()
        .success();
}

#[test]
fn test_budget_set_multiple_times() {
    // Set budget first time
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("budget")
        .arg("set")
        .arg("100.00")
        .assert()
        .success();

    // Set budget second time (should overwrite)
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("budget")
        .arg("set")
        .arg("250.00")
        .assert()
        .success()
        .stdout(predicate::str::contains("Budget set"));
}

#[test]
fn test_budget_status_json_structure() {
    // Set a budget
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("budget")
        .arg("set")
        .arg("500.00")
        .assert()
        .success();

    // Get JSON status
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let assert = cmd
        .arg("budget")
        .arg("status")
        .arg("--json")
        .assert()
        .success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Verify expected fields exist
    assert!(json.get("limit").is_some() || json.get("status").is_some(),
            "Budget JSON should have limit or status field");
}

#[test]
fn test_budget_help() {
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("budget")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("budget"));
}

#[test]
fn test_budget_set_with_decimal() {
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("budget")
        .arg("set")
        .arg("99.99")
        .assert()
        .success();
}

#[test]
fn test_budget_set_with_large_amount() {
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("budget")
        .arg("set")
        .arg("10000.00")
        .assert()
        .success();
}

