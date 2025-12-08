//! Persistent storage for code blocks.

use super::{CodeBlock, CodeBlockError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Selector for choosing which code blocks to retrieve.
#[derive(Debug, Clone)]
pub enum BlockSelector {
    /// Single block by index.
    Single(usize),
    /// Multiple blocks by indices.
    Multiple(Vec<usize>),
    /// Range of blocks (inclusive).
    Range(usize, usize),
}

/// Storage for code blocks organized by session.
pub struct CodeBlockStore {
    /// Workspace root directory.
    workspace_root: PathBuf,
    /// Session ID for this store.
    session_id: String,
    /// Storage directory path.
    storage_dir: PathBuf,
}

/// Internal storage format for blocks.
#[derive(Debug, Serialize, Deserialize)]
struct StoredBlock {
    /// Block index.
    index: usize,
    /// Language identifier.
    language: Option<String>,
    /// Block content.
    content: String,
    /// File path hint.
    file_hint: Option<PathBuf>,
    /// Start line in original text.
    start_line: usize,
    /// Agent ID that produced this block.
    agent_id: String,
}

/// Storage file format.
#[derive(Debug, Serialize, Deserialize)]
struct BlocksFile {
    /// All blocks for this session.
    blocks: Vec<StoredBlock>,
}

impl CodeBlockStore {
    /// Creates a new code block store for a session.
    ///
    /// # Arguments
    /// * `workspace_root` - The workspace root directory
    /// * `session_id` - The session identifier
    ///
    /// # Returns
    /// A new store instance with directory structure created.
    ///
    /// # Errors
    /// Returns error if directory creation fails.
    pub fn new(workspace_root: &Path, session_id: String) -> Result<Self> {
        let storage_dir = workspace_root
            .join(".radium")
            .join("_internals")
            .join("code-blocks")
            .join(&session_id);

        // Create directory structure
        fs::create_dir_all(&storage_dir).map_err(|e| {
            CodeBlockError::StorageCreation(format!(
                "Failed to create storage directory: {}",
                e
            ))
        })?;

        Ok(Self {
            workspace_root: workspace_root.to_path_buf(),
            session_id,
            storage_dir,
        })
    }

    /// Stores code blocks for an agent.
    ///
    /// # Arguments
    /// * `agent_id` - The agent that produced these blocks
    /// * `blocks` - The code blocks to store
    ///
    /// # Errors
    /// Returns error if storage operations fail.
    pub fn store_blocks(&mut self, agent_id: &str, blocks: Vec<CodeBlock>) -> Result<()> {
        if blocks.is_empty() {
            return Ok(());
        }

        // Load existing blocks
        let mut all_blocks = self.load_blocks_internal()?;

        // Convert new blocks to stored format and add agent_id
        let stored_blocks: Vec<StoredBlock> = blocks
            .into_iter()
            .map(|block| StoredBlock {
                index: block.index,
                language: block.language,
                content: block.content,
                file_hint: block.file_hint,
                start_line: block.start_line,
                agent_id: agent_id.to_string(),
            })
            .collect();

        // Append new blocks
        all_blocks.extend(stored_blocks);

        // Save to file
        self.save_blocks_internal(&all_blocks)?;

        Ok(())
    }

    /// Lists all code blocks, optionally filtered by agent.
    ///
    /// # Arguments
    /// * `agent_id` - Optional agent ID to filter by
    ///
    /// # Returns
    /// Vector of all matching code blocks.
    ///
    /// # Errors
    /// Returns error if storage operations fail.
    pub fn list_blocks(&self, agent_id: Option<&str>) -> Result<Vec<CodeBlock>> {
        let stored_blocks = self.load_blocks_internal()?;

        let blocks: Vec<CodeBlock> = stored_blocks
            .into_iter()
            .filter(|block| {
                agent_id
                    .map(|id| block.agent_id == id)
                    .unwrap_or(true)
            })
            .map(|stored| CodeBlock {
                index: stored.index,
                language: stored.language,
                content: stored.content,
                file_hint: stored.file_hint,
                start_line: stored.start_line,
            })
            .collect();

        Ok(blocks)
    }

    /// Retrieves a single code block by index.
    ///
    /// # Arguments
    /// * `index` - The block index (1-based)
    ///
    /// # Returns
    /// The code block if found.
    ///
    /// # Errors
    /// Returns error if block not found or storage operations fail.
    pub fn get_block(&self, index: usize) -> Result<CodeBlock> {
        let blocks = self.list_blocks(None)?;
        blocks
            .into_iter()
            .find(|block| block.index == index)
            .ok_or_else(|| CodeBlockError::NotFound(index))
    }

    /// Retrieves multiple code blocks using a selector.
    ///
    /// # Arguments
    /// * `selector` - The block selector
    ///
    /// # Returns
    /// Vector of matching code blocks.
    ///
    /// # Errors
    /// Returns error if any block not found or storage operations fail.
    pub fn get_blocks(&self, selector: BlockSelector) -> Result<Vec<CodeBlock>> {
        let all_blocks = self.list_blocks(None)?;

        match selector {
            BlockSelector::Single(index) => {
                let block = self.get_block(index)?;
                Ok(vec![block])
            }
            BlockSelector::Multiple(indices) => {
                let mut result = Vec::new();
                for index in indices {
                    let block = all_blocks
                        .iter()
                        .find(|b| b.index == index)
                        .ok_or_else(|| CodeBlockError::NotFound(index))?;
                    result.push(block.clone());
                }
                Ok(result)
            }
            BlockSelector::Range(start, end) => {
                if start > end {
                    return Err(CodeBlockError::InvalidIndex(format!(
                        "Range start {} > end {}",
                        start, end
                    )));
                }
                let mut result = Vec::new();
                for index in start..=end {
                    let block = all_blocks
                        .iter()
                        .find(|b| b.index == index)
                        .ok_or_else(|| CodeBlockError::NotFound(index))?;
                    result.push(block.clone());
                }
                Ok(result)
            }
        }
    }

    /// Loads blocks from storage file.
    fn load_blocks_internal(&self) -> Result<Vec<StoredBlock>> {
        let blocks_file = self.storage_dir.join("blocks.json");

        if !blocks_file.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&blocks_file)?;
        let blocks_file: BlocksFile = serde_json::from_str(&content)?;

        Ok(blocks_file.blocks)
    }

    /// Saves blocks to storage file.
    fn save_blocks_internal(&self, blocks: &[StoredBlock]) -> Result<()> {
        let blocks_file = self.storage_dir.join("blocks.json");
        let blocks_data = BlocksFile {
            blocks: blocks.to_vec(),
        };

        let json = serde_json::to_string_pretty(&blocks_data)?;
        fs::write(&blocks_file, json)?;

        // Also create index.json for quick lookup
        let index: HashMap<usize, usize> = blocks
            .iter()
            .enumerate()
            .map(|(i, block)| (block.index, i))
            .collect();
        let index_file = self.storage_dir.join("index.json");
        let index_json = serde_json::to_string_pretty(&index)?;
        fs::write(&index_file, index_json)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_store_and_retrieve_blocks() {
        let temp_dir = TempDir::new().unwrap();
        let store = CodeBlockStore::new(temp_dir.path(), "test-session".to_string()).unwrap();

        let blocks = vec![
            CodeBlock {
                index: 1,
                language: Some("rust".to_string()),
                content: "fn main() {}".to_string(),
                file_hint: None,
                start_line: 1,
            },
            CodeBlock {
                index: 2,
                language: Some("python".to_string()),
                content: "print('hello')".to_string(),
                file_hint: None,
                start_line: 5,
            },
        ];

        let mut store = store;
        store.store_blocks("agent-1", blocks.clone()).unwrap();

        let retrieved = store.get_block(1).unwrap();
        assert_eq!(retrieved.index, 1);
        assert_eq!(retrieved.language, Some("rust".to_string()));
        assert_eq!(retrieved.content, "fn main() {}");
    }

    #[test]
    fn test_range_selection() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = CodeBlockStore::new(temp_dir.path(), "test-session".to_string()).unwrap();

        // Store 10 blocks
        let blocks: Vec<CodeBlock> = (1..=10)
            .map(|i| CodeBlock {
                index: i,
                language: Some("rust".to_string()),
                content: format!("fn block_{}() {{}}", i),
                file_hint: None,
                start_line: i * 2,
            })
            .collect();

        store.store_blocks("agent-1", blocks).unwrap();

        // Get range 2..5
        let selected = store
            .get_blocks(BlockSelector::Range(2, 5))
            .unwrap();
        assert_eq!(selected.len(), 4);
        assert_eq!(selected[0].index, 2);
        assert_eq!(selected[3].index, 5);
    }

    #[test]
    fn test_multiple_selection() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = CodeBlockStore::new(temp_dir.path(), "test-session".to_string()).unwrap();

        let blocks: Vec<CodeBlock> = (1..=5)
            .map(|i| CodeBlock {
                index: i,
                language: Some("rust".to_string()),
                content: format!("fn block_{}() {{}}", i),
                file_hint: None,
                start_line: i,
            })
            .collect();

        store.store_blocks("agent-1", blocks).unwrap();

        let selected = store
            .get_blocks(BlockSelector::Multiple(vec![1, 3, 5]))
            .unwrap();
        assert_eq!(selected.len(), 3);
        assert_eq!(selected[0].index, 1);
        assert_eq!(selected[1].index, 3);
        assert_eq!(selected[2].index, 5);
    }

    #[test]
    fn test_list_blocks_with_agent_filter() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = CodeBlockStore::new(temp_dir.path(), "test-session".to_string()).unwrap();

        let blocks1 = vec![CodeBlock {
            index: 1,
            language: Some("rust".to_string()),
            content: "fn main() {}".to_string(),
            file_hint: None,
            start_line: 1,
        }];

        let blocks2 = vec![CodeBlock {
            index: 2,
            language: Some("python".to_string()),
            content: "print('hi')".to_string(),
            file_hint: None,
            start_line: 5,
        }];

        store.store_blocks("agent-1", blocks1).unwrap();
        store.store_blocks("agent-2", blocks2).unwrap();

        let agent1_blocks = store.list_blocks(Some("agent-1")).unwrap();
        assert_eq!(agent1_blocks.len(), 1);
        assert_eq!(agent1_blocks[0].index, 1);

        let all_blocks = store.list_blocks(None).unwrap();
        assert_eq!(all_blocks.len(), 2);
    }

    #[test]
    fn test_not_found_error() {
        let temp_dir = TempDir::new().unwrap();
        let store = CodeBlockStore::new(temp_dir.path(), "test-session".to_string()).unwrap();

        let result = store.get_block(999);
        assert!(matches!(result, Err(CodeBlockError::NotFound(999))));
    }
}

