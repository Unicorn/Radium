//! Performance Optimization Module
//!
//! Provides compilation caching, profiling, and performance monitoring:
//! - LRU-based compilation cache
//! - Stage-based compilation profiler
//! - Performance metrics collection

mod cache;
mod profiler;

pub use cache::*;
pub use profiler::*;
