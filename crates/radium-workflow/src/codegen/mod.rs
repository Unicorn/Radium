//! Code generation module
//!
//! Generates TypeScript code from validated workflow definitions using Handlebars templates.

mod typescript;

pub use typescript::{CodeGenerator, GeneratedCode, GenerationError};

use crate::schema::WorkflowDefinition;

/// Generate TypeScript code from a workflow definition
pub fn generate(workflow: &WorkflowDefinition) -> Result<GeneratedCode, GenerationError> {
    let generator = CodeGenerator::new()?;
    generator.generate(workflow)
}
