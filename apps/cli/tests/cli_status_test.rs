//! Comprehensive integration tests for the `rad status` command.

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
fn test_status_no_workspace() {
    // Run status outside of a workspace
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();

    cmd.current_dir(temp_dir.path())
        .arg("status")
        .assert()
        .success() // It exits with 0 even if no workspace is found
        .stdout(predicate::str::contains("workspace not found"));
}

#[test]
fn test_status_in_workspace() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Radium Status"))
        .stdout(predicate::str::contains("Valid: ✓"));
}

#[test]
fn test_status_json_output() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let assert = cmd.current_dir(temp_dir.path()).arg("status").arg("--json").assert().success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify it's valid JSON
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Status JSON output should be valid JSON");

    // Verify JSON structure
    assert!(json.is_object(), "Status JSON should be an object");
    // The exact structure depends on implementation, but should have workspace info
}

#[test]
fn test_status_shows_workspace_path() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains(temp_dir.path().to_str().unwrap()));
}

#[test]
fn test_status_shows_workspace_validity() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Valid: ✓").or(predicate::str::contains("Valid: true")));
}

#[test]
fn test_status_in_subdirectory_of_workspace() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Create a subdirectory
    let subdir = temp_dir.path().join("subdir");
    std::fs::create_dir_all(&subdir).unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(&subdir)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Radium Status"));
}
