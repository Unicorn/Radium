//! Content search tools for repository understanding.
//!
//! This module provides fast, agent-accessible content search capabilities
//! that allow searching across repository files with context lines, filters,
//! and result limits.

pub mod filters;
pub mod grep;

#[cfg(test)]
mod tests;

pub use grep::{search_code, SearchOptions, SearchResult};
