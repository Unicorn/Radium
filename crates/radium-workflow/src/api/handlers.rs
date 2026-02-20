//! API request handlers

use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::codegen::{self, GeneratedCode};
use crate::schema::WorkflowDefinition;
use crate::validation::{self, ValidationError};

/// Compile request body
#[derive(Debug, Deserialize)]
pub struct CompileRequest {
    pub workflow: WorkflowDefinition,
    #[serde(default)]
    pub options: CompileOptions,
}

/// Compile options
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompileOptions {
    pub strict_mode: Option<bool>,
    pub include_comments: Option<bool>,
    pub skip_verification: Option<bool>,
}

/// Compile response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<GeneratedCode>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<CompilerError>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    pub metadata: CompileMetadata,
}

/// Compiler error response format
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompilerError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    pub severity: String,
}

/// Compile metadata
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileMetadata {
    pub node_count: usize,
    pub edge_count: usize,
    pub compilation_time_ms: u64,
    pub version: String,
}

/// Validation response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateResponse {
    pub valid: bool,
    pub errors: Vec<CompilerError>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
}

// Track startup time for uptime calculation
static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

fn get_uptime() -> u64 {
    START_TIME
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_secs()
}

/// POST /compile - Compile a workflow definition
pub async fn compile(
    Json(request): Json<CompileRequest>,
) -> Result<Json<CompileResponse>, (StatusCode, Json<CompileResponse>)> {
    let start = std::time::Instant::now();

    // Validate first
    let validation_result = validation::validate(&request.workflow);

    if !validation_result.is_valid() {
        let errors: Vec<CompilerError> = validation_result
            .errors
            .into_iter()
            .map(|e| convert_validation_error(&e))
            .collect();

        let warnings: Vec<String> = validation_result
            .warnings
            .into_iter()
            .map(|w| format!("{}", w))
            .collect();

        return Ok(Json(CompileResponse {
            success: false,
            code: None,
            errors,
            warnings,
            metadata: CompileMetadata {
                node_count: request.workflow.nodes.len(),
                edge_count: request.workflow.edges.len(),
                compilation_time_ms: start.elapsed().as_millis() as u64,
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }));
    }

    // Generate code
    match codegen::generate(&request.workflow) {
        Ok(code) => {
            let warnings: Vec<String> = validation_result
                .warnings
                .into_iter()
                .map(|w| format!("{}", w))
                .collect();

            Ok(Json(CompileResponse {
                success: true,
                code: Some(code),
                errors: vec![],
                warnings,
                metadata: CompileMetadata {
                    node_count: request.workflow.nodes.len(),
                    edge_count: request.workflow.edges.len(),
                    compilation_time_ms: start.elapsed().as_millis() as u64,
                    version: env!("CARGO_PKG_VERSION").to_string(),
                },
            }))
        },
        Err(e) => Ok(Json(CompileResponse {
            success: false,
            code: None,
            errors: vec![CompilerError {
                code: "CODEGEN_ERROR".to_string(),
                message: e.to_string(),
                node_id: None,
                severity: "error".to_string(),
            }],
            warnings: vec![],
            metadata: CompileMetadata {
                node_count: request.workflow.nodes.len(),
                edge_count: request.workflow.edges.len(),
                compilation_time_ms: start.elapsed().as_millis() as u64,
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        })),
    }
}

/// POST /validate - Validate a workflow definition without generating code
pub async fn validate(Json(workflow): Json<WorkflowDefinition>) -> Json<ValidateResponse> {
    let result = validation::validate(&workflow);

    let errors: Vec<CompilerError> = result
        .errors
        .into_iter()
        .map(|e| convert_validation_error(&e))
        .collect();

    let warnings: Vec<String> = result
        .warnings
        .into_iter()
        .map(|w| format!("{}", w))
        .collect();

    // Generate suggestions based on validation
    let suggestions = generate_suggestions(&workflow);

    Json(ValidateResponse {
        valid: errors.is_empty(),
        errors,
        warnings,
        suggestions,
    })
}

/// GET /health - Health check endpoint
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: get_uptime(),
    })
}

