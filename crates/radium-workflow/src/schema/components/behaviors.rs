//! Shared component behavior types
//!
//! This module defines production-grade behavior configurations that I/O components
//! embed to get consistent retry, rate limiting, circuit breaking, idempotency,
//! observability, and output envelope semantics. These types are both human-readable
//! and machine-parseable — agents read these schemas to understand component behavior.

use serde::{Deserialize, Serialize};
use validator::Validate;

// ---------------------------------------------------------------------------
// Retry
// ---------------------------------------------------------------------------

/// Strategy for spacing out retry attempts after transient failures.
///
/// Controls how the delay between retries grows. Exponential backoff is the
/// recommended default for most network-bound operations.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackoffStrategy {
    /// Delay doubles each attempt (capped by `max_interval_ms`).
    #[default]
    Exponential,
    /// Delay increases by a fixed step each attempt.
    Linear,
    /// No delay between retries.
    None,
}

/// Retry policy applied to component execution.
///
/// Governs how many times a component is retried on transient failure,
/// what backoff curve is used, and which error codes are considered
/// non-retryable. Defaults are tuned for typical HTTP/RPC workloads.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct RetryPolicy {
    /// Maximum number of attempts (including the initial attempt).
    /// Must be at least 1.
    #[serde(default = "default_max_attempts")]
    #[validate(range(min = 1, message = "max_attempts must be at least 1"))]
    pub max_attempts: u32,

    /// Backoff strategy between retry attempts.
    #[serde(default)]
    pub backoff: BackoffStrategy,

    /// Initial delay in milliseconds before the first retry.
    #[serde(default = "default_initial_interval_ms")]
    pub initial_interval_ms: u64,

    /// Upper bound on delay in milliseconds between retries.
    #[serde(default = "default_max_interval_ms")]
    pub max_interval_ms: u64,

    /// Error codes that must never be retried regardless of policy.
    /// An empty list means all errors are eligible for retry.
    #[serde(default)]
    pub non_retryable_errors: Vec<String>,
}

fn default_max_attempts() -> u32 {
    3
}
fn default_initial_interval_ms() -> u64 {
    1000
}
fn default_max_interval_ms() -> u64 {
    30_000
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: default_max_attempts(),
            backoff: BackoffStrategy::default(),
            initial_interval_ms: default_initial_interval_ms(),
            max_interval_ms: default_max_interval_ms(),
            non_retryable_errors: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Idempotency
// ---------------------------------------------------------------------------

/// Strategy for generating idempotency keys.
///
/// `Auto` derives a key from component ID + input hash. `Custom` lets the
/// workflow author supply a key expression.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IdempotencyKeyStrategy {
    /// Automatically derived from component ID and input content hash.
    #[default]
    Auto,
    /// User-provided key expression evaluated at runtime.
    Custom,
}

/// Configuration for idempotent component execution.
///
/// When enabled, the runtime deduplicates executions with the same key
/// so that a component produces at-most-once side effects even if the
/// workflow is replayed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct IdempotencyConfig {
    /// Whether idempotency enforcement is active.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// How the idempotency key is generated.
    #[serde(default)]
    pub key_strategy: IdempotencyKeyStrategy,
}

fn default_true() -> bool {
    true
}

impl Default for IdempotencyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            key_strategy: IdempotencyKeyStrategy::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Rate Limiting
// ---------------------------------------------------------------------------

/// Algorithm used for rate limiting outgoing requests.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitStrategy {
    /// Sliding window counter — smooths bursts over the window period.
    #[default]
    SlidingWindow,
    /// Token bucket — allows controlled bursting up to `burst` tokens.
    TokenBucket,
    /// Fixed window counter — simple per-interval cap.
    FixedWindow,
}

/// Rate limiting configuration for outbound component calls.
///
/// Prevents a component from overwhelming downstream services. The
/// defaults (10 req/s, burst 20) are safe for most third-party APIs.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct RateLimitConfig {
    /// Sustained requests per second allowed.
    #[serde(default = "default_requests_per_second")]
    #[validate(range(min = 1, message = "requests_per_second must be at least 1"))]
    pub requests_per_second: u32,

    /// Maximum burst size above the sustained rate.
    #[serde(default = "default_burst")]
    #[validate(range(min = 1, message = "burst must be at least 1"))]
    pub burst: u32,

    /// Algorithm used to enforce the limit.
    #[serde(default)]
    pub strategy: RateLimitStrategy,
}

