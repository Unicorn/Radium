//! Neo4j graph operations for the discovery service

pub mod client;
pub mod error;
pub mod schema;

#[allow(unused_imports)] // Re-exports used by downstream tasks (Tasks 5-8)
pub use client::{DiscoveryNode, IndexRequest};
#[allow(unused_imports)] // Re-export used by downstream tasks (Tasks 5-8)
pub use error::GraphError;
