//! Source validation utilities for CLI commands.

pub mod display;
pub mod extract;
pub mod helper;
pub mod prompt;

// Re-export types for convenience
pub use radium_core::context::SourceValidationResult;
pub use helper::validate_sources;
pub use prompt::validate_and_prompt;
