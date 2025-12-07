//! Comprehensive integration tests for context file integration across CLI commands.

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

/// Helper to create a test agent configuration
fn create_test_agent(temp_dir: &TempDir, agent_id: &str, name: &str) {
    let agents_dir = temp_dir.path().join("agents");
    fs::create_dir_all(&agents_dir).unwrap();

    // Create prompt file
    let prompts_dir = temp_dir.path().join("prompts");
    fs::create_dir_all(&prompts_dir).unwrap();
    fs::write(
        prompts_dir.join(format!("{}.md", agent_id)),
        format!("# {}\n\nYou are a test agent.\n\n## User Input\n\n{{user_input}}\n\n## Context Files\n\n{{context_files}}", name),
    )
    .unwrap();

    let config_content = format!(
        r#"[agent]
id = "{}"
name = "{}"
description = "A test agent for integration testing"
prompt_path = "prompts/{}.md"
engine = "mock"
model = "test-model"
reasoning_effort = "medium"
category = "test"
"#,
        agent_id, name, agent_id
    );

    fs::write(agents_dir.join(format!("{}.toml", agent_id)), config_content).unwrap();
}

/// Helper to create a context file
fn create_context_file(temp_dir: &TempDir, path: &str, content: &str) {
    let file_path = if path.starts_with('/') {
        temp_dir.path().join(path.strip_prefix('/').unwrap())
    } else {
        temp_dir.path().join(path)
    };
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&file_path, content).unwrap();
}

#[test]
fn test_step_command_with_context_file() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");
    create_context_file(&temp_dir, "GEMINI.md", "# Project Context\n\nThis is project context.");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("step")
        .arg("test-agent")
        .arg("test prompt")
        .assert()
        .success()
        .stdout(predicate::str::contains("Loaded context from"));
}

#[test]
fn test_step_command_with_hierarchical_context() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");
    create_context_file(&temp_dir, "GEMINI.md", "# Project Context\n\nProject level.");
    
    let subdir = temp_dir.path().join("src");
    fs::create_dir_all(&subdir).unwrap();
    create_context_file(&temp_dir, "src/GEMINI.md", "# Subdirectory Context\n\nSubdirectory level.");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(&subdir)
        .arg("step")
        .arg("test-agent")
        .arg("test prompt")
        .assert()
        .success()
        .stdout(predicate::str::contains("Loaded context from").and(predicate::str::contains("2")));
}

#[test]
fn test_step_command_without_context_files() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("step")
        .arg("test-agent")
        .arg("test prompt")
        .assert()
        .success(); // Should succeed even without context files
}

#[test]
fn test_run_command_with_context_file() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");
    create_context_file(&temp_dir, "GEMINI.md", "# Project Context\n\nThis is project context.");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("run")
        .arg("test-agent 'test prompt'")
        .assert()
        .success()
        .stdout(predicate::str::contains("Loaded context from"));
}

#[test]
fn test_run_command_with_dir_flag() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");
    
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir_all(&subdir).unwrap();
    create_context_file(&temp_dir, "subdir/GEMINI.md", "# Subdirectory Context\n\nSubdirectory level.");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("run")
        .arg("--dir")
        .arg("subdir")
        .arg("test-agent 'test prompt'")
        .assert()
        .success()
        .stdout(predicate::str::contains("Loaded context from"));
}

#[test]
fn test_run_command_without_context_files() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_agent(&temp_dir, "test-agent", "Test Agent");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("run")
        .arg("test-agent 'test prompt'")
        .assert()
        .success(); // Should succeed even without context files
}

#[test]
fn test_context_list_command() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_context_file(&temp_dir, "GEMINI.md", "# Project Context");
    
    let subdir = temp_dir.path().join("src");
    fs::create_dir_all(&subdir).unwrap();
    create_context_file(&temp_dir, "src/GEMINI.md", "# Subdirectory Context");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("context")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Found 2 context file(s)").and(predicate::str::contains("GEMINI.md")));
}

#[test]
fn test_context_list_command_no_files() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("context")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No context files found"));
}

#[test]
fn test_context_show_command() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_context_file(&temp_dir, "GEMINI.md", "# Project Context\n\nProject level content.");
    
    let subdir = temp_dir.path().join("src");
    fs::create_dir_all(&subdir).unwrap();
    create_context_file(&temp_dir, "src/GEMINI.md", "# Subdirectory Context\n\nSubdirectory content.");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("context")
        .arg("show")
        .arg("src")
        .assert()
        .success()
        .stdout(predicate::str::contains("Context files for:")
            .and(predicate::str::contains("Project"))
            .and(predicate::str::contains("Subdirectory")));
}

#[test]
fn test_context_show_command_invalid_path() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("context")
        .arg("show")
        .arg("nonexistent/path")
        .assert()
        .success()
        .stdout(predicate::str::contains("Path not found"));
}

#[test]
fn test_context_validate_command_valid() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_context_file(&temp_dir, "GEMINI.md", "# Project Context\n\nValid context file.");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("context")
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("All context files are valid"));
}

#[test]
fn test_context_validate_command_with_imports() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_context_file(&temp_dir, "rules.md", "# Rules\n\n- Rule 1\n- Rule 2");
    create_context_file(&temp_dir, "GEMINI.md", "# Project Context\n\n@rules.md");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("context")
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("All context files are valid"));
}

#[test]
fn test_context_validate_command_circular_import() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_context_file(&temp_dir, "file1.md", "# File 1\n\n@file2.md");
    create_context_file(&temp_dir, "file2.md", "# File 2\n\n@file1.md");
    create_context_file(&temp_dir, "GEMINI.md", "# Project Context\n\n@file1.md");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("context")
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("error").or(predicate::str::contains("Circular")));
}

#[test]
fn test_context_validate_command_missing_import() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_context_file(&temp_dir, "GEMINI.md", "# Project Context\n\n@nonexistent.md");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("context")
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("error").or(predicate::str::contains("not found")));
}

#[test]
fn test_context_validate_command_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_context_file(&temp_dir, "GEMINI.md", "");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("context")
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("warning").or(predicate::str::contains("empty")));
}

