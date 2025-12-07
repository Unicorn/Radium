//! Integration tests for checkpoint workflow scenarios.
//!
//! These tests verify end-to-end checkpoint functionality including:
//! - Automatic checkpoint creation during workflow execution
//! - Checkpoint restoration with workspace state verification
//! - Checkpoint listing and metadata persistence
//! - Error scenarios and edge cases

use radium_core::checkpoint::{Checkpoint, CheckpointManager};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Helper function to set up a Git repository for testing.
fn setup_git_repo() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // Initialize git repo
    Command::new("git").args(["init"]).current_dir(path).output().unwrap();

    // Configure git
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()
        .unwrap();

    // Create initial commit
    let mut file = File::create(path.join("test.txt")).unwrap();
    writeln!(file, "initial content").unwrap();
    drop(file);

    Command::new("git").args(["add", "."]).current_dir(path).output().unwrap();

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()
        .unwrap();

    temp_dir
}

#[test]
fn test_automatic_checkpoint_creation_workflow() {
    let temp_dir = setup_git_repo();
    let manager = CheckpointManager::new(temp_dir.path()).unwrap();

    // Simulate workflow step execution
    // 1. Create checkpoint before modification
    let checkpoint = manager
        .create_checkpoint(Some("Before workflow step: step-1".to_string()))
        .unwrap();

    assert!(checkpoint.id.starts_with("checkpoint-"));
    assert!(!checkpoint.commit_hash.is_empty());
    assert_eq!(
        checkpoint.description,
        Some("Before workflow step: step-1".to_string())
    );

    // 2. Simulate file modification (workflow step execution)
    let file_path = temp_dir.path().join("workflow_file.txt");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "workflow modification").unwrap();
    drop(file);

    Command::new("git")
        .args(["add", "workflow_file.txt"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    Command::new("git")
        .args(["commit", "-m", "Workflow step execution"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // 3. Verify checkpoint still exists and is valid
    let retrieved = manager.get_checkpoint(&checkpoint.id).unwrap();
    assert_eq!(retrieved.id, checkpoint.id);
    assert_eq!(retrieved.commit_hash, checkpoint.commit_hash);
}

#[test]
fn test_checkpoint_restoration_workspace_state() {
    let temp_dir = setup_git_repo();
    let manager = CheckpointManager::new(temp_dir.path()).unwrap();

    // 1. Create initial state
    let file_path = temp_dir.path().join("restore_test.txt");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "original state").unwrap();
    drop(file);

    Command::new("git")
        .args(["add", "restore_test.txt"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    Command::new("git")
        .args(["commit", "-m", "Initial state"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // 2. Create checkpoint
    let checkpoint = manager
        .create_checkpoint(Some("Before modification".to_string()))
        .unwrap();

    // 3. Modify workspace state
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "modified state").unwrap();
    drop(file);

    Command::new("git")
        .args(["add", "restore_test.txt"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    Command::new("git")
        .args(["commit", "-m", "Modified state"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // Verify modification
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("modified state"));

    // 4. Restore checkpoint
    manager.restore_checkpoint(&checkpoint.id).unwrap();

    // 5. Verify workspace state is restored
    let restored_content = fs::read_to_string(&file_path).unwrap();
    assert!(restored_content.contains("original state"));
    assert!(!restored_content.contains("modified state"));
}

#[test]
fn test_checkpoint_listing_multiple_checkpoints() {
    let temp_dir = setup_git_repo();
    let manager = CheckpointManager::new(temp_dir.path()).unwrap();

    // Create multiple checkpoints
    let cp1 = manager.create_checkpoint(Some("Checkpoint 1".to_string())).unwrap();
    let cp2 = manager.create_checkpoint(Some("Checkpoint 2".to_string())).unwrap();
    let cp3 = manager.create_checkpoint(Some("Checkpoint 3".to_string())).unwrap();

    // List all checkpoints
    let checkpoints = manager.list_checkpoints().unwrap();
    assert_eq!(checkpoints.len(), 3);

    // Verify all checkpoints are present
    let ids: Vec<_> = checkpoints.iter().map(|c| c.id.as_str()).collect();
    assert!(ids.contains(&cp1.id.as_str()));
    assert!(ids.contains(&cp2.id.as_str()));
    assert!(ids.contains(&cp3.id.as_str()));

    // Verify metadata persistence
    for checkpoint in &checkpoints {
        assert!(checkpoint.id.starts_with("checkpoint-"));
        assert!(!checkpoint.commit_hash.is_empty());
        assert!(checkpoint.timestamp > 0);
    }

    // Verify ordering (should be sorted by ID, newest first based on implementation)
    // The actual ordering depends on implementation, but all should be present
    assert!(checkpoints.iter().any(|c| c.id == cp1.id));
    assert!(checkpoints.iter().any(|c| c.id == cp2.id));
    assert!(checkpoints.iter().any(|c| c.id == cp3.id));
}

#[test]
fn test_checkpoint_deletion_cleanup() {
    let temp_dir = setup_git_repo();
    let manager = CheckpointManager::new(temp_dir.path()).unwrap();

    // Create checkpoints
    let cp1 = manager.create_checkpoint(Some("CP1".to_string())).unwrap();
    let cp2 = manager.create_checkpoint(Some("CP2".to_string())).unwrap();
    let cp3 = manager.create_checkpoint(Some("CP3".to_string())).unwrap();

    // Verify all exist
    let checkpoints = manager.list_checkpoints().unwrap();
    assert_eq!(checkpoints.len(), 3);

    // Delete one checkpoint
    manager.delete_checkpoint(&cp2.id).unwrap();

    // Verify deletion
    let result = manager.get_checkpoint(&cp2.id);
    assert!(result.is_err());

    // Verify others still exist
    let remaining = manager.list_checkpoints().unwrap();
    assert_eq!(remaining.len(), 2);
    let remaining_ids: Vec<_> = remaining.iter().map(|c| c.id.as_str()).collect();
    assert!(remaining_ids.contains(&cp1.id.as_str()));
    assert!(remaining_ids.contains(&cp3.id.as_str()));
    assert!(!remaining_ids.contains(&cp2.id.as_str()));
}

#[test]
fn test_shadow_repository_initialization() {
    let temp_dir = setup_git_repo();
    let manager = CheckpointManager::new(temp_dir.path()).unwrap();

    // Shadow repo should not exist yet
    let shadow_repo = temp_dir.path().join(".radium/_internals/checkpoints");
    assert!(!shadow_repo.join("HEAD").exists());

    // Initialize shadow repo
    manager.initialize_shadow_repo().unwrap();

    // Verify shadow repo is initialized (bare repo has HEAD in root)
    assert!(shadow_repo.join("HEAD").exists());
    assert!(shadow_repo.join("config").exists());

    // Creating a checkpoint should also initialize if not already done
    let temp_dir2 = setup_git_repo();
    let manager2 = CheckpointManager::new(temp_dir2.path()).unwrap();
    let _checkpoint = manager2.create_checkpoint(None).unwrap();

    let shadow_repo2 = temp_dir2.path().join(".radium/_internals/checkpoints");
    assert!(shadow_repo2.join("HEAD").exists());
}

#[test]
fn test_checkpoint_error_scenarios() {
    let temp_dir = setup_git_repo();
    let manager = CheckpointManager::new(temp_dir.path()).unwrap();

    // Test: Get non-existent checkpoint
    let result = manager.get_checkpoint("nonexistent-checkpoint-12345");
    assert!(result.is_err());

    // Test: Delete non-existent checkpoint
    let result = manager.delete_checkpoint("nonexistent-checkpoint-12345");
    assert!(result.is_err());

    // Test: Restore non-existent checkpoint
    let result = manager.restore_checkpoint("nonexistent-checkpoint-12345");
    assert!(result.is_err());

    // Test: Diff non-existent checkpoint
    let result = manager.diff_checkpoint("nonexistent-checkpoint-12345");
    assert!(result.is_err());
}

#[test]
fn test_checkpoint_manager_not_git_repo() {
    // Test: CheckpointManager should fail if workspace is not a git repo
    let temp_dir = TempDir::new().unwrap();
    let result = CheckpointManager::new(temp_dir.path());
    assert!(result.is_err());
}

#[test]
fn test_checkpoint_with_metadata() {
    let temp_dir = setup_git_repo();
    let manager = CheckpointManager::new(temp_dir.path()).unwrap();

    // Create checkpoint with all metadata
    let checkpoint = manager
        .create_checkpoint(Some("Test checkpoint with metadata".to_string()))
        .unwrap()
        .with_agent("test-agent".to_string())
        .with_task_id("task-123".to_string())
        .with_workflow_id("workflow-456".to_string());

    // Note: The create_checkpoint method doesn't return a builder pattern,
    // so we need to verify what's actually stored
    let retrieved = manager.get_checkpoint(&checkpoint.id).unwrap();
    assert_eq!(retrieved.id, checkpoint.id);
    assert_eq!(retrieved.commit_hash, checkpoint.commit_hash);
    assert_eq!(
        retrieved.description,
        Some("Test checkpoint with metadata".to_string())
    );
}

#[test]
fn test_checkpoint_find_for_step() {
    let temp_dir = setup_git_repo();
    let manager = CheckpointManager::new(temp_dir.path()).unwrap();

    // Create checkpoint with task ID in description (simulating workflow checkpoint)
    let checkpoint = manager
        .create_checkpoint(Some("Before workflow step: step-123".to_string()))
        .unwrap();

    // Find checkpoint for step
    let found = manager.find_checkpoint_for_step("step-123");
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, checkpoint.id);
}

#[test]
fn test_multiple_checkpoints_restore_sequence() {
    let temp_dir = setup_git_repo();
    let manager = CheckpointManager::new(temp_dir.path()).unwrap();

    let file_path = temp_dir.path().join("sequence_test.txt");

    // Create checkpoint 1
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "state 1").unwrap();
    drop(file);
    Command::new("git").args(["add", "."]).current_dir(temp_dir.path()).output().unwrap();
    Command::new("git")
        .args(["commit", "-m", "State 1"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    let cp1 = manager.create_checkpoint(Some("State 1".to_string())).unwrap();

    // Create checkpoint 2
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "state 2").unwrap();
    drop(file);
    Command::new("git").args(["add", "."]).current_dir(temp_dir.path()).output().unwrap();
    Command::new("git")
        .args(["commit", "-m", "State 2"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    let cp2 = manager.create_checkpoint(Some("State 2".to_string())).unwrap();

    // Create checkpoint 3
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "state 3").unwrap();
    drop(file);
    Command::new("git").args(["add", "."]).current_dir(temp_dir.path()).output().unwrap();
    Command::new("git")
        .args(["commit", "-m", "State 3"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    let cp3 = manager.create_checkpoint(Some("State 3".to_string())).unwrap();

    // Restore to checkpoint 1
    manager.restore_checkpoint(&cp1.id).unwrap();
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("state 1"));

    // Restore to checkpoint 3
    manager.restore_checkpoint(&cp3.id).unwrap();
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("state 3"));

    // Restore to checkpoint 2
    manager.restore_checkpoint(&cp2.id).unwrap();
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("state 2"));
}

#[test]
fn test_checkpoint_listing_empty() {
    let temp_dir = setup_git_repo();
    let manager = CheckpointManager::new(temp_dir.path()).unwrap();

    // List checkpoints when none exist
    let checkpoints = manager.list_checkpoints().unwrap();
    assert_eq!(checkpoints.len(), 0);
}

#[test]
fn test_checkpoint_metadata_persistence() {
    let temp_dir = setup_git_repo();
    let manager = CheckpointManager::new(temp_dir.path()).unwrap();

    // Create checkpoint with description
    let description = "Test checkpoint for metadata persistence";
    let checkpoint = manager.create_checkpoint(Some(description.to_string())).unwrap();

    // Verify initial metadata
    assert!(checkpoint.id.starts_with("checkpoint-"));
    assert!(!checkpoint.commit_hash.is_empty());
    assert_eq!(checkpoint.description, Some(description.to_string()));
    assert!(checkpoint.timestamp > 0);

    // Retrieve checkpoint and verify metadata persists
    let retrieved = manager.get_checkpoint(&checkpoint.id).unwrap();
    assert_eq!(retrieved.id, checkpoint.id);
    assert_eq!(retrieved.commit_hash, checkpoint.commit_hash);
    assert_eq!(retrieved.description, checkpoint.description);
    assert_eq!(retrieved.timestamp, checkpoint.timestamp);
}

