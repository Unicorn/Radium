//! Autonomous execution orchestration.
//!
//! Provides end-to-end autonomous execution from high-level goals to completion,
//! coordinating all autonomous capabilities including decomposition, execution,
//! failure detection, recovery, reassignment, and learning.

pub mod orchestrator;

pub use orchestrator::{
    AutonomousConfig, AutonomousError, AutonomousOrchestrator, ExecutionMonitor, ExecutionResult,
    Result as AutonomousResult,
};

