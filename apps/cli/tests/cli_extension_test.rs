//! Comprehensive integration tests for the `rad extension` command.

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
fn test_extension_list_no_extensions() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("extension")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No extensions installed"));
}

#[test]
fn test_extension_list_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("extension")
        .arg("list")
        .arg("--json")
        .assert()
        .success();
}

#[test]
fn test_extension_info_not_found() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("extension")
        .arg("info")
        .arg("nonexistent-extension")
        .assert()
        .failure();
}

#[test]
fn test_extension_search_no_query() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("extension")
        .arg("search")
        .arg("test")
        .assert()
        .success();
}

#[test]
fn test_extension_search_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("extension")
        .arg("search")
        .arg("test")
        .arg("--json")
        .assert()
        .success();
}

#[test]
fn test_extension_create_basic() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("extension")
        .arg("create")
        .arg("test-extension")
        .assert()
        .success();
}

#[test]
fn test_extension_create_with_author() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("extension")
        .arg("create")
        .arg("test-extension")
        .arg("--author")
        .arg("Test Author")
        .assert()
        .success();
}

#[test]
fn test_extension_create_with_description() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("extension")
        .arg("create")
        .arg("test-extension")
        .arg("--description")
        .arg("Test description")
        .assert()
        .success();
}

#[test]
fn test_extension_uninstall_not_found() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("extension")
        .arg("uninstall")
        .arg("nonexistent-extension")
        .assert()
        .failure();
}

#[test]
fn test_extension_list_verbose() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("extension")
        .arg("list")
        .arg("--verbose")
        .assert()
        .success();
}

