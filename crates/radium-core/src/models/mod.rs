//! Data models for Radium Core.
//!
//! This module contains the core data structures used throughout Radium,
//! including agents, workflows, and tasks. These structures serve as the
//! bridge between the gRPC protocol definitions and the internal Rust
//! implementation.

pub mod agent;
pub mod plan;
pub mod proto_convert;
pub mod selector;
pub mod task;
pub mod workflow;

pub use agent::{Agent, AgentConfig, AgentError, AgentState};
pub use plan::{Iteration, Plan, PlanError, PlanManifest, PlanStatus, PlanTask};
pub use selector::{
    ModelSelector, SelectedModel, SelectionError, SelectionOptions, SelectionResult,
};
pub use task::{Task, TaskError, TaskQueue, TaskResult, TaskState};
pub use workflow::{Workflow, WorkflowError, WorkflowState, WorkflowStep};
