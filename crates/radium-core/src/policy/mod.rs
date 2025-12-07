//! Policy engine for fine-grained tool execution control.
//!
//! This module provides a rule-based policy system for controlling tool execution
//! during workflow runs. Policies can allow, deny, or require user approval for
//! tool calls based on pattern matching and priority levels.
//!
//! # Features
//!
//! - TOML-based policy configuration
//! - Tool execution control (allow/deny/ask_user)
//! - Priority-based rule matching (Default/User/Admin)
//! - Approval modes (yolo, autoEdit, ask)
//! - Pattern matching for tool names and arguments
//! - Special handling for shell commands and MCP tools
//! - Session-based constitution rules for per-session enforcement
//!
//! # Example
//!
//! ```no_run
//! use radium_core::policy::{PolicyEngine, ApprovalMode};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let engine = PolicyEngine::new(ApprovalMode::Ask)?;
//!
//! // Check if a tool execution is allowed
//! let decision = engine.evaluate_tool("read_file", &["config.toml"]).await?;
//! # Ok(())
//! # }
//! ```

pub mod constitution;
mod rules;
pub mod templates;
mod types;

pub use constitution::ConstitutionManager;
pub use rules::{PolicyEngine, PolicyRule};
pub use templates::{merge_template, PolicyTemplate, TemplateDiscovery};
pub use types::{
    ApprovalMode, PolicyAction, PolicyDecision, PolicyError, PolicyPriority, PolicyResult,
};
