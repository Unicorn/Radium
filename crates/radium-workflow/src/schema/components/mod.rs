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
mod batch;
mod delay;
mod agent;
mod cache_component;
mod child_service;
mod child_workflow;
mod code_execute;
mod data_transform;
mod encode_decode;
mod event_emit;
mod conditional;
mod file_read;
mod file_write;
mod database_query;
mod graphql_request;
mod grpc_call;
mod http_request;
mod jwt_create;
mod log;
mod loop_component;
mod message;
mod parallel;
mod npm_function;
mod object_storage;
mod schema_validate;
mod queue_consume;
mod queue_publish;
mod secret_read;
mod shell_execute;
mod signal;
mod oauth_token;
mod smtp_send;
mod webhook_send;
mod websocket;
mod start;
mod stop;
mod timer;
mod trigger;

pub use action::{
    ActivityError, ActivityInput, ActivityOutput, RetryConfig, RetryPolicy, TimeoutConfig,
};
pub use batch::{BatchFailStrategy, BatchInput, BatchItemResult, BatchOutput};
pub use cache_component::{CacheAction, CacheInput, CacheOutput};
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
pub use file_read::{FileReadInput, FileReadOutput};
pub use file_write::{FileWriteInput, FileWriteMode, FileWriteOutput};
pub use graphql_request::{
    GraphQlError, GraphQlErrorLocation, GraphQlRequestInput, GraphQlRequestOutput,
};
pub use grpc_call::{GrpcCallInput, GrpcCallOutput};
pub use database_query::{
    ConnectionConfig, DatabaseQueryInput, DatabaseQueryOutput, OrderByClause, QueryOperation,
    ResultFormat, WhereCondition, WhereOperator,
};
pub use http_request::{
    AuthConfig, AuthType, BodyType, HttpMethod, HttpRequestInput, HttpRequestOutput,
};
pub use jwt_create::{JwtAlgorithm, JwtCreateInput, JwtCreateOutput};
pub use log::{LogInput, LogLevel, LogOutput};
pub use loop_component::{BatchConfig, LoopInput, LoopOutput, LoopType};
pub use parallel::{Branch, BranchResult, JoinStrategy, ParallelInput, ParallelOutput};
pub use code_execute::{CodeExecuteInput, CodeExecuteOutput, CodeLanguage};
pub use data_transform::{DataTransformInput, DataTransformOutput, ExpressionLanguage};
pub use encode_decode::{
    EncodeDecodeAction, EncodeDecodeFormat, EncodeDecodeInput, EncodeDecodeOutput, FormatOptions,
};
pub use event_emit::{EventEmitInput, EventEmitOutput};
pub use message::{SignalDirection, SignalInput, SignalOutput};
pub use npm_function::{NpmFunctionInput, NpmFunctionOutput};
pub use object_storage::{
    ObjectInfo, ObjectStorageInput, ObjectStorageOutput, StorageAction, StorageProvider,
};
pub use schema_validate::{SchemaValidateInput, SchemaValidateOutput, SchemaValidationError};
pub use queue_consume::{QueueConsumeInput, QueueConsumeOutput, QueueMessage};
pub use queue_publish::{QueueProvider, QueuePublishInput, QueuePublishOutput};
pub use secret_read::{SecretReadInput, SecretReadOutput};
pub use shell_execute::{CaptureMode, ShellExecuteInput, ShellExecuteOutput};
pub use oauth_token::{OAuthGrantType, OAuthTokenInput, OAuthTokenOutput};
pub use smtp_send::{EmailAttachment, EmailContentType, SmtpSendInput, SmtpSendOutput};
pub use webhook_send::{SigningAlgorithm, WebhookMethod, WebhookSendInput, WebhookSendOutput};
pub use websocket::{WebSocketAction, WebSocketInput, WebSocketMessageType, WebSocketOutput};
pub use start::{StartInput, StartOutput};
pub use stop::{StopInput, StopOutput};
pub use delay::{delay_default_behaviors, DelayInput, DelayOutput, DelayUnit};
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
