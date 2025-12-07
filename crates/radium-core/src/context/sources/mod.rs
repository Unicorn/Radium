//! Source reading and verification.
//!
//! This module provides a unified abstraction for reading sources from different
//! protocols (file, HTTP, Jira, Braingrid, etc.). The main trait is `SourceReader`,
//! which implementations provide for specific protocols.
//!
//! # Example
//!
//! ```rust,no_run
//! use radium_core::context::sources::{SourceReader, SourceMetadata, SourceError};
//!
//! # async fn example() -> Result<(), SourceError> {
//! // Get a reader for a specific scheme
//! // let reader = registry.get_reader("file:///path/to/file.txt")?;
//!
//! // Verify source exists
//! // let metadata = reader.verify("file:///path/to/file.txt").await?;
//! // assert!(metadata.accessible);
//!
//! // Fetch full content
//! // let content = reader.fetch("file:///path/to/file.txt").await?;
//! # Ok(())
//! # }
//! ```

mod braingrid;
mod http;
mod jira;
mod local;
mod traits;
mod types;

pub use braingrid::BraingridReader;
pub use http::HttpReader;
pub use jira::JiraReader;
pub use local::LocalFileReader;
pub use traits::SourceReader;
pub use types::{SourceError, SourceMetadata};
