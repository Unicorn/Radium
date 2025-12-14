//! Schema definitions for workflow definitions
//!
//! This module defines all the types needed to represent workflow definitions
//! in a type-safe manner. These types mirror the TypeScript definitions in
//! the workflow-builder package.

pub mod components;
pub mod state;
pub mod variables;
mod edge;
mod node;
mod settings;
mod variable;
mod workflow;

pub use components::{LogInput, LogLevel, LogOutput, StartInput, StartOutput, StopInput, StopOutput};
pub use edge::WorkflowEdge;
pub use node::{NodeData, NodeType, Position, RetryPolicy, RetryStrategy, WorkflowNode};
pub use settings::WorkflowSettings;
// Re-export from old variable module for backward compatibility
pub use variable::{VariableType as LegacyVariableType, WorkflowVariable};
// Re-export from new variables module
pub use variables::{
    VariableConstraints, VariableDefinition, VariableReference, VariableScope, VariableType,
    VariableValue,
};
// Re-export from state module
pub use state::{
    ActivityContext, ActivityError, ActivityState, ActivityStatus, BatchProgress, ContinuationInfo,
    ExecutionState, ProgressMarker, StateError, StateSnapshot, WorkflowState,
};
pub use workflow::WorkflowDefinition;
