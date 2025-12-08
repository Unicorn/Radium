//! Git snapshot management for checkpointing agent work.

use super::error::{CheckpointError, Result};
use serde_json;
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
    /// Task ID associated with this checkpoint (for recovery).
    pub task_id: Option<String>,
    /// Workflow ID associated with this checkpoint (for recovery).
    pub workflow_id: Option<String>,
    /// Execution duration from workflow start to checkpoint (seconds).
    pub execution_duration_secs: Option<u64>,
    /// Memory usage at checkpoint time (MB).
    pub memory_usage_mb: Option<f64>,
    /// CPU time consumed up to checkpoint (seconds).
    pub cpu_time_secs: Option<f64>,
    /// Total tokens used up to checkpoint time.
    pub tokens_used: Option<u64>,
}

/// Represents changes between two checkpoints.
#[derive(Debug, Clone)]
pub struct CheckpointDiff {
    /// Files that were added.
    pub added: Vec<String>,
    /// Files that were modified.
    pub modified: Vec<String>,
    /// Files that were deleted.
    pub deleted: Vec<String>,
    /// Raw Git diff output.
    pub raw_diff: String,
    /// Statistics: number of insertions.
    pub insertions: usize,
    /// Statistics: number of deletions.
    pub deletions: usize,
}

impl CheckpointDiff {
    /// Creates a new empty diff.
    pub fn new() -> Self {
        Self {
            added: Vec::new(),
            modified: Vec::new(),
            deleted: Vec::new(),
            raw_diff: String::new(),
            insertions: 0,
            deletions: 0,
        }
    }

    /// Gets the total number of files changed.
    pub fn files_changed(&self) -> usize {
        self.added.len() + self.modified.len() + self.deleted.len()
    }
}

impl Default for CheckpointDiff {
    fn default() -> Self {
        Self::new()
    }
}

impl Checkpoint {
    /// Creates a new checkpoint.
    pub fn new(id: String, commit_hash: String) -> Self {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        Self {
            id,
            commit_hash,
            agent_id: None,
            timestamp,
            description: None,
            task_id: None,
            workflow_id: None,
            execution_duration_secs: None,
            memory_usage_mb: None,
            cpu_time_secs: None,
            tokens_used: None,
        }
    }

    /// Sets the agent ID.
    #[must_use]
    pub fn with_agent(mut self, agent_id: String) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    /// Sets the description.
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets the task ID.
    #[must_use]
    pub fn with_task_id(mut self, task_id: String) -> Self {
        self.task_id = Some(task_id);
        self
    }

    /// Sets the workflow ID.
    #[must_use]
    pub fn with_workflow_id(mut self, workflow_id: String) -> Self {
        self.workflow_id = Some(workflow_id);
        self
    }

    /// Sets execution duration.
    #[must_use]
    pub fn with_execution_duration(mut self, duration_secs: u64) -> Self {
        self.execution_duration_secs = Some(duration_secs);
        self
    }

    /// Sets memory usage.
    #[must_use]
    pub fn with_memory_usage(mut self, memory_mb: f64) -> Self {
        self.memory_usage_mb = Some(memory_mb);
        self
    }

    /// Sets CPU time.
    #[must_use]
    pub fn with_cpu_time(mut self, cpu_secs: f64) -> Self {
        self.cpu_time_secs = Some(cpu_secs);
        self
    }

