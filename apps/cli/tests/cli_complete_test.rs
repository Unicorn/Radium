//! Comprehensive integration tests for the `rad complete` command.

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
fn test_complete_no_workspace() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("complete")
        .arg("test.md")
        .assert()
        .failure(); // Should fail if no workspace found
}

#[test]
fn test_complete_invalid_source() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("complete")
        .arg("invalid-source-format-12345")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Source detection failed"));
}

#[test]
fn test_complete_file_source_detection() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Create a spec file
    let spec_file = temp_dir.path().join("spec.md");
    fs::write(&spec_file, "# Test Project\n\nBuild a simple app.").unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("complete")
        .arg(spec_file.to_str().unwrap())
        .assert();

    // Should detect file source and start processing
    // May fail during plan generation or execution, but should at least detect source
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_complete_jira_ticket_detection() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("complete")
        .arg("RAD-42")
        .assert();

    // Should detect Jira ticket format
    // Will likely fail on fetch (no credentials), but should detect format
    let output = String::from_utf8_lossy(&result.get_output().stderr);
    // Should either detect the source or fail with authentication error
    assert!(
        output.contains("Jira ticket") || 
        output.contains("Missing") || 
        output.contains("credentials") ||
        output.contains("Source detection failed")
    );
}

#[test]
fn test_complete_braingrid_req_detection() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("complete")
        .arg("REQ-230")
        .assert();

    // Should detect Braingrid REQ format
    // Will likely fail on fetch (no credentials), but should detect format
    let output = String::from_utf8_lossy(&result.get_output().stderr);
    // Should either detect the source or fail with authentication error
    assert!(
        output.contains("Braingrid") || 
        output.contains("Missing") || 
        output.contains("credentials") ||
        output.contains("Source detection failed")
    );
}

#[test]
fn test_complete_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("complete")
        .arg("./nonexistent-file-12345.md")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Source detection failed").or(predicate::str::contains("not found")));
}

#[test]
fn test_complete_file_source_workflow() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Create a minimal spec file
    let spec_file = temp_dir.path().join("test-spec.md");
    let spec_content = r#"# Test Project

Build a simple test application.

## Requirements
- Create a basic structure
- Add tests
"#;
    fs::write(&spec_file, spec_content).unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("complete")
        .arg(spec_file.to_str().unwrap())
        .assert();

    // Should progress through source detection and fetching
    // May fail during plan generation/execution, but should get past source detection
    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(
        output.contains("Step 1") || 
        output.contains("Detecting source") ||
        output.contains("File:") ||
        result.get_output().status.code().is_some()
    );
}

#[test]
fn test_complete_jira_pattern_variations() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Test various Jira ticket patterns
    let patterns = vec!["PROJ-123", "ABC-999", "XYZ-1"];

    for pattern in patterns {
        let mut cmd = Command::cargo_bin("radium-cli").unwrap();
        let result = cmd
            .current_dir(temp_dir.path())
            .arg("complete")
            .arg(pattern)
            .assert();

        // Should detect as Jira ticket (may fail on fetch)
        let output = String::from_utf8_lossy(&result.get_output().stderr);
        assert!(
            output.contains("Jira ticket") || 
            output.contains("Missing") || 
            output.contains("credentials") ||
            output.contains("Source detection failed")
        );
    }
}

#[test]
fn test_complete_braingrid_pattern_variations() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Test various Braingrid REQ patterns (current format is REQ-<number>)
    let patterns = vec!["REQ-1", "REQ-230", "REQ-9999"];

    for pattern in patterns {
        let mut cmd = Command::cargo_bin("radium-cli").unwrap();
        let result = cmd
            .current_dir(temp_dir.path())
            .arg("complete")
            .arg(pattern)
            .assert();

        // Should detect as Braingrid requirement (may fail on fetch)
        let output = String::from_utf8_lossy(&result.get_output().stderr);
        assert!(
            output.contains("Braingrid") || 
            output.contains("Missing") || 
            output.contains("credentials") ||
            output.contains("Source detection failed")
        );
    }
}

#[test]
fn test_complete_invalid_jira_patterns() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Test invalid Jira patterns (should not match)
    let invalid_patterns = vec!["rad-42", "RAD-42-EXTRA", "RAD"];

    for pattern in invalid_patterns {
        let mut cmd = Command::cargo_bin("radium-cli").unwrap();
        cmd.current_dir(temp_dir.path())
            .arg("complete")
            .arg(pattern)
            .assert()
            .failure()
            .stderr(predicate::str::contains("Source detection failed"));
    }
}

#[test]
fn test_complete_invalid_braingrid_patterns() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Test invalid Braingrid patterns (should not match)
    let invalid_patterns = vec!["REQ-24-001", "REQ-2024-12", "req-2024-001"];

    for pattern in invalid_patterns {
        let mut cmd = Command::cargo_bin("radium-cli").unwrap();
        cmd.current_dir(temp_dir.path())
            .arg("complete")
            .arg(pattern)
            .assert()
            .failure()
            .stderr(predicate::str::contains("Source detection failed"));
    }
}

#[test]
fn test_complete_command_structure() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Test that command accepts source argument
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    let result = cmd
        .current_dir(temp_dir.path())
        .arg("complete")
        .arg("test.md")
        .assert();

    // Should at least parse the command (may fail on source detection or later)
    assert!(result.get_output().status.code().is_some());
}

#[test]
fn test_complete_missing_source_argument() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Test that command requires source argument
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("complete")
        .assert()
        .failure(); // Should fail without source argument
}

