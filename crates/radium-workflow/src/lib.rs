//! Workflow Compiler - Rust-based workflow validation and TypeScript code generation
//!
//! This crate provides:
//! - Schema definitions for workflow definitions
//! - Validation of workflow graphs
//! - Type-safe TypeScript code generation
//! - Expression parsing and evaluation
//! - Verification pipeline (tsc + ESLint)
//! - Component migration framework

pub mod api;
pub mod codegen;
pub mod expressions;
pub mod migration;
pub mod schema;
pub mod validation;
pub mod verification;

pub use expressions::{Expression, ExpressionParser, ExpressionEvaluator, TypeScriptGenerator};
pub use migration::{ComponentMigration, MigrationRecord, MigrationRunner, VerificationResult};
pub use schema::WorkflowDefinition;
