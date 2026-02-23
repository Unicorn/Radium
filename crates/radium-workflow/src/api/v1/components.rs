//! Component type registry and custom component CRUD.
//!
//! Exposes the 37 built-in workflow component types so that agents and the UI
//! can discover what is available without hard-coding knowledge of the schema.
//!
//! Three legacy aliases (activity, child_workflow, signal) are also included
//! with `deprecated: true` so existing callers continue to resolve them while
//! being guided toward their canonical replacements.
//!
//! Additionally provides authenticated endpoints for creating, listing, and
//! deleting user-defined custom component types stored in Supabase.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::api::auth::{self, AuthenticatedUser};
use crate::api::state::AppState;
use crate::supabase::SupabaseError;
use crate::versioning::SemVer;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// The 11 valid component categories.
const VALID_CATEGORIES: &[&str] = &[
    "control_flow",
    "activities",
    "agent",
    "orchestration",
    "execution",
    "data",
    "security",
    "storage",
    "networking",
    "messaging",
    "flow_control",
];

/// The 4 valid behavior tiers.
const VALID_BEHAVIOR_TIERS: &[&str] = &["pure", "stateful", "io", "n/a"];

/// Default component_type_id for custom components.
const DEFAULT_COMPONENT_TYPE_ID: &str = "00000000-0000-0000-0000-000000000001";

/// Default visibility_id for custom components.
const DEFAULT_VISIBILITY_ID: &str = "00000000-0000-0000-0000-000000000001";

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// A single field within a component's configuration.
#[derive(Debug, Clone, Serialize)]
pub struct ConfigField {
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub description: String,
}

/// A workflow component type with its metadata and configuration schema.
#[derive(Debug, Clone, Serialize)]
pub struct ComponentType {
    pub name: String,
    pub category: String,
    pub description: String,
    pub config_fields: Vec<ConfigField>,
    /// Semantic version for this component definition.
    pub version: String,
    /// Execution tier: "pure", "stateful", "io", or "n/a".
    pub behavior_tier: String,
    /// `true` when this entry is a deprecated alias for a canonical name.
    pub deprecated: bool,
    /// For deprecated aliases, the preferred canonical component name.
    pub canonical_name: Option<String>,
}

// ---------------------------------------------------------------------------
// Request / response types for custom component CRUD
// ---------------------------------------------------------------------------

/// A config field in the create request body.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigFieldInput {
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub description: String,
}

/// Request body for `POST /v1/components`.
#[derive(Debug, Deserialize)]
pub struct CreateComponentRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub category: String,
    pub version: String,
    pub behavior_tier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
    #[serde(default)]
    pub config_fields: Vec<ConfigFieldInput>,
}

/// Row inserted into the Supabase `components` table.
#[derive(Debug, Serialize)]
struct InsertComponentRow {
    name: String,
    display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    component_type_id: String,
    version: String,
    created_by: String,
    visibility_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    config_schema: Option<serde_json::Value>,
}

/// Response from Supabase after inserting or reading a custom component.
#[derive(Debug, Serialize, Deserialize)]
pub struct CustomComponentResponse {
    pub id: String,
    pub name: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub component_type_id: String,
    pub version: String,
    pub created_by: String,
    pub visibility_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// List response envelope for custom components.
#[derive(Debug, Serialize)]
pub struct CustomComponentListResponse {
    pub components: Vec<CustomComponentResponse>,
    pub total: usize,
}

// ---------------------------------------------------------------------------
// Error type (same pattern as workflows.rs)
// ---------------------------------------------------------------------------

/// Structured error envelope matching the spec:
/// `{ "error": { "code": "...", "message": "...", "details": [...] } }`
#[derive(Debug, Serialize)]
struct ErrorBody {
    code: String,
    message: String,
    details: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

/// Handler-level error that converts into a JSON response automatically.
#[derive(Debug)]
pub struct ComponentError {
    status: StatusCode,
    code: String,
    message: String,
    details: Vec<String>,
}

impl ComponentError {
    fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "UNAUTHORIZED".to_string(),
            message: "Authorization header with Bearer token is required".to_string(),
            details: vec![],
        }
    }

    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "BAD_REQUEST".to_string(),
            message: message.into(),
            details: vec![],
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "NOT_FOUND".to_string(),
            message: message.into(),
            details: vec![],
        }
    }

    fn validation_failed(message: impl Into<String>, details: Vec<String>) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "VALIDATION_FAILED".to_string(),
            message: message.into(),
            details,
        }
    }

    fn conflict(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code: "CONFLICT".to_string(),
            message: message.into(),
            details: vec![],
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "INTERNAL_ERROR".to_string(),
            message: message.into(),
            details: vec![],
        }
    }

    fn from_supabase(err: &SupabaseError) -> Self {
        match err {
            SupabaseError::NotFound { .. } => Self::not_found(err.to_string()),
            SupabaseError::ApiError { status, .. } if *status == 404 => {
                Self::not_found(err.to_string())
            }
            _ => {
                tracing::error!("Supabase error: {err}");
                Self::internal("Database operation failed")
            }
        }
    }
}