    /// Sets token usage.
    #[must_use]
    pub fn with_tokens_used(mut self, tokens: u64) -> Self {
        self.tokens_used = Some(tokens);
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
    /// Serializes checkpoint metadata to JSON for storage in Git tag annotation.
    fn serialize_metadata(checkpoint: &Checkpoint) -> String {
        serde_json::json!({
            "description": checkpoint.description,
            "agent_id": checkpoint.agent_id,
            "task_id": checkpoint.task_id,
            "workflow_id": checkpoint.workflow_id,
            "timestamp": checkpoint.timestamp,
            "execution_duration_secs": checkpoint.execution_duration_secs,
            "memory_usage_mb": checkpoint.memory_usage_mb,
            "cpu_time_secs": checkpoint.cpu_time_secs,
            "tokens_used": checkpoint.tokens_used,
        })
        .to_string()
    }

    /// Deserializes checkpoint metadata from Git tag annotation.
    fn deserialize_metadata(tag_id: &str, commit_hash: String, tag_message: &str) -> Checkpoint {
        let mut checkpoint = Checkpoint::new(tag_id.to_string(), commit_hash);
        
        // Try to parse JSON metadata from tag message
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(tag_message) {
            if let Some(desc) = json.get("description").and_then(|v| v.as_str()) {
                checkpoint = checkpoint.with_description(desc.to_string());
            }
            if let Some(agent_id) = json.get("agent_id").and_then(|v| v.as_str()) {
                checkpoint = checkpoint.with_agent(agent_id.to_string());
            }
            if let Some(task_id) = json.get("task_id").and_then(|v| v.as_str()) {
                checkpoint = checkpoint.with_task_id(task_id.to_string());
            }
            if let Some(workflow_id) = json.get("workflow_id").and_then(|v| v.as_str()) {
                checkpoint = checkpoint.with_workflow_id(workflow_id.to_string());
            }
            if let Some(timestamp) = json.get("timestamp").and_then(|v| v.as_u64()) {
                checkpoint.timestamp = timestamp;
            }
            if let Some(duration) = json.get("execution_duration_secs").and_then(|v| v.as_u64()) {
                checkpoint.execution_duration_secs = Some(duration);
            }
            if let Some(memory) = json.get("memory_usage_mb").and_then(|v| v.as_f64()) {
                checkpoint.memory_usage_mb = Some(memory);
            }
            if let Some(cpu) = json.get("cpu_time_secs").and_then(|v| v.as_f64()) {
                checkpoint.cpu_time_secs = Some(cpu);
            }
            if let Some(tokens) = json.get("tokens_used").and_then(|v| v.as_u64()) {
                checkpoint.tokens_used = Some(tokens);
            }
        }
        
        checkpoint
    }

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
            return Err(CheckpointError::RepositoryNotFound(workspace_root.display().to_string()));
        }

        // Create shadow repo directory
        fs::create_dir_all(&shadow_repo)?;

        Ok(Self { workspace_root, shadow_repo })
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
        let output =
            Command::new("git").args(["init", "--bare"]).current_dir(&self.shadow_repo).output()?;

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
    /// * `execution_start_time` - Optional execution start time for duration calculation
    /// * `tokens_used` - Optional total tokens used up to checkpoint
    ///
    /// # Returns
    /// The created checkpoint
    ///
    /// # Errors
    /// Returns error if git operations fail
    pub fn create_checkpoint(
        &self,
        description: Option<String>,
    ) -> Result<Checkpoint> {
        self.create_checkpoint_with_metrics(description, None, None)
    }

