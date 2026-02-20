//! Batch processing module for parallel execution of async operations.
//!
//! This module provides core batch processing functionality that is shared
//! across radium-core and radium-orchestrator.

pub mod error;
pub mod processor;
pub mod types;

pub use error::BatchError;
pub use processor::BatchProcessor;
pub use types::{BatchResult, RetryPolicy};
