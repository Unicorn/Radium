//! Playbook system for embedding organizational knowledge into agent behavior.
//!
//! Playbooks are YAML-frontmattered markdown files that contain organizational
//! knowledge, SOPs, and procedures. They are automatically loaded into agent
//! context based on scope and tags.

pub mod braingrid_storage;
pub mod discovery;
pub mod error;
pub mod parser;
pub mod registry;
pub mod storage;
pub mod types;

pub use discovery::PlaybookDiscovery;
pub use error::{PlaybookError, Result};
pub use parser::PlaybookParser;
pub use registry::PlaybookRegistry;
pub use storage::PlaybookStorage;
pub use types::{Playbook, PlaybookPriority};

