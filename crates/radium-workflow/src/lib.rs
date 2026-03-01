//! Workflow Compiler - Rust-based workflow validation and TypeScript code generation
//!
//! This crate provides:
//! - Schema definitions for workflow definitions
//! - Validation of workflow graphs
//! - Type-safe TypeScript code generation
//! - Expression parsing and evaluation
//! - Verification pipeline (tsc + ESLint)
//! - Component migration framework

pub mod api;
pub mod change_detection;
pub mod codegen;
pub mod deploy_pipeline;
pub mod types;
pub mod discovery;
pub mod errors;
pub mod expressions;
pub mod kong_client;
pub mod migration;
pub mod monitoring;
pub mod performance;
pub mod schema;
pub mod security;
pub mod supabase;
pub mod temporal_client;
pub mod validation;
pub mod verification;
pub mod versioning;
pub mod yaml_format;

pub use errors::{ErrorCategory, ErrorCode, ErrorLocation, ErrorSeverity, WorkflowError, WorkflowErrors};
pub use expressions::{Expression, ExpressionParser, ExpressionEvaluator, TypeScriptGenerator};
pub use migration::{ComponentMigration, MigrationRecord, MigrationRunner, VerificationResult};
pub use monitoring::{HealthChecker, HealthReport, HealthStatus, MetricsRegistry, MetricsSnapshot, WorkflowMetrics, TraceCollector, TraceContext};
pub use performance::{CompilationCache, CompilationProfiler, CompilationProfile, ProfileBuilder, CompilationStage};
pub use schema::WorkflowDefinition;
pub use security::{AuditEvent, AuditEventType, AuditLog, AuditSeverity, InputSanitizer, RateLimitConfig, RateLimitResult, SlidingWindowLimiter, TokenBucketLimiter};
pub use yaml_format::{transform, TransformError, YamlWorkflow};
pub use supabase::{SupabaseClient, SupabaseConfig, SupabaseError};
pub use temporal_client::{TemporalClient, TemporalConfig, TemporalError};
