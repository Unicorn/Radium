//! Git snapshot management for checkpointing agent work.

use super::error::{CheckpointError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Checkpoint metadata.
#[derive(Debug, Clone)]
pub struct Checkpoint {
    /// Unique checkpoint identifier.
    pub id: String,
    /// Git commit hash.
    pub commit_hash: String,
    /// Agent ID that created this checkpoint.
    pub agent_id: Option<String>,
    /// Creation timestamp (Unix epoch seconds).
    pub timestamp: u64,
    /// User-provided description.
    pub description: Option<String>,
}

impl Checkpoint {
    /// Creates a new checkpoint.
    pub fn new(id: String, commit_hash: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            commit_hash,
            agent_id: None,
            timestamp,
            description: None,
        }
    }

    /// Sets the agent ID.
    pub fn with_agent(mut self, agent_id: String) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

/// Checkpoint manager for git snapshots.
pub struct CheckpointManager {
    /// Workspace root directory.
    workspace_root: PathBuf,
    /// Shadow git repository path.
    shadow_repo: PathBuf,
}

impl CheckpointManager {
    /// Creates a new checkpoint manager.
    ///
    /// # Arguments
    /// * `workspace_root` - Root directory of the workspace
    ///
    /// # Errors
    /// Returns error if workspace is not a git repository
    pub fn new(workspace_root: impl AsRef<Path>) -> Result<Self> {
        let workspace_root = workspace_root.as_ref().to_path_buf();
        let shadow_repo = workspace_root.join(".radium").join("_internals").join("checkpoints");

        // Verify workspace is a git repository
        if !workspace_root.join(".git").exists() {
            return Err(CheckpointError::RepositoryNotFound(
                workspace_root.display().to_string(),
            ));
        }

        // Create shadow repo directory
        fs::create_dir_all(&shadow_repo)?;

        Ok(Self {
            workspace_root,
            shadow_repo,
        })
    }

    /// Initializes the shadow git repository.
    ///
    /// # Errors
    /// Returns error if git initialization fails
    pub fn initialize_shadow_repo(&self) -> Result<()> {
        // Check if already initialized (bare repo has HEAD file in root)
        if self.shadow_repo.join("HEAD").exists() {
            return Ok(());
        }

        // Initialize bare git repository
        let output = Command::new("git")
            .args(["init", "--bare"])
            .current_dir(&self.shadow_repo)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CheckpointError::ShadowRepoInitFailed(stderr.to_string()));
        }

        Ok(())
    }

    /// Creates a checkpoint of the current state.
    ///
    /// # Arguments
    /// * `description` - Optional description for the checkpoint
    ///
    /// # Returns
    /// The created checkpoint
    ///
    /// # Errors
    /// Returns error if git operations fail
    pub fn create_checkpoint(&self, description: Option<String>) -> Result<Checkpoint> {
        // Ensure shadow repo is initialized
        self.initialize_shadow_repo()?;

        // Get current commit hash
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.workspace_root)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CheckpointError::GitCommandFailed(stderr.to_string()));
        }

        let commit_hash = String::from_utf8(output.stdout)?.trim().to_string();

        // Generate unique checkpoint ID
        let uuid = Uuid::new_v4();
        let checkpoint_id = format!("checkpoint-{}", uuid.simple());

        // Create tag in shadow repo pointing to this commit
        let tag_name = format!("refs/tags/{}", checkpoint_id);
        let output = Command::new("git")
            .args(["tag", &checkpoint_id, &commit_hash])
            .current_dir(&self.workspace_root)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CheckpointError::GitCommandFailed(stderr.to_string()));
        }

        let mut checkpoint = Checkpoint::new(checkpoint_id, commit_hash);
        if let Some(desc) = description {
            checkpoint = checkpoint.with_description(desc);
        }

        Ok(checkpoint)
    }

    /// Lists all checkpoints.
    ///
    /// # Returns
    /// List of checkpoint metadata
    ///
    /// # Errors
    /// Returns error if git operations fail
    pub fn list_checkpoints(&self) -> Result<Vec<Checkpoint>> {
        let output = Command::new("git")
            .args(["tag", "-l", "checkpoint-*"])
            .current_dir(&self.workspace_root)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CheckpointError::GitCommandFailed(stderr.to_string()));
        }

        let tags = String::from_utf8(output.stdout)?;
        let mut checkpoints = Vec::new();

        for tag in tags.lines() {
            if tag.is_empty() {
                continue;
            }

            // Get commit hash for this tag
            let output = Command::new("git")
                .args(["rev-list", "-n", "1", tag])
                .current_dir(&self.workspace_root)
                .output()?;

            if output.status.success() {
                let commit_hash = String::from_utf8(output.stdout)?.trim().to_string();
                checkpoints.push(Checkpoint::new(tag.to_string(), commit_hash));
            }
        }

        // Sort by ID (which includes timestamp)
        checkpoints.sort_by(|a, b| b.id.cmp(&a.id));

        Ok(checkpoints)
    }

    /// Gets a specific checkpoint by ID.
    ///
    /// # Arguments
    /// * `checkpoint_id` - Checkpoint identifier
    ///
    /// # Returns
    /// Checkpoint metadata
    ///
    /// # Errors
    /// Returns error if checkpoint not found or git operations fail
    pub fn get_checkpoint(&self, checkpoint_id: &str) -> Result<Checkpoint> {
        let output = Command::new("git")
            .args(["rev-list", "-n", "1", checkpoint_id])
            .current_dir(&self.workspace_root)
            .output()?;

        if !output.status.success() {
            return Err(CheckpointError::CheckpointNotFound(checkpoint_id.to_string()));
        }

        let commit_hash = String::from_utf8(output.stdout)?.trim().to_string();
        Ok(Checkpoint::new(checkpoint_id.to_string(), commit_hash))
    }

    /// Restores a checkpoint.
    ///
    /// # Arguments
    /// * `checkpoint_id` - Checkpoint identifier to restore
    ///
    /// # Errors
    /// Returns error if restore fails
    pub fn restore_checkpoint(&self, checkpoint_id: &str) -> Result<()> {
        // Verify checkpoint exists
        let checkpoint = self.get_checkpoint(checkpoint_id)?;

        // Checkout the commit
        let output = Command::new("git")
            .args(["checkout", &checkpoint.commit_hash])
            .current_dir(&self.workspace_root)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CheckpointError::RestoreFailed(stderr.to_string()));
        }

        Ok(())
    }

    /// Deletes a checkpoint.
    ///
    /// # Arguments
    /// * `checkpoint_id` - Checkpoint identifier to delete
    ///
    /// # Errors
    /// Returns error if deletion fails
    pub fn delete_checkpoint(&self, checkpoint_id: &str) -> Result<()> {
        let output = Command::new("git")
            .args(["tag", "-d", checkpoint_id])
            .current_dir(&self.workspace_root)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CheckpointError::GitCommandFailed(stderr.to_string()));
        }

        Ok(())
    }

    /// Gets the diff between current state and a checkpoint.
    ///
    /// # Arguments
    /// * `checkpoint_id` - Checkpoint to compare against
    ///
    /// # Returns
    /// Git diff output
    ///
    /// # Errors
    /// Returns error if diff fails
    pub fn diff_checkpoint(&self, checkpoint_id: &str) -> Result<String> {
        let checkpoint = self.get_checkpoint(checkpoint_id)?;

        let output = Command::new("git")
            .args(["diff", &checkpoint.commit_hash])
            .current_dir(&self.workspace_root)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CheckpointError::GitCommandFailed(stderr.to_string()));
        }

        Ok(String::from_utf8(output.stdout)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_git_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Initialize git repo
        Command::new("git")
            .args(["init"])
            .current_dir(path)
            .output()
            .unwrap();

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

        Command::new("git")
            .args(["add", "."])
            .current_dir(path)
            .output()
            .unwrap();

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(path)
            .output()
            .unwrap();

        temp_dir
    }

    #[test]
    fn test_checkpoint_manager_new() {
        let temp_dir = setup_git_repo();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();
        assert!(manager.workspace_root.exists());
    }

    #[test]
    fn test_checkpoint_manager_not_git_repo() {
        let temp_dir = TempDir::new().unwrap();
        let result = CheckpointManager::new(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_initialize_shadow_repo() {
        let temp_dir = setup_git_repo();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();
        manager.initialize_shadow_repo().unwrap();

        // Bare repo has HEAD file in root, not .git subdirectory
        let shadow_head = temp_dir.path().join(".radium/_internals/checkpoints/HEAD");
        assert!(shadow_head.exists());
    }

    #[test]
    fn test_create_checkpoint() {
        let temp_dir = setup_git_repo();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        let checkpoint = manager
            .create_checkpoint(Some("Test checkpoint".to_string()))
            .unwrap();

        assert!(checkpoint.id.starts_with("checkpoint-"));
        assert!(!checkpoint.commit_hash.is_empty());
        assert_eq!(checkpoint.description, Some("Test checkpoint".to_string()));
    }

    #[test]
    fn test_list_checkpoints() {
        let temp_dir = setup_git_repo();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        manager.create_checkpoint(Some("CP1".to_string())).unwrap();
        manager.create_checkpoint(Some("CP2".to_string())).unwrap();

        let checkpoints = manager.list_checkpoints().unwrap();
        assert_eq!(checkpoints.len(), 2);
    }

    #[test]
    fn test_get_checkpoint() {
        let temp_dir = setup_git_repo();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        let created = manager.create_checkpoint(None).unwrap();
        let retrieved = manager.get_checkpoint(&created.id).unwrap();

        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.commit_hash, created.commit_hash);
    }

    #[test]
    fn test_delete_checkpoint() {
        let temp_dir = setup_git_repo();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        let checkpoint = manager.create_checkpoint(None).unwrap();
        manager.delete_checkpoint(&checkpoint.id).unwrap();

        let result = manager.get_checkpoint(&checkpoint.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_diff_checkpoint() {
        let temp_dir = setup_git_repo();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        let checkpoint = manager.create_checkpoint(None).unwrap();

        // Modify file
        let mut file = File::create(temp_dir.path().join("test.txt")).unwrap();
        writeln!(file, "modified content").unwrap();

        let diff = manager.diff_checkpoint(&checkpoint.id).unwrap();
        assert!(diff.contains("modified content"));
    }

    #[test]
    fn test_checkpoint_with_agent() {
        let checkpoint = Checkpoint::new("test-id".to_string(), "abc123".to_string())
            .with_agent("my-agent".to_string());

        assert_eq!(checkpoint.agent_id, Some("my-agent".to_string()));
        assert_eq!(checkpoint.id, "test-id");
        assert_eq!(checkpoint.commit_hash, "abc123");
    }

    #[test]
    fn test_checkpoint_with_agent_and_description() {
        let checkpoint = Checkpoint::new("test-id".to_string(), "abc123".to_string())
            .with_agent("my-agent".to_string())
            .with_description("Test description".to_string());

        assert_eq!(checkpoint.agent_id, Some("my-agent".to_string()));
        assert_eq!(checkpoint.description, Some("Test description".to_string()));
    }

    #[test]
    fn test_restore_checkpoint() {
        let temp_dir = setup_git_repo();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        // Create initial file
        let file_path = temp_dir.path().join("restore_test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "original content").unwrap();
        drop(file);

        // Commit the file
        Command::new("git")
            .args(["add", "restore_test.txt"])
            .current_dir(temp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "Add restore test file"])
            .current_dir(temp_dir.path())
            .output()
            .unwrap();

        // Create checkpoint
        let checkpoint = manager.create_checkpoint(Some("Before modification".to_string())).unwrap();

        // Modify file
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "modified content").unwrap();
        drop(file);

        // Commit modification
        Command::new("git")
            .args(["add", "restore_test.txt"])
            .current_dir(temp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "Modify file"])
            .current_dir(temp_dir.path())
            .output()
            .unwrap();

        // Restore checkpoint
        manager.restore_checkpoint(&checkpoint.id).unwrap();

        // Verify content is restored
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("original content"));
        assert!(!content.contains("modified content"));
    }

    #[test]
    fn test_get_checkpoint_not_found() {
        let temp_dir = setup_git_repo();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        let result = manager.get_checkpoint("nonexistent-checkpoint");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_checkpoint_not_found() {
        let temp_dir = setup_git_repo();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        let result = manager.delete_checkpoint("nonexistent-checkpoint");
        assert!(result.is_err());
    }

    #[test]
    fn test_restore_checkpoint_not_found() {
        let temp_dir = setup_git_repo();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        let result = manager.restore_checkpoint("nonexistent-checkpoint");
        assert!(result.is_err());
    }

    #[test]
    fn test_diff_checkpoint_not_found() {
        let temp_dir = setup_git_repo();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        let result = manager.diff_checkpoint("nonexistent-checkpoint");
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_checkpoints_order() {
        let temp_dir = setup_git_repo();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        let cp1 = manager.create_checkpoint(Some("First".to_string())).unwrap();
        let cp2 = manager.create_checkpoint(Some("Second".to_string())).unwrap();
        let cp3 = manager.create_checkpoint(Some("Third".to_string())).unwrap();

        let checkpoints = manager.list_checkpoints().unwrap();
        assert_eq!(checkpoints.len(), 3);

        // Verify all checkpoints are present
        let ids: Vec<_> = checkpoints.iter().map(|c| c.id.as_str()).collect();
        assert!(ids.contains(&cp1.id.as_str()));
        assert!(ids.contains(&cp2.id.as_str()));
        assert!(ids.contains(&cp3.id.as_str()));
    }

    #[test]
    fn test_create_checkpoint_without_description() {
        let temp_dir = setup_git_repo();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        let checkpoint = manager.create_checkpoint(None).unwrap();
        assert!(checkpoint.description.is_none());
        assert!(!checkpoint.id.is_empty());
        assert!(!checkpoint.commit_hash.is_empty());
    }

    #[test]
    fn test_list_checkpoints_empty() {
        let temp_dir = setup_git_repo();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        let checkpoints = manager.list_checkpoints().unwrap();
        assert_eq!(checkpoints.len(), 0);
    }
}
