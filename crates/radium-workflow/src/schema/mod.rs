//! Schema definitions for workflow definitions
//!
//! This module defines all the types needed to represent workflow definitions
//! in a type-safe manner. These types mirror the TypeScript definitions in
//! the workflow-builder package.

pub mod components;
mod edge;
mod node;
mod settings;
mod variable;
mod workflow;

pub use components::{LogInput, LogLevel, LogOutput, StartInput, StartOutput, StopInput, StopOutput};
pub use edge::WorkflowEdge;
pub use node::{NodeData, NodeType, Position, RetryPolicy, RetryStrategy, WorkflowNode};
pub use settings::WorkflowSettings;
pub use variable::{VariableType, WorkflowVariable};
pub use workflow::WorkflowDefinition;
