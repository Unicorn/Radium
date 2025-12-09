//! Model caching system for optimizing model lifecycle.
//!
//! This module provides a caching layer for AI models, reducing initialization
//! overhead by keeping frequently-used models in memory and automatically
//! unloading inactive models.

pub mod cache;
pub mod config;
pub mod types;

// ModelCache will be exported in Task 2
// pub use cache::ModelCache;
pub use config::CacheConfig;
pub use types::{CacheKey, CacheStats, CachedModel};

