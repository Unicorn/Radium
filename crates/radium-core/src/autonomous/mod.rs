//! Autonomous execution orchestration.
//!
//! Provides end-to-end autonomous execution from high-level goals to completion,
//! coordinating all autonomous capabilities including decomposition, execution,
//! failure detection, recovery, reassignment, and learning.

#[cfg(feature = "orchestrator-integration")]
pub mod orchestrator;

#[cfg(feature = "orchestrator-integration")]
pub use orchestrator::{
    AutonomousConfig, AutonomousError, AutonomousOrchestrator, ExecutionMonitor, ExecutionResult,
    Result as AutonomousResult,
};

