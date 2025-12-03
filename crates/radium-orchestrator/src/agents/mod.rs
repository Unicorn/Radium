//! Example agent implementations.
//!
//! This module provides example agents that demonstrate different use cases
//! and patterns for implementing the `Agent` trait.

pub mod chat;
pub mod simple;

pub use chat::ChatAgent;
pub use simple::SimpleAgent;
