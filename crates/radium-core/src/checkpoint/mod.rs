//! Checkpointing system for agent work snapshots.
//!
//! This module provides git-based checkpointing for preserving agent work state.
//!
//! # Example
//!
//! ```rust,no_run
//! use radium_core::checkpoint::{CheckpointManager, Checkpoint};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let manager = CheckpointManager::new(Path::new("/workspace"))?;
//!
//! // Create a checkpoint
//! let checkpoint = manager.create_checkpoint(Some("Before refactoring".to_string()))?;
//!
//! // Do some work...
//!
//! // Restore if needed
//! manager.restore_checkpoint(&checkpoint.id)?;
//! # Ok(())
//! # }
//! ```

mod error;
mod snapshot;

pub use error::{CheckpointError, Result};
pub use snapshot::{Checkpoint, CheckpointDiff, CheckpointManager};
