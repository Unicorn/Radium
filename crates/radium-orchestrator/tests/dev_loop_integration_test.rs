//! Integration tests for common development loop scenarios.
//!
//! These tests validate that git tools, terminal commands, and approval flows
//! work correctly together in typical development workflows.

use radium_orchestrator::orchestration::{
    git_extended_tools,
    terminal_tool::{self, WorkspaceRootProvider},
    tool::{Tool, ToolArguments},
};
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;
use tokio::process::Command;

/// Test workspace root provider
struct TestWorkspaceRoot {
    root: PathBuf,
}

impl WorkspaceRootProvider for TestWorkspaceRoot {
    fn workspace_root(&self) -> Option<PathBuf> {
        Some(self.root.clone())
    }
}

/// Setup a test git repository with some commits and changes
async fn setup_test_git_repo(temp_dir: &TempDir) -> Result<(), Box<dyn std::error::Error>> {
    let repo_path = temp_dir.path();

    // Initialize git repo
    Command::new("git")
        .arg("init")
        .current_dir(repo_path)
        .output()
        .await?;

    // Configure git user (required for commits)
    Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()
        .await?;

    Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()
        .await?;

    // Create initial file and commit
    fs::write(repo_path.join("README.md"), "# Test Project\n\nInitial commit.").await?;
    Command::new("git")
        .args(&["add", "README.md"])
        .current_dir(repo_path)
        .output()
        .await?;

    Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()
        .await?;

    // Create a source file
    fs::write(
        repo_path.join("src/main.rs"),
        "fn main() {\n    println!(\"Hello, world!\");\n}\n",
    )
    .await?;

    Command::new("git")
        .args(&["add", "src/main.rs"])
        .current_dir(repo_path)
        .output()
        .await?;

    Command::new("git")
        .args(&["commit", "-m", "Add main.rs"])
        .current_dir(repo_path)
        .output()
        .await?;

    // Make some uncommitted changes
    fs::write(
        repo_path.join("src/main.rs"),
        "fn main() {\n    println!(\"Hello, Radium!\");\n    println!(\"Updated\");\n}\n",
    )
    .await?;

    fs::write(repo_path.join("new_file.txt"), "New untracked file").await?;

    Ok(())
}

#[tokio::test]
async fn test_git_status_in_clean_repo() {
    let temp_dir = TempDir::new().unwrap();
    setup_test_git_repo(&temp_dir).await.unwrap();

    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tool = git_extended_tools::create_git_status_tool(workspace_root);
    let args = ToolArguments::new(serde_json::json!({
        "format": "short"
    }));

    let result = tool.execute(&args).await.unwrap();
    assert!(result.success);
    assert!(result.output.contains("Branch:"));
    // Should show modified and untracked files
    assert!(
        result.output.contains("Modified") || result.output.contains("Untracked"),
        "Output: {}",
        result.output
    );
}

#[tokio::test]
async fn test_git_status_detailed_format() {
    let temp_dir = TempDir::new().unwrap();
    setup_test_git_repo(&temp_dir).await.unwrap();

    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tool = git_extended_tools::create_git_status_tool(workspace_root);
    let args = ToolArguments::new(serde_json::json!({
        "format": "detailed"
    }));

    let result = tool.execute(&args).await.unwrap();
    assert!(result.success);
    assert!(result.output.contains("Branch:"));
    assert!(result.output.contains("Modified:") || result.output.contains("Untracked:"));
}

#[tokio::test]
async fn test_git_status_not_a_repo() {
    let temp_dir = TempDir::new().unwrap();
    // Don't initialize git

    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tool = git_extended_tools::create_git_status_tool(workspace_root);
    let args = ToolArguments::new(serde_json::json!({}));

    let result = tool.execute(&args).await.unwrap();
    // Should return error for non-git repo
    assert!(
        !result.success || result.output.contains("Not a git repository"),
        "Expected error for non-git repo, got: {}",
        result.output
    );
}

#[tokio::test]
async fn test_git_diff_working_directory() {
    let temp_dir = TempDir::new().unwrap();
    setup_test_git_repo(&temp_dir).await.unwrap();

    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tool = git_extended_tools::create_git_diff_tool(workspace_root);
    let args = ToolArguments::new(serde_json::json!({}));

    let result = tool.execute(&args).await.unwrap();
    assert!(result.success);
    // Should show diff of modified files
    assert!(
        result.output.contains("diff") || result.output.contains("No differences found"),
        "Output: {}",
        result.output
    );
}

