//! Workflow Compiler - Rust-based workflow validation and TypeScript code generation
//!
//! This crate provides:
//! - Schema definitions for workflow definitions
//! - Validation of workflow graphs
//! - Type-safe TypeScript code generation
//! - Verification pipeline (tsc + ESLint)

pub mod api;
pub mod codegen;
pub mod schema;
pub mod validation;
pub mod verification;

pub use schema::WorkflowDefinition;
