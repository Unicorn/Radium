//! Comprehensive integration tests for the `rad custom` command.

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

/// Helper to create a test custom command
fn create_test_command(temp_dir: &TempDir, name: &str, description: &str, template: &str) {
    let commands_dir = temp_dir.path().join(".radium").join("commands");
    fs::create_dir_all(&commands_dir).unwrap();

    let toml_content = format!(
        r#"[command]
name = "{}"
description = "{}"
template = "{}"
"#,
        name, description, template
    );

    fs::write(commands_dir.join(format!("{}.toml", name)), toml_content).unwrap();
}

/// Helper to create a namespaced test command
fn create_namespaced_command(temp_dir: &TempDir, namespace: &str, name: &str, description: &str, template: &str) {
    let commands_dir = temp_dir.path().join(".radium").join("commands").join(namespace);
    fs::create_dir_all(&commands_dir).unwrap();

    let toml_content = format!(
        r#"[command]
name = "{}"
description = "{}"
template = "{}"
"#,
        name, description, template
    );

    fs::write(commands_dir.join(format!("{}.toml", name)), toml_content).unwrap();
}

#[test]
fn test_list_commands_empty() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("custom")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No custom commands found"));
}

#[test]
fn test_list_commands_with_commands() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_command(&temp_dir, "test-cmd", "Test command", "!{echo 'test'}");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("custom")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-cmd"))
        .stdout(predicate::str::contains("Test command"));
}

#[test]
fn test_list_commands_with_namespace() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_namespaced_command(&temp_dir, "git", "status", "Git status", "!{git status}");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("custom")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("git:status"));
}

#[test]
fn test_list_commands_verbose() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_command(&temp_dir, "test-cmd", "Test command", "!{echo 'test'}");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("custom")
        .arg("list")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-cmd"));
}

#[test]
fn test_execute_command_shell_injection() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_command(&temp_dir, "echo-test", "Echo test", "!{echo 'Hello World'}");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("custom")
        .arg("execute")
        .arg("echo-test")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello World"));
}

#[test]
fn test_execute_command_file_injection() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    
    // Create a test file
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "File content here").unwrap();
    
    create_test_command(&temp_dir, "file-test", "File test", "Content: @{test.txt}");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("custom")
        .arg("execute")
        .arg("file-test")
        .assert()
        .success()
        .stdout(predicate::str::contains("File content here"));
}

#[test]
fn test_execute_command_args_substitution() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_command(&temp_dir, "arg-test", "Arg test", "Hello {{arg1}}!");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("custom")
        .arg("execute")
        .arg("arg-test")
        .arg("World")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello World!"));
}

#[test]
fn test_execute_command_not_found() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("custom")
        .arg("execute")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_create_command_project() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("custom")
        .arg("create")
        .arg("new-cmd")
        .arg("--description")
        .arg("New command")
        .arg("--template")
        .arg("!{echo 'test'}")
        .assert()
        .success();

    // Verify file was created
    let cmd_file = temp_dir.path().join(".radium").join("commands").join("new-cmd.toml");
    assert!(cmd_file.exists(), "Command file should be created");
    
    let content = fs::read_to_string(&cmd_file).unwrap();
    assert!(content.contains("new-cmd"));
    assert!(content.contains("New command"));
}

#[test]
fn test_create_command_namespace() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("custom")
        .arg("create")
        .arg("status")
        .arg("--namespace")
        .arg("git")
        .arg("--description")
        .arg("Git status")
        .arg("--template")
        .arg("!{git status}")
        .assert()
        .success();

    // Verify file was created in namespace directory
    let cmd_file = temp_dir.path().join(".radium").join("commands").join("git").join("status.toml");
    assert!(cmd_file.exists(), "Command file should be created in namespace directory");
}

#[test]
fn test_validate_command_valid() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_command(&temp_dir, "valid-cmd", "Valid command", "!{echo 'test'}");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("custom")
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("Valid:"));
}

#[test]
fn test_validate_command_invalid_toml() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    
    // Create invalid TOML file
    let commands_dir = temp_dir.path().join(".radium").join("commands");
    fs::create_dir_all(&commands_dir).unwrap();
    fs::write(commands_dir.join("invalid.toml"), "invalid toml content {").unwrap();

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    // This will fail during discovery, so we expect an error
    cmd.current_dir(temp_dir.path())
        .arg("custom")
        .arg("validate")
        .assert()
        .failure();
}

#[test]
fn test_validate_command_missing_file() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_command(&temp_dir, "file-ref", "File reference", "Content: @{nonexistent.txt}");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("custom")
        .arg("validate")
        .assert()
        .failure()
        .stdout(predicate::str::contains("Invalid"));
}

#[test]
fn test_validate_command_specific() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_command(&temp_dir, "test-cmd", "Test", "!{echo 'test'}");
    create_test_command(&temp_dir, "other-cmd", "Other", "!{echo 'other'}");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("custom")
        .arg("validate")
        .arg("test-cmd")
        .assert()
        .success();
}

#[test]
fn test_execute_command_with_all_args() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_test_command(&temp_dir, "all-args", "All args", "Args: {{args}}");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("custom")
        .arg("execute")
        .arg("all-args")
        .arg("arg1")
        .arg("arg2")
        .arg("arg3")
        .assert()
        .success()
        .stdout(predicate::str::contains("arg1 arg2 arg3"));
}

#[test]
fn test_list_commands_namespace_filter() {
    let temp_dir = TempDir::new().unwrap();
    init_workspace(&temp_dir);
    create_namespaced_command(&temp_dir, "git", "status", "Git status", "!{git status}");
    create_namespaced_command(&temp_dir, "docker", "ps", "Docker ps", "!{docker ps}");

    let mut cmd = Command::cargo_bin("radium-cli").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("custom")
        .arg("list")
        .arg("--namespace")
        .arg("git")
        .assert()
        .success()
        .stdout(predicate::str::contains("git:status"))
        .stdout(predicate::str::contains("docker:ps").not());
}

