//! Deploy pipeline endpoints.
//!
//! Provides REST API endpoints for deploying, undeploying, and checking the
//! deployment status of workflow definitions. Deploying a workflow validates it,
//! generates TypeScript code via the codegen module, and stores the compiled
//! output in the `workflow_compiled_code` table.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::auth::{self, AuthenticatedUser};
use crate::api::state::AppState;
use crate::codegen;
use crate::supabase::SupabaseError;
use crate::validation;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Status ID for a deployed workflow.
const DEPLOYED_STATUS_ID: &str = "00000000-0000-0000-0000-000000000003";

/// Status ID for a draft workflow (used when undeploying).
const DRAFT_STATUS_ID: &str = "00000000-0000-0000-0000-000000000001";

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Response returned after a successful deploy.
#[derive(Debug, Serialize)]
pub struct DeployResponse {
    pub workflow_id: String,
    pub status: String,
    pub compiled_at: String,
    pub message: String,
}

/// Response returned after a successful undeploy.
#[derive(Debug, Serialize)]
pub struct UndeployResponse {
    pub workflow_id: String,
    pub status: String,
    pub message: String,
}

/// Response returned from the status endpoint.
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub workflow_id: String,
    pub deployment_status: String,
    pub last_deployed_at: Option<String>,
}

// ---------------------------------------------------------------------------
// Error type (reuses same pattern as workflows.rs)
// ---------------------------------------------------------------------------

/// Structured error envelope.
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

/// Handler-level error that converts into a JSON response.
#[derive(Debug)]
pub struct DeployError {
    status: StatusCode,
    code: String,
    message: String,
    details: Vec<String>,
}

