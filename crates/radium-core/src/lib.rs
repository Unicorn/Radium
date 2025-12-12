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
pub mod analytics;
pub mod autonomous;
pub mod auth;
pub mod batch;
pub mod checkpoint;
pub mod clipboard;
pub mod code_blocks;
#[cfg(feature = "server")]
pub mod client;
// pub mod collaboration;  // TEMPORARILY DISABLED: depends on radium-orchestrator (circular dependency)
pub mod commands;
pub mod config;
pub mod context;
pub mod engines;
pub mod error;
pub mod extensions;
pub mod hooks;
pub mod learning;
pub mod mcp;  // Re-enabled for REQ-51 MCP integration work
pub mod memory;
pub mod models;
pub mod monitoring;
pub mod oversight;
pub mod planning;
pub mod playbooks;
pub mod policy;
pub mod prompts;
pub mod sandbox;
// pub mod server;  // TEMPORARILY DISABLED: depends on radium-orchestrator (circular dependency)
pub mod security;
#[cfg(feature = "syntax")]
pub mod syntax;
pub mod storage;
pub mod terminal;
#[cfg(feature = "workflow")]
pub mod workflow;
pub mod training;
pub mod workspace;

/// Generated protobuf code for the Radium gRPC API.
#[allow(clippy::similar_names)]
#[allow(clippy::doc_markdown)]
pub mod proto {
    tonic::include_proto!("radium");
}

pub use agents::config::{AgentConfigError, AgentConfigFile, ReasoningEffort};
pub use agents::discovery::{AgentDiscovery, DiscoveryError, DiscoveryOptions};
pub use agents::registry::{AgentRegistry, RegistryError};
#[cfg(feature = "monitoring")]
pub use analytics::{ReportFormatter, SessionAnalytics, SessionMetrics, SessionReport};
pub use auth::{AuthError, AuthResult, CredentialStore, ProviderType};
pub use checkpoint::{Checkpoint, CheckpointError, CheckpointManager, Result as CheckpointResult};
#[cfg(feature = "server")]
pub use client::ClientHelper;
pub use commands::{CommandError, CommandRegistry, CustomCommand, Result as CommandResult};
pub use config::{Config, cli_config::{CliConfig, CliConfigError, CliConfigResult}};
pub use context::{
    ContextError, ContextInjector, ContextManager, InjectionDirective, Result as ContextResult,
};
pub use engines::{
    BinaryDetector, Engine, EngineError, EngineMetadata, EngineRegistry, ExecutionRequest,
    ExecutionResponse, Result as EngineResult, TokenUsage,
};
pub use error::{RadiumError, Result};
pub use extensions::{
    Extension, ExtensionDiscovery, ExtensionError, ExtensionManager, ExtensionManifest,
    ExtensionManifestError, ExtensionStructureError, InstallOptions, Result as ExtensionResult,
};
pub use hooks::{
    Hook, HookConfig, HookContext, HookError, HookPriority, HookRegistry,
    HookResult as HookExecutionResult, HookResult, HookType, ModelHookContext, ModelHookType,
    TelemetryHookContext, ToolHookContext, ToolHookType,
};
#[cfg(feature = "orchestrator-integration")]
pub use hooks::{HookRegistryAdapter, OrchestratorHooks};
#[cfg(feature = "workflow")]
pub use hooks::{BehaviorEvaluatorAdapter, BehaviorHookRegistrar};
pub use learning::{
    CategorySummary, LearningEntry, LearningError, LearningStore, LearningType,
    Result as LearningResult, STANDARD_CATEGORIES, STANDARD_SECTIONS, Skill, SkillStatus,
    UpdateBatch, UpdateOperation, UpdateOperationType,
};
pub use memory::{
    FileAdapter, MemoryAdapter, MemoryEntry, MemoryError, MemoryStore, Result as MemoryResult,
};
pub use models::agent::{Agent, AgentConfig, AgentError, AgentState};
pub use models::plan::{Iteration, Plan, PlanError, PlanManifest, PlanStatus, PlanTask};
pub use models::task::{Task, TaskError, TaskQueue, TaskResult, TaskState};
pub use models::workflow::{Workflow, WorkflowError, WorkflowState, WorkflowStep};
// pub use monitoring::{  // DISABLED: monitoring module
//     AgentRecord, AgentStatus, LogManager, MonitoringError, MonitoringService,
//     Result as MonitoringResult, TelemetryParser, TelemetryRecord, TelemetryTracking,
//     initialize_schema,
// };
#[cfg(feature = "workflow")]
pub use oversight::{
    MetacognitiveError, MetacognitiveService, OversightRequest, OversightResponse,
    Result as OversightResult,
};
pub use planning::{
    generate_plan_files, ExecutionConfig, ExecutionError, ParsedIteration, ParsedPlan, ParsedTask,
    PlanExecutor, PlanGenerator, PlanGeneratorConfig, PlanParser, RunMode,
    TaskResult as PlanTaskResult,
};
pub use policy::{
    ApprovalMode, ConstitutionManager, PolicyAction, PolicyDecision, PolicyEngine, PolicyError,
    PolicyPriority, PolicyResult, PolicyRule,
};
pub use prompts::{PromptContext, PromptError, PromptTemplate};
pub use proto::radium_client;
pub use proto::{PingRequest, PingResponse};
pub use sandbox::{
    NetworkMode, NoSandbox, Result as SandboxResult, Sandbox, SandboxConfig, SandboxError,
    SandboxFactory, SandboxProfile, SandboxType,
};
#[cfg(feature = "orchestrator-integration")]
pub use sandbox::AgentSandboxManager;
pub use security::{
    SecretManager, SecurityError, SecurityResult,
};
pub use storage::{
    AgentRepository, Database, SqliteAgentRepository, SqliteTaskRepository,
    SqliteWorkflowRepository, StorageError, TaskRepository, WorkflowRepository,
};
#[cfg(feature = "workflow")]
pub use workflow::{
    BehaviorAction, BehaviorError, CheckpointDecision, CheckpointEvaluator, CheckpointState,
    ExecutionContext, LoopBehaviorConfig, LoopCounters, LoopDecision, LoopEvaluator, StepRecord,
    StepResult, StepStatus, StepTracker, StepTrackingError, TriggerBehaviorConfig, TriggerDecision,
    TriggerEvaluator, VibeCheckDecision, VibeCheckEvaluator, VibeCheckState, WorkflowEngine,
    WorkflowEngineError, WorkflowTemplate, WorkflowTemplateError,
};
pub use workspace::{
    DiscoveredPlan, PlanDiscovery, PlanDiscoveryOptions, RequirementId, RequirementIdError, SortBy,
    SortOrder, Workspace, WorkspaceConfig, WorkspaceError, WorkspaceStructure,
};
