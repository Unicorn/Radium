//! Batch processing module for parallel execution of async operations.

pub mod error;
pub mod processor;
pub mod types;

pub use error::BatchError;
pub use processor::BatchProcessor;
pub use types::{BatchResult, RetryPolicy};

