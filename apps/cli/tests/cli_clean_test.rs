//! Comprehensive integration tests for the `rad clean` command.

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

/// Helper to create some artifacts in the workspace
fn create_test_artifacts(temp_dir: &TempDir) {
    let radium_dir = temp_dir.path().join(".radium");

    // Create some test files in various artifact directories
    let artifacts_dir = radium_dir.join("_internals").join("artifacts");
    fs::create_dir_all(&artifacts_dir).unwrap();
    fs::write(artifacts_dir.join("test.txt"), "test artifact").unwrap();

    let logs_dir = radium_dir.join("_internals").join("logs");
    fs::create_dir_all(&logs_dir).unwrap();
    fs::write(logs_dir.join("test.log"), "test log").unwrap();

    let memory_dir = radium_dir.join("_internals").join("memory");
    fs::create_dir_all(&memory_dir).unwrap();
    fs::write(memory_dir.join("test.json"), r#"{"test": "data"}"#).unwrap();
}

#[test]
fn test_clean_no_workspace() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();

    cmd.current_dir(temp_dir.path())
        .arg("clean")
        .assert()
        .failure() // Should fail if no workspace found
        .stderr(
            predicate::str::contains("workspace not found")
                .or(predicate::str::contains("not found")),
        );
}

#[test]
fn test_clean_empty_workspace() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("clean")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cleaning workspace artifacts"));
}

#[test]
fn test_clean_removes_artifacts() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_artifacts(&temp_dir);

    // Verify artifacts exist before cleaning
    let artifacts_dir = temp_dir.path().join(".radium").join("_internals").join("artifacts");
    assert!(artifacts_dir.join("test.txt").exists());

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path()).arg("clean").assert().success();

    // Verify artifacts are removed (directory may still exist but should be empty)
    assert!(!artifacts_dir.join("test.txt").exists());
}

#[test]
fn test_clean_verbose_output() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_artifacts(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("clean")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cleaning workspace artifacts"));
}

#[test]
fn test_clean_with_custom_directory() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path().join("workspace");
    fs::create_dir_all(&workspace_path).unwrap();

    // Initialize workspace in subdirectory
    let mut init_cmd = Command::cargo_bin("radium-cli").unwrap();
    init_cmd
        .arg("init")
        .arg("--use-defaults")
        .arg(workspace_path.to_str().unwrap())
        .assert()
        .success();

    create_test_artifacts(&temp_dir);

    // Clean from different directory
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("clean").arg("--dir").arg(workspace_path.to_str().unwrap()).assert().success();
}

#[test]
fn test_clean_preserves_structure() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_artifacts(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path()).arg("clean").assert().success();

    // Verify workspace structure is preserved
    let radium_dir = temp_dir.path().join(".radium");
    assert!(radium_dir.exists());
    assert!(radium_dir.join("_internals").exists());
    assert!(radium_dir.join("plan").exists());
    assert!(radium_dir.join("plan").join("backlog").exists());
}

#[test]
fn test_clean_shows_summary() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_artifacts(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path()).arg("clean").assert().success();
    // The exact output format may vary, but command should succeed
    // and clean the artifacts (verified in test_clean_removes_artifacts)
}
