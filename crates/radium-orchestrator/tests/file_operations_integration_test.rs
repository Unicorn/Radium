//! Integration tests for file operation tools
//!
//! These tests verify the complete lifecycle of file operations with real filesystem,
//! including the critical leading slash handling fix and workspace boundary protection.

use radium_orchestrator::orchestration::file_tools::{create_file_operation_tools, WorkspaceRootProvider};
use radium_orchestrator::orchestration::tool::ToolArguments;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

/// Test workspace root provider
struct TestWorkspaceRoot {
    root: PathBuf,
}

impl WorkspaceRootProvider for TestWorkspaceRoot {
    fn workspace_root(&self) -> Option<PathBuf> {
        Some(self.root.clone())
    }
}

/// Complete file lifecycle: create_dir → write_file → read_file → rename_file → delete_file
#[tokio::test]
async fn test_complete_file_lifecycle() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tools = create_file_operation_tools(workspace_root);

    // Find our tools
    let create_dir = tools.iter().find(|t| t.name == "create_dir").unwrap();
    let write_file = tools.iter().find(|t| t.name == "write_file").unwrap();
    let read_file = tools.iter().find(|t| t.name == "read_file").unwrap();
    let rename_file = tools.iter().find(|t| t.name == "rename_file").unwrap();
    let delete_file = tools.iter().find(|t| t.name == "delete_file").unwrap();

    // Step 1: Create directory
    let args = ToolArguments::new(serde_json::json!({
        "dir_path": "my_project/docs"
    }));
    let result = create_dir.execute(&args).await.unwrap();
    assert!(result.success, "Failed to create directory: {}", result.output);
    assert!(temp_dir.path().join("my_project/docs").is_dir());

    // Step 2: Write file
    let args = ToolArguments::new(serde_json::json!({
        "file_path": "my_project/docs/README.md",
        "contents": "# My Project\n\nThis is a test project."
    }));
    let result = write_file.execute(&args).await.unwrap();
    assert!(result.success, "Failed to write file: {}", result.output);
    assert!(temp_dir.path().join("my_project/docs/README.md").exists());

    // Step 3: Read file
    let args = ToolArguments::new(serde_json::json!({
        "file_path": "my_project/docs/README.md"
    }));
    let result = read_file.execute(&args).await.unwrap();
    assert!(result.success, "Failed to read file: {}", result.output);
    assert!(result.output.contains("# My Project"));

    // Step 4: Rename file
    let args = ToolArguments::new(serde_json::json!({
        "old_path": "my_project/docs/README.md",
        "new_path": "my_project/docs/GETTING_STARTED.md"
    }));
    let result = rename_file.execute(&args).await.unwrap();
    assert!(result.success, "Failed to rename file: {}", result.output);
    assert!(!temp_dir.path().join("my_project/docs/README.md").exists());
    assert!(temp_dir.path().join("my_project/docs/GETTING_STARTED.md").exists());

    // Step 5: Delete file
    let args = ToolArguments::new(serde_json::json!({
        "file_path": "my_project/docs/GETTING_STARTED.md"
    }));
    let result = delete_file.execute(&args).await.unwrap();
    assert!(result.success, "Failed to delete file: {}", result.output);
    assert!(!temp_dir.path().join("my_project/docs/GETTING_STARTED.md").exists());
}

