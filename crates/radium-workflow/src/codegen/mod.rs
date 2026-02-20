//! Code generation module
//!
//! Generates TypeScript code from validated workflow definitions using Handlebars templates.

// Codegen types are part of the public interface
#![allow(dead_code)]
#![allow(unused_imports)]

mod state_generator;
mod typescript;

pub use state_generator::{StateGenError, StateGenerator, StateTemplateContext, VariableTemplateData};
pub use typescript::{CodeGenerator, GeneratedCode, GenerationError};

use crate::schema::WorkflowDefinition;

/// Generate TypeScript code from a workflow definition
pub fn generate(workflow: &WorkflowDefinition) -> Result<GeneratedCode, GenerationError> {
    let generator = CodeGenerator::new()?;
    generator.generate(workflow)
}