#[tokio::test]
async fn test_git_diff_specific_file() {
    let temp_dir = TempDir::new().unwrap();
    setup_test_git_repo(&temp_dir).await.unwrap();

    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tool = git_extended_tools::create_git_diff_tool(workspace_root);
    let args = ToolArguments::new(serde_json::json!({
        "file_path": "src/main.rs"
    }));

    let result = tool.execute(&args).await.unwrap();
    assert!(result.success);
}

#[tokio::test]
async fn test_git_log_default() {
    let temp_dir = TempDir::new().unwrap();
    setup_test_git_repo(&temp_dir).await.unwrap();

    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tool = git_extended_tools::create_git_log_tool(workspace_root);
    let args = ToolArguments::new(serde_json::json!({}));

    let result = tool.execute(&args).await.unwrap();
    assert!(result.success);
    // Should show commit history
    assert!(
        result.output.contains("Initial commit") || result.output.contains("Add main.rs"),
        "Output: {}",
        result.output
    );
}

#[tokio::test]
async fn test_git_log_with_limit() {
    let temp_dir = TempDir::new().unwrap();
    setup_test_git_repo(&temp_dir).await.unwrap();

    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tool = git_extended_tools::create_git_log_tool(workspace_root);
    let args = ToolArguments::new(serde_json::json!({
        "max_entries": 1
    }));

    let result = tool.execute(&args).await.unwrap();
    assert!(result.success);
}

#[tokio::test]
async fn test_terminal_cmd_basic_execution() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tool = terminal_tool::create_terminal_command_tool(workspace_root, None, Some(30));
    let args = ToolArguments::new(serde_json::json!({
        "command": "echo 'Hello, World!'"
    }));

    let result = tool.execute(&args).await.unwrap();
    assert!(result.success);
    assert!(result.output.contains("Hello, World!"));
    assert!(result.metadata.contains_key("exit_code"));
    assert_eq!(result.metadata.get("exit_code"), Some(&"0".to_string()));
}

#[tokio::test]
async fn test_terminal_cmd_with_working_dir() {
    let temp_dir = TempDir::new().unwrap();
    fs::create_dir_all(temp_dir.path().join("subdir")).await.unwrap();

    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tool = terminal_tool::create_terminal_command_tool(workspace_root, None, Some(30));
    let args = ToolArguments::new(serde_json::json!({
        "command": "pwd",
        "working_dir": "subdir"
    }));

    let result = tool.execute(&args).await.unwrap();
    assert!(result.success);
    // Should show the subdirectory path
    assert!(
        result.output.contains("subdir") || result.metadata.get("working_dir").map(|s| s.contains("subdir")).unwrap_or(false),
        "Output: {}",
        result.output
    );
}

#[tokio::test]
async fn test_terminal_cmd_with_env_vars() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tool = terminal_tool::create_terminal_command_tool(workspace_root, None, Some(30));
    let args = ToolArguments::new(serde_json::json!({
        "command": "echo $TEST_VAR",
        "env": {
            "TEST_VAR": "test_value"
        }
    }));

    let result = tool.execute(&args).await.unwrap();
    assert!(result.success);
    // Should show the environment variable value
    assert!(
        result.output.contains("test_value"),
        "Output: {}",
        result.output
    );
}

#[tokio::test]
async fn test_terminal_cmd_timeout() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tool = terminal_tool::create_terminal_command_tool(workspace_root, None, Some(30));
    let args = ToolArguments::new(serde_json::json!({
        "command": "sleep 5",
        "timeout_seconds": 1
    }));

    let result = tool.execute(&args).await.unwrap();
    // Should timeout
    assert!(
        !result.success || result.output.contains("timed out"),
        "Expected timeout, got: {}",
        result.output
    );
}

