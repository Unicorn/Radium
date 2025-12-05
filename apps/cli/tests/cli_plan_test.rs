//! Comprehensive integration tests for the `rad plan` command.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to initialize a workspace for testing
fn init_workspace(temp_dir: &TempDir) {
    let temp_path = temp_dir.path().to_str().unwrap();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("init")
        .arg("--use-defaults")
        .arg(temp_path)
        .assert()
        .success();
}

#[test]
fn test_plan_no_workspace() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("plan")
        .arg("Test specification")
        .assert()
        .failure(); // Should fail if no workspace found
}

#[test]
fn test_plan_with_direct_input() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("plan")
        .arg("Build a simple calculator app")
        .assert()
        .success()
        .stdout(predicate::str::contains("rad plan"));
}

#[test]
fn test_plan_with_file_input() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Create a spec file
    let spec_file = temp_dir.path().join("spec.md");
    fs::write(&spec_file, "# Calculator App\n\nBuild a simple calculator.").unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("plan")
        .arg(spec_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("rad plan"));
}

#[test]
fn test_plan_with_custom_id() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("plan")
        .arg("--id")
        .arg("REQ-042")
        .arg("Test specification")
        .assert()
        .success();
}

#[test]
fn test_plan_with_custom_name() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("plan")
        .arg("--name")
        .arg("my-project")
        .arg("Test specification")
        .assert()
        .success();
}

#[test]
fn test_plan_creates_plan_directory() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("plan")
        .arg("Test specification")
        .assert()
        .success();

    // Verify plan directory structure was created
    // Plans are created in workspace.root()/radium/backlog/ (note: "radium" not ".radium")
    // or workspace.root()/.radium/plan/backlog/ depending on implementation
    let possible_paths = [
        temp_dir.path().join("radium").join("backlog"),
        temp_dir.path().join(".radium").join("plan").join("backlog"),
        temp_dir.path().join(".radium").join("backlog"),
    ];
    
    // At least one of these should exist
    assert!(
        possible_paths.iter().any(|p| p.exists()) || 
        temp_dir.path().join(".radium").join("plan").exists() ||
        temp_dir.path().join("radium").exists()
    );
}

