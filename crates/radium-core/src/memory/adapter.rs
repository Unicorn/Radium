//! Storage adapters for memory entries.
//!
//! Provides abstraction over different storage backends (file system, database, etc.).

use super::error::Result;
use super::store::MemoryEntry;
use async_trait::async_trait;
use std::path::Path;

/// Trait for memory storage adapters.
///
/// Adapters implement different storage backends for memory entries.
#[async_trait]
pub trait MemoryAdapter: Send + Sync {
    /// Writes a memory entry to storage.
    ///
    /// # Arguments
    /// * `agent_id` - The agent identifier
    /// * `entry` - The memory entry to store
    ///
    /// # Errors
    /// Returns error if write fails
    async fn write(&self, agent_id: &str, entry: &MemoryEntry) -> Result<()>;

    /// Reads a memory entry from storage.
    ///
    /// # Arguments
    /// * `agent_id` - The agent identifier
    ///
    /// # Returns
    /// The memory entry if found
    ///
    /// # Errors
    /// Returns error if read fails or entry not found
    async fn read(&self, agent_id: &str) -> Result<MemoryEntry>;

    /// Lists all agent IDs with stored memory.
    ///
    /// # Errors
    /// Returns error if listing fails
    async fn list(&self) -> Result<Vec<String>>;

    /// Deletes a memory entry.
    ///
    /// # Arguments
    /// * `agent_id` - The agent identifier
    ///
    /// # Errors
    /// Returns error if deletion fails
    async fn delete(&self, agent_id: &str) -> Result<()>;

    /// Deletes all memory entries.
    ///
    /// # Errors
    /// Returns error if deletion fails
    async fn clear(&self) -> Result<()>;

    /// Appends content to an existing memory entry.
    ///
    /// # Arguments
    /// * `agent_id` - The agent identifier
    /// * `content` - The content to append
    ///
    /// # Errors
    /// Returns error if append fails or entry not found
    async fn append(&self, agent_id: &str, content: &str) -> Result<()>;
}

/// File system-based memory adapter.
///
/// Stores memory entries as JSON files in a directory.
pub struct FileAdapter {
    /// Root directory for memory storage.
    root_path: std::path::PathBuf,
}

impl FileAdapter {
    /// Creates a new file adapter.
    ///
    /// # Arguments
    /// * `root_path` - The root directory for storage
    ///
    /// # Returns
    /// A new file adapter instance
    ///
    /// # Errors
    /// Returns error if directory creation fails
    pub fn new(root_path: impl AsRef<Path>) -> Result<Self> {
        let root_path = root_path.as_ref().to_path_buf();
        std::fs::create_dir_all(&root_path)?;
        Ok(Self { root_path })
    }

    /// Returns the file path for an agent's memory.
    fn agent_path(&self, agent_id: &str) -> std::path::PathBuf {
        self.root_path.join(format!("{}.json", agent_id))
    }
}

#[async_trait]
impl MemoryAdapter for FileAdapter {
    async fn write(&self, agent_id: &str, entry: &MemoryEntry) -> Result<()> {
        let path = self.agent_path(agent_id);
        let json = serde_json::to_string_pretty(entry)?;
        tokio::fs::write(path, json).await?;
        Ok(())
    }

    async fn read(&self, agent_id: &str) -> Result<MemoryEntry> {
        let path = self.agent_path(agent_id);
        let content = tokio::fs::read_to_string(path).await?;
        let entry = serde_json::from_str(&content)?;
        Ok(entry)
    }

