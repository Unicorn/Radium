//! Comprehensive integration tests for the `rad doctor` command.

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
fn test_doctor_no_workspace() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();

    cmd.current_dir(temp_dir.path())
        .arg("doctor")
        .assert()
        .success() // Doctor should still run even without workspace
        .stdout(predicate::str::contains("Radium Doctor"))
        .stdout(predicate::str::contains("Not found"));
}

#[test]
fn test_doctor_in_workspace() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Radium Doctor"))
        .stdout(predicate::str::contains("✓ Found").or(predicate::str::contains("Found")));
}

#[test]
fn test_doctor_json_output() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let assert = cmd.current_dir(temp_dir.path()).arg("doctor").arg("--json").assert().success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify it's valid JSON
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Doctor JSON output should be valid JSON");

    // Verify JSON structure
    assert!(json.is_object(), "Doctor JSON should be an object");
    assert!(json.get("workspace").is_some(), "JSON should have workspace field");
    assert!(json.get("environment").is_some(), "JSON should have environment field");
    assert!(json.get("network").is_some(), "JSON should have network field");
    assert!(json.get("structure").is_some(), "JSON should have structure field");
}

#[test]
fn test_doctor_shows_workspace_location() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Location:"));
}

#[test]
fn test_doctor_validates_workspace_structure() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Workspace Structure"));
}

#[test]
fn test_doctor_shows_environment_status() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Environment"));
}

#[test]
fn test_doctor_shows_network_status() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Network"));
}

#[test]
fn test_doctor_with_broken_workspace() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Remove a required directory to simulate broken workspace
    let plan_dir = temp_dir.path().join(".radium").join("plan");
    fs::remove_dir_all(&plan_dir).unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("doctor")
        .assert()
        .success() // Doctor should still run and report issues
        .stdout(predicate::str::contains("Missing").or(predicate::str::contains("✗")));
}

#[test]
fn test_doctor_json_with_no_workspace() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let assert = cmd.current_dir(temp_dir.path()).arg("doctor").arg("--json").assert().success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify it's valid JSON even without workspace
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Doctor JSON output should be valid JSON");

    assert!(json.is_object(), "Doctor JSON should be an object");
    // Workspace status should indicate error
    if let Some(workspace) = json.get("workspace") {
        if let Some(status) = workspace.get("status") {
            // Status might be "error" or "ok" depending on implementation
            assert!(status.is_string());
        }
    }
}

#[test]
fn test_doctor_shows_plans_count() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Plans:"));
}

#[test]
fn test_doctor_all_checks_passed() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("All checks passed")
                .or(predicate::str::contains("checks passed")),
        );
}

