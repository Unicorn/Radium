use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("--version").assert().success().stdout(predicate::str::contains("rad 0.1.0"));
}

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Radium (rad) is a high-performance"));
}

#[test]
fn test_init_command() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();

    // Run init in the temp directory
    cmd.arg("init")
        .arg("--use-defaults")
        .arg(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Workspace initialized successfully!"));

    // Verify directory structure
    let radium_dir = temp_dir.path().join(".radium");
    assert!(radium_dir.exists());

    let internals_dir = radium_dir.join("_internals");
    assert!(internals_dir.exists());
    assert!(internals_dir.join("agents").exists());
    assert!(internals_dir.join("prompts").exists());

    let plan_dir = radium_dir.join("plan");
    assert!(plan_dir.exists());
    assert!(plan_dir.join("backlog").exists());
    assert!(plan_dir.join("development").exists());
}

#[test]
fn test_status_command_no_workspace() {
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
fn test_status_command_in_workspace() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Initialize first
    let mut init_cmd = Command::cargo_bin("radium-cli").unwrap();
    init_cmd.arg("init").arg("--use-defaults").arg(temp_path.to_str().unwrap()).assert().success();

    // Then run status
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_path)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Radium Status"))
        .stdout(predicate::str::contains("Valid: ✓"));
}
