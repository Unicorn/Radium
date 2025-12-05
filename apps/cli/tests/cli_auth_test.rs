//! Comprehensive integration tests for the `rad auth` command.

use assert_cmd::Command;
use predicates::prelude::*;
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
fn test_auth_status() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("auth")
        .arg("status")
        .assert()
        .success();
}

#[test]
fn test_auth_status_json() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let assert = cmd.current_dir(temp_dir.path())
        .arg("auth")
        .arg("status")
        .arg("--json")
        .assert()
        .success();

    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Verify it's valid JSON
    let _json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Auth status JSON output should be valid JSON");
}

#[test]
fn test_auth_login_invalid_provider() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("auth")
        .arg("login")
        .arg("invalid-provider")
        .assert()
        .failure(); // Should fail for invalid provider
}

#[test]
fn test_auth_logout_invalid_provider() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("auth")
        .arg("logout")
        .arg("invalid-provider")
        .assert()
        .failure(); // Should fail for invalid provider
}

// Note: Login/logout tests that require interactive input are harder to test
// They would need mocking or non-interactive flags. For now, we test the
// status command and error cases for invalid providers.

