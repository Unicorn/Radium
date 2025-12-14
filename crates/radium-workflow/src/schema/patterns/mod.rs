//! Workflow Composition Patterns
//!
//! Common workflow patterns for orchestration:
//! - Saga: Distributed transactions with compensation
//! - Scatter-Gather: Parallel execution with result collection
//! - Pipeline: Sequential data transformation
//! - Map-Reduce: Parallel processing with aggregation

pub mod map_reduce;
pub mod pipeline;
pub mod saga;
pub mod scatter_gather;

pub use map_reduce::*;
pub use pipeline::*;
pub use saga::*;
pub use scatter_gather::*;

/// Common trait for workflow patterns
pub trait WorkflowPattern {
    /// Get the pattern name
    fn pattern_name(&self) -> &'static str;

    /// Validate the pattern configuration
    fn validate_pattern(&self) -> Result<(), Vec<String>>;

    /// Generate TypeScript code for this pattern
    fn to_typescript(&self) -> String;
}