fn default_requests_per_second() -> u32 {
    10
}
fn default_burst() -> u32 {
    20
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: default_requests_per_second(),
            burst: default_burst(),
            strategy: RateLimitStrategy::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Circuit Breaker
// ---------------------------------------------------------------------------

/// Circuit breaker configuration for component execution.
///
/// Opens the circuit after `failure_threshold` consecutive failures,
/// waits `cooldown_ms`, then enters half-open state allowing
/// `half_open_max_requests` probes before fully closing again.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before the circuit opens.
    #[serde(default = "default_failure_threshold")]
    #[validate(range(min = 1, message = "failure_threshold must be at least 1"))]
    pub failure_threshold: u32,

    /// Time in milliseconds the circuit stays open before probing.
    #[serde(default = "default_cooldown_ms")]
    pub cooldown_ms: u64,

    /// Number of probe requests allowed in half-open state.
    #[serde(default = "default_half_open_max_requests")]
    #[validate(range(min = 1, message = "half_open_max_requests must be at least 1"))]
    pub half_open_max_requests: u32,
}

fn default_failure_threshold() -> u32 {
    5
}
fn default_cooldown_ms() -> u64 {
    30_000
}
fn default_half_open_max_requests() -> u32 {
    1
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: default_failure_threshold(),
            cooldown_ms: default_cooldown_ms(),
            half_open_max_requests: default_half_open_max_requests(),
        }
    }
}

// ---------------------------------------------------------------------------
// Payload Limits
// ---------------------------------------------------------------------------

/// Byte-size limits on component input and output payloads.
///
/// Prevents runaway memory usage when components exchange large data.
/// Defaults to 10 MiB in each direction.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct PayloadLimits {
    /// Maximum allowed input payload size in bytes (default 10 MiB).
    #[serde(default = "default_payload_limit")]
    #[validate(range(min = 1, message = "max_input_bytes must be at least 1"))]
    pub max_input_bytes: u64,

    /// Maximum allowed output payload size in bytes (default 10 MiB).
    #[serde(default = "default_payload_limit")]
    #[validate(range(min = 1, message = "max_output_bytes must be at least 1"))]
    pub max_output_bytes: u64,
}

fn default_payload_limit() -> u64 {
    10_485_760 // 10 MiB
}

impl Default for PayloadLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: default_payload_limit(),
            max_output_bytes: default_payload_limit(),
        }
    }
}

// ---------------------------------------------------------------------------
// Observability
// ---------------------------------------------------------------------------

/// Log level for component observability telemetry.
///
/// Distinct from the workflow `LogLevel` used by the Log component —
/// this controls the verbosity of the runtime's own telemetry for a
/// component execution.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BehaviorLogLevel {
    /// Emit all telemetry including detailed input/output snapshots.
    Debug,
    /// Standard operational telemetry (default).
    #[default]
    Info,
    /// Only warnings and errors.
    Warning,
    /// Only error-level telemetry.
    Error,
}

/// Observability configuration for component execution telemetry.
///
/// Controls what the runtime captures when a component runs — timing,
/// payload sizes, status codes, and the verbosity of structured logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ObservabilityConfig {
    /// Minimum log level for this component's telemetry.
    #[serde(default)]
    pub log_level: BehaviorLogLevel,

    /// Whether to record execution duration.
    #[serde(default = "default_true")]
    pub capture_timing: bool,

    /// Whether to record input/output payload sizes.
    #[serde(default = "default_true")]
    pub capture_size: bool,

    /// Whether to record HTTP/gRPC status codes.
    #[serde(default = "default_true")]
    pub capture_status: bool,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            log_level: BehaviorLogLevel::default(),
            capture_timing: true,
            capture_size: true,
            capture_status: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Component Behaviors (aggregate)
// ---------------------------------------------------------------------------

