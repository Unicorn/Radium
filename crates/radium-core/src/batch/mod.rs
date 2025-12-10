//! Batch processing module for parallel execution of async operations.

pub mod error;
pub mod formats;
pub mod input;
pub mod processor;
pub mod types;

pub use error::BatchError;
pub use formats::{detect_format, InputFormat};
pub use input::{parse_input_file, BatchInput};
pub use processor::BatchProcessor;
pub use types::{BatchResult, RetryPolicy};

