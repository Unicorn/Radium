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

#[test]
fn test_context_init_command_basic() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("context")
        .arg("init")
        .arg("--template")
        .arg("basic")
        .write_stdin("n\n") // Don't overwrite if exists (shouldn't exist)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created context file")
            .and(predicate::str::contains("basic")));

    // Verify file was created
    let context_file = temp_dir.path().join("GEMINI.md");
    assert!(context_file.exists());
    let content = fs::read_to_string(&context_file).unwrap();
    assert!(content.contains("Project Context"));
    assert!(content.contains("Guidelines"));
}

#[test]
fn test_context_init_command_coding_standards() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("context")
        .arg("init")
        .arg("--template")
        .arg("coding-standards")
        .write_stdin("n\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created context file")
            .and(predicate::str::contains("coding-standards")));

    // Verify file was created with correct template
    let context_file = temp_dir.path().join("GEMINI.md");
    assert!(context_file.exists());
    let content = fs::read_to_string(&context_file).unwrap();
    assert!(content.contains("Coding Standards"));
    assert!(content.contains("Code Formatting"));
}

#[test]
fn test_context_init_command_architecture() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("context")
        .arg("init")
        .arg("--template")
        .arg("architecture")
        .write_stdin("n\n")
        .assert()
        .success();

    // Verify file was created with correct template
    let context_file = temp_dir.path().join("GEMINI.md");
    assert!(context_file.exists());
    let content = fs::read_to_string(&context_file).unwrap();
    assert!(content.contains("Architecture"));
    assert!(content.contains("Components"));
}

#[test]
fn test_context_init_command_team_conventions() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("context")
        .arg("init")
        .arg("--template")
        .arg("team-conventions")
        .write_stdin("n\n")
        .assert()
        .success();

    // Verify file was created with correct template
    let context_file = temp_dir.path().join("GEMINI.md");
    assert!(context_file.exists());
    let content = fs::read_to_string(&context_file).unwrap();
    assert!(content.contains("Team Conventions"));
    assert!(content.contains("Communication"));
}

#[test]
fn test_context_init_command_custom_path() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let custom_path = temp_dir.path().join("custom").join("CONTEXT.md");
    fs::create_dir_all(custom_path.parent().unwrap()).unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("context")
        .arg("init")
        .arg("--path")
        .arg("custom/CONTEXT.md")
        .write_stdin("n\n")
        .assert()
        .success();

    // Verify file was created at custom path
    assert!(custom_path.exists());
    let content = fs::read_to_string(&custom_path).unwrap();
    assert!(content.contains("Project Context"));
}

#[test]
fn test_context_init_command_invalid_template() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("context")
        .arg("init")
        .arg("--template")
        .arg("invalid-template")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid template type"));
}

#[test]
fn test_context_init_command_default_template() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    // Test that default template (basic) is used when not specified
    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("context")
        .arg("init")
        .write_stdin("n\n")
        .assert()
        .success();

    // Verify file was created with basic template (default)
    let context_file = temp_dir.path().join("GEMINI.md");
    assert!(context_file.exists());
    let content = fs::read_to_string(&context_file).unwrap();
    assert!(content.contains("Project Context"));
}