impl IntoResponse for ComponentError {
    fn into_response(self) -> Response {
        let envelope = ErrorEnvelope {
            error: ErrorBody {
                code: self.code,
                message: self.message,
                details: self.details,
            },
        };
        (self.status, Json(envelope)).into_response()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a `ConfigField` in one line.
fn field(name: &str, field_type: &str, required: bool, description: &str) -> ConfigField {
    ConfigField {
        name: name.to_string(),
        field_type: field_type.to_string(),
        required,
        description: description.to_string(),
    }
}

/// Build a canonical (non-deprecated) `ComponentType`.
fn component(
    name: &str,
    category: &str,
    description: &str,
    behavior_tier: &str,
    config_fields: Vec<ConfigField>,
) -> ComponentType {
    ComponentType {
        name: name.to_string(),
        category: category.to_string(),
        description: description.to_string(),
        config_fields,
        version: "1.0.0".to_string(),
        behavior_tier: behavior_tier.to_string(),
        deprecated: false,
        canonical_name: None,
    }
}

/// Build a deprecated alias that points to a canonical component name.
fn deprecated_alias(
    alias: &str,
    canonical: &str,
    category: &str,
    description: &str,
    behavior_tier: &str,
    config_fields: Vec<ConfigField>,
) -> ComponentType {
    ComponentType {
        name: alias.to_string(),
        category: category.to_string(),
        description: description.to_string(),
        config_fields,
        version: "1.0.0".to_string(),
        behavior_tier: behavior_tier.to_string(),
        deprecated: true,
        canonical_name: Some(canonical.to_string()),
    }
}

/// Validate the Bearer token against the Supabase `api_keys` table and check
/// the per-user rate limit.
///
/// Returns the authenticated user on success, or a `ComponentError` when the
/// token is missing, invalid, expired, revoked, or rate-limited.
async fn require_auth(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<AuthenticatedUser, ComponentError> {
    let mut request = axum::http::Request::builder()
        .uri("http://localhost/")
        .body(())
        .unwrap();
    *request.headers_mut() = headers.clone();
    let (parts, ()) = request.into_parts();

    let token =
        auth::extract_bearer_token(&parts).ok_or_else(ComponentError::unauthorized)?;

    let user = auth::validate_api_key(
        state.supabase.http_client(),
        state.supabase.url(),
        state.supabase.service_role_key(),
        &token,
    )
    .await
    .map_err(|_| ComponentError::unauthorized())?;

    // Check rate limit (keyed by user_id).
    let result = state.rate_limiter.check(&user.user_id);
    if !result.allowed {
        return Err(ComponentError {
            status: StatusCode::TOO_MANY_REQUESTS,
            code: "RATE_LIMITED".to_string(),
            message: format!(
                "Rate limit exceeded. Try again in {} seconds.",
                result.reset_in_seconds()
            ),
            details: vec![],
        });
    }

    Ok(user)
}

/// Return the set of all built-in component names (canonical + deprecated aliases).
fn builtin_component_names() -> Vec<String> {
    all_components().into_iter().map(|c| c.name).collect()
}

/// Validate a `CreateComponentRequest` and return a list of validation errors.
/// Returns an empty vec if the request is valid.
pub fn validate_create_request(req: &CreateComponentRequest) -> Vec<String> {
    let mut errors = Vec::new();

    // name: required, non-empty
    if req.name.is_empty() {
        errors.push("'name' is required and must be non-empty".to_string());
    } else {
        // name: lowercase alphanumeric + underscores only
        let valid_name = req
            .name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        if !valid_name {
            errors.push(format!(
                "'name' must contain only lowercase alphanumeric characters and underscores, got: '{}'",
                req.name
            ));
        }

        // name: must not conflict with built-in names
        let builtins = builtin_component_names();
        if builtins.iter().any(|b| b == &req.name) {
            errors.push(format!(
                "'name' conflicts with built-in component '{}'. Choose a different name.",
                req.name
            ));
        }
    }

    // category: must be one of the 11 valid categories
    if !VALID_CATEGORIES.contains(&req.category.as_str()) {
        errors.push(format!(
            "'category' must be one of {:?}, got: '{}'",
            VALID_CATEGORIES, req.category
        ));
    }

    // behavior_tier: must be one of the 4 valid tiers
    if !VALID_BEHAVIOR_TIERS.contains(&req.behavior_tier.as_str()) {
        errors.push(format!(
            "'behavior_tier' must be one of {:?}, got: '{}'",
            VALID_BEHAVIOR_TIERS, req.behavior_tier
        ));
    }

    // version: must be valid semver
    if let Err(e) = SemVer::parse(&req.version) {
        errors.push(format!("'version' is not valid semver: {e}"));
    }

    errors
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

/// Return the full list of component types: 37 canonical + 3 deprecated aliases.
fn all_components() -> Vec<ComponentType> {
    vec![
        // ----------------------------------------------------------------
        // control_flow (behavior_tier = "n/a")
        // ----------------------------------------------------------------
        component(
            "trigger",
            "control_flow",
            "Entry point that starts a workflow execution. Defines what event or condition initiates the workflow.",
            "n/a",
            vec![
                field("trigger_type", "string", false, "The type of trigger (e.g. manual, schedule, webhook, signal)."),
            ],
        ),
        component(
            "start",
            "control_flow",
            "Marks the beginning of a workflow graph. Connects the trigger to the first processing node.",
            "n/a",
            vec![],
        ),
        component(
            "stop",
            "control_flow",
            "Terminates the workflow execution. Every workflow path must eventually reach a stop node.",
            "n/a",
            vec![
                field("status", "string", false, "Optional terminal status to emit: 'success' (default), 'failure', or 'cancelled'."),
            ],
        ),
        component(
            "conditional",
            "control_flow",
            "Evaluates a boolean expression and routes execution to the matching branch.",
            "n/a",
            vec![
                field("expression", "string", true, "The boolean expression to evaluate for branching."),
            ],
        ),
        component(
            "loop",
            "control_flow",
            "Repeats a block of steps based on a collection, condition, or fixed count.",
            "n/a",
            vec![
                field("loop_type", "string", true, "The loop strategy: 'for_each', 'while', or 'count'."),
                field("items", "string", false, "Expression resolving to the collection to iterate (for_each loops)."),
                field("condition", "string", false, "Boolean expression evaluated before each iteration (while loops)."),
            ],
        ),
        component(
            "parallel",
            "control_flow",
            "Executes multiple branches concurrently and waits for them to complete based on a join strategy.",
            "n/a",
            vec![
                field("join_strategy", "string", false, "How to join parallel branches: 'all' (default), 'any', or 'n_of_m'."),
            ],
        ),

        // ----------------------------------------------------------------
        // activities (behavior_tier = "io" or "pure")
        // ----------------------------------------------------------------
        component(
            "action",
            "activities",
            "A unit of work executed as a Temporal activity. Supports configurable timeouts and retry policies.",
            "io",
            vec![
                field("name", "string", true, "The activity function name to invoke."),
                field("timeout", "string", false, "Maximum duration the activity may run (e.g. '30s', '5m')."),
                field("retry.max_attempts", "integer", false, "Maximum number of retry attempts on failure."),
            ],
        ),
        component(
            "log",
            "activities",
            "Emits a structured log entry for debugging and observability during workflow execution.",
            "pure",
            vec![
                field("level", "string", false, "Log level: 'debug', 'info' (default), 'warn', or 'error'."),
                field("message", "string", true, "The log message template. Supports variable interpolation."),
            ],
        ),
        component(
            "http_request",
            "activities",
            "Makes an outbound HTTP request to an external service and captures the response.",
            "io",
            vec![
                field("url", "string", true, "The target URL for the HTTP request."),
                field("method", "string", true, "HTTP method (GET, POST, PUT, PATCH, DELETE)."),
                field("headers", "object", false, "Key-value map of HTTP headers to include in the request."),
            ],
        ),
        component(
            "database_query",
            "activities",
            "Executes a database operation (select, insert, update, delete) against a configured data store.",
            "io",
            vec![
                field("operation", "string", true, "The database operation type (select, insert, update, delete)."),
                field("table", "string", true, "The target database table name."),
                field("conditions", "object", false, "Filter conditions for the query (e.g. WHERE clauses)."),
            ],
        ),

        // ----------------------------------------------------------------
        // agent
        // ----------------------------------------------------------------
        component(
            "agent",
            "agent",
            "Invokes an AI agent with a prompt and optional tool access to perform intelligent processing.",
            "io",
            vec![
                field("provider", "string", false, "The AI provider to use (e.g. openai, anthropic)."),
                field("model", "string", false, "The specific model identifier (e.g. gpt-4, claude-3-opus)."),
                field("prompt", "string", true, "The prompt template sent to the agent. Supports variable interpolation."),
            ],
        ),

        // ----------------------------------------------------------------
        // orchestration
        // ----------------------------------------------------------------
        component(
            "child_service",
            "orchestration",
            "Starts a child workflow execution and optionally waits for its result.",
            "io",
            vec![
                field("workflow_id", "string", true, "The identifier of the child workflow definition to execute."),
                field("input", "object", false, "Input data to pass to the child workflow."),
            ],
        ),
        component(
            "message",
            "orchestration",
            "Sends or receives an inter-workflow message for coordination between running workflows.",
            "io",
            vec![
                field("signal_name", "string", true, "The name of the signal/message to send or wait for."),
                field("direction", "string", true, "Whether this node sends ('send') or receives ('receive') the message."),
            ],
        ),
        component(
            "timer",
            "orchestration",
            "Pauses workflow execution for a duration or until a specific point in time.",
            "stateful",
            vec![
                field("timer_type", "string", true, "The timer mode: 'duration' or 'until'."),
                field("duration", "string", false, "How long to wait (e.g. '30s', '1h'). Used with timer_type 'duration'."),
                field("until", "string", false, "ISO-8601 timestamp to wait until. Used with timer_type 'until'."),
            ],
        ),

        // ----------------------------------------------------------------
        // execution
        // ----------------------------------------------------------------
        component(
            "shell_execute",
            "execution",
            "Runs a shell command in a sandboxed environment and captures stdout/stderr.",
            "io",
            vec![
                field("command", "string", true, "The shell command to execute."),
                field("timeout", "string", false, "Maximum duration the command may run."),
            ],
        ),
        component(
            "npm_function",
            "execution",
            "Executes a Node.js function from a registered npm package as a workflow step.",
            "io",
            vec![
                field("package", "string", true, "The npm package name containing the function."),
                field("function", "string", true, "The exported function name to call."),
                field("args", "object", false, "Arguments to pass to the function."),
            ],
        ),
        component(
            "code_execute",
            "execution",
            "Executes an inline code snippet (JavaScript or Python) within a managed runtime.",
            "stateful",
            vec![
                field("language", "string", true, "Runtime language: 'javascript' or 'python'."),
                field("code", "string", true, "The source code to execute."),
            ],
        ),

        // ----------------------------------------------------------------
        // data
        // ----------------------------------------------------------------
        component(
            "data_transform",
            "data",
            "Applies a transformation expression to reshape or map data between workflow steps.",
            "pure",
            vec![
                field("expression", "string", true, "JSONPath or JMESPath expression defining the transformation."),
                field("output_key", "string", false, "Key under which the result is stored in the workflow context."),
            ],
        ),
        component(
            "schema_validate",
            "data",
            "Validates a data payload against a JSON Schema and fails the step on mismatch.",
            "pure",
            vec![
                field("schema", "object", true, "The JSON Schema definition to validate against."),
                field("input", "string", false, "Expression referencing the data to validate."),
            ],
        ),
        component(
            "encode_decode",
            "data",
            "Encodes or decodes a value using a specified format (base64, hex, url, etc.).",
            "pure",
            vec![
                field("format", "string", true, "Encoding format: 'base64', 'hex', or 'url'."),
                field("direction", "string", true, "Operation direction: 'encode' or 'decode'."),
                field("value", "string", true, "The value to encode or decode."),
            ],
        ),

        // ----------------------------------------------------------------
        // security
        // ----------------------------------------------------------------
        component(
            "secret_read",
            "security",
            "Reads a named secret from the configured secrets backend and injects it into the workflow context.",
            "io",
            vec![
                field("secret_name", "string", true, "The name of the secret to retrieve."),
                field("output_key", "string", false, "Context key under which the secret value is stored."),
            ],
        ),
        component(
            "oauth_token",
            "security",
            "Fetches or refreshes an OAuth2 access token using a configured credential set.",
            "io",
            vec![
                field("provider", "string", true, "The OAuth2 provider configuration name."),
                field("scopes", "array", false, "List of OAuth2 scopes to request."),
            ],
        ),
        component(
            "jwt_create",
            "security",
            "Creates a signed JWT token from a claims payload and a configured signing key.",
            "pure",
            vec![
                field("claims", "object", true, "The JWT claims to embed in the token."),
                field("algorithm", "string", false, "Signing algorithm: 'HS256' (default), 'RS256', etc."),
            ],
        ),

        // ----------------------------------------------------------------
        // storage
        // ----------------------------------------------------------------
        component(
            "cache",
            "storage",
            "Reads from or writes to a distributed cache (e.g. Redis) keyed by a workflow variable.",
            "io",
            vec![
                field("operation", "string", true, "Cache operation: 'get', 'set', or 'delete'."),
                field("key", "string", true, "The cache key expression."),
                field("ttl", "string", false, "Time-to-live for set operations (e.g. '5m', '1h')."),
            ],
        ),
        component(
            "file_write",
            "storage",
            "Writes content to a file in the configured storage backend.",
            "io",
            vec![
                field("path", "string", true, "Destination file path within the storage backend."),
                field("content", "string", true, "The content to write. Supports variable interpolation."),
            ],
        ),
        component(
            "file_read",
            "storage",
            "Reads a file from the configured storage backend into the workflow context.",
            "io",
            vec![
                field("path", "string", true, "Source file path within the storage backend."),
                field("output_key", "string", false, "Context key under which the file content is stored."),
            ],
        ),
        component(
            "object_storage",
            "storage",
            "Interacts with an S3-compatible object store (upload, download, delete, list).",
            "io",
            vec![
                field("operation", "string", true, "Object operation: 'upload', 'download', 'delete', or 'list'."),
                field("bucket", "string", true, "The target bucket name."),
                field("key", "string", false, "Object key within the bucket."),
            ],
        ),

        // ----------------------------------------------------------------
        // networking
        // ----------------------------------------------------------------
        component(
            "graphql_request",
            "networking",
            "Executes a GraphQL query or mutation against a configured endpoint.",
            "io",
            vec![
                field("url", "string", true, "The GraphQL endpoint URL."),
                field("query", "string", true, "The GraphQL query or mutation string."),
                field("variables", "object", false, "Variable bindings for the query."),
            ],
        ),
        component(
            "grpc_call",
            "networking",
            "Invokes a gRPC service method and returns the response message.",
            "io",
            vec![
                field("service", "string", true, "The fully-qualified gRPC service name."),
                field("method", "string", true, "The RPC method to call."),
                field("payload", "object", false, "The request message payload."),
            ],
        ),
        component(
            "websocket",
            "networking",
            "Opens or interacts with a WebSocket connection for real-time bidirectional communication.",
            "io",
            vec![
                field("url", "string", true, "The WebSocket server URL."),
                field("action", "string", true, "Action: 'connect', 'send', 'receive', or 'close'."),
            ],
        ),
        component(
            "smtp_send",
            "networking",
            "Sends an email via SMTP using a configured mail provider.",
            "io",
            vec![
                field("to", "string", true, "Recipient email address(es)."),
                field("subject", "string", true, "Email subject line."),
                field("body", "string", true, "Email body. Supports HTML or plain text."),
            ],
        ),

        // ----------------------------------------------------------------
        // messaging
        // ----------------------------------------------------------------
        component(
            "webhook_send",
            "messaging",
            "Delivers a payload to an external webhook URL.",
            "io",
            vec![
                field("url", "string", true, "The destination webhook URL."),
                field("payload", "object", false, "The JSON payload to send."),
            ],
        ),
        component(
            "queue_publish",
            "messaging",
            "Publishes a message to a queue or topic (e.g. SQS, Kafka, RabbitMQ).",
            "io",
            vec![
                field("queue", "string", true, "The queue or topic name."),
                field("message", "object", true, "The message payload to publish."),
            ],
        ),
        component(
            "queue_consume",
            "messaging",
            "Reads one or more messages from a queue and injects them into the workflow context.",
            "io",
            vec![
                field("queue", "string", true, "The queue or topic name to consume from."),
                field("max_messages", "integer", false, "Maximum number of messages to read per invocation."),
            ],
        ),
        component(
            "event_emit",
            "messaging",
            "Emits a named domain event that other workflows or external subscribers can react to.",
            "io",
            vec![
                field("event_name", "string", true, "The name of the event to emit."),
                field("payload", "object", false, "Event data to attach to the emitted event."),
            ],
        ),

        // ----------------------------------------------------------------
        // flow_control
        // ----------------------------------------------------------------
        component(
            "delay",
            "flow_control",
            "Inserts a fixed pause into workflow execution without consuming a Temporal timer resource.",
            "stateful",
            vec![
                field("duration", "string", true, "How long to delay (e.g. '5s', '2m')."),
            ],
        ),
        component(
            "batch",
            "flow_control",
            "Collects items until a size or time threshold is met, then forwards them as a single batch.",
            "io",
            vec![
                field("max_size", "integer", false, "Maximum number of items in a batch before flushing."),
                field("max_wait", "string", false, "Maximum time to wait before flushing an incomplete batch (e.g. '10s')."),
            ],
        ),

        // ----------------------------------------------------------------
        // Deprecated aliases (deprecated: true)
        // ----------------------------------------------------------------
        deprecated_alias(
            "activity",
            "action",
            "activities",
            "Deprecated alias for 'action'. Use 'action' instead.",
            "io",
            vec![
                field("name", "string", true, "The activity function name to invoke."),
                field("timeout", "string", false, "Maximum duration the activity may run (e.g. '30s', '5m')."),
                field("retry.max_attempts", "integer", false, "Maximum number of retry attempts on failure."),
            ],
        ),
        deprecated_alias(
            "child_workflow",
            "child_service",
            "orchestration",
            "Deprecated alias for 'child_service'. Use 'child_service' instead.",
            "io",
            vec![
                field("workflow_id", "string", true, "The identifier of the child workflow definition to execute."),
                field("input", "object", false, "Input data to pass to the child workflow."),
            ],
        ),
        deprecated_alias(
            "signal",
            "message",
            "orchestration",
            "Deprecated alias for 'message'. Use 'message' instead.",
            "io",
            vec![
                field("signal_name", "string", true, "The name of the signal to send or wait for."),
                field("direction", "string", true, "Whether this node sends ('send') or receives ('receive') the signal."),
            ],
        ),
    ]
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /v1/components` -- list all available built-in component types.
pub async fn list_components() -> Json<Vec<ComponentType>> {
    Json(all_components())
}

/// `GET /v1/components/custom` -- list user's custom components from Supabase.
pub async fn list_custom_components(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<CustomComponentListResponse>, ComponentError> {
    let user = require_auth(&headers, &state).await?;

    let user_filter = format!("eq.{}", user.user_id);
    let components: Vec<CustomComponentResponse> = state
        .supabase
        .select(
            "components",
            &[
                ("created_by", &user_filter),
                ("is_active", "eq.true"),
                ("order", "created_at.desc"),
            ],
        )
        .await
        .map_err(|e| ComponentError::from_supabase(&e))?;

    let total = components.len();
    Ok(Json(CustomComponentListResponse { components, total }))
}

/// `GET /v1/components/:component_type` -- get a single built-in component type by name.
pub async fn get_component(
    Path(component_type): Path<String>,
) -> Result<Json<ComponentType>, StatusCode> {
    all_components()
        .into_iter()
        .find(|c| c.name == component_type)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// `POST /v1/components` -- create a custom component type in Supabase.
pub async fn create_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateComponentRequest>,
) -> Result<impl IntoResponse, ComponentError> {
    let user = require_auth(&headers, &state).await?;

    // Validate the request.
    let validation_errors = validate_create_request(&req);
    if !validation_errors.is_empty() {
        return Err(ComponentError::validation_failed(
            "Component validation failed",
            validation_errors,
        ));
    }

    // Serialize config_fields to JSONB for the config_schema column.
    let config_schema = if req.config_fields.is_empty() {
        None
    } else {
        Some(serde_json::to_value(&req.config_fields).map_err(|e| {
            ComponentError::internal(format!("Failed to serialize config_fields: {e}"))
        })?)
    };

    let display_name = req
        .display_name
        .unwrap_or_else(|| req.name.clone());

    let row = InsertComponentRow {
        name: req.name,
        display_name,
        description: req.description,
        component_type_id: DEFAULT_COMPONENT_TYPE_ID.to_string(),
        version: req.version,
        created_by: user.user_id,
        visibility_id: DEFAULT_VISIBILITY_ID.to_string(),
        input_schema: req.input_schema,
        output_schema: req.output_schema,
        config_schema,
    };

    let created: CustomComponentResponse = state
        .supabase
        .insert("components", &row)
        .await
        .map_err(|e| {
            // Check for unique constraint violations (duplicate name).
            if let SupabaseError::ApiError { status, ref message } = e {
                if status == 409 || message.contains("duplicate") || message.contains("unique") {
                    return ComponentError::conflict(format!(
                        "A component with name '{}' already exists",
                        row.name
                    ));
                }
            }
            ComponentError::from_supabase(&e)
        })?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// `DELETE /v1/components/custom/:name` -- delete a user's custom component.
pub async fn delete_custom_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Result<StatusCode, ComponentError> {
    let user = require_auth(&headers, &state).await?;

    // First, verify the component exists and belongs to the user.
    let user_filter = format!("eq.{}", user.user_id);
    let name_filter = format!("eq.{name}");
    let existing: Vec<CustomComponentResponse> = state
        .supabase
        .select(
            "components",
            &[
                ("name", &name_filter),
                ("created_by", &user_filter),
                ("select", "id,name,display_name,component_type_id,version,created_by,visibility_id"),
            ],
        )
        .await
        .map_err(|e| ComponentError::from_supabase(&e))?;

    if existing.is_empty() {
        return Err(ComponentError::not_found(format!(
            "Custom component '{name}' not found"
        )));
    }

    // Delete the component.
    state
        .supabase
        .delete(
            "components",
            &[("name", &name_filter), ("created_by", &user_filter)],
        )
        .await
        .map_err(|e| ComponentError::from_supabase(&e))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// 37 canonical components + 3 deprecated aliases = 40 total.
    const EXPECTED_TOTAL: usize = 40;
    const EXPECTED_CANONICAL: usize = 37;

    // -----------------------------------------------------------------------
    // Built-in registry tests (unchanged)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_list_components_returns_all_types() {
        let Json(components) = list_components().await;
        assert_eq!(
            components.len(),
            EXPECTED_TOTAL,
            "Expected {EXPECTED_TOTAL} component entries (37 canonical + 3 deprecated aliases), got {}",
            components.len()
        );

        // Verify all 37 canonical names are present.
        let canonical_names: Vec<&str> = components
            .iter()
            .filter(|c| !c.deprecated)
            .map(|c| c.name.as_str())
            .collect();
        assert_eq!(
            canonical_names.len(),
            EXPECTED_CANONICAL,
            "Expected {EXPECTED_CANONICAL} canonical components"
        );

        let expected_canonical = [
            // control_flow
            "trigger", "start", "stop", "conditional", "loop", "parallel",
            // activities
            "action", "log", "http_request", "database_query",
            // agent
            "agent",
            // orchestration
            "child_service", "message", "timer",
            // execution
            "shell_execute", "npm_function", "code_execute",
            // data
            "data_transform", "schema_validate", "encode_decode",
            // security
            "secret_read", "oauth_token", "jwt_create",
            // storage
            "cache", "file_write", "file_read", "object_storage",
            // networking
            "graphql_request", "grpc_call", "websocket", "smtp_send",
            // messaging
            "webhook_send", "queue_publish", "queue_consume", "event_emit",
            // flow_control
            "delay", "batch",
        ];
        for name in &expected_canonical {
            assert!(
                canonical_names.contains(name),
                "Missing canonical component type: {name}"
            );
        }

        // Verify the 3 deprecated aliases are present.
        let deprecated_names: Vec<&str> = components
            .iter()
            .filter(|c| c.deprecated)
            .map(|c| c.name.as_str())
            .collect();
        assert_eq!(deprecated_names.len(), 3, "Expected 3 deprecated aliases");
        for alias in &["activity", "child_workflow", "signal"] {
            assert!(
                deprecated_names.contains(alias),
                "Missing deprecated alias: {alias}"
            );
        }
    }

    #[tokio::test]
    async fn test_get_component_found() {
        // Canonical name lookup.
        let result = get_component(Path("action".to_string())).await;
        assert!(result.is_ok(), "Expected Ok for 'action'");
        let Json(component) = result.unwrap();
        assert_eq!(component.name, "action");
        assert_eq!(component.category, "activities");
        assert!(!component.description.is_empty());
        assert!(!component.config_fields.is_empty());
    }

    #[tokio::test]
    async fn test_get_deprecated_alias_resolves() {
        // Deprecated alias lookup still resolves.
        let result = get_component(Path("activity".to_string())).await;
        assert!(result.is_ok(), "Expected Ok for deprecated alias 'activity'");
        let Json(component) = result.unwrap();
        assert!(component.deprecated, "'activity' must be marked deprecated");
        assert_eq!(
            component.canonical_name.as_deref(),
            Some("action"),
            "'activity' must point to canonical name 'action'"
        );
    }

    #[tokio::test]
    async fn test_get_component_not_found() {
        let result = get_component(Path("nonexistent".to_string())).await;
        assert!(result.is_err(), "Expected Err for 'nonexistent'");
        assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_all_components_have_descriptions() {
        let Json(components) = list_components().await;
        for component in &components {
            assert!(
                !component.description.is_empty(),
                "Component '{}' has an empty description",
                component.name
            );
        }
    }

    #[tokio::test]
    async fn test_all_components_have_valid_version() {
        let Json(components) = list_components().await;
        for component in &components {
            assert_eq!(
                component.version, "1.0.0",
                "Component '{}' must have version '1.0.0'",
                component.name
            );
        }
    }

    #[tokio::test]
    async fn test_all_components_have_valid_behavior_tier() {
        let Json(components) = list_components().await;
        for component in &components {
            assert!(
                VALID_BEHAVIOR_TIERS.contains(&component.behavior_tier.as_str()),
                "Component '{}' has unexpected behavior_tier '{}'",
                component.name,
                component.behavior_tier,
            );
        }
    }

    #[tokio::test]
    async fn test_control_flow_components_are_na_tier() {
        let Json(components) = list_components().await;
        let control_flow: Vec<&ComponentType> = components
            .iter()
            .filter(|c| c.category == "control_flow" && !c.deprecated)
            .collect();
        assert!(!control_flow.is_empty(), "Expected at least one control_flow component");
        for c in &control_flow {
            assert_eq!(
                c.behavior_tier, "n/a",
                "Control flow component '{}' must have behavior_tier 'n/a'",
                c.name
            );
        }
    }

    #[tokio::test]
    async fn test_deprecated_aliases_have_canonical_names() {
        let Json(components) = list_components().await;
        for component in components.iter().filter(|c| c.deprecated) {
            assert!(
                component.canonical_name.is_some(),
                "Deprecated component '{}' must have a canonical_name",
                component.name
            );
        }
        // Non-deprecated components must NOT have a canonical_name.
        for component in components.iter().filter(|c| !c.deprecated) {
            assert!(
                component.canonical_name.is_none(),
                "Canonical component '{}' must not have a canonical_name set",
                component.name
            );
        }
    }

    #[tokio::test]
    async fn test_required_action_has_name_field() {
        let result = get_component(Path("action".to_string())).await;
        let Json(component) = result.unwrap();

        let name_field = component
            .config_fields
            .iter()
            .find(|f| f.name == "name");
        assert!(
            name_field.is_some(),
            "Action component must have a 'name' config field"
        );
        assert!(
            name_field.unwrap().required,
            "Action 'name' field must be required"
        );
    }

    #[tokio::test]
    async fn test_component_categories_are_valid() {
        let Json(components) = list_components().await;
        for component in &components {
            assert!(
                VALID_CATEGORIES.contains(&component.category.as_str()),
                "Component '{}' has unexpected category '{}'",
                component.name,
                component.category,
            );
        }
    }

    #[tokio::test]
    async fn test_no_duplicate_names() {
        let Json(components) = list_components().await;
        let mut seen = std::collections::HashSet::new();
        for component in &components {
            assert!(
                seen.insert(component.name.clone()),
                "Duplicate component name found: '{}'",
                component.name
            );
        }
    }

    // -----------------------------------------------------------------------
    // CreateComponentRequest validation tests
    // -----------------------------------------------------------------------

    /// Helper to build a valid CreateComponentRequest for tests.
    fn valid_request() -> CreateComponentRequest {
        CreateComponentRequest {
            name: "my_custom_step".to_string(),
            display_name: Some("My Custom Step".to_string()),
            description: Some("Does something custom".to_string()),
            category: "activities".to_string(),
            version: "1.0.0".to_string(),
            behavior_tier: "io".to_string(),
            input_schema: None,
            output_schema: None,
            config_fields: vec![ConfigFieldInput {
                name: "url".to_string(),
                field_type: "string".to_string(),
                required: true,
                description: "Target URL".to_string(),
            }],
        }
    }

    #[test]
    fn test_validate_valid_request() {
        let req = valid_request();
        let errors = validate_create_request(&req);
        assert!(
            errors.is_empty(),
            "Expected no validation errors, got: {errors:?}"
        );
    }

    #[test]
    fn test_validate_empty_name() {
        let mut req = valid_request();
        req.name = "".to_string();
        let errors = validate_create_request(&req);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("non-empty"));
    }

    #[test]
    fn test_validate_name_with_uppercase() {
        let mut req = valid_request();
        req.name = "MyStep".to_string();
        let errors = validate_create_request(&req);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("lowercase"));
    }

    #[test]
    fn test_validate_name_with_hyphens() {
        let mut req = valid_request();
        req.name = "my-step".to_string();
        let errors = validate_create_request(&req);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("lowercase alphanumeric"));
    }

    #[test]
    fn test_validate_name_with_spaces() {
        let mut req = valid_request();
        req.name = "my step".to_string();
        let errors = validate_create_request(&req);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("lowercase alphanumeric"));
    }

    #[test]
    fn test_validate_name_with_underscores_ok() {
        let mut req = valid_request();
        req.name = "my_custom_step_v2".to_string();
        let errors = validate_create_request(&req);
        assert!(errors.is_empty(), "Underscores should be allowed: {errors:?}");
    }

    #[test]
    fn test_validate_name_with_digits_ok() {
        let mut req = valid_request();
        req.name = "step123".to_string();
        let errors = validate_create_request(&req);
        assert!(errors.is_empty(), "Digits should be allowed: {errors:?}");
    }

    #[test]
    fn test_validate_name_conflicts_with_builtin_canonical() {
        let mut req = valid_request();
        req.name = "trigger".to_string();
        let errors = validate_create_request(&req);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("conflicts with built-in"));
    }

    #[test]
    fn test_validate_name_conflicts_with_builtin_action() {
        let mut req = valid_request();
        req.name = "action".to_string();
        let errors = validate_create_request(&req);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("conflicts with built-in"));
    }

    #[test]
    fn test_validate_name_conflicts_with_deprecated_alias() {
        let mut req = valid_request();
        req.name = "activity".to_string();
        let errors = validate_create_request(&req);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("conflicts with built-in"));
    }

    #[test]
    fn test_validate_name_conflicts_with_child_workflow_alias() {
        let mut req = valid_request();
        req.name = "child_workflow".to_string();
        let errors = validate_create_request(&req);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("conflicts with built-in"));
    }

    #[test]
    fn test_validate_name_conflicts_with_signal_alias() {
        let mut req = valid_request();
        req.name = "signal".to_string();
        let errors = validate_create_request(&req);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("conflicts with built-in"));
    }

    #[test]
    fn test_validate_all_37_canonical_names_conflict() {
        let builtins = builtin_component_names();
        let canonical_count = all_components()
            .iter()
            .filter(|c| !c.deprecated)
            .count();
        assert_eq!(canonical_count, 37);

        for name in &builtins {
            let mut req = valid_request();
            req.name = name.clone();
            let errors = validate_create_request(&req);
            assert!(
                errors.iter().any(|e| e.contains("conflicts with built-in")),
                "Built-in name '{}' should be rejected",
                name
            );
        }
    }

    #[test]
    fn test_validate_invalid_category() {
        let mut req = valid_request();
        req.category = "invalid_category".to_string();
        let errors = validate_create_request(&req);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("category"));
    }

    #[test]
    fn test_validate_all_valid_categories_accepted() {
        for &cat in VALID_CATEGORIES {
            let mut req = valid_request();
            req.category = cat.to_string();
            let errors = validate_create_request(&req);
            assert!(
                errors.is_empty(),
                "Category '{}' should be valid, got: {:?}",
                cat,
                errors
            );
        }
    }

    #[test]
    fn test_validate_invalid_behavior_tier() {
        let mut req = valid_request();
        req.behavior_tier = "unknown".to_string();
        let errors = validate_create_request(&req);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("behavior_tier"));
    }

    #[test]
    fn test_validate_all_valid_behavior_tiers_accepted() {
        for &tier in VALID_BEHAVIOR_TIERS {
            let mut req = valid_request();
            req.behavior_tier = tier.to_string();
            let errors = validate_create_request(&req);
            assert!(
                errors.is_empty(),
                "Behavior tier '{}' should be valid, got: {:?}",
                tier,
                errors
            );
        }
    }

    #[test]
    fn test_validate_invalid_semver() {
        let mut req = valid_request();
        req.version = "not.a.version".to_string();
        let errors = validate_create_request(&req);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("semver"));
    }

    #[test]
    fn test_validate_invalid_semver_too_few_parts() {
        let mut req = valid_request();
        req.version = "1.0".to_string();
        let errors = validate_create_request(&req);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("semver"));
    }

    #[test]
    fn test_validate_empty_version() {
        let mut req = valid_request();
        req.version = "".to_string();
        let errors = validate_create_request(&req);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("semver"));
    }

    #[test]
    fn test_validate_valid_semver_prerelease() {
        let mut req = valid_request();
        req.version = "1.0.0-beta.1".to_string();
        let errors = validate_create_request(&req);
        assert!(errors.is_empty(), "Pre-release semver should be valid: {errors:?}");
    }

    #[test]
    fn test_validate_multiple_errors() {
        let req = CreateComponentRequest {
            name: "".to_string(),
            display_name: None,
            description: None,
            category: "bad".to_string(),
            version: "nope".to_string(),
            behavior_tier: "wrong".to_string(),
            input_schema: None,
            output_schema: None,
            config_fields: vec![],
        };
        let errors = validate_create_request(&req);
        // Should have errors for: name (empty), category, behavior_tier, version
        assert_eq!(
            errors.len(),
            4,
            "Expected 4 validation errors, got {}: {:?}",
            errors.len(),
            errors
        );
    }

    // -----------------------------------------------------------------------
    // Error response formatting
    // -----------------------------------------------------------------------

    #[test]
    fn test_error_envelope_serialization() {
        let envelope = ErrorEnvelope {
            error: ErrorBody {
                code: "VALIDATION_FAILED".to_string(),
                message: "Something went wrong".to_string(),
                details: vec!["detail 1".to_string()],
            },
        };

        let json = serde_json::to_value(&envelope).unwrap();
        assert_eq!(json["error"]["code"], "VALIDATION_FAILED");
        assert_eq!(json["error"]["message"], "Something went wrong");
        assert_eq!(json["error"]["details"][0], "detail 1");
    }

    #[test]
    fn test_component_error_unauthorized() {
        let err = ComponentError::unauthorized();
        assert_eq!(err.status, StatusCode::UNAUTHORIZED);
        assert_eq!(err.code, "UNAUTHORIZED");
    }

    #[test]
    fn test_component_error_bad_request() {
        let err = ComponentError::bad_request("invalid input");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.code, "BAD_REQUEST");
        assert_eq!(err.message, "invalid input");
    }

    #[test]
    fn test_component_error_not_found() {
        let err = ComponentError::not_found("Component 'abc' not found");
        assert_eq!(err.status, StatusCode::NOT_FOUND);
        assert_eq!(err.code, "NOT_FOUND");
    }

    #[test]
    fn test_component_error_validation_failed() {
        let err = ComponentError::validation_failed(
            "Validation failed",
            vec!["error 1".to_string(), "error 2".to_string()],
        );
        assert_eq!(err.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(err.code, "VALIDATION_FAILED");
        assert_eq!(err.details.len(), 2);
    }

    #[test]
    fn test_component_error_conflict() {
        let err = ComponentError::conflict("Name already exists");
        assert_eq!(err.status, StatusCode::CONFLICT);
        assert_eq!(err.code, "CONFLICT");
    }

    #[test]
    fn test_component_error_internal() {
        let err = ComponentError::internal("Something broke");
        assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.code, "INTERNAL_ERROR");
    }

    // -----------------------------------------------------------------------
    // Request deserialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_create_request_deserialization() {
        let json = r#"{
            "name": "my_custom_step",
            "display_name": "My Custom Step",
            "description": "Does something custom",
            "category": "activities",
            "version": "1.0.0",
            "behavior_tier": "io",
            "input_schema": { "type": "object" },
            "output_schema": { "type": "object" },
            "config_fields": [
                { "name": "url", "field_type": "string", "required": true, "description": "Target URL" }
            ]
        }"#;

        let req: CreateComponentRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "my_custom_step");
        assert_eq!(req.display_name, Some("My Custom Step".to_string()));
        assert_eq!(req.description, Some("Does something custom".to_string()));
        assert_eq!(req.category, "activities");
        assert_eq!(req.version, "1.0.0");
        assert_eq!(req.behavior_tier, "io");
        assert!(req.input_schema.is_some());
        assert!(req.output_schema.is_some());
        assert_eq!(req.config_fields.len(), 1);
        assert_eq!(req.config_fields[0].name, "url");
    }

    #[test]
    fn test_create_request_minimal_deserialization() {
        let json = r#"{
            "name": "minimal_step",
            "category": "data",
            "version": "1.0.0",
            "behavior_tier": "pure"
        }"#;

        let req: CreateComponentRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "minimal_step");
        assert!(req.display_name.is_none());
        assert!(req.description.is_none());
        assert!(req.input_schema.is_none());
        assert!(req.output_schema.is_none());
        assert!(req.config_fields.is_empty());
    }

    // -----------------------------------------------------------------------
    // Response serialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_custom_component_response_serialization() {
        let resp = CustomComponentResponse {
            id: "some-uuid".to_string(),
            name: "my_step".to_string(),
            display_name: "My Step".to_string(),
            description: Some("A step".to_string()),
            component_type_id: DEFAULT_COMPONENT_TYPE_ID.to_string(),
            version: "1.0.0".to_string(),
            created_by: "user-uuid".to_string(),
            visibility_id: DEFAULT_VISIBILITY_ID.to_string(),
            input_schema: None,
            output_schema: None,
            config_schema: None,
            is_active: Some(true),
            deprecated: Some(false),
            created_at: Some("2026-01-01T00:00:00Z".to_string()),
            updated_at: Some("2026-01-01T00:00:00Z".to_string()),
        };

        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["id"], "some-uuid");
        assert_eq!(json["name"], "my_step");
        assert_eq!(json["display_name"], "My Step");
        assert_eq!(json["description"], "A step");
        // Optional None fields should be omitted
        assert!(json.get("input_schema").is_none());
        assert!(json.get("output_schema").is_none());
        assert!(json.get("config_schema").is_none());
    }

    #[test]
    fn test_custom_component_list_response_serialization() {
        let resp = CustomComponentListResponse {
            components: vec![],
            total: 0,
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["total"], 0);
        assert!(json["components"].as_array().unwrap().is_empty());
    }

    // -----------------------------------------------------------------------
    // Built-in name registry completeness
    // -----------------------------------------------------------------------

    #[test]
    fn test_builtin_names_includes_all_40() {
        let names = builtin_component_names();
        assert_eq!(
            names.len(),
            EXPECTED_TOTAL,
            "builtin_component_names should return {EXPECTED_TOTAL} names, got {}",
            names.len()
        );
    }

    #[test]
    fn test_builtin_names_includes_deprecated_aliases() {
        let names = builtin_component_names();
        assert!(names.contains(&"activity".to_string()));
        assert!(names.contains(&"child_workflow".to_string()));
        assert!(names.contains(&"signal".to_string()));
    }

    // -----------------------------------------------------------------------
    // Config field serialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_config_field_input_serializes_to_json() {
        let fields = vec![
            ConfigFieldInput {
                name: "url".to_string(),
                field_type: "string".to_string(),
                required: true,
                description: "Target URL".to_string(),
            },
            ConfigFieldInput {
                name: "timeout".to_string(),
                field_type: "integer".to_string(),
                required: false,
                description: "Timeout in seconds".to_string(),
            },
        ];
        let json = serde_json::to_value(&fields).unwrap();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 2);
        assert_eq!(json[0]["name"], "url");
        assert_eq!(json[0]["required"], true);
        assert_eq!(json[1]["name"], "timeout");
        assert_eq!(json[1]["required"], false);
    }

    // -----------------------------------------------------------------------
    // Integration tests that need Supabase (marked #[ignore])
    // -----------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_create_component_integration() {
        // This test would POST a custom component to a real Supabase instance.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_list_custom_components_integration() {
        // This test would list custom components from a real Supabase instance.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_delete_custom_component_integration() {
        // This test would delete a custom component from a real Supabase instance.
    }
}
