//! Expression parsing and evaluation module
//!
//! Provides expression parsing, evaluation, and TypeScript code generation
//! for computed variables and conditional logic in workflows.

mod parser;
mod evaluator;
mod typescript_gen;

pub use parser::{Expression, ParseError, ExpressionParser};
pub use evaluator::{EvaluationError, ExpressionEvaluator};
pub use typescript_gen::{TypeScriptGenerator, TsGenError};