/// Test leading slash handling across ALL file operation tools
#[tokio::test]
async fn test_leading_slash_handling_all_tools() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tools = create_file_operation_tools(workspace_root);

    // Find our tools
    let create_dir = tools.iter().find(|t| t.name == "create_dir").unwrap();
    let write_file = tools.iter().find(|t| t.name == "write_file").unwrap();
    let read_file = tools.iter().find(|t| t.name == "read_file").unwrap();
    let list_dir = tools.iter().find(|t| t.name == "list_dir").unwrap();
    let rename_file = tools.iter().find(|t| t.name == "rename_file").unwrap();
    let delete_file = tools.iter().find(|t| t.name == "delete_file").unwrap();

    // Test create_dir with leading slash
    let args = ToolArguments::new(serde_json::json!({
        "dir_path": "/test_dir"
    }));
    let result = create_dir.execute(&args).await.unwrap();
    assert!(result.success, "create_dir failed with leading slash: {}", result.output);
    assert!(temp_dir.path().join("test_dir").is_dir(), "Directory not created at workspace root");

    // Test write_file with leading slash
    let args = ToolArguments::new(serde_json::json!({
        "file_path": "/test_dir/file.txt",
        "contents": "test content"
    }));
    let result = write_file.execute(&args).await.unwrap();
    assert!(result.success, "write_file failed with leading slash: {}", result.output);
    assert!(temp_dir.path().join("test_dir/file.txt").exists(), "File not created at workspace root");

    // Test read_file with leading slash
    let args = ToolArguments::new(serde_json::json!({
        "file_path": "/test_dir/file.txt"
    }));
    let result = read_file.execute(&args).await.unwrap();
    assert!(result.success, "read_file failed with leading slash: {}", result.output);
    assert_eq!(result.output, "test content");

    // Test list_dir with leading slash
    let args = ToolArguments::new(serde_json::json!({
        "dir_path": "/test_dir"
    }));
    let result = list_dir.execute(&args).await.unwrap();
    assert!(result.success, "list_dir failed with leading slash: {}", result.output);
    assert!(result.output.contains("file.txt"));

    // Test rename_file with leading slashes
    let args = ToolArguments::new(serde_json::json!({
        "old_path": "/test_dir/file.txt",
        "new_path": "/test_dir/renamed.txt"
    }));
    let result = rename_file.execute(&args).await.unwrap();
    assert!(result.success, "rename_file failed with leading slash: {}", result.output);
    assert!(temp_dir.path().join("test_dir/renamed.txt").exists(), "Renamed file not at workspace root");

    // Test delete_file with leading slash
    let args = ToolArguments::new(serde_json::json!({
        "file_path": "/test_dir/renamed.txt"
    }));
    let result = delete_file.execute(&args).await.unwrap();
    assert!(result.success, "delete_file failed with leading slash: {}", result.output);
    assert!(!temp_dir.path().join("test_dir/renamed.txt").exists());
}

/// Test workspace boundary protection - should reject paths that escape workspace
#[tokio::test]
async fn test_workspace_boundary_protection() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tools = create_file_operation_tools(workspace_root);
    let write_file = tools.iter().find(|t| t.name == "write_file").unwrap();

    // Try to write outside workspace using ../
    let args = ToolArguments::new(serde_json::json!({
        "file_path": "../outside.txt",
        "contents": "should not work"
    }));

    // The execute() returns a Result that contains an error (not a ToolResult with success=false)
    // because the boundary validation happens in resolve_path() which returns Result
    let result = write_file.execute(&args).await;
    assert!(result.is_err(), "Should return error for path traversal");

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("outside workspace boundary"),
        "Error message should mention boundary, got: {}", error_msg);
}

/// Test error handling for non-existent files
#[tokio::test]
async fn test_error_handling_nonexistent_files() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tools = create_file_operation_tools(workspace_root);

    // Try to read non-existent file
    let read_file = tools.iter().find(|t| t.name == "read_file").unwrap();
    let args = ToolArguments::new(serde_json::json!({
        "file_path": "does_not_exist.txt"
    }));
    let result = read_file.execute(&args).await.unwrap();
    assert!(!result.success);
    assert!(result.output.contains("Failed to read file"));

    // Try to delete non-existent file
    let delete_file = tools.iter().find(|t| t.name == "delete_file").unwrap();
    let args = ToolArguments::new(serde_json::json!({
        "file_path": "does_not_exist.txt"
    }));
    let result = delete_file.execute(&args).await.unwrap();
    assert!(!result.success);
    assert!(result.output.contains("not found"));

    // Try to rename non-existent file
    let rename_file = tools.iter().find(|t| t.name == "rename_file").unwrap();
    let args = ToolArguments::new(serde_json::json!({
        "old_path": "does_not_exist.txt",
        "new_path": "new_name.txt"
    }));
    let result = rename_file.execute(&args).await.unwrap();
    assert!(!result.success);
    assert!(result.output.contains("not found"));
}

/// Test nested directory creation
#[tokio::test]
async fn test_nested_directory_creation() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tools = create_file_operation_tools(workspace_root);
    let create_dir = tools.iter().find(|t| t.name == "create_dir").unwrap();

    // Create deeply nested directory structure
    let args = ToolArguments::new(serde_json::json!({
        "dir_path": "level1/level2/level3/level4"
    }));
    let result = create_dir.execute(&args).await.unwrap();
    assert!(result.success, "Failed to create nested directories: {}", result.output);
    assert!(temp_dir.path().join("level1/level2/level3/level4").is_dir());
}

