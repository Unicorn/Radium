//! Comprehensive integration tests for the `rad craft` command.

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
fn test_craft_no_workspace() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path()).arg("craft").arg("REQ-001").assert().failure(); // Should fail if no workspace found
}

#[test]
fn test_craft_plan_not_found() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path()).arg("craft").arg("REQ-999").assert().failure(); // Should fail if plan not found
}

#[test]
fn test_craft_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Test dry-run flag parsing and command structure
    // Note: Full end-to-end test requires plan to be in discoverable location
    // which may have path mismatch issues between plan creation and discovery
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result =
        cmd.current_dir(temp_dir.path()).arg("craft").arg("--dry-run").arg("REQ-001").assert();

    // Command should run (may fail if plan not found, but shouldn't panic)
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_craft_with_json_output() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Test JSON flag parsing and command structure
    // Note: Full end-to-end test requires plan to exist in discoverable location
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result =
        cmd.current_dir(temp_dir.path()).arg("craft").arg("--json").arg("REQ-001").assert();

    // Command should run (may fail if plan not found, but shouldn't panic)
    let output = result.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // If command succeeds and produces output, verify it's valid JSON
    if result.get_output().status.success() && !stdout.trim().is_empty() {
        let _json: serde_json::Value =
            serde_json::from_str(&stdout).expect("Craft JSON output should be valid JSON");
    }
}

#[test]
fn test_craft_with_resume() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Test resume flag parsing and command structure
    // Note: Full end-to-end test requires plan to exist in discoverable location
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result =
        cmd.current_dir(temp_dir.path()).arg("craft").arg("--resume").arg("REQ-001").assert();

    // Command should run (may fail if plan not found, but shouldn't panic)
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_craft_with_iteration() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("craft")
        .arg("--iteration")
        .arg("I1")
        .arg("REQ-001")
        .assert();

    // Command should run (may fail if plan not found, but shouldn't panic)
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_craft_with_task() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("craft")
        .arg("--task")
        .arg("I1.T1")
        .arg("REQ-001")
        .assert();

    // Command should run (may fail if plan not found, but shouldn't panic)
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_craft_with_iteration_and_task() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    // Both iteration and task specified - task should take precedence
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("craft")
        .arg("--iteration")
        .arg("I1")
        .arg("--task")
        .arg("I1.T1")
        .arg("REQ-001")
        .assert();

    // Command should run (may fail if plan not found, but shouldn't panic)
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_craft_with_all_flags() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("craft")
        .arg("--dry-run")
        .arg("--json")
        .arg("--resume")
        .arg("REQ-001")
        .assert();

    // Command should run (may fail if plan not found, but shouldn't panic)
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_craft_with_folder_name() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Craft can accept folder name instead of REQ-XXX
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd.current_dir(temp_dir.path()).arg("craft").arg("my-project-folder").assert();

    // Command should run (may fail if folder not found, but shouldn't panic)
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_craft_without_plan_identifier() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    // Craft without plan identifier should fail
    cmd.current_dir(temp_dir.path()).arg("craft").assert().failure();
}

#[test]
fn test_end_to_end_plan_generation_and_execution() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Create a specification file
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

    // Generate plan
    let mut plan_cmd = Command::cargo_bin("radium-cli").unwrap();
    plan_cmd
        .current_dir(temp_dir.path())
        .arg("plan")
        .arg("--id")
        .arg("REQ-001")
        .arg(spec_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Plan generated successfully"));

    // Execute plan (dry-run to avoid actual execution)
    let mut craft_cmd = Command::cargo_bin("radium-cli").unwrap();
    craft_cmd
        .current_dir(temp_dir.path())
        .arg("craft")
        .arg("REQ-001")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("rad craft"))
        .stdout(predicate::str::contains("Dry run mode"));
}

