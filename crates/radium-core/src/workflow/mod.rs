//! Workflow execution engine for Radium Core.
//!
//! This module provides functionality for executing workflows, including
//! sequential execution, conditional branching, and parallel execution,
//! plus workflow behaviors (loop, trigger, checkpoint).

pub mod behaviors;
pub mod complete;
pub mod control_flow;
pub mod engine;
pub mod executor;
pub mod failure;
pub mod parallel;
pub mod recovery;
pub mod reassignment;
pub mod service;
pub mod step_tracking;
pub mod template_discovery;
pub mod templates;

pub use behaviors::{
    BehaviorAction, BehaviorError, BehaviorEvaluator, CheckpointDecision, CheckpointEvaluator,
    CheckpointState, LoopBehaviorConfig, LoopCounters, LoopDecision, LoopEvaluator,
    TriggerBehaviorConfig, TriggerDecision, TriggerEvaluator, VibeCheckDecision,
    VibeCheckEvaluator, VibeCheckState,
};
pub use engine::{ExecutionContext, StepResult, WorkflowEngine, WorkflowEngineError};
pub use executor::WorkflowExecutor;
pub use failure::{
    FailureClassifier, FailureHistory, FailurePolicy, FailureRecord, FailureType,
};
pub use recovery::{
    RecoveryContext, RecoveryError, RecoveryManager, RecoveryStrategy, Result as RecoveryResult,
};
pub use reassignment::{
    AgentPerformanceTracker, AgentReassignment, AgentSelector, AgentStats, ReassignmentError,
    ReassignmentReason, ReassignmentRecord,
};
pub use service::{WorkflowExecution, WorkflowService};
pub use step_tracking::{StepRecord, StepStatus, StepTracker, StepTrackingError};
pub use template_discovery::TemplateDiscovery;
pub use complete::{
    detect_source, fetch_source_content, SourceDetectionError, SourceDetectionResult,
    SourceFetchError, SourceFetchResult, SourceType,
};
pub use templates::{
    ModuleBehavior, ModuleBehaviorAction, ModuleBehaviorType, WorkflowStep, WorkflowStepConfig,
    WorkflowStepType, WorkflowTemplate, WorkflowTemplateError,
};
