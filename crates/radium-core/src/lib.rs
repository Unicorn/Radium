//! Radium Core - High-performance agent orchestration backend.
//!
//! This crate provides the core functionality for Radium, including:
//! - gRPC server for client communication
//! - Configuration management
//! - Error handling
//!
//! # Example
//!
//! ```rust,no_run
//! use radium_core::{config::Config, server};
//!
//! #[tokio::main]
//! async fn main() -> radium_core::error::Result<()> {
//!     let config = Config::load()?;
//!     server::run(&config).await
//! }
//! ```

pub mod agents;
pub mod config;
pub mod error;
pub mod models;
pub mod prompts;
pub mod server;
pub mod storage;
pub mod workflow;
pub mod workspace;

/// Generated protobuf code for the Radium gRPC API.
#[allow(clippy::similar_names)]
#[allow(clippy::doc_markdown)]
pub mod proto {
    tonic::include_proto!("radium");
}

pub use agents::config::{AgentConfigError, AgentConfigFile, ReasoningEffort};
pub use agents::discovery::{AgentDiscovery, DiscoveryError, DiscoveryOptions};
pub use agents::prompt_loader::{AgentPromptLoader, PromptLoaderError};
pub use config::Config;
pub use error::{RadiumError, Result};
pub use models::agent::{Agent, AgentConfig, AgentError, AgentState};
pub use models::plan::{Iteration, Plan, PlanError, PlanManifest, PlanStatus, PlanTask};
pub use models::task::{Task, TaskError, TaskQueue, TaskResult, TaskState};
pub use models::workflow::{Workflow, WorkflowError, WorkflowState, WorkflowStep};
pub use prompts::{
    PromptCache, PromptContext, PromptError, PromptTemplate, ValidationIssue, ValidationResult,
    validate_prompt,
};
pub use proto::radium_client;
pub use proto::{PingRequest, PingResponse};
pub use storage::{
    AgentRepository, Database, SqliteAgentRepository, SqliteTaskRepository,
    SqliteWorkflowRepository, StorageError, TaskRepository, WorkflowRepository,
};
pub use workflow::{
    BehaviorAction, BehaviorError, CheckpointDecision, CheckpointEvaluator, CheckpointState,
    ExecutionContext, LoopBehaviorConfig, LoopCounters, LoopDecision, LoopEvaluator, StepRecord,
    StepResult, StepStatus, StepTracker, StepTrackingError, TriggerBehaviorConfig, TriggerDecision,
    TriggerEvaluator, WorkflowEngine, WorkflowEngineError, WorkflowTemplate, WorkflowTemplateError,
};
pub use workspace::{
    DiscoveredPlan, PlanDiscovery, PlanDiscoveryOptions, RequirementId, RequirementIdError, SortBy,
    SortOrder, Workspace, WorkspaceConfig, WorkspaceError, WorkspaceStructure,
};
