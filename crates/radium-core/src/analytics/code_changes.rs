//! Code change tracking for session analytics.

use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// Code change statistics.
#[derive(Debug, Clone, Default)]
pub struct CodeChanges {
    /// Lines added
    pub lines_added: i64,
    /// Lines removed
    pub lines_removed: i64,
    /// Files changed
    pub files_changed: usize,
}

impl CodeChanges {
    /// Calculate code changes between two git commits or from a base commit.
    pub fn from_git_diff(workspace_root: &Path, base_commit: Option<&str>) -> Result<Self> {
        let mut changes = CodeChanges::default();

        // Check if this is a git repository
        let git_dir = workspace_root.join(".git");
        if !git_dir.exists() {
            // Not a git repo, return zeros
            return Ok(changes);
        }

        // Get git diff statistics
        let mut cmd = Command::new("git");
        cmd.current_dir(workspace_root);
        cmd.arg("diff");
        cmd.arg("--numstat");

        if let Some(base) = base_commit {
            cmd.arg(base);
        } else {
            // Compare against HEAD (uncommitted changes)
            cmd.arg("HEAD");
        }

        let output = cmd.output()?;

        if !output.status.success() {
            // Git command failed, return zeros
            return Ok(changes);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Parse numstat output: "added\tremoved\tfilename"
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let (Ok(added), Ok(removed)) = (
                    parts[0].parse::<i64>(),
                    parts[1].parse::<i64>(),
                ) {
                    changes.lines_added += added;
                    changes.lines_removed += removed;
                    changes.files_changed += 1;
                }
            }
        }

        Ok(changes)
    }

    /// Calculate code changes since a specific timestamp.
    pub fn from_git_since(workspace_root: &Path, since_timestamp: u64) -> Result<Self> {
        // Convert timestamp to git date format
        let since_date = chrono::DateTime::from_timestamp(since_timestamp as i64, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "1970-01-01 00:00:00".to_string());

        // Get commits since timestamp
        let mut cmd = Command::new("git");
        cmd.current_dir(workspace_root);
        cmd.args(&["log", "--since", &since_date, "--pretty=format:%H", "--reverse"]);
        
        let output = cmd.output()?;
        
        if !output.status.success() || output.stdout.is_empty() {
            // No commits or git command failed
            return CodeChanges::from_git_diff(workspace_root, None);
        }

        // Get first commit hash
        let first_commit_str = String::from_utf8_lossy(&output.stdout);
        let first_commit = first_commit_str
            .lines()
            .next()
            .unwrap_or("HEAD")
            .to_string();

        // Calculate diff from first commit to HEAD
        CodeChanges::from_git_diff(workspace_root, Some(&first_commit))
    }
}

