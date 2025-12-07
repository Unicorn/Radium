//! Hooks system for execution flow interception and customization.
//!
//! This module provides a hooks system that enables users to intercept and customize
//! behavior at various points in the execution flow, including:
//! - Model call hooks (before/after)
//! - Tool execution hooks (before/after/selection)
//! - Error handling hooks (interception, transformation, recovery, logging)
//! - Telemetry hooks (collection, logging, metrics, performance monitoring)

pub mod config;
pub mod error;
pub mod error_hooks;
pub mod integration;
pub mod model;
pub mod registry;
pub mod telemetry;
pub mod tool;
pub mod types;

pub use config::HookConfig;
pub use error::{HookError, Result as HookResult};
pub use error_hooks::{ErrorHook, ErrorHookContext, ErrorHookType};
pub use integration::OrchestratorHooks;
pub use model::{ModelHook, ModelHookContext, ModelHookType};
pub use registry::{Hook, HookRegistry, HookType};
pub use telemetry::TelemetryHookContext;
pub use tool::{ToolHook, ToolHookContext, ToolHookType};
pub use types::{HookContext, HookPriority, HookResult as HookExecutionResult};

