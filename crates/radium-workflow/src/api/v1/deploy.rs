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
use serde::{Deserialize, Serialize};

use crate::api::auth::{self, AuthenticatedUser};
use crate::api::state::AppState;
use crate::deploy_pipeline::{self, DeployFailureKind, SingleServiceResult, DEPLOYED_STATUS_ID};
use crate::supabase::SupabaseError;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

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
// Error type (reuses same pattern as services.rs)
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

/// `POST /v1/services/:id/deploy` -- Validate, compile, and deploy a workflow.
pub async fn deploy_workflow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, DeployError> {
    let user = require_auth(&headers, &state).await?;

    match deploy_pipeline::deploy_single_service(&state, &id, &user.user_id).await {
        SingleServiceResult::Success {
            service_id,
            compiled_at,
        } => Ok((
            StatusCode::OK,
            Json(DeployResponse {
                workflow_id: service_id,
                status: "deployed".to_string(),
                compiled_at,
                message: "Workflow compiled and deployed successfully".to_string(),
            }),
        )),
        SingleServiceResult::Failure { kind, error, .. } => match kind {
            DeployFailureKind::NotFound => {
                Err(DeployError::not_found(format!("Workflow '{id}' not found")))
            }
            DeployFailureKind::ValidationFailed(details) => {
                Err(DeployError::validation_failed(
                    "Workflow validation failed",
                    details,
                ))
            }
            _ => Err(DeployError::internal(error)),
        },
    }
}

/// `POST /v1/services/:id/undeploy` -- Revert a workflow to draft status.
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

/// `GET /v1/services/:id/status` -- Check deployment status of a workflow.
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
    // from_supabase error mapping tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_deploy_error_from_supabase_not_found() {
        let err = DeployError::from_supabase(SupabaseError::NotFound {
            resource: "workflows".to_string(),
            key: "id".to_string(),
            value: "abc-123".to_string(),
        });
        assert_eq!(err.status, StatusCode::NOT_FOUND);
        assert_eq!(err.code, "NOT_FOUND");
    }

    #[test]
    fn test_deploy_error_from_supabase_api_error_404() {
        let err = DeployError::from_supabase(SupabaseError::ApiError {
            status: 404,
            message: "Not found".to_string(),
        });
        assert_eq!(err.status, StatusCode::NOT_FOUND);
        assert_eq!(err.code, "NOT_FOUND");
    }

    #[test]
    fn test_deploy_error_from_supabase_api_error_500() {
        let err = DeployError::from_supabase(SupabaseError::ApiError {
            status: 500,
            message: "Internal server error".to_string(),
        });
        assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.code, "INTERNAL_ERROR");
    }

    #[test]
    fn test_deploy_error_from_supabase_config_error() {
        let err = DeployError::from_supabase(SupabaseError::ConfigError(
            "Missing URL".to_string(),
        ));
        assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.code, "INTERNAL_ERROR");
    }

    #[test]
    fn test_deploy_error_from_supabase_deserialization_error() {
        let err = DeployError::from_supabase(SupabaseError::DeserializationError(
            "Invalid JSON".to_string(),
        ));
        assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.code, "INTERNAL_ERROR");
    }

    #[test]
    fn test_deploy_error_rate_limited() {
        let err = DeployError {
            status: StatusCode::TOO_MANY_REQUESTS,
            code: "RATE_LIMITED".to_string(),
            message: "Rate limit exceeded. Try again in 60 seconds.".to_string(),
            details: vec![],
        };
        assert_eq!(err.status, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(err.code, "RATE_LIMITED");
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
            .route("/services/{id}/deploy", post(deploy_workflow))
            .route("/services/{id}/undeploy", post(undeploy_workflow))
            .route("/services/{id}/status", get(workflow_status));
    }

    // -----------------------------------------------------------------------
    // Integration tests (need Supabase)
    // -----------------------------------------------------------------------

    // Integration tests for deploy/undeploy/status live in
    // `crates/radium-workflow/tests/api_integration.rs` which spins up a real
    // Axum server and exercises the full request path including Supabase.
    // See: test_deploy_valid_workflow, test_undeploy_deployed_workflow,
    // test_workflow_status_draft, etc.
}
