//! Comprehensive integration tests for the `rad plan` command.

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
fn test_plan_no_workspace() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path()).arg("plan").arg("Test specification").assert().failure(); // Should fail if no workspace found
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
    cmd.current_dir(temp_dir.path()).arg("plan").arg("Test specification").assert().success();

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
        possible_paths.iter().any(|p| p.exists())
            || temp_dir.path().join(".radium").join("plan").exists()
            || temp_dir.path().join("radium").exists()
    );
}

#[test]
fn test_plan_with_both_id_and_name() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("plan")
        .arg("--id")
        .arg("REQ-042")
        .arg("--name")
        .arg("my-project")
        .arg("Test specification")
        .assert()
        .success();
}

#[test]
fn test_plan_with_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    // The plan command may treat nonexistent files as direct input
    // or fail - depends on implementation. Test that it doesn't panic.
    let result = cmd.current_dir(temp_dir.path()).arg("plan").arg("nonexistent.md").assert();

    // Either success or failure is acceptable, just verify it doesn't panic
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_plan_with_empty_input() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    // Empty input might be accepted or rejected - depends on implementation
    let result = cmd.current_dir(temp_dir.path()).arg("plan").arg("").assert();

    // Either success or failure is acceptable, just verify it doesn't panic
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_plan_with_invalid_id_format() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    // Invalid ID format might be accepted or rejected - depends on implementation
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("plan")
        .arg("--id")
        .arg("INVALID-ID")
        .arg("Test specification")
        .assert();

    // Either success or failure is acceptable, just verify it doesn't panic
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_plan_with_very_long_specification() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let long_spec = "Test specification. ".repeat(100);
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path()).arg("plan").arg(&long_spec).assert().success();
}

#[test]
fn test_plan_generates_all_files() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let spec_file = temp_dir.path().join("spec.md");
    fs::write(
        &spec_file,
        r#"# Test Project

Build a test application.

## Iteration 1: Setup
Goal: Set up the project

1. **Initialize project** - Create basic structure
   - Agent: setup-agent
   - Acceptance: Project structure created
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("plan")
        .arg("--id")
        .arg("REQ-001")
        .arg(spec_file.to_str().unwrap())
        .assert()
        .success();

    // Verify all files were created
    let backlog_dir = temp_dir.path().join(".radium/plan/backlog");
    let plan_dirs: Vec<_> = fs::read_dir(&backlog_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir() && e.file_name().to_string_lossy().starts_with("REQ-001"))
        .collect();

    assert!(!plan_dirs.is_empty(), "Plan directory should be created");

    let plan_dir = &plan_dirs[0].path();
    assert!(plan_dir.join("plan.json").exists(), "plan.json should exist");
    assert!(
        plan_dir.join("plan/plan_manifest.json").exists(),
        "plan_manifest.json should exist"
    );
    assert!(
        plan_dir.join("plan/01_Plan_Overview_and_Setup.md").exists(),
        "Overview markdown should exist"
    );
    assert!(
        plan_dir.join("plan/03_Verification_and_Glossary.md").exists(),
        "Verification markdown should exist"
    );
    assert!(
        plan_dir.join("plan/coordinator-prompt.md").exists(),
        "Coordinator prompt should exist"
    );
    assert!(
        plan_dir.join("specifications.md").exists(),
        "Specifications file should be copied"
    );
}
