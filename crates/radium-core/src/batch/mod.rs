//! Batch processing module for parallel execution of async operations.

pub mod error;
pub mod formats;
pub mod input;
pub mod processor;
pub mod progress;
pub mod types;
pub mod ui;

pub use error::BatchError;
pub use formats::{detect_format, InputFormat};
pub use input::{parse_input_file, BatchInput};
pub use processor::BatchProcessor;
pub use progress::BatchProgressTracker;
pub use types::{BatchResult, RetryPolicy};
pub use ui::{render_progress, render_summary};