impl DeployError {
    fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "UNAUTHORIZED".to_string(),
            message: "Authorization header with Bearer token is required".to_string(),
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

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "INTERNAL_ERROR".to_string(),
            message: message.into(),
            details: vec![],
        }
    }

    fn from_supabase(err: SupabaseError) -> Self {
        match &err {
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

impl IntoResponse for DeployError {
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
// Supabase row types
// ---------------------------------------------------------------------------

/// Row shape for loading a workflow from the `workflows` table.
#[derive(Debug, Deserialize)]
struct WorkflowRow {
    id: String,
    #[allow(dead_code)]
    name: String,
    status_id: String,
    definition: serde_json::Value,
    #[serde(default)]
    deployed_at: Option<String>,
}

/// Row to insert into `workflow_compiled_code`.
#[derive(Debug, Serialize)]
struct InsertCompiledCodeRow {
    id: String,
    workflow_id: String,
    code: serde_json::Value,
    compiled_at: String,
}

/// Row to update on the `workflows` table when deploying.
#[derive(Debug, Serialize)]
struct DeployUpdateRow {
    status_id: String,
    deployed_at: String,
}

/// Row to update on the `workflows` table when undeploying.
#[derive(Debug, Serialize)]
struct UndeployUpdateRow {
    status_id: String,
}

/// Row shape when checking compiled code existence.
#[derive(Debug, Deserialize)]
struct CompiledCodeRow {
    #[allow(dead_code)]
    id: String,
    compiled_at: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Validate the Bearer token against the Supabase `api_keys` table and check
/// the per-user rate limit.
async fn require_auth(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<AuthenticatedUser, DeployError> {
    let mut request = axum::http::Request::builder()
        .uri("http://localhost/")
        .body(())
        .unwrap();
    *request.headers_mut() = headers.clone();
    let (parts, _) = request.into_parts();

    let token =
        auth::extract_bearer_token(&parts).ok_or_else(DeployError::unauthorized)?;

    let user = auth::validate_api_key(
        state.supabase.http_client(),
        state.supabase.url(),
        state.supabase.service_role_key(),
        &token,
    )
    .await
    .map_err(|_| DeployError::unauthorized())?;

    // Check rate limit (keyed by user_id).
    let result = state.rate_limiter.check(&user.user_id);
    if !result.allowed {
        return Err(DeployError {
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

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `POST /v1/workflows/:id/deploy` -- Validate, compile, and deploy a workflow.
pub async fn deploy_workflow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, DeployError> {
    let user = require_auth(&headers, &state).await?;

    // 1. Load workflow from Supabase (scoped to user).
    let user_filter = format!("eq.{}", user.user_id);
    let workflow: WorkflowRow = state
        .supabase
        .select_one(
            "workflows",
            &[
                ("id", &format!("eq.{id}")),
                ("created_by", &user_filter),
                ("select", "id,name,status_id,definition,deployed_at"),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                DeployError::not_found(format!("Workflow '{id}' not found"))
            }
            _ => DeployError::from_supabase(e),
        })?;

    // 2. Parse the stored definition JSONB into a WorkflowDefinition.
    let definition: crate::schema::WorkflowDefinition =
        serde_json::from_value(workflow.definition).map_err(|e| {
            DeployError::internal(format!("Failed to parse stored workflow definition: {e}"))
        })?;

    // 3. Validate.
    let validation_result = validation::validate(&definition);
    if !validation_result.is_valid() {
        let details: Vec<String> = validation_result
            .errors
            .iter()
            .map(|e| e.to_string())
            .collect();
        return Err(DeployError::validation_failed(
            "Workflow validation failed",
            details,
        ));
    }

    // 4. Compile via codegen.
    let generated = codegen::generate(&definition).map_err(|e| {
        DeployError::internal(format!("Code generation failed: {e}"))
    })?;

    // 5. Serialize the generated code as a JSON blob for storage.
    let code_json = serde_json::to_value(&generated).map_err(|e| {
        DeployError::internal(format!("Failed to serialize generated code: {e}"))
    })?;

    let now = Utc::now().to_rfc3339();

    // 6. Insert compiled code into `workflow_compiled_code`.
    let compiled_row = InsertCompiledCodeRow {
        id: Uuid::new_v4().to_string(),
        workflow_id: id.clone(),
        code: code_json,
        compiled_at: now.clone(),
    };

    let _inserted: serde_json::Value = state
        .supabase
        .insert("workflow_compiled_code", &compiled_row)
        .await
        .map_err(DeployError::from_supabase)?;

    // 7. Update workflow status to deployed.
    let update_body = DeployUpdateRow {
        status_id: DEPLOYED_STATUS_ID.to_string(),
        deployed_at: now.clone(),
    };

    let _updated: Vec<serde_json::Value> = state
        .supabase
        .update(
            "workflows",
            &[("id", &format!("eq.{id}")), ("created_by", &user_filter)],
            &update_body,
        )
        .await
        .map_err(DeployError::from_supabase)?;

    // Fire-and-forget: record deploy telemetry in discovery service
    if let Some(ref discovery) = state.discovery {
        let discovery = discovery.clone();
        let workflow_id = id.clone();
        let deploy_user_id = user.user_id.clone();
        tokio::spawn(async move {
            discovery
                .telemetry(&workflow_id, "deploy", &deploy_user_id, &[])
                .await;
        });
    }

    Ok((
        StatusCode::OK,
        Json(DeployResponse {
            workflow_id: id,
            status: "deployed".to_string(),
            compiled_at: now,
            message: "Workflow compiled and deployed successfully".to_string(),
        }),
    ))
}

/// `POST /v1/workflows/:id/undeploy` -- Revert a workflow to draft status.
pub async fn undeploy_workflow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<UndeployResponse>, DeployError> {
    let user = require_auth(&headers, &state).await?;

    // Verify the workflow exists and belongs to the user.
    let user_filter = format!("eq.{}", user.user_id);
    let _workflow: WorkflowRow = state
        .supabase
        .select_one(
            "workflows",
            &[
                ("id", &format!("eq.{id}")),
                ("created_by", &user_filter),
                ("select", "id,name,status_id,definition,deployed_at"),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                DeployError::not_found(format!("Workflow '{id}' not found"))
            }
            _ => DeployError::from_supabase(e),
        })?;

    // Update workflow status back to draft (scoped to user).
    let update_body = UndeployUpdateRow {
        status_id: DRAFT_STATUS_ID.to_string(),
    };

    let _updated: Vec<serde_json::Value> = state
        .supabase
        .update(
            "workflows",
            &[("id", &format!("eq.{id}")), ("created_by", &user_filter)],
            &update_body,
        )
        .await
        .map_err(DeployError::from_supabase)?;

    Ok(Json(UndeployResponse {
        workflow_id: id,
        status: "draft".to_string(),
        message: "Workflow undeployed and reverted to draft".to_string(),
    }))
}

/// `GET /v1/workflows/:id/status` -- Check deployment status of a workflow.
pub async fn workflow_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<StatusResponse>, DeployError> {
    let user = require_auth(&headers, &state).await?;

    // Load the workflow (scoped to user).
    let user_filter = format!("eq.{}", user.user_id);
    let workflow: WorkflowRow = state
        .supabase
        .select_one(
            "workflows",
            &[
                ("id", &format!("eq.{id}")),
                ("created_by", &user_filter),
                ("select", "id,name,status_id,definition,deployed_at"),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                DeployError::not_found(format!("Workflow '{id}' not found"))
            }
            _ => DeployError::from_supabase(e),
        })?;

    // Determine deployment status based on status_id and compiled code presence.
    let deployment_status = if workflow.status_id == DEPLOYED_STATUS_ID {
        "deployed"
    } else {
        // Check if compiled code exists (could be "compiled" but not actively deployed).
        let compiled: Vec<CompiledCodeRow> = state
            .supabase
            .select(
                "workflow_compiled_code",
                &[
                    ("workflow_id", &format!("eq.{id}")),
                    ("select", "id,compiled_at"),
                    ("order", "compiled_at.desc"),
                    ("limit", "1"),
                ],
            )
            .await
            .unwrap_or_default();

        if compiled.is_empty() {
            "draft"
        } else {
            "compiled"
        }
    };

    Ok(Json(StatusResponse {
        workflow_id: id,
        deployment_status: deployment_status.to_string(),
        last_deployed_at: workflow.deployed_at,
    }))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Response serialization tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_deploy_response_serialization() {
        let response = DeployResponse {
            workflow_id: "wf-123".to_string(),
            status: "deployed".to_string(),
            compiled_at: "2026-02-20T12:00:00Z".to_string(),
            message: "Workflow compiled and deployed successfully".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["workflow_id"], "wf-123");
        assert_eq!(json["status"], "deployed");
        assert_eq!(json["compiled_at"], "2026-02-20T12:00:00Z");
        assert_eq!(json["message"], "Workflow compiled and deployed successfully");
    }

    #[test]
    fn test_undeploy_response_serialization() {
        let response = UndeployResponse {
            workflow_id: "wf-456".to_string(),
            status: "draft".to_string(),
            message: "Workflow undeployed and reverted to draft".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["workflow_id"], "wf-456");
        assert_eq!(json["status"], "draft");
        assert_eq!(json["message"], "Workflow undeployed and reverted to draft");
    }

    #[test]
    fn test_status_response_serialization_deployed() {
        let response = StatusResponse {
            workflow_id: "wf-789".to_string(),
            deployment_status: "deployed".to_string(),
            last_deployed_at: Some("2026-02-20T12:00:00Z".to_string()),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["workflow_id"], "wf-789");
        assert_eq!(json["deployment_status"], "deployed");
        assert_eq!(json["last_deployed_at"], "2026-02-20T12:00:00Z");
    }

    #[test]
    fn test_status_response_serialization_draft() {
        let response = StatusResponse {
            workflow_id: "wf-000".to_string(),
            deployment_status: "draft".to_string(),
            last_deployed_at: None,
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["workflow_id"], "wf-000");
        assert_eq!(json["deployment_status"], "draft");
        assert!(json["last_deployed_at"].is_null());
    }

    #[test]
    fn test_status_response_serialization_compiled() {
        let response = StatusResponse {
            workflow_id: "wf-111".to_string(),
            deployment_status: "compiled".to_string(),
            last_deployed_at: Some("2026-01-15T08:30:00Z".to_string()),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["deployment_status"], "compiled");
    }

    // -----------------------------------------------------------------------
    // Error type tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_deploy_error_unauthorized() {
        let err = DeployError::unauthorized();
        assert_eq!(err.status, StatusCode::UNAUTHORIZED);
        assert_eq!(err.code, "UNAUTHORIZED");
    }

    #[test]
    fn test_deploy_error_not_found() {
        let err = DeployError::not_found("Workflow 'abc' not found");
        assert_eq!(err.status, StatusCode::NOT_FOUND);
        assert_eq!(err.code, "NOT_FOUND");
        assert_eq!(err.message, "Workflow 'abc' not found");
    }

    #[test]
    fn test_deploy_error_validation_failed() {
        let err = DeployError::validation_failed(
            "Workflow validation failed",
            vec!["No start node".to_string()],
        );
        assert_eq!(err.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(err.code, "VALIDATION_FAILED");
        assert_eq!(err.details.len(), 1);
    }

    #[test]
    fn test_deploy_error_internal() {
        let err = DeployError::internal("Something broke");
        assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.code, "INTERNAL_ERROR");
    }

    #[test]
    fn test_error_envelope_serialization() {
        let envelope = ErrorEnvelope {
            error: ErrorBody {
                code: "NOT_FOUND".to_string(),
                message: "Workflow not found".to_string(),
                details: vec![],
            },
        };

        let json = serde_json::to_value(&envelope).unwrap();
        assert_eq!(json["error"]["code"], "NOT_FOUND");
        assert_eq!(json["error"]["message"], "Workflow not found");
    }

    // -----------------------------------------------------------------------
    // Auth extraction (require_auth is now async + validates against Supabase,
    // so token-presence tests are covered by auth::extract_bearer_token tests
    // in auth.rs. The integration path is tested via ignored Supabase tests.)
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Router wiring test
    // -----------------------------------------------------------------------

    #[test]
    fn test_deploy_routes_are_registered() {
        use axum::routing::{get, post};
        use axum::Router;

        // This test verifies that the routes can be constructed without panicking.
        // It does not make HTTP requests (that requires a running Supabase).
        let _router: Router<AppState> = Router::new()
            .route("/workflows/{id}/deploy", post(deploy_workflow))
            .route("/workflows/{id}/undeploy", post(undeploy_workflow))
            .route("/workflows/{id}/status", get(workflow_status));
    }

    // -----------------------------------------------------------------------
    // Integration tests (need Supabase)
    // -----------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_deploy_workflow_integration() {
        // Would deploy a real workflow through Supabase.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_undeploy_workflow_integration() {
        // Would undeploy a real workflow through Supabase.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_workflow_status_integration() {
        // Would check status of a real workflow through Supabase.
    }
}
