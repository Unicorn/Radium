//! Component schemas for workflow nodes
//!
//! This module contains the type definitions for all workflow components:
//! - Control Flow: Start, Stop, Trigger, Conditional, Loop
//! - Activities: Log, Activity, HTTP Request, Database Query
//! - Agent: AI model invocation
//! - Advanced: Child Workflow, Signal, Timer, Parallel
//! - Behaviors: Shared retry, rate limit, circuit breaker, idempotency, output envelope

pub mod behaviors;
mod action;
mod activity;
mod agent;
mod child_service;
mod child_workflow;
mod conditional;
mod database_query;
mod http_request;
mod log;
mod loop_component;
mod message;
mod parallel;
mod npm_function;
mod shell_execute;
mod signal;
mod start;
mod stop;
mod timer;
mod trigger;

pub use action::{
    ActivityError, ActivityInput, ActivityOutput, RetryConfig, RetryPolicy, TimeoutConfig,
};
pub use agent::{
    AgentInput, AgentOutput, AIProvider, AnthropicModel, FinishReason, Message, MessageRole,
    ModelConfig, TokenUsage, Tool, ToolCall,
};
pub use child_service::{
    ChildWorkflowInput, ChildWorkflowOutput, ParentClosePolicy, WorkflowStatus,
};
pub use conditional::{
    ComparisonOperator, Condition, ConditionGroup, ConditionalInput, ConditionalOutput,
    LogicalOperator,
};
pub use database_query::{
    ConnectionConfig, DatabaseQueryInput, DatabaseQueryOutput, OrderByClause, QueryOperation,
    ResultFormat, WhereCondition, WhereOperator,
};
pub use http_request::{
    AuthConfig, AuthType, BodyType, HttpMethod, HttpRequestInput, HttpRequestOutput,
};
pub use log::{LogInput, LogLevel, LogOutput};
pub use loop_component::{BatchConfig, LoopInput, LoopOutput, LoopType};
pub use parallel::{Branch, BranchResult, JoinStrategy, ParallelInput, ParallelOutput};
pub use message::{SignalDirection, SignalInput, SignalOutput};
pub use npm_function::{NpmFunctionInput, NpmFunctionOutput};
pub use shell_execute::{CaptureMode, ShellExecuteInput, ShellExecuteOutput};
pub use start::{StartInput, StartOutput};
pub use stop::{StopInput, StopOutput};
pub use timer::{DurationUnit, TimerInput, TimerOutput, TimerType};
pub use trigger::{ScheduleConfig, TriggerInput, TriggerOutput, TriggerType, WebhookConfig};

// Shared behavior types — re-exported for convenient access.
// NOTE: `behaviors::RetryPolicy` is the shared struct-based retry config.
// `activity::RetryPolicy` is the legacy enum (NoRetry/Linear/Exponential/Custom).
// Both are available; the shared version is accessed via `behaviors::RetryPolicy`
// or the alias `SharedRetryPolicy` to avoid ambiguity.
pub use behaviors::{
    BackoffStrategy, BehaviorLogLevel, CircuitBreakerConfig, ComponentBehaviors, ComponentError,
    ComponentOutput, ErrorCatalogEntry, IdempotencyConfig, IdempotencyKeyStrategy,
    ObservabilityConfig, OutputMetadata, PayloadLimits, RateLimitConfig, RateLimitStrategy,
    RetryPolicy as SharedRetryPolicy,
};
