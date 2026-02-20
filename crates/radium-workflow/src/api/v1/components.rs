//! Component type registry.
//!
//! Exposes the 13 built-in workflow component types so that agents and the UI
//! can discover what is available without hard-coding knowledge of the schema.

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
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

/// Return the full list of the 13 built-in component types.
fn all_components() -> Vec<ComponentType> {
    vec![
        ComponentType {
            name: "trigger".to_string(),
            category: "control_flow".to_string(),
            description: "Entry point that starts a workflow execution. Defines what event or condition initiates the workflow.".to_string(),
            config_fields: vec![
                ConfigField {
                    name: "trigger_type".to_string(),
                    field_type: "string".to_string(),
                    required: false,
                    description: "The type of trigger (e.g. manual, schedule, webhook, signal).".to_string(),
                },
            ],
        },
        ComponentType {
            name: "stop".to_string(),
            category: "control_flow".to_string(),
            description: "Terminates the workflow execution. Every workflow path must eventually reach a stop node.".to_string(),
            config_fields: vec![],
        },
        ComponentType {
            name: "activity".to_string(),
            category: "activities".to_string(),
            description: "A unit of work executed as a Temporal activity. Supports configurable timeouts and retry policies.".to_string(),
            config_fields: vec![
                ConfigField {
                    name: "name".to_string(),
                    field_type: "string".to_string(),
                    required: true,
                    description: "The activity function name to invoke.".to_string(),
                },
                ConfigField {
                    name: "timeout".to_string(),
                    field_type: "string".to_string(),
                    required: false,
                    description: "Maximum duration the activity may run (e.g. '30s', '5m').".to_string(),
                },
                ConfigField {
                    name: "retry.max_attempts".to_string(),
                    field_type: "integer".to_string(),
                    required: false,
                    description: "Maximum number of retry attempts on failure.".to_string(),
                },
                ConfigField {
                    name: "retry.backoff".to_string(),
                    field_type: "string".to_string(),
                    required: false,
                    description: "Backoff strategy between retries (e.g. 'exponential', 'fixed').".to_string(),
                },
            ],
        },
        ComponentType {
            name: "http_request".to_string(),
            category: "activities".to_string(),
            description: "Makes an outbound HTTP request to an external service and captures the response.".to_string(),
            config_fields: vec![
                ConfigField {
                    name: "url".to_string(),
                    field_type: "string".to_string(),
                    required: true,
                    description: "The target URL for the HTTP request.".to_string(),
                },
                ConfigField {
                    name: "method".to_string(),
                    field_type: "string".to_string(),
                    required: true,
                    description: "HTTP method (GET, POST, PUT, PATCH, DELETE).".to_string(),
                },
                ConfigField {
                    name: "headers".to_string(),
                    field_type: "object".to_string(),
                    required: false,
                    description: "Key-value map of HTTP headers to include in the request.".to_string(),
                },
                ConfigField {
                    name: "body".to_string(),
                    field_type: "string".to_string(),
                    required: false,
                    description: "The request body (typically JSON-encoded).".to_string(),
                },
            ],
        },
        ComponentType {
            name: "database_query".to_string(),
            category: "activities".to_string(),
            description: "Executes a database operation (select, insert, update, delete) against a configured data store.".to_string(),
            config_fields: vec![
                ConfigField {
                    name: "operation".to_string(),
                    field_type: "string".to_string(),
                    required: true,
                    description: "The database operation type (select, insert, update, delete).".to_string(),
                },
                ConfigField {
                    name: "table".to_string(),
                    field_type: "string".to_string(),
                    required: true,
                    description: "The target database table name.".to_string(),
                },
                ConfigField {
                    name: "conditions".to_string(),
                    field_type: "object".to_string(),
                    required: false,
                    description: "Filter conditions for the query (e.g. WHERE clauses).".to_string(),
                },
            ],
        },
        ComponentType {
            name: "agent".to_string(),
            category: "activities".to_string(),
            description: "Invokes an AI agent with a prompt and optional tool access to perform intelligent processing.".to_string(),
            config_fields: vec![
                ConfigField {
                    name: "provider".to_string(),
                    field_type: "string".to_string(),
                    required: false,
                    description: "The AI provider to use (e.g. openai, anthropic).".to_string(),
                },
                ConfigField {
                    name: "model".to_string(),
                    field_type: "string".to_string(),
                    required: false,
                    description: "The specific model identifier (e.g. gpt-4, claude-3-opus).".to_string(),
                },
                ConfigField {
                    name: "prompt".to_string(),
                    field_type: "string".to_string(),
                    required: true,
                    description: "The prompt template sent to the agent. Supports variable interpolation.".to_string(),
                },
                ConfigField {
                    name: "tools".to_string(),
                    field_type: "array".to_string(),
                    required: false,
                    description: "List of tool definitions the agent may invoke.".to_string(),
                },
            ],
        },
        ComponentType {
            name: "conditional".to_string(),
            category: "control_flow".to_string(),
            description: "Evaluates a boolean expression and routes execution to the matching branch.".to_string(),
            config_fields: vec![
                ConfigField {
                    name: "expression".to_string(),
                    field_type: "string".to_string(),
                    required: true,
                    description: "The boolean expression to evaluate for branching.".to_string(),
                },
            ],
        },
        ComponentType {
            name: "loop".to_string(),
            category: "control_flow".to_string(),
            description: "Repeats a block of steps based on a collection, condition, or fixed count.".to_string(),
            config_fields: vec![
                ConfigField {
                    name: "loop_type".to_string(),
                    field_type: "string".to_string(),
                    required: true,
                    description: "The loop strategy: 'for_each', 'while', or 'count'.".to_string(),
                },
                ConfigField {
                    name: "items".to_string(),
                    field_type: "string".to_string(),
                    required: false,
                    description: "Expression resolving to the collection to iterate (for_each loops).".to_string(),
                },
                ConfigField {
                    name: "condition".to_string(),
                    field_type: "string".to_string(),
                    required: false,
                    description: "Boolean expression evaluated before each iteration (while loops).".to_string(),
                },
                ConfigField {
                    name: "count".to_string(),
                    field_type: "integer".to_string(),
                    required: false,
                    description: "Fixed number of iterations (count loops).".to_string(),
                },
            ],
        },
        ComponentType {
            name: "parallel".to_string(),
            category: "control_flow".to_string(),
            description: "Executes multiple branches concurrently and waits for them to complete based on a join strategy.".to_string(),
            config_fields: vec![
                ConfigField {
                    name: "join_strategy".to_string(),
                    field_type: "string".to_string(),
                    required: false,
                    description: "How to join parallel branches: 'all' (default), 'any', or 'n_of_m'.".to_string(),
                },
            ],
        },
        ComponentType {
            name: "signal".to_string(),
            category: "communication".to_string(),
            description: "Sends or receives an inter-workflow signal for coordination between running workflows.".to_string(),
            config_fields: vec![
                ConfigField {
                    name: "signal_name".to_string(),
                    field_type: "string".to_string(),
                    required: true,
                    description: "The name of the signal to send or wait for.".to_string(),
                },
                ConfigField {
                    name: "direction".to_string(),
                    field_type: "string".to_string(),
                    required: true,
                    description: "Whether this node sends ('send') or receives ('receive') the signal.".to_string(),
                },
            ],
        },
        ComponentType {
            name: "timer".to_string(),
            category: "control_flow".to_string(),
            description: "Pauses workflow execution for a duration or until a specific point in time.".to_string(),
            config_fields: vec![
                ConfigField {
                    name: "timer_type".to_string(),
                    field_type: "string".to_string(),
                    required: true,
                    description: "The timer mode: 'duration' or 'until'.".to_string(),
                },
                ConfigField {
                    name: "duration".to_string(),
                    field_type: "string".to_string(),
                    required: false,
                    description: "How long to wait (e.g. '30s', '1h'). Used with timer_type 'duration'.".to_string(),
                },
                ConfigField {
                    name: "until".to_string(),
                    field_type: "string".to_string(),
                    required: false,
                    description: "ISO-8601 timestamp to wait until. Used with timer_type 'until'.".to_string(),
                },
            ],
        },
        ComponentType {
            name: "child_workflow".to_string(),
            category: "orchestration".to_string(),
            description: "Starts a child workflow execution and optionally waits for its result.".to_string(),
            config_fields: vec![
                ConfigField {
                    name: "workflow_id".to_string(),
                    field_type: "string".to_string(),
                    required: true,
                    description: "The identifier of the child workflow definition to execute.".to_string(),
                },
                ConfigField {
                    name: "input".to_string(),
                    field_type: "object".to_string(),
                    required: false,
                    description: "Input data to pass to the child workflow.".to_string(),
                },
            ],
        },
        ComponentType {
            name: "log".to_string(),
            category: "observability".to_string(),
            description: "Emits a structured log entry for debugging and observability during workflow execution.".to_string(),
            config_fields: vec![
                ConfigField {
                    name: "level".to_string(),
                    field_type: "string".to_string(),
                    required: false,
                    description: "Log level: 'debug', 'info' (default), 'warn', or 'error'.".to_string(),
                },
                ConfigField {
                    name: "message".to_string(),
                    field_type: "string".to_string(),
                    required: true,
                    description: "The log message template. Supports variable interpolation.".to_string(),
                },
            ],
        },
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

    #[tokio::test]
    async fn test_list_components_returns_all_types() {
        let Json(components) = list_components().await;
        assert_eq!(
            components.len(),
            13,
            "Expected 13 component types, got {}",
            components.len()
        );

        let names: Vec<&str> = components.iter().map(|c| c.name.as_str()).collect();
        let expected = [
            "trigger",
            "stop",
            "activity",
            "http_request",
            "database_query",
            "agent",
            "conditional",
            "loop",
            "parallel",
            "signal",
            "timer",
            "child_workflow",
            "log",
        ];
        for name in &expected {
            assert!(
                names.contains(name),
                "Missing component type: {name}"
            );
        }
    }

    #[tokio::test]
    async fn test_get_component_found() {
        let result = get_component(Path("activity".to_string())).await;
        assert!(result.is_ok(), "Expected Ok for 'activity'");

        let Json(component) = result.unwrap();
        assert_eq!(component.name, "activity");
        assert_eq!(component.category, "activities");
        assert!(!component.description.is_empty());
        assert!(!component.config_fields.is_empty());
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
    async fn test_required_activity_has_name_field() {
        let result = get_component(Path("activity".to_string())).await;
        let Json(component) = result.unwrap();

        let name_field = component
            .config_fields
            .iter()
            .find(|f| f.name == "name");
        assert!(
            name_field.is_some(),
            "Activity component must have a 'name' config field"
        );
        assert!(
            name_field.unwrap().required,
            "Activity 'name' field must be required"
        );
    }

    #[tokio::test]
    async fn test_component_categories_are_valid() {
        let valid_categories = [
            "control_flow",
            "activities",
            "communication",
            "orchestration",
            "observability",
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
}
