//! Plan-scoped memory management for agent outputs.
//!
//! This module provides memory storage for agent execution outputs, scoped to
//! individual plans. Each plan has its own memory directory where agent outputs
//! are stored and can be retrieved for context in subsequent executions.
//!
//! # Example
//!
//! ```rust,no_run
//! use radium_core::memory::{MemoryStore, MemoryEntry};
//! use radium_core::workspace::RequirementId;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let workspace_root = Path::new("/path/to/workspace");
//! let req_id = RequirementId::new(1);
//!
//! // Create memory store for a plan
//! let mut store = MemoryStore::new(workspace_root, req_id)?;
//!
//! // Store agent output
//! let entry = MemoryEntry::new(
//!     "architect".to_string(),
//!     "System design complete...".to_string()
//! );
//! store.store(entry)?;
//!
//! // Retrieve agent output
//! let entry = store.get("architect")?;
//! println!("Last output: {}", entry.output);
//! # Ok(())
//! # }
//! ```

mod adapter;
mod error;
mod store;

pub use adapter::{FileAdapter, MemoryAdapter};
pub use error::{MemoryError, Result};
pub use store::{MemoryEntry, MemoryStore};
