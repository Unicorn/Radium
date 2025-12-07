//! Plan generation and AI-powered planning services.
//!
//! This module provides AI-powered plan generation from specifications,
//! including iteration structuring, task extraction, and dependency analysis.

mod dag;
mod executor;
mod generator;
pub mod markdown;
mod parser;

pub use dag::{DagError, DependencyGraph, Result as DagResult};
pub use executor::{ExecutionConfig, ExecutionError, PlanExecutor, RunMode, TaskResult};
pub use generator::{PlanGenerator, PlanGeneratorConfig};
pub use markdown::generate_plan_files;
pub use parser::{ParsedIteration, ParsedPlan, ParsedTask, PlanParser};