/// Aggregate behavior configuration embedded by I/O components.
///
/// Provides a single struct combining retry, timeout, idempotency,
/// rate limiting, circuit breaking, payload limits, and observability.
/// Every field has a sensible default so components can opt in
/// incrementally.
///
/// # Example (YAML)
///
/// ```yaml
/// behaviors:
///   retry_policy:
///     max_attempts: 5
///     backoff: linear
///   timeout_ms: 60000
///   rate_limit:
///     requests_per_second: 5
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct ComponentBehaviors {
    /// Retry policy for transient failures.
    #[serde(default)]
    #[validate(nested)]
    pub retry_policy: RetryPolicy,

    /// Maximum wall-clock time in milliseconds for a single attempt.
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,

    /// Optional heartbeat interval in milliseconds for long-running components.
    /// When set, the runtime expects periodic heartbeats and will cancel
    /// the component if none arrive within this interval.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heartbeat_interval_ms: Option<u64>,

    /// Idempotency enforcement configuration.
    #[serde(default)]
    pub idempotency: IdempotencyConfig,

    /// Rate limiting for outbound calls.
    #[serde(default)]
    #[validate(nested)]
    pub rate_limit: RateLimitConfig,

    /// Circuit breaker for downstream protection.
    #[serde(default)]
    #[validate(nested)]
    pub circuit_breaker: CircuitBreakerConfig,

    /// Payload size limits.
    #[serde(default)]
    #[validate(nested)]
    pub payload_limits: PayloadLimits,

    /// Observability and telemetry settings.
    #[serde(default)]
    pub observability: ObservabilityConfig,

    /// Whether secret values must use `${{ secrets.NAME }}` references
    /// rather than inline plaintext. Enforced at compile time.
    #[serde(default = "default_true")]
    pub enforce_secret_references: bool,
}

fn default_timeout_ms() -> u64 {
    30_000
}

impl Default for ComponentBehaviors {
    fn default() -> Self {
        Self {
            retry_policy: RetryPolicy::default(),
            timeout_ms: default_timeout_ms(),
            heartbeat_interval_ms: None,
            idempotency: IdempotencyConfig::default(),
            rate_limit: RateLimitConfig::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
            payload_limits: PayloadLimits::default(),
            observability: ObservabilityConfig::default(),
            enforce_secret_references: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Output types
// ---------------------------------------------------------------------------

/// Entry in a component's error catalog.
///
/// Each component can declare the known error codes it may produce.
/// This allows workflow authors and tooling to reason about failure
/// modes at design time.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ErrorCatalogEntry {
    /// Machine-readable error code (e.g. `"TIMEOUT"`, `"AUTH_FAILED"`).
    pub code: String,

    /// Whether this error is eligible for automatic retry.
    pub retryable: bool,

    /// Human-readable description of the error condition.
    pub description: String,
}

/// Error details attached to a failed component execution.
///
/// Carried inside [`ComponentOutput`] when `success` is `false`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ComponentError {
    /// Machine-readable error code matching an [`ErrorCatalogEntry`].
    pub code: String,

    /// Human-readable error message.
    pub message: String,

    /// Whether this specific occurrence is retryable.
    pub retryable: bool,

    /// Optional structured details (stack traces, upstream response, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Metadata captured alongside every component execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OutputMetadata {
    /// Wall-clock duration of the execution in milliseconds.
    pub duration_ms: u64,

    /// Total number of attempts (1 = no retries occurred).
    pub attempt_count: u32,

    /// Unique identifier of the component instance that ran.
    pub component_id: String,

    /// Idempotency key used for this execution.
    pub idempotency_key: String,
}

/// Standard output envelope for all I/O components.
///
/// Provides a uniform shape for downstream nodes to consume:
/// check `success`, read `data` on happy path, inspect `error` on
/// failure, and use `metadata` for observability.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ComponentOutput {
    /// Whether the component completed successfully.
    pub success: bool,

    /// The component's result payload. Contains `null` on failure.
    pub data: serde_json::Value,

    /// Error information when `success` is `false`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ComponentError>,

