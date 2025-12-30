//! Batch processing module for parallel execution of async operations.
//!
//! Core batch processing (processor, types, error) is provided by radium-abstraction.
//! This module extends it with UI-specific functionality (formats, input parsing, progress, UI rendering).

// Re-export core batch types from radium-abstraction
pub use radium_abstraction::batch::{BatchError, BatchProcessor, BatchResult, RetryPolicy};

// UI-specific modules remain in radium-core
pub mod formats;
pub mod input;
pub mod progress;
pub mod ui;

pub use formats::{detect_format, InputFormat};
pub use input::{parse_input_file, BatchInput};
pub use progress::BatchProgressTracker;
pub use ui::{render_progress, render_summary};

