//! Plan-scoped memory store for agent outputs.

use super::error::{MemoryError, Result};
use crate::workspace::RequirementId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Maximum characters to store from agent output.
const MAX_OUTPUT_CHARS: usize = 2000;

/// Memory entry for an agent's output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Agent ID that produced this output.
    pub agent_id: String,

    /// Timestamp when this entry was created.
    pub timestamp: SystemTime,

    /// Agent output (last 2000 characters).
    pub output: String,

    /// Optional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl MemoryEntry {
    /// Creates a new memory entry.
    ///
    /// # Arguments
    /// * `agent_id` - The agent identifier
    /// * `output` - The agent's output (will be truncated to last 2000 chars)
    pub fn new(agent_id: String, output: String) -> Self {
        let truncated_output = if output.len() > MAX_OUTPUT_CHARS {
            // Take last 2000 characters
            output.chars().rev().take(MAX_OUTPUT_CHARS).collect::<String>().chars().rev().collect()
        } else {
            output
        };

        Self {
            agent_id,
            timestamp: SystemTime::now(),
            output: truncated_output,
            metadata: HashMap::new(),
        }
    }

    /// Adds metadata to the entry.
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Plan-scoped memory store.
///
/// Stores agent outputs in a plan-specific directory structure:
/// `<workspace_root>/.radium/plan/<REQ-XXX>/memory/`
pub struct MemoryStore {
    /// Root path for this plan's memory.
    memory_root: PathBuf,

    /// Requirement ID for this plan.
    requirement_id: RequirementId,

    /// In-memory cache of entries.
    cache: HashMap<String, MemoryEntry>,
}

impl MemoryStore {
    /// Creates a new memory store for a plan.
    ///
    /// # Arguments
    /// * `workspace_root` - The workspace root directory
    /// * `requirement_id` - The plan's requirement ID
    ///
    /// # Returns
    /// A new memory store instance
    ///
    /// # Errors
    /// Returns error if directory creation fails
    pub fn new(workspace_root: &Path, requirement_id: RequirementId) -> Result<Self> {
        let memory_root = workspace_root
            .join(".radium")
            .join("plan")
            .join(requirement_id.to_string())
            .join("memory");

        // Create memory directory if it doesn't exist
        fs::create_dir_all(&memory_root)?;

        Ok(Self { memory_root, requirement_id, cache: HashMap::new() })
    }

    /// Opens an existing memory store.
    ///
    /// # Arguments
    /// * `workspace_root` - The workspace root directory
    /// * `requirement_id` - The plan's requirement ID
    ///
    /// # Returns
    /// A memory store instance with cached entries loaded
    ///
    /// # Errors
    /// Returns error if directory doesn't exist or loading fails
    pub fn open(workspace_root: &Path, requirement_id: RequirementId) -> Result<Self> {
        let memory_root = workspace_root
            .join(".radium")
            .join("plan")
            .join(requirement_id.to_string())
            .join("memory");

        if !memory_root.exists() {
            return Err(MemoryError::NotInitialized(requirement_id.to_string()));
        }

        let mut store = Self { memory_root, requirement_id, cache: HashMap::new() };

        // Load all existing entries into cache
        store.load_all()?;

        Ok(store)
    }

    /// Stores an agent's output.
    ///
    /// # Arguments
    /// * `entry` - The memory entry to store
    ///
    /// # Errors
    /// Returns error if writing fails
    pub fn store(&mut self, entry: MemoryEntry) -> Result<()> {
        let agent_id = entry.agent_id.clone();

        // Write to file
        let file_path = self.memory_root.join(format!("{}.json", agent_id));
        let json = serde_json::to_string_pretty(&entry)?;
        fs::write(&file_path, json)?;

        // Update cache
        self.cache.insert(agent_id, entry);

        Ok(())
    }

    /// Retrieves an agent's last output.
    ///
    /// # Arguments
    /// * `agent_id` - The agent identifier
    ///
    /// # Returns
    /// The memory entry if found
    ///
    /// # Errors
    /// Returns error if entry doesn't exist
    pub fn get(&self, agent_id: &str) -> Result<&MemoryEntry> {
        self.cache.get(agent_id).ok_or_else(|| MemoryError::NotFound(agent_id.to_string()))
    }

    /// Retrieves an agent's last output (mutable).
    ///
    /// # Arguments
    /// * `agent_id` - The agent identifier
    ///
    /// # Returns
    /// The mutable memory entry if found
    ///
    /// # Errors
    /// Returns error if entry doesn't exist
    pub fn get_mut(&mut self, agent_id: &str) -> Result<&mut MemoryEntry> {
        self.cache.get_mut(agent_id).ok_or_else(|| MemoryError::NotFound(agent_id.to_string()))
    }

    /// Lists all agent IDs with stored memory.
    pub fn list_agents(&self) -> Vec<String> {
        self.cache.keys().cloned().collect()
    }

    /// Returns all memory entries.
    pub fn all_entries(&self) -> &HashMap<String, MemoryEntry> {
        &self.cache
    }