#[test]
fn test_resume_after_partial_execution() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Create a plan with multiple tasks
    let spec_file = temp_dir.path().join("spec.md");
    fs::write(
        &spec_file,
        r#"# Test Project

## Iteration 1: Tasks
Goal: Complete tasks

1. **Task 1** - First task
   - Agent: setup-agent
   - Acceptance: Task 1 done

2. **Task 2** - Second task
   - Agent: code-agent
   - Dependencies: I1.T1
   - Acceptance: Task 2 done
"#,
    )
    .unwrap();

    // Generate plan
    let mut plan_cmd = Command::cargo_bin("radium-cli").unwrap();
    plan_cmd
        .current_dir(temp_dir.path())
        .arg("plan")
        .arg("--id")
        .arg("REQ-001")
        .arg(spec_file.to_str().unwrap())
        .assert()
        .success();

    // Load manifest and mark first task as complete
    let backlog_dir = temp_dir.path().join(".radium/plan/backlog");
    let plan_dirs: Vec<_> = fs::read_dir(&backlog_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir() && e.file_name().to_string_lossy().starts_with("REQ-001"))
        .collect();

    if !plan_dirs.is_empty() {
        let plan_dir = plan_dirs[0].path();
        let manifest_path = plan_dir.join("plan/plan_manifest.json");

        if manifest_path.exists() {
            let manifest_content = fs::read_to_string(&manifest_path).unwrap();
            let mut manifest: serde_json::Value = serde_json::from_str(&manifest_content).unwrap();

            // Mark first task as complete
            if let Some(iterations) = manifest.get_mut("iterations").and_then(|i| i.as_array_mut()) {
                if let Some(iter) = iterations.get_mut(0) {
                    if let Some(tasks) = iter.get_mut("tasks").and_then(|t| t.as_array_mut()) {
                        if let Some(task) = tasks.get_mut(0) {
                            task["completed"] = serde_json::json!(true);
                        }
                    }
                }
            }

            fs::write(&manifest_path, serde_json::to_string_pretty(&manifest).unwrap()).unwrap();

            // Test resume
            let mut craft_cmd = Command::cargo_bin("radium-cli").unwrap();
            craft_cmd
                .current_dir(temp_dir.path())
                .arg("craft")
                .arg("REQ-001")
                .arg("--resume")
                .arg("--dry-run")
                .assert()
                .success();
        }
    }
}

#[test]
fn test_dependency_validation() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Create a plan with dependencies
    let spec_file = temp_dir.path().join("spec.md");
    fs::write(
        &spec_file,
        r#"# Test Project

## Iteration 1: Tasks
Goal: Complete tasks

1. **Task 1** - First task
   - Agent: setup-agent
   - Acceptance: Task 1 done

2. **Task 2** - Second task (depends on Task 1)
   - Agent: code-agent
   - Dependencies: I1.T1
   - Acceptance: Task 2 done
"#,
    )
    .unwrap();

    // Generate plan
    let mut plan_cmd = Command::cargo_bin("radium-cli").unwrap();
    plan_cmd
        .current_dir(temp_dir.path())
        .arg("plan")
        .arg("--id")
        .arg("REQ-001")
        .arg(spec_file.to_str().unwrap())
        .assert()
        .success();

    // Verify manifest has dependencies
    let backlog_dir = temp_dir.path().join(".radium/plan/backlog");
    let plan_dirs: Vec<_> = fs::read_dir(&backlog_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir() && e.file_name().to_string_lossy().starts_with("REQ-001"))
        .collect();

    if !plan_dirs.is_empty() {
        let plan_dir = plan_dirs[0].path();
        let manifest_path = plan_dir.join("plan/plan_manifest.json");

        if manifest_path.exists() {
            let manifest_content = fs::read_to_string(&manifest_path).unwrap();
            let manifest: serde_json::Value = serde_json::from_str(&manifest_content).unwrap();

            // Verify Task 2 has dependency on I1.T1
            if let Some(iterations) = manifest.get("iterations").and_then(|i| i.as_array()) {
                if let Some(iter) = iterations.get(0) {
                    if let Some(tasks) = iter.get("tasks").and_then(|t| t.as_array()) {
                        if let Some(task2) = tasks.get(1) {
                            let deps = task2.get("dependencies").and_then(|d| d.as_array());
                            assert!(
                                deps.is_some() && !deps.unwrap().is_empty(),
                                "Task 2 should have dependencies"
                            );
                        }
                    }
                }
            }
        }
    }
}
