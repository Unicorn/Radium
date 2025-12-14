//! Monitoring & Observability Module
//!
//! Provides comprehensive monitoring infrastructure:
//! - Metrics collection and aggregation
//! - Health check endpoints
//! - Distributed tracing support
//! - Event logging

mod health;
mod metrics;
mod tracing_support;

pub use health::*;
pub use metrics::*;
pub use tracing_support::*;
