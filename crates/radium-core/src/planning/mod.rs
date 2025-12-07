//! Plan generation and AI-powered planning services.
//!
//! This module provides AI-powered plan generation from specifications,
//! including iteration structuring, task extraction, and dependency analysis.

#[cfg(feature = "workflow")]
mod autonomous;
mod dag;
mod executor;
mod generator;
pub mod markdown;
mod parser;

#[cfg(feature = "workflow")]
pub use autonomous::{
    AutonomousPlan, AutonomousPlanner, PlanValidator, PlanningError, Result as PlanningResult,
    ValidationReport, WorkflowGenerator,
};
pub use dag::{DagError, DependencyGraph, Result as DagResult};
pub use executor::{ErrorCategory, ExecutionConfig, ExecutionError, PlanExecutor, RunMode, TaskResult};
pub use generator::{PlanGenerator, PlanGeneratorConfig};
pub use markdown::generate_plan_files;
pub use parser::{ParsedIteration, ParsedPlan, ParsedTask, PlanParser};
