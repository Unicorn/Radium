//! Integration tests for file operations in orchestration

use radium_orchestrator::orchestration::file_tools::{create_file_operation_tools, WorkspaceRootProvider};
use radium_orchestrator::orchestration::tool::{ToolArguments, ToolResult};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;

struct TestWorkspaceRoot {
    root: PathBuf,
}

impl WorkspaceRootProvider for TestWorkspaceRoot {
    fn workspace_root(&self) -> Option<PathBuf> {
        Some(self.root.clone())
    }
}

#[tokio::test]
async fn test_read_file_tool() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    tokio::fs::write(&test_file, "Hello, world!").await.unwrap();

    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tools = create_file_operation_tools(workspace_root);
    let read_tool = tools.iter().find(|t| t.name == "read_file").unwrap();

    let args = ToolArguments::new(serde_json::json!({
        "file_path": "test.txt"
    }));

    let result = read_tool.execute(&args).await.unwrap();
    assert!(result.success);
    assert_eq!(result.output, "Hello, world!");
}

#[tokio::test]
async fn test_write_file_tool() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tools = create_file_operation_tools(workspace_root);
    let write_tool = tools.iter().find(|t| t.name == "write_file").unwrap();

    let args = ToolArguments::new(serde_json::json!({
        "file_path": "new_file.txt",
        "contents": "Test content"
    }));

    let result = write_tool.execute(&args).await.unwrap();
    assert!(result.success);

    let content = tokio::fs::read_to_string(temp_dir.path().join("new_file.txt")).await.unwrap();
    assert_eq!(content, "Test content");
}

#[tokio::test]
async fn test_search_replace_tool() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    tokio::fs::write(&test_file, "Hello, world! Hello again!").await.unwrap();

    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tools = create_file_operation_tools(workspace_root);
    let replace_tool = tools.iter().find(|t| t.name == "search_replace").unwrap();

    let args = ToolArguments::new(serde_json::json!({
        "file_path": "test.txt",
        "old_string": "Hello",
        "new_string": "Hi"
    }));

    let result = replace_tool.execute(&args).await.unwrap();
    assert!(result.success);

    let content = tokio::fs::read_to_string(&test_file).await.unwrap();
    assert_eq!(content, "Hi, world! Hi again!");
}

#[tokio::test]
async fn test_list_dir_tool() {
    let temp_dir = TempDir::new().unwrap();
    tokio::fs::write(temp_dir.path().join("file1.txt"), "content1").await.unwrap();
    tokio::fs::write(temp_dir.path().join("file2.txt"), "content2").await.unwrap();
    tokio::fs::create_dir(temp_dir.path().join("subdir")).await.unwrap();

    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tools = create_file_operation_tools(workspace_root);
    let list_tool = tools.iter().find(|t| t.name == "list_dir").unwrap();

    let args = ToolArguments::new(serde_json::json!({
        "dir_path": "."
    }));

    let result = list_tool.execute(&args).await.unwrap();
    assert!(result.success);
    assert!(result.output.contains("file1.txt"));
    assert!(result.output.contains("file2.txt"));
}

#[tokio::test]
async fn test_file_not_found_error() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tools = create_file_operation_tools(workspace_root);
    let read_tool = tools.iter().find(|t| t.name == "read_file").unwrap();

    let args = ToolArguments::new(serde_json::json!({
        "file_path": "nonexistent.txt"
    }));

    let result = read_tool.execute(&args).await.unwrap();
    assert!(!result.success);
    assert!(result.output.contains("Failed to read"));
}

