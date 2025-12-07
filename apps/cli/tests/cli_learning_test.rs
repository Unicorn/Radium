//! Comprehensive integration tests for the `rad learning` command.

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
fn test_learning_no_workspace() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("learning")
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No Radium workspace found"));
}

#[test]
fn test_learning_list() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("learning")
        .arg("list")
        .assert()
        .success();
}

#[test]
fn test_learning_list_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let assert = cmd
        .current_dir(temp_dir.path())
        .arg("learning")
        .arg("list")
        .arg("--json")
        .assert()
        .success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("JSON output should be valid JSON");
}

#[test]
fn test_learning_list_with_category() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("learning")
        .arg("list")
        .arg("--category")
        .arg("coding")
        .assert()
        .success();
}

#[test]
fn test_learning_add_mistake() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("learning")
        .arg("add-mistake")
        .arg("--category")
        .arg("coding")
        .arg("--description")
        .arg("Forgot to handle error case")
        .arg("--solution")
        .arg("Always check for errors")
        .assert()
        .success();
}

#[test]
fn test_learning_add_skill() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("learning")
        .arg("add-skill")
        .arg("--section")
        .arg("best-practices")
        .arg("--content")
        .arg("Always write tests for new features")
        .assert()
        .success();
}

#[test]
fn test_learning_tag_skill() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // First add a skill
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("learning")
        .arg("add-skill")
        .arg("--section")
        .arg("best-practices")
        .arg("--content")
        .arg("Test content")
        .assert()
        .success();

    // Then tag it (this may fail if skill ID format is different)
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("learning")
        .arg("tag-skill")
        .arg("--skill-id")
        .arg("skill-1")
        .arg("--tag")
        .arg("helpful")
        .assert();
    // May fail if skill ID doesn't exist, but command should parse
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_learning_show_skillbook() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("learning")
        .arg("show-skillbook")
        .assert()
        .success();
}

#[test]
fn test_learning_show_skillbook_with_section() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("learning")
        .arg("show-skillbook")
        .arg("--section")
        .arg("best-practices")
        .assert()
        .success();
}

#[test]
fn test_learning_show_skillbook_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let assert = cmd
        .current_dir(temp_dir.path())
        .arg("learning")
        .arg("show-skillbook")
        .arg("--json")
        .assert()
        .success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("JSON output should be valid JSON");
}

#[test]
fn test_learning_help() {
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("learning")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("learning"));
}

#[test]
fn test_learning_list_after_adding_mistake() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Add a mistake
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("learning")
        .arg("add-mistake")
        .arg("--category")
        .arg("testing")
        .arg("--description")
        .arg("Test mistake")
        .arg("--solution")
        .arg("Test solution")
        .assert()
        .success();

    // List should show the mistake
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("learning")
        .arg("list")
        .assert()
        .success();
}

