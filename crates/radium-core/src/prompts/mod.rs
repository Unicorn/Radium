//! Prompt template system.
//!
//! Provides functionality for loading, processing, and managing prompt templates
//! for agents.
//!
//! # Example
//!
//! ```rust,no_run
//! use radium_core::prompts::{PromptTemplate, PromptContext};
//! use std::path::Path;
//!
//! # fn main() -> anyhow::Result<()> {
//! let template = PromptTemplate::load(Path::new("prompts/test.md"))?;
//! let mut context = PromptContext::new();
//! context.set("name", "World");
//! let prompt = template.render(&context)?;
//! # Ok(())
//! # }
//! ```

pub mod cache;
pub mod processing;
pub mod templates;

pub use cache::{CacheStats, PromptCache};
pub use processing::{process_with_file_injection, validate_prompt, ProcessingError};
pub use templates::{PromptContext, PromptError, PromptTemplate};