    /// Creates a checkpoint with resource metrics.
    ///
    /// # Arguments
    /// * `description` - Optional description for the checkpoint
    /// * `execution_start_time` - Optional execution start time (SystemTime) for duration calculation
    /// * `tokens_used` - Optional total tokens used up to checkpoint
    ///
    /// # Returns
    /// The created checkpoint
    ///
    /// # Errors
    /// Returns error if git operations fail
    pub fn create_checkpoint_with_metrics(
        &self,
        description: Option<String>,
        execution_start_time: Option<SystemTime>,
        tokens_used: Option<u64>,
    ) -> Result<Checkpoint> {
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

        // Create checkpoint struct with metadata
        let mut checkpoint = Checkpoint::new(checkpoint_id.clone(), commit_hash.clone());
        if let Some(desc) = description {
            checkpoint = checkpoint.with_description(desc);
        }

        // Add resource metrics if provided
        if let Some(start_time) = execution_start_time {
            let duration = SystemTime::now()
                .duration_since(start_time)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            checkpoint = checkpoint.with_execution_duration(duration);
        }
        if let Some(tokens) = tokens_used {
            checkpoint = checkpoint.with_tokens_used(tokens);
        }

        // Serialize metadata to JSON for tag annotation
        let metadata_json = Self::serialize_metadata(&checkpoint);

        // Create annotated tag with metadata in message
        let output = Command::new("git")
            .args(["tag", "-a", &checkpoint_id, &commit_hash, "-m", &metadata_json])
            .current_dir(&self.workspace_root)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CheckpointError::GitCommandFailed(stderr.to_string()));
        }

        Ok(checkpoint)
    }

    /// Creates a checkpoint with task and workflow metadata.
    ///
    /// # Arguments
    /// * `description` - Optional description for the checkpoint
    /// * `task_id` - Optional task ID associated with this checkpoint
    /// * `workflow_id` - Optional workflow ID associated with this checkpoint
    ///
    /// # Returns
    /// The created checkpoint
    ///
    /// # Errors
    /// Returns error if git operations fail
    pub fn create_checkpoint_with_metadata(
        &self,
        description: Option<String>,
        task_id: Option<String>,
        workflow_id: Option<String>,
    ) -> Result<Checkpoint> {
        let mut checkpoint = self.create_checkpoint(description)?;
        if let Some(tid) = task_id {
            checkpoint = checkpoint.with_task_id(tid);
        }
        if let Some(wid) = workflow_id {
            checkpoint = checkpoint.with_workflow_id(wid);
        }
        Ok(checkpoint)
    }

    /// Finds a checkpoint for a specific step.
    ///
    /// # Arguments
    /// * `step_id` - The step ID to find checkpoint for
    ///
    /// # Returns
    /// The checkpoint if found, None otherwise
    pub fn find_checkpoint_for_step(&self, step_id: &str) -> Option<Checkpoint> {
        if let Ok(checkpoints) = self.list_checkpoints() {
            checkpoints
                .into_iter()
                .find(|cp| {
                    cp.task_id.as_ref().map(|tid| tid == step_id).unwrap_or(false)
                        || cp.description
                            .as_ref()
                            .map(|d| d.contains(step_id))
                            .unwrap_or(false)
                })
        } else {
            None
        }
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
                
                // Get tag message (annotation) if it exists
                let tag_message = Command::new("git")
                    .args(["tag", "-l", "--format=%(contents)", tag])
                    .current_dir(&self.workspace_root)
                    .output()
                    .ok()
                    .and_then(|o| {
                        if o.status.success() {
                            String::from_utf8(o.stdout).ok()
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "{}".to_string());
                
                let checkpoint = Self::deserialize_metadata(tag, commit_hash, &tag_message);
                checkpoints.push(checkpoint);
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
        
        // Get tag message (annotation) if it exists
        let tag_message = Command::new("git")
            .args(["tag", "-l", "--format=%(contents)", checkpoint_id])
            .current_dir(&self.workspace_root)
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout).ok()
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "{}".to_string());
        
        Ok(Self::deserialize_metadata(checkpoint_id, commit_hash, &tag_message))
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

    /// Gets the diff between two checkpoints.
    ///
    /// # Arguments
    /// * `from_id` - Source checkpoint ID
    /// * `to_id` - Target checkpoint ID
    ///
    /// # Returns
    /// Structured diff information
    ///
    /// # Errors
    /// Returns error if diff fails or checkpoints not found
    pub fn diff_checkpoints(&self, from_id: &str, to_id: &str) -> Result<CheckpointDiff> {
        let from_checkpoint = self.get_checkpoint(from_id)?;
        let to_checkpoint = self.get_checkpoint(to_id)?;

        // Get raw diff output
        let output = Command::new("git")
            .args(["diff", "--name-status", &from_checkpoint.commit_hash, &to_checkpoint.commit_hash])
            .current_dir(&self.workspace_root)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CheckpointError::GitCommandFailed(stderr.to_string()));
        }

        let name_status = String::from_utf8(output.stdout)?;

        // Get full diff for statistics
        let full_output = Command::new("git")
            .args(["diff", "--numstat", &from_checkpoint.commit_hash, &to_checkpoint.commit_hash])
            .current_dir(&self.workspace_root)
            .output()?;

        let mut diff = CheckpointDiff::new();

        // Parse name-status output (format: STATUS\tFILE)
        for line in name_status.lines() {
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 2 {
                continue;
            }

            let status = parts[0];
            let file = parts[1..].join("\t"); // Handle filenames with tabs

            match status {
                "A" | "A+" => diff.added.push(file),
                "M" | "M+" => diff.modified.push(file),
                "D" => diff.deleted.push(file),
                "R" | "R+" => {
                    // Renamed file (format: R100\told\tnew)
                    if parts.len() >= 3 {
                        diff.deleted.push(parts[1].to_string());
                        diff.added.push(parts[2].to_string());
                    }
                }
                "C" => {
                    // Copied file
                    if parts.len() >= 3 {
                        diff.added.push(parts[2].to_string());
                    }
                }
                _ => {
                    // Unknown status, treat as modified
                    diff.modified.push(file);
                }
            }
        }

        // Parse numstat for insertions/deletions
        if full_output.status.success() {
            let numstat = String::from_utf8(full_output.stdout)?;
            for line in numstat.lines() {
                if line.is_empty() {
                    continue;
                }

                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(insertions) = parts[0].parse::<usize>() {
                        diff.insertions += insertions;
                    }
                    if let Ok(deletions) = parts[1].parse::<usize>() {
                        diff.deletions += deletions;
                    }
                }
            }
        }

        // Get raw diff output for full context
        let raw_output = Command::new("git")
            .args(["diff", &from_checkpoint.commit_hash, &to_checkpoint.commit_hash])
            .current_dir(&self.workspace_root)
            .output()?;

        if raw_output.status.success() {
            diff.raw_diff = String::from_utf8(raw_output.stdout)?;
        }

        Ok(diff)
    }

    /// Cleans up old checkpoints, keeping only the most recent N checkpoints.
    ///
    /// # Arguments
    /// * `keep_count` - Number of most recent checkpoints to keep
    ///
    /// # Returns
    /// Number of checkpoints deleted
    ///
    /// # Errors
    /// Returns error if cleanup fails
    pub fn cleanup_old_checkpoints(&self, keep_count: usize) -> Result<usize> {
        let all_checkpoints = self.list_checkpoints()?;

        if all_checkpoints.len() <= keep_count {
            return Ok(0);
        }

        // Sort by timestamp (newest first), then take the ones to delete (oldest)
        let mut sorted = all_checkpoints;
        sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        let to_delete = sorted.split_off(keep_count);
        let deleted_count = to_delete.len();

        // Delete old checkpoints
        for checkpoint in &to_delete {
            if let Err(e) = self.delete_checkpoint(&checkpoint.id) {
                // Log error but continue with other deletions
                eprintln!("Warning: Failed to delete checkpoint {}: {}", checkpoint.id, e);
            }
        }

        // Run Git garbage collection to reclaim disk space
        if deleted_count > 0 {
            let _ = Command::new("git")
                .args(["gc", "--prune=now"])
                .current_dir(&self.workspace_root)
                .output();
        }

        Ok(deleted_count)
    }

    /// Gets the size of the shadow repository in bytes.
    ///
    /// # Returns
    /// Size in bytes, or 0 if size cannot be determined
    pub fn get_shadow_repo_size(&self) -> u64 {
        if !self.shadow_repo.exists() {
            return 0;
        }

        // Calculate directory size recursively
        Self::calculate_directory_size(&self.shadow_repo).unwrap_or(0)
    }

    /// Calculates the total size of a directory recursively.
    fn calculate_directory_size(path: &Path) -> std::io::Result<u64> {
        let mut total_size = 0u64;

        if path.is_file() {
            return Ok(path.metadata()?.len());
        }

        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let entry_path = entry.path();
                total_size += Self::calculate_directory_size(&entry_path)?;
            }
        }

        Ok(total_size)
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

        let checkpoint = manager.create_checkpoint(Some("Test checkpoint".to_string())).unwrap();

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
        let checkpoint =
            manager.create_checkpoint(Some("Before modification".to_string())).unwrap();

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

    #[test]
    fn test_cleanup_old_checkpoints() {
        let temp_dir = setup_git_repo();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        // Create 10 checkpoints
        for i in 0..10 {
            manager
                .create_checkpoint(Some(format!("Checkpoint {}", i)))
                .unwrap();
        }

        // Verify all exist
        let checkpoints = manager.list_checkpoints().unwrap();
        assert_eq!(checkpoints.len(), 10);

        // Cleanup, keeping only 3 most recent
        let deleted = manager.cleanup_old_checkpoints(3).unwrap();
        assert_eq!(deleted, 7);

        // Verify only 3 remain
        let remaining = manager.list_checkpoints().unwrap();
        assert_eq!(remaining.len(), 3);
    }

    #[test]
    fn test_cleanup_no_checkpoints() {
        let temp_dir = setup_git_repo();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        // Cleanup when no checkpoints exist
        let deleted = manager.cleanup_old_checkpoints(5).unwrap();
        assert_eq!(deleted, 0);
    }

    #[test]
    fn test_cleanup_keep_all() {
        let temp_dir = setup_git_repo();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        // Create 5 checkpoints
        for i in 0..5 {
            manager
                .create_checkpoint(Some(format!("Checkpoint {}", i)))
                .unwrap();
        }

        // Cleanup keeping all (keep_count >= total)
        let deleted = manager.cleanup_old_checkpoints(10).unwrap();
        assert_eq!(deleted, 0);

        // Verify all still exist
        let checkpoints = manager.list_checkpoints().unwrap();
        assert_eq!(checkpoints.len(), 5);
    }

    #[test]
    fn test_get_shadow_repo_size() {
        let temp_dir = setup_git_repo();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        // Size should be 0 before initialization
        let size_before = manager.get_shadow_repo_size();
        assert_eq!(size_before, 0);

        // Initialize shadow repo
        manager.initialize_shadow_repo().unwrap();

        // Size should be > 0 after initialization
        let size_after = manager.get_shadow_repo_size();
        assert!(size_after > 0);

        // Create checkpoints and verify size increases
        manager.create_checkpoint(Some("Test checkpoint".to_string())).unwrap();
        let size_with_checkpoint = manager.get_shadow_repo_size();
        assert!(size_with_checkpoint >= size_after);
    }

    #[test]
    fn test_diff_checkpoints() {
        let temp_dir = setup_git_repo();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        // Create first checkpoint
        let cp1 = manager.create_checkpoint(Some("Checkpoint 1".to_string())).unwrap();

        // Create a file
        let file_path = temp_dir.path().join("test_file.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "initial content").unwrap();
        drop(file);

        Command::new("git").args(["add", "test_file.txt"]).current_dir(temp_dir.path()).output().unwrap();
        Command::new("git")
            .args(["commit", "-m", "Add test file"])
            .current_dir(temp_dir.path())
            .output()
            .unwrap();

        // Create second checkpoint
        let cp2 = manager.create_checkpoint(Some("Checkpoint 2".to_string())).unwrap();

        // Modify file
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "modified content").unwrap();
        drop(file);

        Command::new("git").args(["add", "test_file.txt"]).current_dir(temp_dir.path()).output().unwrap();
        Command::new("git")
            .args(["commit", "-m", "Modify test file"])
            .current_dir(temp_dir.path())
            .output()
            .unwrap();

        // Create third checkpoint
        let cp3 = manager.create_checkpoint(Some("Checkpoint 3".to_string())).unwrap();

        // Diff between cp1 and cp2
        let diff = manager.diff_checkpoints(&cp1.id, &cp2.id).unwrap();
        assert!(diff.files_changed() > 0);

        // Diff between cp2 and cp3
        let diff2 = manager.diff_checkpoints(&cp2.id, &cp3.id).unwrap();
        assert!(diff2.files_changed() > 0);
    }

    #[test]
    fn test_diff_checkpoints_no_changes() {
        let temp_dir = setup_git_repo();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        // Create two checkpoints without changes
        let cp1 = manager.create_checkpoint(Some("Checkpoint 1".to_string())).unwrap();
        let cp2 = manager.create_checkpoint(Some("Checkpoint 2".to_string())).unwrap();

        // Diff should show no changes (same commit)
        let diff = manager.diff_checkpoints(&cp1.id, &cp2.id).unwrap();
        // Note: If checkpoints point to the same commit, there may be no changes
        // This test verifies the method doesn't panic
        let _ = diff.files_changed(); // Just verify it doesn't panic
    }

    #[test]
    fn test_diff_checkpoints_not_found() {
        let temp_dir = setup_git_repo();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        let result = manager.diff_checkpoints("nonexistent-1", "nonexistent-2");
        assert!(result.is_err());
    }
}