    /// Clears all memory for this plan.
    ///
    /// # Errors
    /// Returns error if deletion fails
    pub fn clear(&mut self) -> Result<()> {
        // Remove all files
        for entry in fs::read_dir(&self.memory_root)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                fs::remove_file(entry.path())?;
            }
        }

        // Clear cache
        self.cache.clear();

        Ok(())
    }

    /// Loads all entries from disk into cache.
    fn load_all(&mut self) -> Result<()> {
        if !self.memory_root.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.memory_root)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                let memory_entry: MemoryEntry = serde_json::from_str(&content)?;
                self.cache.insert(memory_entry.agent_id.clone(), memory_entry);
            }
        }

        Ok(())
    }

    /// Returns the memory root directory path.
    pub fn memory_root(&self) -> &Path {
        &self.memory_root
    }

    /// Returns the requirement ID for this store.
    pub fn requirement_id(&self) -> &RequirementId {
        &self.requirement_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_memory_entry_new() {
        let entry = MemoryEntry::new("test-agent".to_string(), "output".to_string());
        assert_eq!(entry.agent_id, "test-agent");
        assert_eq!(entry.output, "output");
        assert!(entry.metadata.is_empty());
    }

    #[test]
    fn test_memory_entry_truncation() {
        let long_output = "x".repeat(3000);
        let entry = MemoryEntry::new("test-agent".to_string(), long_output.clone());
        assert_eq!(entry.output.len(), 2000);
        // Should contain last 2000 characters
        assert_eq!(entry.output, long_output.chars().skip(1000).collect::<String>());
    }

    #[test]
    fn test_memory_entry_with_metadata() {
        let entry = MemoryEntry::new("test-agent".to_string(), "output".to_string())
            .with_metadata("key1".to_string(), "value1".to_string())
            .with_metadata("key2".to_string(), "value2".to_string());

        assert_eq!(entry.metadata.len(), 2);
        assert_eq!(entry.metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(entry.metadata.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_memory_store_new() {
        let temp_dir = TempDir::new().unwrap();
        let req_id = RequirementId::new(1);

        let store = MemoryStore::new(temp_dir.path(), req_id).unwrap();
        assert!(store.memory_root().exists());
        assert_eq!(store.requirement_id(), &req_id);
    }

    #[test]
    fn test_memory_store_store_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let req_id = RequirementId::new(1);

        let mut store = MemoryStore::new(temp_dir.path(), req_id).unwrap();

        let entry = MemoryEntry::new("agent-1".to_string(), "test output".to_string());
        store.store(entry.clone()).unwrap();

        let retrieved = store.get("agent-1").unwrap();
        assert_eq!(retrieved.agent_id, "agent-1");
        assert_eq!(retrieved.output, "test output");
    }

    #[test]
    fn test_memory_store_get_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let req_id = RequirementId::new(1);

        let store = MemoryStore::new(temp_dir.path(), req_id).unwrap();
        let result = store.get("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_store_list_agents() {
        let temp_dir = TempDir::new().unwrap();
        let req_id = RequirementId::new(1);

        let mut store = MemoryStore::new(temp_dir.path(), req_id).unwrap();

        store.store(MemoryEntry::new("agent-1".to_string(), "output1".to_string())).unwrap();
        store.store(MemoryEntry::new("agent-2".to_string(), "output2".to_string())).unwrap();

        let agents = store.list_agents();
        assert_eq!(agents.len(), 2);
        assert!(agents.contains(&"agent-1".to_string()));
        assert!(agents.contains(&"agent-2".to_string()));
    }

    #[test]
    fn test_memory_store_clear() {
        let temp_dir = TempDir::new().unwrap();
        let req_id = RequirementId::new(1);

        let mut store = MemoryStore::new(temp_dir.path(), req_id).unwrap();

        store.store(MemoryEntry::new("agent-1".to_string(), "output1".to_string())).unwrap();
        store.store(MemoryEntry::new("agent-2".to_string(), "output2".to_string())).unwrap();

        assert_eq!(store.list_agents().len(), 2);

        store.clear().unwrap();
        assert_eq!(store.list_agents().len(), 0);
    }

    #[test]
    fn test_memory_store_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let req_id = RequirementId::new(1);

        // Create store and save entry
        {
            let mut store = MemoryStore::new(temp_dir.path(), req_id).unwrap();
            store.store(MemoryEntry::new("agent-1".to_string(), "test output".to_string())).unwrap();
        }

        // Open store and verify entry persisted
        {
            let store = MemoryStore::open(temp_dir.path(), req_id).unwrap();
            let entry = store.get("agent-1").unwrap();
            assert_eq!(entry.output, "test output");
        }
    }

    #[test]
    fn test_memory_store_open_not_initialized() {
        let temp_dir = TempDir::new().unwrap();
        let req_id = RequirementId::new(1);

        let result = MemoryStore::open(temp_dir.path(), req_id);
        assert!(result.is_err());
    }
}
