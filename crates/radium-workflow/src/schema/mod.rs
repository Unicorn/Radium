//! Schema definitions for workflow definitions
//!
//! This module defines all the types needed to represent workflow definitions
//! in a type-safe manner. These types mirror the TypeScript definitions in
//! the workflow-builder package.

pub mod advanced;
pub mod components;
pub mod patterns;
pub mod state;
pub mod variables;
mod edge;
mod node;
mod settings;
mod variable;
mod workflow;

pub use components::{LogInput, LogLevel, LogOutput, StartInput, StartOutput, StopInput, StopOutput};
pub use edge::WorkflowEdge;
pub use node::{NodeData, NodeType, Position, RetryPolicy, RetryStrategy, WorkflowNode};
pub use settings::WorkflowSettings;
// Re-export from old variable module for backward compatibility
pub use variable::{VariableType as LegacyVariableType, WorkflowVariable};
// Re-export from new variables module
pub use variables::{
    VariableConstraints, VariableDefinition, VariableReference, VariableScope, VariableType,
    VariableValue,
};
// Re-export from state module
pub use state::{
    ActivityContext, ActivityError, ActivityState, ActivityStatus, BatchProgress, ContinuationInfo,
    ExecutionState, ProgressMarker, StateError, StateSnapshot, WorkflowState,
};
pub use workflow::WorkflowDefinition;

// Re-export from advanced module
pub use advanced::{
    // Child workflow orchestration
    CancellationType, ChildWorkflowExecutionResult, ChildWorkflowHandle, ChildWorkflowOrchestration,
    SearchAttributeValue as AdvancedSearchAttributeValue, WorkflowExecutionError, WorkflowIdReusePolicy,
    WorkflowIdStrategy,
    // Signals
    SignalBuffering, SignalDefinition, SignalHandler, SignalHandlerLogic, SignalSchema,
    SignalSchemaField, SignalWithHandler, VariableSource, VariableUpdate, WorkflowSignals,
    // Queries
    QueryDefinition, QueryHandlerLogic, QuerySchema, QuerySchemaField, WorkflowQueries,
    // Cancellation
    CancellationScope, CleanupActivity, CleanupConfig, StateUpdate, WorkflowCancellationHandler,
    // Search Attributes
    SearchAttributeDefinition, SearchAttributeType, SearchAttributeUpdate,
    TypedSearchAttributeValue, WorkflowSearchAttributes,
    // Versioning
    VersionBranch, VersionChangePoint, VersionInfo, VersioningConfig,
};

// Re-export from patterns module
pub use patterns::{
    // Workflow patterns trait
    WorkflowPattern,
    // Saga pattern
    CompensationBehavior, SagaAction, SagaDefinition, SagaStep,
    // Scatter-Gather pattern
    ErrorHandling, GatherConfig, GatherStrategy, InputDistribution, ResultAggregation,
    ScatterConfig, ScatterGatherDefinition, ScatterWorker,
    // Pipeline pattern
    PipelineDefinition, PipelineErrorHandling, PipelineStage, StageProcessor,
    // Map-Reduce pattern
    MapConfig, MapReduceDefinition, Mapper, ReduceConfig, Reducer,
};
