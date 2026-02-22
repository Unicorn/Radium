//! CLI integration tests for radium-workflow binary.
//!
//! These tests exercise the compiled binary using `assert_cmd` to verify:
//! - Help output and version flag work
//! - Validate command accepts valid YAML and rejects invalid YAML
//! - Error output is structured JSON on stderr
//! - Config-dependent commands fail gracefully without a config file
//!
//! NOTE: Tests that hit the real API (create, list, deploy, etc.) live in
//! `crates/radium-workflow/tests/api_integration.rs` and require a running
//! Supabase instance. These CLI tests are self-contained and run offline.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Get a Command for the radium-workflow binary.
#[allow(deprecated)] // cargo_bin_cmd! macro is the new API but not yet stable
fn cli() -> Command {
    Command::cargo_bin("radium-workflow").expect("binary radium-workflow should exist")
}

// ---------------------------------------------------------------------------
// Help & Version
// ---------------------------------------------------------------------------

#[test]
fn help_flag_shows_usage() {
    cli()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Radium Workflow CLI"))
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn version_flag_shows_version() {
    cli()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("radium-workflow"));
}

#[test]
fn no_subcommand_shows_help() {
    cli()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage:"));
}

// ---------------------------------------------------------------------------
// Subcommand help
// ---------------------------------------------------------------------------

#[test]
fn validate_help() {
    cli()
        .args(["validate", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Validate"));
}

#[test]
fn create_help() {
    cli()
        .args(["create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Create"));
}

#[test]
fn login_help() {
    cli()
        .args(["login", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("credentials").or(predicate::str::contains("API")));
}

#[test]
fn components_help() {
    cli()
        .args(["components", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("components").or(predicate::str::contains("component")));
}

// ---------------------------------------------------------------------------
// Validate command — file handling
// ---------------------------------------------------------------------------

#[test]
fn validate_missing_file_fails() {
    cli()
        .args(["validate", "/tmp/nonexistent-file-abc123.yaml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn validate_empty_file_fails() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("empty.yaml");
    fs::write(&file, "").unwrap();

    cli()
        .args(["validate", file.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn validate_invalid_yaml_syntax_fails() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("bad-syntax.yaml");
    fs::write(
        &file,
        r#"
name: Bad Workflow
components:
  - id: start
    type: trigger
  - this is not valid yaml: [[[
"#,
    )
    .unwrap();

    cli()
        .args(["validate", file.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn validate_yaml_missing_required_fields_fails() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("missing-fields.yaml");
    // Missing 'name' and 'components' — should fail deserialization or API validation
    fs::write(&file, "description: just a description\n").unwrap();

    cli()
        .args(["validate", file.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

// ---------------------------------------------------------------------------
// Create command — file handling
// ---------------------------------------------------------------------------

#[test]
fn create_missing_file_fails() {
    cli()
        .args(["create", "/tmp/nonexistent-file-abc123.yaml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

// ---------------------------------------------------------------------------
// Commands that need a config profile — graceful failure without config
// ---------------------------------------------------------------------------

#[test]
fn list_without_config_fails_gracefully() {
    // Use a HOME that has no .radium/config.toml so the config lookup fails
    let dir = TempDir::new().unwrap();

    cli()
        .env("HOME", dir.path())
        .args(["list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn show_without_config_fails_gracefully() {
    let dir = TempDir::new().unwrap();

    cli()
        .env("HOME", dir.path())
        .args(["show", "550e8400-e29b-41d4-a716-446655440000"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn components_without_config_fails_gracefully() {
    let dir = TempDir::new().unwrap();

    cli()
        .env("HOME", dir.path())
        .args(["components"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn deploy_without_config_fails_gracefully() {
    let dir = TempDir::new().unwrap();

    cli()
        .env("HOME", dir.path())
        .args(["deploy", "550e8400-e29b-41d4-a716-446655440000"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

// ---------------------------------------------------------------------------
// Error output format — stderr should be JSON
// ---------------------------------------------------------------------------

#[test]
fn error_output_is_json() {
    let dir = TempDir::new().unwrap();

    let output = cli()
        .env("HOME", dir.path())
        .args(["list"])
        .output()
        .expect("failed to execute process");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    // The CLI wraps errors in JSON: {"error": "..."}
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(stderr.trim());
    assert!(
        parsed.is_ok(),
        "stderr should be valid JSON, got: {}",
        stderr
    );
    let json = parsed.unwrap();
    assert!(
        json.get("error").is_some(),
        "JSON error output should have an 'error' key, got: {}",
        json
    );
}

// ---------------------------------------------------------------------------
// Login command — writes config
// ---------------------------------------------------------------------------

#[test]
fn login_creates_config_file() {
    let dir = TempDir::new().unwrap();

    cli()
        .env("HOME", dir.path())
        .args([
            "login",
            "--url",
            "https://radium.example.com",
            "--key",
            "rk_test_key_123",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok"));

    let config_path = dir.path().join(".radium").join("config.toml");
    assert!(config_path.exists(), "config.toml should be created");

    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("radium.example.com"));
    assert!(content.contains("rk_test_key_123"));
}

#[test]
fn login_with_custom_profile() {
    let dir = TempDir::new().unwrap();

    cli()
        .env("HOME", dir.path())
        .args([
            "--profile",
            "staging",
            "login",
            "--url",
            "https://staging.radium.example.com",
            "--key",
            "rk_staging_key",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("staging"));

    let config_path = dir.path().join(".radium").join("config.toml");
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("staging"));
    assert!(content.contains("staging.radium.example.com"));
}

// ---------------------------------------------------------------------------
// Profile flag
// ---------------------------------------------------------------------------

#[test]
fn nonexistent_profile_fails() {
    let dir = TempDir::new().unwrap();

    // First create a config with "default" profile
    cli()
        .env("HOME", dir.path())
        .args([
            "login",
            "--url",
            "https://radium.example.com",
            "--key",
            "rk_test",
        ])
        .assert()
        .success();

    // Then try to use a profile that doesn't exist
    cli()
        .env("HOME", dir.path())
        .args(["--profile", "nonexistent", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

// ---------------------------------------------------------------------------
// Missing required arguments
// ---------------------------------------------------------------------------

#[test]
fn login_missing_url_fails() {
    cli()
        .args(["login", "--key", "rk_test"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--url"));
}

#[test]
fn login_missing_key_fails() {
    cli()
        .args(["login", "--url", "https://example.com"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--key"));
}

#[test]
fn validate_missing_file_arg_fails() {
    cli()
        .args(["validate"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("<FILE>").or(predicate::str::contains("required")));
}

#[test]
fn create_missing_file_arg_fails() {
    cli()
        .args(["create"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("<FILE>").or(predicate::str::contains("required")));
}

#[test]
fn show_missing_id_fails() {
    cli()
        .args(["show"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("<ID>").or(predicate::str::contains("required")));
}

#[test]
fn delete_missing_id_fails() {
    cli()
        .args(["delete"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("<ID>").or(predicate::str::contains("required")));
}
