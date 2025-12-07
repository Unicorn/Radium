//! Command implementations for the Radium CLI.

pub mod agents;
pub mod auth;
// pub mod autonomous;  // DISABLED: depends on radium_core::autonomous (circular dependency)
pub mod budget;
// pub mod chat;  // DISABLED: depends on radium_core::mcp and analytics (disabled modules)
pub mod checkpoint;
pub mod clean;
// pub mod complete;  // DISABLED: depends on radium_core::workflow (circular dependency)
pub mod context;
pub mod craft;
// pub mod custom;  // DISABLED: depends on radium_core::commands (disabled)
pub mod doctor;
pub mod engines;
pub mod extension;
pub mod hooks;
pub mod init;
// pub mod mcp;  // DISABLED: depends on radium_core::mcp (circular dependency)
pub mod monitor;
pub mod plan;
pub mod policy;
pub mod constitution;
pub mod run;
pub mod sandbox;
pub mod stats;
pub mod status;
pub mod step;
// pub mod templates;  // DISABLED: depends on radium_core::workflow (circular dependency)
pub mod learning;
pub mod types;
pub mod validate;
// pub mod vibecheck;  // DISABLED: depends on radium_core::oversight and policy (disabled)

// Re-export types for convenience
pub use types::{AgentsCommand, AuthCommand, EnginesCommand, ExtensionCommand, HooksCommand, MigrateSubcommand};
// pub use types::{CustomCommand, TemplatesCommand};  // DISABLED: commands disabled
pub use context::ContextCommand;
pub use budget::BudgetCommand;
pub use policy::{execute_policy_command, PolicyCommand};
pub use constitution::{execute_constitution_command, ConstitutionCommand};