/// Test write_file automatically creates parent directories
#[tokio::test]
async fn test_write_file_creates_parents() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tools = create_file_operation_tools(workspace_root);
    let write_file = tools.iter().find(|t| t.name == "write_file").unwrap();

    // Write to nested path that doesn't exist
    let args = ToolArguments::new(serde_json::json!({
        "file_path": "nested/path/to/file.txt",
        "contents": "auto-created parents"
    }));
    let result = write_file.execute(&args).await.unwrap();
    assert!(result.success, "Failed to auto-create parent dirs: {}", result.output);
    assert!(temp_dir.path().join("nested/path/to").is_dir());
    assert!(temp_dir.path().join("nested/path/to/file.txt").exists());
}

/// Test rename_file creates destination parent directories
#[tokio::test]
async fn test_rename_file_creates_destination_parents() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tools = create_file_operation_tools(workspace_root);
    let write_file = tools.iter().find(|t| t.name == "write_file").unwrap();
    let rename_file = tools.iter().find(|t| t.name == "rename_file").unwrap();

    // Create initial file
    let args = ToolArguments::new(serde_json::json!({
        "file_path": "source.txt",
        "contents": "test"
    }));
    write_file.execute(&args).await.unwrap();

    // Rename to nested destination
    let args = ToolArguments::new(serde_json::json!({
        "old_path": "source.txt",
        "new_path": "deep/nested/path/destination.txt"
    }));
    let result = rename_file.execute(&args).await.unwrap();
    assert!(result.success, "Failed to create destination parents: {}", result.output);
    assert!(temp_dir.path().join("deep/nested/path").is_dir());
    assert!(temp_dir.path().join("deep/nested/path/destination.txt").exists());
}

/// Test that delete_file rejects directories
#[tokio::test]
async fn test_delete_file_safety_check() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tools = create_file_operation_tools(workspace_root);
    let create_dir = tools.iter().find(|t| t.name == "create_dir").unwrap();
    let delete_file = tools.iter().find(|t| t.name == "delete_file").unwrap();

    // Create a directory
    let args = ToolArguments::new(serde_json::json!({
        "dir_path": "test_directory"
    }));
    create_dir.execute(&args).await.unwrap();

    // Try to delete it with delete_file (should fail)
    let args = ToolArguments::new(serde_json::json!({
        "file_path": "test_directory"
    }));
    let result = delete_file.execute(&args).await.unwrap();
    assert!(!result.success, "Should reject directory deletion");
    assert!(result.output.contains("not a file"), "Error should mention it's not a file");
    assert!(temp_dir.path().join("test_directory").exists(), "Directory should still exist");
}

/// Test rename_file rejects when destination exists
#[tokio::test]
async fn test_rename_file_destination_exists() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tools = create_file_operation_tools(workspace_root);
    let write_file = tools.iter().find(|t| t.name == "write_file").unwrap();
    let rename_file = tools.iter().find(|t| t.name == "rename_file").unwrap();

    // Create two files
    write_file.execute(&ToolArguments::new(serde_json::json!({
        "file_path": "file1.txt",
        "contents": "file 1"
    }))).await.unwrap();

    write_file.execute(&ToolArguments::new(serde_json::json!({
        "file_path": "file2.txt",
        "contents": "file 2"
    }))).await.unwrap();

    // Try to rename file1 to file2 (should fail)
    let args = ToolArguments::new(serde_json::json!({
        "old_path": "file1.txt",
        "new_path": "file2.txt"
    }));
    let result = rename_file.execute(&args).await.unwrap();
    assert!(!result.success, "Should reject when destination exists");
    assert!(result.output.contains("already exists"));
}

/// Test create_dir is idempotent
#[tokio::test]
async fn test_create_dir_idempotent() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = Arc::new(TestWorkspaceRoot {
        root: temp_dir.path().to_path_buf(),
    });

    let tools = create_file_operation_tools(workspace_root);
    let create_dir = tools.iter().find(|t| t.name == "create_dir").unwrap();

    let args = ToolArguments::new(serde_json::json!({
        "dir_path": "my_dir"
    }));

    // Create once
    let result1 = create_dir.execute(&args).await.unwrap();
    assert!(result1.success);
    assert_eq!(result1.metadata.get("created"), Some(&"true".to_string()));

    // Create again (should succeed, not error)
    let result2 = create_dir.execute(&args).await.unwrap();
    assert!(result2.success);
    assert_eq!(result2.metadata.get("already_existed"), Some(&"true".to_string()));
}