    /// Execution metadata (timing, attempts, identifiers).
    pub metadata: OutputMetadata,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_retry_policy() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.backoff, BackoffStrategy::Exponential);
        assert_eq!(policy.initial_interval_ms, 1000);
        assert_eq!(policy.max_interval_ms, 30_000);
        assert!(policy.non_retryable_errors.is_empty());
    }

    #[test]
    fn test_default_behaviors() {
        let b = ComponentBehaviors::default();

        // Retry
        assert_eq!(b.retry_policy.max_attempts, 3);
        assert_eq!(b.retry_policy.backoff, BackoffStrategy::Exponential);

        // Timeout
        assert_eq!(b.timeout_ms, 30_000);
        assert!(b.heartbeat_interval_ms.is_none());

        // Idempotency
        assert!(b.idempotency.enabled);
        assert_eq!(b.idempotency.key_strategy, IdempotencyKeyStrategy::Auto);

        // Rate limit
        assert_eq!(b.rate_limit.requests_per_second, 10);
        assert_eq!(b.rate_limit.burst, 20);
        assert_eq!(b.rate_limit.strategy, RateLimitStrategy::SlidingWindow);

        // Circuit breaker
        assert_eq!(b.circuit_breaker.failure_threshold, 5);
        assert_eq!(b.circuit_breaker.cooldown_ms, 30_000);
        assert_eq!(b.circuit_breaker.half_open_max_requests, 1);

        // Payload limits
        assert_eq!(b.payload_limits.max_input_bytes, 10_485_760);
        assert_eq!(b.payload_limits.max_output_bytes, 10_485_760);

        // Observability
        assert_eq!(b.observability.log_level, BehaviorLogLevel::Info);
        assert!(b.observability.capture_timing);
        assert!(b.observability.capture_size);
        assert!(b.observability.capture_status);

        // Secret enforcement
        assert!(b.enforce_secret_references);
    }

    #[test]
    fn test_behaviors_serialize_deserialize() {
        let original = ComponentBehaviors::default();
        let yaml = serde_yaml::to_string(&original).expect("serialize to YAML");
        let restored: ComponentBehaviors =
            serde_yaml::from_str(&yaml).expect("deserialize from YAML");

        assert_eq!(restored.retry_policy.max_attempts, original.retry_policy.max_attempts);
        assert_eq!(restored.timeout_ms, original.timeout_ms);
        assert_eq!(
            restored.rate_limit.requests_per_second,
            original.rate_limit.requests_per_second
        );
        assert_eq!(
            restored.circuit_breaker.failure_threshold,
            original.circuit_breaker.failure_threshold
        );
        assert!(restored.enforce_secret_references);
    }

    #[test]
    fn test_behaviors_partial_override() {
        let yaml = r#"
retry_policy:
  max_attempts: 10
timeout_ms: 60000
"#;
        let b: ComponentBehaviors = serde_yaml::from_str(yaml).expect("partial deserialize");

        // Overridden fields
        assert_eq!(b.retry_policy.max_attempts, 10);
        assert_eq!(b.timeout_ms, 60_000);

        // Defaults preserved
        assert_eq!(b.retry_policy.backoff, BackoffStrategy::Exponential);
        assert_eq!(b.retry_policy.initial_interval_ms, 1000);
        assert_eq!(b.rate_limit.requests_per_second, 10);
        assert_eq!(b.circuit_breaker.failure_threshold, 5);
        assert!(b.idempotency.enabled);
        assert!(b.enforce_secret_references);
    }

    #[test]
    fn test_component_output_success() {
        let output = ComponentOutput {
            success: true,
            data: serde_json::json!({"order_id": "abc-123"}),
            error: None,
            metadata: OutputMetadata {
                duration_ms: 250,
                attempt_count: 1,
                component_id: "http_1".to_string(),
                idempotency_key: "key-001".to_string(),
            },
        };

        assert!(output.success);
        assert!(output.error.is_none());
        assert_eq!(output.metadata.duration_ms, 250);
        assert_eq!(output.metadata.attempt_count, 1);
        assert_eq!(output.metadata.component_id, "http_1");
        assert_eq!(output.data["order_id"], "abc-123");
    }

    #[test]
    fn test_component_output_failure() {
        let output = ComponentOutput {
            success: false,
            data: serde_json::Value::Null,
            error: Some(ComponentError {
                code: "TIMEOUT".to_string(),
                message: "Request timed out after 30s".to_string(),
                retryable: true,
                details: Some(serde_json::json!({"elapsed_ms": 30000})),
            }),
            metadata: OutputMetadata {
                duration_ms: 30_000,
                attempt_count: 3,
                component_id: "http_1".to_string(),
                idempotency_key: "key-002".to_string(),
            },
        };

        assert!(!output.success);
        assert!(output.error.is_some());
        let err = output.error.as_ref().unwrap();
        assert_eq!(err.code, "TIMEOUT");
        assert!(err.retryable);
        assert!(err.details.is_some());
        assert_eq!(output.metadata.attempt_count, 3);
    }

    #[test]
    fn test_error_catalog_entry() {
        let entry = ErrorCatalogEntry {
            code: "AUTH_FAILED".to_string(),
            retryable: false,
            description: "Authentication credentials were rejected".to_string(),
        };

        let yaml = serde_yaml::to_string(&entry).expect("serialize");
        let restored: ErrorCatalogEntry = serde_yaml::from_str(&yaml).expect("deserialize");

        assert_eq!(restored.code, "AUTH_FAILED");
        assert!(!restored.retryable);
        assert_eq!(restored.description, entry.description);
    }
}
