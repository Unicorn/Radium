//! Context management for agent execution.
//!
//! This module provides context gathering and injection capabilities for agents,
//! including:
//! - Plan-scoped context (metadata, status, etc.)
//! - Memory context from previous agent executions
//! - File injection via syntax like `agent[input:file1.md,file2.md]`
//! - Tail context support like `agent[tail:50]`
//! - Architecture documentation injection
//! - Context files (GEMINI.md) with hierarchical loading and imports
//!
//! ## Context Files
//!
//! The context files system (see [`files`](files) module) provides persistent
//! instructions to agents through hierarchical `GEMINI.md` files. For detailed
//! user-facing documentation on context files, see
//! [Context Files Documentation](../../../docs/features/context-files.md).
//!
//! # Example
//!
//! ```rust,no_run
//! use radium_core::context::ContextManager;
//! use radium_core::workspace::{Workspace, RequirementId};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let workspace = Workspace::open(Path::new("/path/to/workspace"))?;
//! let req_id = RequirementId::new(1);
//!
//! // Create context manager for a plan
//! let manager = ContextManager::for_plan(&workspace, req_id)?;
//!
//! // Build context with file injection
//! let context = manager.build_context("architect[input:spec.md]", Some(req_id))?;
//!
//! // Context now contains:
//! // - Plan information
//! // - Architecture documentation (if exists)
//! // - Memory from previous runs
//! // - Contents of spec.md
//!
//! println!("Context: {}", context);
//! # Ok(())
//! # }
//! ```

mod error;
mod files;
mod history;
mod injection;
mod manager;
pub mod sources;
mod templates;
mod validator;

pub use error::{ContextError, Result};
pub use files::ContextFileLoader;
pub use history::{HistoryError, HistoryManager, Interaction, Result as HistoryResult};
pub use injection::{ContextInjector, InjectionDirective};
pub use manager::ContextManager;
pub use sources::{
    BraingridReader, HttpReader, JiraReader, LocalFileReader, SourceError, SourceMetadata,
    SourceReader, SourceRegistry,
};
pub use templates::{generate_template, TemplateType};
pub use validator::{SourceValidator, SourceValidationResult};
