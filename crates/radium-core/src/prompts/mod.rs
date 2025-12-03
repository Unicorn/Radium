//! Prompt template system.
//!
//! Provides functionality for loading, processing, and managing prompt templates
//! for agents.
//!
//! # Example
//!
//! ```rust,no_run
//! use radium_core::prompts::{PromptTemplate, PromptContext, PromptCache};
//! use std::path::Path;
//!
//! # fn main() -> anyhow::Result<()> {
//! let cache = PromptCache::new();
//! let template = cache.load(Path::new("prompts/test.md"))?;
//! let mut context = PromptContext::new();
//! context.set("name", "World");
//! let prompt = template.render(&context)?;
//! # Ok(())
//! # }
//! ```

pub mod processing;
pub mod templates;

pub use processing::{PromptCache, ValidationIssue, ValidationResult, validate_prompt};
pub use templates::{PromptContext, PromptError, PromptTemplate};
