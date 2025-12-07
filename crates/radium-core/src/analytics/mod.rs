//! Session analytics and reporting for Radium.
//!
//! Provides comprehensive session reporting with metrics, token tracking,
//! and cost transparency.

pub mod code_changes;
pub mod comparison;
pub mod report;
pub mod session;
pub mod storage;

pub use comparison::{ComparisonFormatter, SessionComparison};
pub use report::{ReportFormatter, SessionReport};
pub use session::{ModelUsageStats, SessionAnalytics, SessionMetrics};
pub use storage::{SessionMetadata, SessionStorage};