/// Convert validation error to API error format
fn convert_validation_error(error: &ValidationError) -> CompilerError {
    match error {
        ValidationError::NoStartNode => CompilerError {
            code: "NO_START_NODE".to_string(),
            message: error.to_string(),
            node_id: None,
            severity: "error".to_string(),
        },
        ValidationError::MultipleStartNodes(nodes) => CompilerError {
            code: "MULTIPLE_START_NODES".to_string(),
            message: error.to_string(),
            node_id: nodes.first().cloned(),
            severity: "error".to_string(),
        },
        ValidationError::NoEndNode => CompilerError {
            code: "NO_END_NODE".to_string(),
            message: error.to_string(),
            node_id: None,
            severity: "error".to_string(),
        },
        ValidationError::OrphanNode(id) => CompilerError {
            code: "ORPHAN_NODE".to_string(),
            message: error.to_string(),
            node_id: Some(id.clone()),
            severity: "error".to_string(),
        },
        ValidationError::CycleDetected(_) => CompilerError {
            code: "CYCLE_DETECTED".to_string(),
            message: error.to_string(),
            node_id: None,
            severity: "error".to_string(),
        },
        ValidationError::InvalidEdgeSource(id) => CompilerError {
            code: "INVALID_EDGE_SOURCE".to_string(),
            message: error.to_string(),
            node_id: Some(id.clone()),
            severity: "error".to_string(),
        },
        ValidationError::InvalidEdgeTarget(id) => CompilerError {
            code: "INVALID_EDGE_TARGET".to_string(),
            message: error.to_string(),
            node_id: Some(id.clone()),
            severity: "error".to_string(),
        },
        ValidationError::MissingActivityName { node_id } => CompilerError {
            code: "MISSING_ACTIVITY_NAME".to_string(),
            message: error.to_string(),
            node_id: Some(node_id.clone()),
            severity: "error".to_string(),
        },
        ValidationError::InvalidConfig { node_id, .. } => CompilerError {
            code: "INVALID_CONFIG".to_string(),
            message: error.to_string(),
            node_id: Some(node_id.clone()),
            severity: "error".to_string(),
        },
        ValidationError::UnknownVariable { node_id, .. } => CompilerError {
            code: "UNKNOWN_VARIABLE".to_string(),
            message: error.to_string(),
            node_id: Some(node_id.clone()),
            severity: "error".to_string(),
        },
        ValidationError::MissingRequiredField(_) => CompilerError {
            code: "MISSING_REQUIRED_FIELD".to_string(),
            message: error.to_string(),
            node_id: None,
            severity: "error".to_string(),
        },
        ValidationError::TriggerHasIncomingEdges { node_id } => CompilerError {
            code: "TRIGGER_HAS_INCOMING_EDGES".to_string(),
            message: error.to_string(),
            node_id: Some(node_id.clone()),
            severity: "error".to_string(),
        },
        ValidationError::UnreachableNode { node_id } => CompilerError {
            code: "UNREACHABLE_NODE".to_string(),
            message: error.to_string(),
            node_id: Some(node_id.clone()),
            severity: "error".to_string(),
        },
    }
}

/// Generate improvement suggestions
fn generate_suggestions(workflow: &WorkflowDefinition) -> Vec<String> {
    let mut suggestions = Vec::new();

    // Check for activities without retry policies
    let activities_without_retry: Vec<_> = workflow
        .nodes
        .iter()
        .filter(|n| n.node_type.is_activity() && n.data.retry_policy.is_none())
        .collect();

    if !activities_without_retry.is_empty() {
        suggestions.push(format!(
            "Consider adding retry policy to {} activities",
            activities_without_retry.len()
        ));
    }

    // Check for long-running workflows
    if !workflow.settings.is_long_running() && workflow.nodes.len() > 10 {
        suggestions.push(
            "Consider enabling long-running workflow settings for workflows with many nodes"
                .to_string(),
        );
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{
        NodeData, NodeType, Position, WorkflowEdge, WorkflowNode, WorkflowSettings,
    };

    fn create_valid_workflow() -> WorkflowDefinition {
        WorkflowDefinition {
            id: "wf_test".to_string(),
            name: "Test Workflow".to_string(),
            nodes: vec![
                WorkflowNode {
                    id: "trigger".to_string(),
                    node_type: NodeType::Trigger,
                    data: NodeData {
                        label: "Start".to_string(),
                        ..Default::default()
                    },
                    position: Position::default(),
                },
                WorkflowNode {
                    id: "activity".to_string(),
                    node_type: NodeType::Activity,
                    data: NodeData {
                        label: "Activity".to_string(),
                        activity_name: Some("doSomething".to_string()),
                        ..Default::default()
                    },
                    position: Position::default(),
                },
                WorkflowNode {
                    id: "end".to_string(),
                    node_type: NodeType::End,
                    data: NodeData {
                        label: "End".to_string(),
                        ..Default::default()
                    },
                    position: Position::default(),
                },
            ],
            edges: vec![
                WorkflowEdge::new("e1", "trigger", "activity"),
                WorkflowEdge::new("e2", "activity", "end"),
            ],
            variables: vec![],
            settings: WorkflowSettings::default(),
        }
    }

    #[tokio::test]
    async fn test_compile_valid_workflow() {
        let request = CompileRequest {
            workflow: create_valid_workflow(),
            options: CompileOptions::default(),
        };

        let result = compile(Json(request)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert!(response.success);
        assert!(response.code.is_some());
        assert!(response.errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_endpoint() {
        let workflow = create_valid_workflow();
        let response = validate(Json(workflow)).await;

        assert!(response.valid);
        assert!(response.errors.is_empty());
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let response = health().await;
        assert_eq!(response.status, "healthy");
        assert!(!response.version.is_empty());
    }
}
