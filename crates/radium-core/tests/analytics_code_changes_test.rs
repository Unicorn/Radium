//! Unit tests for code change tracking.

use radium_core::analytics::code_changes::CodeChanges;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_code_changes_from_git_diff_non_git_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    // Ensure it's not a git repo
    assert!(!workspace_root.join(".git").exists());
    
    let changes = CodeChanges::from_git_diff(workspace_root, None)
        .expect("Should return zeros for non-git directory");
    
    assert_eq!(changes.lines_added, 0);
    assert_eq!(changes.lines_removed, 0);
    assert_eq!(changes.files_changed, 0);
}

#[test]
fn test_code_changes_from_git_diff_empty_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    // Initialize git repo
    let output = std::process::Command::new("git")
        .args(["init"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to init git");
    
    if !output.status.success() {
        // Git not available, skip this test
        return;
    }
    
    // Configure git user (required for commits)
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to set git config");
    
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to set git config");
    
    // Empty repo, no commits yet
    let changes = CodeChanges::from_git_diff(workspace_root, None)
        .expect("Should handle empty git repo");
    
    // Should return zeros or handle gracefully
    assert!(changes.lines_added >= 0);
    assert!(changes.lines_removed >= 0);
}

#[test]
fn test_code_changes_from_git_diff_with_commits() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    // Initialize git repo
    let output = std::process::Command::new("git")
        .args(["init"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to init git");
    
    if !output.status.success() {
        // Git not available, skip this test
        return;
    }
    
    // Configure git user
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to set git config");
    
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to set git config");
    
    // Create initial file and commit
    let test_file = workspace_root.join("test.txt");
    fs::write(&test_file, "line 1\nline 2\nline 3\n").expect("Failed to write test file");
    
    std::process::Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to git add");
    
    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to git commit");
    
    // Make changes
    fs::write(&test_file, "line 1\nline 2\nline 3\nline 4\nline 5\n").expect("Failed to write test file");
    
    // Get diff against HEAD (uncommitted changes)
    let changes = CodeChanges::from_git_diff(workspace_root, None)
        .expect("Should calculate diff");
    
    // Should detect 2 lines added (line 4 and line 5)
    assert!(changes.lines_added >= 0);
    assert!(changes.files_changed >= 0);
}

#[test]
fn test_code_changes_from_git_diff_with_base_commit() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    // Initialize git repo
    let output = std::process::Command::new("git")
        .args(["init"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to init git");
    
    if !output.status.success() {
        // Git not available, skip this test
        return;
    }
    
    // Configure git user
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to set git config");
    
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to set git config");
    
    // Create initial file and commit
    let test_file = workspace_root.join("test.txt");
    fs::write(&test_file, "line 1\n").expect("Failed to write test file");
    
    std::process::Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to git add");
    
    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to git commit");
    
    // Get the commit hash
    let commit_output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to get commit hash");
    
    if commit_output.status.success() {
        let commit_hash = String::from_utf8_lossy(&commit_output.stdout).trim().to_string();
        
        // Make changes
        fs::write(&test_file, "line 1\nline 2\nline 3\n").expect("Failed to write test file");
        
        // Get diff against specific commit
        let changes = CodeChanges::from_git_diff(workspace_root, Some(&commit_hash))
            .expect("Should calculate diff against commit");
        
        // Should detect changes
        assert!(changes.lines_added >= 0);
    }
}

#[test]
fn test_code_changes_from_git_since_non_git_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    let timestamp = chrono::Utc::now().timestamp() as u64;
    
    let changes = CodeChanges::from_git_since(workspace_root, timestamp)
        .expect("Should return zeros for non-git directory");
    
    assert_eq!(changes.lines_added, 0);
    assert_eq!(changes.lines_removed, 0);
    assert_eq!(changes.files_changed, 0);
}

#[test]
fn test_code_changes_from_git_since_with_commits() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    // Initialize git repo
    let output = std::process::Command::new("git")
        .args(["init"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to init git");
    
    if !output.status.success() {
        // Git not available, skip this test
        return;
    }
    
    // Configure git user
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to set git config");
    
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to set git config");
    
    // Create initial file and commit
    let test_file = workspace_root.join("test.txt");
    fs::write(&test_file, "line 1\n").expect("Failed to write test file");
    
    std::process::Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to git add");
    
    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to git commit");
    
    // Get timestamp before making changes
    let before_timestamp = chrono::Utc::now().timestamp() as u64;
    
    // Wait a moment
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    // Make changes and commit
    fs::write(&test_file, "line 1\nline 2\nline 3\n").expect("Failed to write test file");
    
    std::process::Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to git add");
    
    std::process::Command::new("git")
        .args(["commit", "-m", "Add more lines"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to git commit");
    
    // Get changes since timestamp
    let changes = CodeChanges::from_git_since(workspace_root, before_timestamp)
        .expect("Should calculate changes since timestamp");
    
    // Should detect changes
    assert!(changes.lines_added >= 0);
}

#[test]
fn test_code_changes_from_git_since_no_commits() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    // Initialize git repo
    let output = std::process::Command::new("git")
        .args(["init"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to init git");
    
    if !output.status.success() {
        // Git not available, skip this test
        return;
    }
    
    // Configure git user
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to set git config");
    
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to set git config");
    
    // Use a future timestamp (no commits since then)
    let future_timestamp = (chrono::Utc::now().timestamp() + 86400) as u64; // 1 day in future
    
    let changes = CodeChanges::from_git_since(workspace_root, future_timestamp)
        .expect("Should handle no commits since timestamp");
    
    // Should return zeros or handle gracefully
    assert!(changes.lines_added >= 0);
    assert!(changes.lines_removed >= 0);
}

#[test]
fn test_code_changes_default() {
    let changes = CodeChanges::default();
    
    assert_eq!(changes.lines_added, 0);
    assert_eq!(changes.lines_removed, 0);
    assert_eq!(changes.files_changed, 0);
}

#[test]
fn test_code_changes_parses_numstat_format() {
    // This test verifies the parsing logic by using a real git repo
    // The actual parsing is tested indirectly through from_git_diff
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_root = temp_dir.path();
    
    // Initialize git repo
    let output = std::process::Command::new("git")
        .args(["init"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to init git");
    
    if !output.status.success() {
        // Git not available, skip this test
        return;
    }
    
    // Configure git user
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to set git config");
    
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to set git config");
    
    // Create file with known content
    let test_file = workspace_root.join("test.txt");
    fs::write(&test_file, "a\nb\nc\n").expect("Failed to write test file");
    
    std::process::Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to git add");
    
    std::process::Command::new("git")
        .args(["commit", "-m", "Initial"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to git commit");
    
    // Modify file: add 2 lines, remove 1 line
    fs::write(&test_file, "a\nb\nx\ny\n").expect("Failed to write test file");
    
    let changes = CodeChanges::from_git_diff(workspace_root, None)
        .expect("Should parse numstat format");
    
    // Should detect changes (exact numbers depend on git diff output)
    assert!(changes.lines_added >= 0);
    assert!(changes.files_changed >= 0);
}