#[tokio::test]
async fn test_terminal_cmd_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tool = terminal_tool::create_terminal_command_tool(workspace_root, None, Some(30));
    let args = ToolArguments::new(serde_json::json!({
        "command": "false" // Command that exits with non-zero code
    }));

    let result = tool.execute(&args).await.unwrap();
    // Should indicate failure
    assert!(
        !result.success || result.metadata.get("exit_code").map(|s| s != "0").unwrap_or(false),
        "Expected failure, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_dev_loop_build_scenario() {
    // Simulate: git status -> build command -> git diff
    let temp_dir = TempDir::new().unwrap();
    setup_test_git_repo(&temp_dir).await.unwrap();

    // Create a simple Rust project structure
    fs::create_dir_all(temp_dir.path().join("src")).await.unwrap();
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"[package]
name = "test-project"
version = "0.1.0"
"#,
    )
    .await
    .unwrap();

    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    // 1. Check git status
    let status_tool = git_extended_tools::create_git_status_tool(Arc::clone(&workspace_root));
    let status_args = ToolArguments::new(serde_json::json!({}));
    let status_result = status_tool.execute(&status_args).await.unwrap();
    assert!(status_result.success);

    // 2. Run a build-like command (just echo for test)
    let cmd_tool = terminal_tool::create_terminal_command_tool(Arc::clone(&workspace_root), None, Some(30));
    let cmd_args = ToolArguments::new(serde_json::json!({
        "command": "echo 'Building...'"
    }));
    let cmd_result = cmd_tool.execute(&cmd_args).await.unwrap();
    assert!(cmd_result.success);

    // 3. Check git diff
    let diff_tool = git_extended_tools::create_git_diff_tool(workspace_root);
    let diff_args = ToolArguments::new(serde_json::json!({}));
    let diff_result = diff_tool.execute(&diff_args).await.unwrap();
    assert!(diff_result.success);
}

#[tokio::test]
async fn test_dev_loop_git_workflow() {
    // Simulate: git log -> git status -> git diff
    let temp_dir = TempDir::new().unwrap();
    setup_test_git_repo(&temp_dir).await.unwrap();

    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    // 1. Check recent commits
    let log_tool = git_extended_tools::create_git_log_tool(Arc::clone(&workspace_root));
    let log_args = ToolArguments::new(serde_json::json!({
        "max_entries": 5
    }));
    let log_result = log_tool.execute(&log_args).await.unwrap();
    assert!(log_result.success);
    assert!(log_result.output.contains("commit") || log_result.output.contains("Initial"));

    // 2. Check current status
    let status_tool = git_extended_tools::create_git_status_tool(Arc::clone(&workspace_root));
    let status_args = ToolArguments::new(serde_json::json!({}));
    let status_result = status_tool.execute(&status_args).await.unwrap();
    assert!(status_result.success);

    // 3. Check what changed
    let diff_tool = git_extended_tools::create_git_diff_tool(workspace_root);
    let diff_args = ToolArguments::new(serde_json::json!({}));
    let diff_result = diff_tool.execute(&diff_args).await.unwrap();
    assert!(diff_result.success);
}

#[tokio::test]
async fn test_terminal_cmd_with_shell_and_env() {
    // Test combined features: shell execution with environment variables
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tool = terminal_tool::create_terminal_command_tool(workspace_root, None, Some(30));
    let args = ToolArguments::new(serde_json::json!({
        "command": "echo $VAR1 and $VAR2",
        "use_shell": true,
        "env": {
            "VAR1": "value1",
            "VAR2": "value2"
        }
    }));

    let result = tool.execute(&args).await.unwrap();
    assert!(result.success);
    assert!(result.output.contains("value1"));
    assert!(result.output.contains("value2"));
}

#[tokio::test]
async fn test_git_status_hide_untracked() {
    let temp_dir = TempDir::new().unwrap();
    setup_test_git_repo(&temp_dir).await.unwrap();

    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tool = git_extended_tools::create_git_status_tool(workspace_root);
    let args = ToolArguments::new(serde_json::json!({
        "show_untracked": false
    }));

    let result = tool.execute(&args).await.unwrap();
    assert!(result.success);
    // Should still show status, but untracked files may be hidden
}

#[tokio::test]
async fn test_git_diff_staged_only() {
    let temp_dir = TempDir::new().unwrap();
    setup_test_git_repo(&temp_dir).await.unwrap();

    // Stage a file
    Command::new("git")
        .args(&["add", "src/main.rs"])
        .current_dir(temp_dir.path())
        .output()
        .await
        .unwrap();

    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tool = git_extended_tools::create_git_diff_tool(workspace_root);
    let args = ToolArguments::new(serde_json::json!({
        "staged_only": true
    }));

    let result = tool.execute(&args).await.unwrap();
    assert!(result.success);
}

#[tokio::test]
async fn test_git_log_file_filter() {
    let temp_dir = TempDir::new().unwrap();
    setup_test_git_repo(&temp_dir).await.unwrap();

    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tool = git_extended_tools::create_git_log_tool(workspace_root);
    let args = ToolArguments::new(serde_json::json!({
        "file_path": "src/main.rs",
        "max_entries": 10
    }));

    let result = tool.execute(&args).await.unwrap();
    assert!(result.success);
}
