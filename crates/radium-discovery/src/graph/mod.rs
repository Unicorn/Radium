//! Neo4j graph operations for the discovery service

pub mod client;
pub mod error;
pub mod queries;
pub mod schema;

#[allow(unused_imports)] // DiscoveryNode re-exported for search endpoint (Task 6)
pub use client::DiscoveryNode;
pub use client::IndexRequest;
pub use error::GraphError;
