//! Advanced workflow features
//!
//! This module provides advanced Temporal workflow capabilities:
//! - Child workflow orchestration with ID strategies
//! - Signal handlers for workflow communication
//! - Query handlers for state inspection
//! - Cancellation scopes for graceful cleanup
//! - Search attributes for workflow discovery
//! - Workflow versioning for safe deployments

pub mod cancellation;
pub mod child_orchestration;
pub mod queries;
pub mod search_attributes;
pub mod signals;
pub mod versioning;

pub use cancellation::*;
pub use child_orchestration::*;
pub use queries::*;
pub use search_attributes::*;
pub use signals::*;
pub use versioning::*;
