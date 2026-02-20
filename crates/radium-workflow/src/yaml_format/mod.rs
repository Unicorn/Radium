//! YAML workflow format and transformer
//!
//! This module provides a human-friendly YAML format for defining workflows
//! and a transformer that converts it into the compiler's internal
//! `WorkflowDefinition` struct. The YAML format hides React Flow positions,
//! edge types, handles, and other UI concerns so that agents can write
//! simple, readable workflow definitions.
//!
//! # Example
//!
//! ```yaml
//! name: My Workflow
//! components:
//!   - id: start
//!     type: trigger
//!   - id: fetch
//!     type: http_request
//!     config:
//!       name: fetchData
//!       url: "https://api.example.com"
//!   - id: end
//!     type: stop
//! connections:
//!   - from: start
//!     to: fetch
//!   - from: fetch
//!     to: end
//! ```

pub mod transformer;
pub mod types;

pub use transformer::{transform, TransformError};
pub use types::{
    YamlComponent, YamlComponentType, YamlConnection, YamlSettings, YamlVariable, YamlWorkflow,
};