    async fn list(&self) -> Result<Vec<String>> {
        let mut agents = Vec::new();
        let mut entries = tokio::fs::read_dir(&self.root_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                    agents.push(file_stem.to_string());
                }
            }
        }

        Ok(agents)
    }

    async fn delete(&self, agent_id: &str) -> Result<()> {
        let path = self.agent_path(agent_id);
        tokio::fs::remove_file(path).await?;
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let agents = self.list().await?;
        for agent_id in agents {
            self.delete(&agent_id).await?;
        }
        Ok(())
    }

    async fn append(&self, agent_id: &str, content: &str) -> Result<()> {
        let mut entry = self.read(agent_id).await?;
        entry.output.push_str(content);

        // Truncate to last 2000 characters if needed
        if entry.output.len() > 2000 {
            entry.output =
                entry.output.chars().rev().take(2000).collect::<String>().chars().rev().collect();
        }

        self.write(agent_id, &entry).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_adapter_new() {
        let temp_dir = TempDir::new().unwrap();
        let _adapter = FileAdapter::new(temp_dir.path()).unwrap();
        assert!(temp_dir.path().exists());
    }

    #[tokio::test]
    async fn test_file_adapter_write_and_read() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = FileAdapter::new(temp_dir.path()).unwrap();

        let entry = MemoryEntry::new("test-agent".to_string(), "test output".to_string());
        adapter.write("test-agent", &entry).await.unwrap();

        let read_entry = adapter.read("test-agent").await.unwrap();
        assert_eq!(read_entry.agent_id, "test-agent");
        assert_eq!(read_entry.output, "test output");
    }

    #[tokio::test]
    async fn test_file_adapter_list() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = FileAdapter::new(temp_dir.path()).unwrap();

        let entry1 = MemoryEntry::new("agent-1".to_string(), "output1".to_string());
        let entry2 = MemoryEntry::new("agent-2".to_string(), "output2".to_string());

        adapter.write("agent-1", &entry1).await.unwrap();
        adapter.write("agent-2", &entry2).await.unwrap();

        let agents = adapter.list().await.unwrap();
        assert_eq!(agents.len(), 2);
        assert!(agents.contains(&"agent-1".to_string()));
        assert!(agents.contains(&"agent-2".to_string()));
    }

    #[tokio::test]
    async fn test_file_adapter_delete() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = FileAdapter::new(temp_dir.path()).unwrap();

        let entry = MemoryEntry::new("test-agent".to_string(), "test output".to_string());
        adapter.write("test-agent", &entry).await.unwrap();

        adapter.delete("test-agent").await.unwrap();

        let result = adapter.read("test-agent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_file_adapter_clear() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = FileAdapter::new(temp_dir.path()).unwrap();

        let entry1 = MemoryEntry::new("agent-1".to_string(), "output1".to_string());
        let entry2 = MemoryEntry::new("agent-2".to_string(), "output2".to_string());

        adapter.write("agent-1", &entry1).await.unwrap();
        adapter.write("agent-2", &entry2).await.unwrap();

        adapter.clear().await.unwrap();

        let agents = adapter.list().await.unwrap();
        assert_eq!(agents.len(), 0);
    }

    #[tokio::test]
    async fn test_file_adapter_append() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = FileAdapter::new(temp_dir.path()).unwrap();

        let entry = MemoryEntry::new("test-agent".to_string(), "initial ".to_string());
        adapter.write("test-agent", &entry).await.unwrap();

        adapter.append("test-agent", "appended").await.unwrap();

        let read_entry = adapter.read("test-agent").await.unwrap();
        assert_eq!(read_entry.output, "initial appended");
    }

    #[tokio::test]
    async fn test_file_adapter_append_truncation() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = FileAdapter::new(temp_dir.path()).unwrap();

        let long_output = "x".repeat(1900);
        let entry = MemoryEntry::new("test-agent".to_string(), long_output.clone());
        adapter.write("test-agent", &entry).await.unwrap();

        // Append another 200 characters (total 2100)
        adapter.append("test-agent", &"y".repeat(200)).await.unwrap();

        let read_entry = adapter.read("test-agent").await.unwrap();
        assert_eq!(read_entry.output.len(), 2000);
        // Should end with 'y's
        assert!(read_entry.output.ends_with(&"y".repeat(200)));
    }

    #[tokio::test]
    async fn test_file_adapter_read_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = FileAdapter::new(temp_dir.path()).unwrap();

        let result = adapter.read("nonexistent-agent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_file_adapter_list_empty() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = FileAdapter::new(temp_dir.path()).unwrap();

        let agents = adapter.list().await.unwrap();
        assert_eq!(agents.len(), 0);
    }

    #[tokio::test]
    async fn test_file_adapter_delete_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = FileAdapter::new(temp_dir.path()).unwrap();

        // Delete should succeed even if file doesn't exist
        let result = adapter.delete("nonexistent-agent").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_file_adapter_append_creates_if_missing() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = FileAdapter::new(temp_dir.path()).unwrap();

        // Append to non-existent agent should create it
        adapter.append("new-agent", "content").await.unwrap();

        let entry = adapter.read("new-agent").await.unwrap();
        assert_eq!(entry.agent_id, "new-agent");
        assert_eq!(entry.output, "content");
    }

    #[tokio::test]
    async fn test_file_adapter_multiple_appends() {
        let temp_dir = TempDir::new().unwrap();
        let adapter = FileAdapter::new(temp_dir.path()).unwrap();

        adapter.append("agent", "part1").await.unwrap();
        adapter.append("agent", " part2").await.unwrap();
        adapter.append("agent", " part3").await.unwrap();

        let entry = adapter.read("agent").await.unwrap();
        assert_eq!(entry.output, "part1 part2 part3");
    }
}
