//! Component type registry.
//!
//! Exposes the 37 built-in workflow component types so that agents and the UI
//! can discover what is available without hard-coding knowledge of the schema.
//!
//! Three legacy aliases (activity, child_workflow, signal) are also included
//! with `deprecated: true` so existing callers continue to resolve them while
//! being guided toward their canonical replacements.

use axum::{
    extract::Path,
    http::StatusCode,
    Json,
};
use serde::Serialize;

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

/// `GET /v1/components` -- list all available component types.
pub async fn list_components() -> Json<Vec<ComponentType>> {
    Json(all_components())
}

/// `GET /v1/components/:component_type` -- get a single component type by name.
pub async fn get_component(
    Path(component_type): Path<String>,
) -> Result<Json<ComponentType>, StatusCode> {
    all_components()
        .into_iter()
        .find(|c| c.name == component_type)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
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
        let valid_tiers = ["pure", "stateful", "io", "n/a"];
        let Json(components) = list_components().await;
        for component in &components {
            assert!(
                valid_tiers.contains(&component.behavior_tier.as_str()),
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
        let valid_categories = [
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
        let Json(components) = list_components().await;
        for component in &components {
            assert!(
                valid_categories.contains(&component.category.as_str()),
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
}
