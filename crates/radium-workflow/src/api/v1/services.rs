//! Workflow CRUD endpoints.
//!
//! Provides REST API endpoints for creating, reading, updating, and deleting
//! workflow definitions stored in Supabase. Supports both YAML and JSON input
//! formats for workflow definitions.

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::api::auth::{self, AuthenticatedUser};
use crate::api::state::AppState;
use crate::supabase::SupabaseError;
use crate::validation;
use crate::yaml_format;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default status_id for newly created workflows (maps to 'draft' in workflow_statuses).
const DRAFT_STATUS_ID: &str = "00000000-0000-0000-0000-000000000001";

/// Default visibility_id for newly created workflows.
const DEFAULT_VISIBILITY_ID: &str = "00000000-0000-0000-0000-000000000001";

/// Public visibility_id (component_visibility seed: 'public').
const PUBLIC_VISIBILITY_ID: &str = "00000000-0000-0000-0000-000000000003";

/// Private visibility_id (component_visibility seed: 'private').
const PRIVATE_VISIBILITY_ID: &str = "00000000-0000-0000-0000-000000000001";

/// Team visibility_id (component_visibility seed: 'team').
const TEAM_VISIBILITY_ID: &str = "00000000-0000-0000-0000-000000000002";

// ---------------------------------------------------------------------------
// Query parameters
// ---------------------------------------------------------------------------

/// Optional query parameters for service creation.
#[derive(Debug, Deserialize)]
pub struct CreateServiceQuery {
    pub project_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Full workflow response returned from GET and create/update operations.
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowResponse {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status_id: String,
    pub version: String,
    pub definition: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

/// Summary response for listing workflows.
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowSummary {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status_id: String,
    pub version: String,
    pub created_at: String,
}

/// List response envelope.
#[derive(Debug, Serialize)]
pub struct WorkflowListResponse {
    pub workflows: Vec<WorkflowSummary>,
    pub total: usize,
}

/// Validation endpoint response.
#[derive(Debug, Serialize)]
pub struct ValidateWorkflowResponse {
    pub valid: bool,
    pub errors: Vec<CompilerErrorResponse>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

/// A single validation/compiler error in the API response.
#[derive(Debug, Serialize)]
pub struct CompilerErrorResponse {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    pub severity: String,
}

// ---------------------------------------------------------------------------
// Catalog request/response types
// ---------------------------------------------------------------------------

/// Request body for importing a service from the catalog.
#[derive(Debug, Deserialize)]
pub struct ImportServiceRequest {
    pub project_id: String,
}

/// Response envelope for catalog listing.
#[derive(Debug, Serialize)]
pub struct CatalogListResponse {
    pub services: Vec<WorkflowSummary>,
    pub total: usize,
}

/// Response for publish/unpublish operations.
#[derive(Debug, Serialize)]
pub struct PublishResponse {
    pub status: String,
    pub message: String,
}

/// Body sent to Supabase to update only the visibility_id column.
#[derive(Debug, Serialize)]
struct UpdateVisibilityRow {
    visibility_id: String,
}

/// Extended workflow row returned when fetching a source service for import.
/// Includes fields not present in `WorkflowResponse` (e.g. `display_name`,
/// `visibility_id`, `project_id`).
#[derive(Debug, Deserialize)]
struct SourceWorkflowRow {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub definition: serde_json::Value,
    pub visibility_id: String,
}

// ---------------------------------------------------------------------------
// Error type
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
pub struct WorkflowError {
    status: StatusCode,
    code: String,
    message: String,
    details: Vec<String>,
}

impl WorkflowError {
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

impl IntoResponse for WorkflowError {
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
// Supabase row types (what we INSERT / what Supabase returns)
// ---------------------------------------------------------------------------

/// Body sent to Supabase for creating a workflow.
#[derive(Debug, Serialize)]
struct InsertWorkflowRow {
    name: String,
    display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    definition: serde_json::Value,
    version: String,
    status_id: String,
    visibility_id: String,
    created_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_workflow_id: Option<String>,
}

/// Body sent to Supabase for updating a workflow.
#[derive(Debug, Serialize)]
struct UpdateWorkflowRow {
    name: String,
    display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    definition: serde_json::Value,
    version: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Validate the Bearer token against the Supabase `api_keys` table and check
/// the per-user rate limit.
///
/// Returns the authenticated user on success, or a `WorkflowError` when the
/// token is missing, invalid, expired, revoked, or rate-limited.
async fn require_auth(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<AuthenticatedUser, WorkflowError> {
    // Build minimal Parts to extract the bearer token.
    let mut request = axum::http::Request::builder()
        .uri("http://localhost/")
        .body(())
        .unwrap();
    *request.headers_mut() = headers.clone();
    let (parts, ()) = request.into_parts();

    let token =
        auth::extract_bearer_token(&parts).ok_or_else(WorkflowError::unauthorized)?;

    let user = auth::validate_api_key(
        state.supabase.http_client(),
        state.supabase.url(),
        state.supabase.service_role_key(),
        &token,
    )
    .await
    .map_err(|_| WorkflowError::unauthorized())?;

    // Check rate limit (keyed by user_id).
    let result = state.rate_limiter.check(&user.user_id);
    if !result.allowed {
        return Err(WorkflowError {
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

/// Parse a request body as either YAML or JSON into a `YamlWorkflow`.
///
/// Content-Type detection:
/// - `application/x-yaml` or `text/yaml` -> YAML
/// - Everything else (including `application/json`) -> JSON
fn parse_workflow_body(
    headers: &HeaderMap,
    body: &[u8],
) -> Result<yaml_format::YamlWorkflow, WorkflowError> {
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json");

    let is_yaml = content_type.contains("yaml");

    if is_yaml {
        serde_yaml::from_slice(body).map_err(|e| {
            WorkflowError::bad_request(format!("Invalid YAML workflow definition: {e}"))
        })
    } else {
        serde_json::from_slice(body).map_err(|e| {
            WorkflowError::bad_request(format!("Invalid JSON workflow definition: {e}"))
        })
    }
}

/// Convert a validation error into the API response format.
fn convert_validation_error(error: &validation::ValidationError) -> CompilerErrorResponse {
    let (code, node_id) = match error {
        validation::ValidationError::NoStartNode => ("NO_START_NODE", None),
        validation::ValidationError::MultipleStartNodes(nodes) => {
            ("MULTIPLE_START_NODES", nodes.first().cloned())
        }
        validation::ValidationError::NoEndNode => ("NO_END_NODE", None),
        validation::ValidationError::OrphanNode(id) => ("ORPHAN_NODE", Some(id.clone())),
        validation::ValidationError::CycleDetected(_) => ("CYCLE_DETECTED", None),
        validation::ValidationError::InvalidEdgeSource(id) => {
            ("INVALID_EDGE_SOURCE", Some(id.clone()))
        }
        validation::ValidationError::InvalidEdgeTarget(id) => {
            ("INVALID_EDGE_TARGET", Some(id.clone()))
        }
        validation::ValidationError::MissingActivityName { node_id } => {
            ("MISSING_ACTIVITY_NAME", Some(node_id.clone()))
        }
        validation::ValidationError::InvalidConfig { node_id, .. } => {
            ("INVALID_CONFIG", Some(node_id.clone()))
        }
        validation::ValidationError::UnknownVariable { node_id, .. } => {
            ("UNKNOWN_VARIABLE", Some(node_id.clone()))
        }
        validation::ValidationError::MissingRequiredField(_) => ("MISSING_REQUIRED_FIELD", None),
        validation::ValidationError::TriggerHasIncomingEdges { node_id } => {
            ("TRIGGER_HAS_INCOMING_EDGES", Some(node_id.clone()))
        }
        validation::ValidationError::UnreachableNode { node_id } => {
            ("UNREACHABLE_NODE", Some(node_id.clone()))
        }
    };

    CompilerErrorResponse {
        code: code.to_string(),
        message: error.to_string(),
        node_id,
        severity: "error".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `POST /v1/services` -- Create a workflow from YAML or JSON body.
pub async fn create_workflow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CreateServiceQuery>,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse, WorkflowError> {
    let user = require_auth(&headers, &state).await?;

    let yaml_workflow = parse_workflow_body(&headers, &body)?;

    // Transform YAML types into the compiler's WorkflowDefinition.
    let definition = yaml_format::transform(&yaml_workflow).map_err(|e| {
        WorkflowError::validation_failed(
            format!("Workflow transformation failed: {e}"),
            vec![e.to_string()],
        )
    })?;

    // Serialize the definition to JSONB for storage.
    let definition_json = serde_json::to_value(&definition).map_err(|e| {
        WorkflowError::internal(format!("Failed to serialize workflow definition: {e}"))
    })?;

    let user_id_for_discovery = user.user_id.clone();

    let row = InsertWorkflowRow {
        name: yaml_workflow.name.clone(),
        display_name: yaml_workflow.name.clone(),
        description: yaml_workflow.description.clone(),
        definition: definition_json,
        version: "1.0.0".to_string(),
        status_id: DRAFT_STATUS_ID.to_string(),
        visibility_id: DEFAULT_VISIBILITY_ID.to_string(),
        created_by: user.user_id,
        project_id: query.project_id,
        parent_workflow_id: None,
    };

    let created: WorkflowResponse = state
        .supabase
        .insert("workflows", &row)
        .await
        .map_err(|e| WorkflowError::from_supabase(&e))?;

    // Fire-and-forget: index in discovery service
    if let Some(ref discovery) = state.discovery {
        let index_req = serde_json::json!({
            "id": created.id,
            "kind": "service",
            "name": created.name,
            "description": created.description.as_deref().unwrap_or(""),
            "category": "workflow",
            "visibility": "private",
            "owner_id": user_id_for_discovery,
            "tags": [],
            "definition": created.definition,
        });
        let discovery = discovery.clone();
        tokio::spawn(async move { discovery.index(&index_req).await });
    }

    Ok((StatusCode::CREATED, Json(created)))
}

/// `GET /v1/services` -- List all workflows.
pub async fn list_workflows(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<WorkflowListResponse>, WorkflowError> {
    let user = require_auth(&headers, &state).await?;

    let user_filter = format!("eq.{}", user.user_id);
    let workflows: Vec<WorkflowSummary> = state
        .supabase
        .select(
            "workflows",
            &[
                ("select", "id,name,description,status_id,version,created_at"),
                ("order", "created_at.desc"),
                ("created_by", &user_filter),
            ],
        )
        .await
        .map_err(|e| WorkflowError::from_supabase(&e))?;

    let total = workflows.len();
    Ok(Json(WorkflowListResponse { workflows, total }))
}

/// `GET /v1/services/:id` -- Get a single workflow by ID.
pub async fn get_workflow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<WorkflowResponse>, WorkflowError> {
    let user = require_auth(&headers, &state).await?;

    let user_filter = format!("eq.{}", user.user_id);
    let workflow: WorkflowResponse = state
        .supabase
        .select_one(
            "workflows",
            &[
                ("id", &format!("eq.{id}")),
                ("created_by", &user_filter),
                (
                    "select",
                    "id,name,description,status_id,version,definition,created_at,updated_at",
                ),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                WorkflowError::not_found(format!("Workflow '{id}' not found"))
            }
            _ => WorkflowError::from_supabase(&e),
        })?;

    Ok(Json(workflow))
}

/// `PUT /v1/services/:id` -- Update a workflow definition.
pub async fn update_workflow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    body: axum::body::Bytes,
) -> Result<Json<WorkflowResponse>, WorkflowError> {
    let user = require_auth(&headers, &state).await?;

    let yaml_workflow = parse_workflow_body(&headers, &body)?;

    let definition = yaml_format::transform(&yaml_workflow).map_err(|e| {
        WorkflowError::validation_failed(
            format!("Workflow transformation failed: {e}"),
            vec![e.to_string()],
        )
    })?;

    let definition_json = serde_json::to_value(&definition).map_err(|e| {
        WorkflowError::internal(format!("Failed to serialize workflow definition: {e}"))
    })?;

    let update_body = UpdateWorkflowRow {
        name: yaml_workflow.name.clone(),
        display_name: yaml_workflow.name.clone(),
        description: yaml_workflow.description.clone(),
        definition: definition_json,
        version: "1.0.0".to_string(),
    };

    let user_id_for_discovery = user.user_id.clone();
    let user_filter = format!("eq.{}", user.user_id);
    let updated: Vec<WorkflowResponse> = state
        .supabase
        .update(
            "workflows",
            &[("id", &format!("eq.{id}")), ("created_by", &user_filter)],
            &update_body,
        )
        .await
        .map_err(|e| WorkflowError::from_supabase(&e))?;

    let workflow = updated.into_iter().next().ok_or_else(|| {
        WorkflowError::not_found(format!("Workflow '{id}' not found"))
    })?;

    // Fire-and-forget: re-index in discovery service
    if let Some(ref discovery) = state.discovery {
        let index_req = serde_json::json!({
            "id": workflow.id,
            "kind": "service",
            "name": workflow.name,
            "description": workflow.description.as_deref().unwrap_or(""),
            "category": "workflow",
            "visibility": "private",
            "owner_id": user_id_for_discovery,
            "tags": [],
            "definition": workflow.definition,
        });
        let discovery = discovery.clone();
        tokio::spawn(async move { discovery.index(&index_req).await });
    }

    Ok(Json(workflow))
}

/// `DELETE /v1/services/:id` -- Delete a workflow.
pub async fn delete_workflow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, WorkflowError> {
    let user = require_auth(&headers, &state).await?;

    let user_filter = format!("eq.{}", user.user_id);
    state
        .supabase
        .delete(
            "workflows",
            &[("id", &format!("eq.{id}")), ("created_by", &user_filter)],
        )
        .await
        .map_err(|e| WorkflowError::from_supabase(&e))?;

    Ok(StatusCode::NO_CONTENT)
}

/// `POST /v1/services/:id/validate` -- Validate a stored workflow.
pub async fn validate_workflow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ValidateWorkflowResponse>, WorkflowError> {
    let user = require_auth(&headers, &state).await?;

    // Load the workflow from Supabase (scoped to user).
    let user_filter = format!("eq.{}", user.user_id);
    let workflow: WorkflowResponse = state
        .supabase
        .select_one(
            "workflows",
            &[
                ("id", &format!("eq.{id}")),
                ("created_by", &user_filter),
                (
                    "select",
                    "id,name,description,status_id,version,definition,created_at,updated_at",
                ),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                WorkflowError::not_found(format!("Workflow '{id}' not found"))
            }
            _ => WorkflowError::from_supabase(&e),
        })?;

    // Parse the stored definition JSONB back into a WorkflowDefinition.
    let definition: crate::schema::WorkflowDefinition =
        serde_json::from_value(workflow.definition).map_err(|e| {
            WorkflowError::internal(format!(
                "Failed to parse stored workflow definition: {e}"
            ))
        })?;

    // Run validation.
    let result = validation::validate(&definition);

    let errors: Vec<CompilerErrorResponse> = result
        .errors
        .iter()
        .map(convert_validation_error)
        .collect();

    let warnings: Vec<String> = result.warnings.iter().map(ToString::to_string).collect();

    // Generate suggestions (same logic as the existing validate handler).
    let mut suggestions = Vec::new();
    let activities_without_retry: Vec<_> = definition
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
    if !definition.settings.is_long_running() && definition.nodes.len() > 10 {
        suggestions.push(
            "Consider enabling long-running workflow settings for workflows with many nodes"
                .to_string(),
        );
    }

    Ok(Json(ValidateWorkflowResponse {
        valid: errors.is_empty(),
        errors,
        warnings,
        suggestions,
    }))
}

// ---------------------------------------------------------------------------
// Catalog handlers
// ---------------------------------------------------------------------------

/// `GET /v1/services/catalog` -- Browse public services from other users.
pub async fn list_catalog(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<CatalogListResponse>, WorkflowError> {
    let user = require_auth(&headers, &state).await?;

    let visibility_filter = format!("eq.{PUBLIC_VISIBILITY_ID}");
    let exclude_self = format!("neq.{}", user.user_id);

    let services: Vec<WorkflowSummary> = state
        .supabase
        .select(
            "workflows",
            &[
                ("select", "id,name,description,status_id,version,created_at"),
                ("visibility_id", &visibility_filter),
                ("created_by", &exclude_self),
                ("order", "created_at.desc"),
            ],
        )
        .await
        .map_err(|e| WorkflowError::from_supabase(&e))?;

    let total = services.len();
    Ok(Json(CatalogListResponse { services, total }))
}

/// `POST /v1/services/{id}/publish` -- Make a service visible in the catalog.
pub async fn publish_service(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<PublishResponse>, WorkflowError> {
    let user = require_auth(&headers, &state).await?;

    // Ownership check: select the workflow scoped to the current user.
    let user_filter = format!("eq.{}", user.user_id);
    let _workflow: WorkflowResponse = state
        .supabase
        .select_one(
            "workflows",
            &[
                ("id", &format!("eq.{id}")),
                ("created_by", &user_filter),
                (
                    "select",
                    "id,name,description,status_id,version,definition,created_at,updated_at",
                ),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                WorkflowError::not_found(format!("Workflow '{id}' not found"))
            }
            _ => WorkflowError::from_supabase(&e),
        })?;

    // Update visibility to public.
    let update_body = UpdateVisibilityRow {
        visibility_id: PUBLIC_VISIBILITY_ID.to_string(),
    };
    let user_filter = format!("eq.{}", user.user_id);
    let _updated: Vec<WorkflowResponse> = state
        .supabase
        .update(
            "workflows",
            &[("id", &format!("eq.{id}")), ("created_by", &user_filter)],
            &update_body,
        )
        .await
        .map_err(|e| WorkflowError::from_supabase(&e))?;

    Ok(Json(PublishResponse {
        status: "published".to_string(),
        message: format!("Service '{id}' is now public in the catalog"),
    }))
}

/// `POST /v1/services/{id}/unpublish` -- Remove a service from the catalog.
pub async fn unpublish_service(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<PublishResponse>, WorkflowError> {
    let user = require_auth(&headers, &state).await?;

    // Ownership check.
    let user_filter = format!("eq.{}", user.user_id);
    let _workflow: WorkflowResponse = state
        .supabase
        .select_one(
            "workflows",
            &[
                ("id", &format!("eq.{id}")),
                ("created_by", &user_filter),
                (
                    "select",
                    "id,name,description,status_id,version,definition,created_at,updated_at",
                ),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                WorkflowError::not_found(format!("Workflow '{id}' not found"))
            }
            _ => WorkflowError::from_supabase(&e),
        })?;

    // Update visibility to private.
    let update_body = UpdateVisibilityRow {
        visibility_id: PRIVATE_VISIBILITY_ID.to_string(),
    };
    let user_filter = format!("eq.{}", user.user_id);
    let _updated: Vec<WorkflowResponse> = state
        .supabase
        .update(
            "workflows",
            &[("id", &format!("eq.{id}")), ("created_by", &user_filter)],
            &update_body,
        )
        .await
        .map_err(|e| WorkflowError::from_supabase(&e))?;

    Ok(Json(PublishResponse {
        status: "unpublished".to_string(),
        message: format!("Service '{id}' is now private"),
    }))
}

/// `POST /v1/services/catalog/{source_id}/import` -- Import a public/team
/// service into the caller's project, creating a private copy with lineage.
pub async fn import_service(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(source_id): Path<String>,
    Json(body): Json<ImportServiceRequest>,
) -> Result<impl IntoResponse, WorkflowError> {
    let user = require_auth(&headers, &state).await?;

    // Fetch the source service (must exist and be public or team visibility).
    let source: SourceWorkflowRow = state
        .supabase
        .select_one(
            "workflows",
            &[
                ("id", &format!("eq.{source_id}")),
                (
                    "visibility_id",
                    &format!("in.({PUBLIC_VISIBILITY_ID},{TEAM_VISIBILITY_ID})"),
                ),
                (
                    "select",
                    "id,name,display_name,description,definition,visibility_id",
                ),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => WorkflowError::not_found(format!(
                "Service '{source_id}' not found or is not available for import"
            )),
            _ => WorkflowError::from_supabase(&e),
        })?;

    // Build the imported copy.
    let imported_name = format!("{} (imported)", source.name);
    let imported_display_name = source
        .display_name
        .map_or_else(|| imported_name.clone(), |dn| format!("{dn} (imported)"));

    let row = InsertWorkflowRow {
        name: imported_name,
        display_name: imported_display_name,
        description: source.description,
        definition: source.definition,
        version: "1.0.0".to_string(),
        status_id: DRAFT_STATUS_ID.to_string(),
        visibility_id: PRIVATE_VISIBILITY_ID.to_string(),
        created_by: user.user_id,
        project_id: Some(body.project_id),
        parent_workflow_id: Some(source.id),
    };

    let created: WorkflowResponse = state
        .supabase
        .insert("workflows", &row)
        .await
        .map_err(|e| WorkflowError::from_supabase(&e))?;

    Ok((StatusCode::CREATED, Json(created)))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Content-type detection
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_yaml_content_type() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "content-type",
            "application/x-yaml".parse().unwrap(),
        );

        let yaml_body = br#"
name: Test Workflow
components:
  - id: start
    type: trigger
  - id: end
    type: stop
connections:
  - from: start
    to: end
"#;

        let result = parse_workflow_body(&headers, yaml_body);
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        let wf = result.unwrap();
        assert_eq!(wf.name, "Test Workflow");
        assert_eq!(wf.components.len(), 2);
    }

    #[test]
    fn test_parse_json_content_type() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "content-type",
            "application/json".parse().unwrap(),
        );

        let json_body = br#"{
            "name": "Test Workflow",
            "components": [
                { "id": "start", "type": "trigger" },
                { "id": "end", "type": "stop" }
            ],
            "connections": [
                { "from": "start", "to": "end" }
            ]
        }"#;

        let result = parse_workflow_body(&headers, json_body);
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        let wf = result.unwrap();
        assert_eq!(wf.name, "Test Workflow");
        assert_eq!(wf.components.len(), 2);
    }

    #[test]
    fn test_parse_text_yaml_content_type() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "content-type",
            "text/yaml".parse().unwrap(),
        );

        let yaml_body = br#"
name: YAML Test
components:
  - id: start
    type: trigger
  - id: end
    type: stop
connections:
  - from: start
    to: end
"#;

        let result = parse_workflow_body(&headers, yaml_body);
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        assert_eq!(result.unwrap().name, "YAML Test");
    }

    #[test]
    fn test_parse_defaults_to_json_when_no_content_type() {
        let headers = HeaderMap::new();

        let json_body = br#"{
            "name": "Default JSON",
            "components": [
                { "id": "start", "type": "trigger" },
                { "id": "end", "type": "stop" }
            ],
            "connections": [
                { "from": "start", "to": "end" }
            ]
        }"#;

        let result = parse_workflow_body(&headers, json_body);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "Default JSON");
    }

    #[test]
    fn test_parse_invalid_yaml_returns_error() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "content-type",
            "application/x-yaml".parse().unwrap(),
        );

        let bad_body = b"this is: [not valid: yaml: {{{}}}";
        let result = parse_workflow_body(&headers, bad_body);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_json_returns_error() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "content-type",
            "application/json".parse().unwrap(),
        );

        let bad_body = b"{ not json }";
        let result = parse_workflow_body(&headers, bad_body);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // YAML parsing + transformation end-to-end
    // -----------------------------------------------------------------------

    #[test]
    fn test_yaml_parse_and_transform() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "content-type",
            "application/x-yaml".parse().unwrap(),
        );

        let yaml_body = br#"
name: E2E Test Workflow
description: Tests the full parse-and-transform path
components:
  - id: start
    type: trigger
  - id: fetch
    type: http_request
    config:
      name: fetchData
      url: "https://api.example.com"
      method: GET
  - id: end
    type: stop
connections:
  - from: start
    to: fetch
  - from: fetch
    to: end
"#;

        let yaml_wf = parse_workflow_body(&headers, yaml_body).unwrap();
        let definition = yaml_format::transform(&yaml_wf).unwrap();

        assert_eq!(definition.name, "E2E Test Workflow");
        assert_eq!(definition.nodes.len(), 3);
        assert_eq!(definition.edges.len(), 2);

        // Transformed definition should pass validation.
        let validation_result = validation::validate(&definition);
        assert!(
            validation_result.is_valid(),
            "Transformed workflow should be valid, errors: {:?}",
            validation_result.errors
        );
    }

    #[test]
    fn test_json_parse_and_transform() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "content-type",
            "application/json".parse().unwrap(),
        );

        let json_body = br#"{
            "name": "JSON E2E Workflow",
            "components": [
                { "id": "start", "type": "trigger" },
                { "id": "process", "type": "activity", "config": { "name": "doWork" } },
                { "id": "end", "type": "stop" }
            ],
            "connections": [
                { "from": "start", "to": "process" },
                { "from": "process", "to": "end" }
            ]
        }"#;

        let yaml_wf = parse_workflow_body(&headers, json_body).unwrap();
        let definition = yaml_format::transform(&yaml_wf).unwrap();

        assert_eq!(definition.name, "JSON E2E Workflow");
        assert_eq!(definition.nodes.len(), 3);

        let validation_result = validation::validate(&definition);
        assert!(
            validation_result.is_valid(),
            "Transformed workflow should be valid, errors: {:?}",
            validation_result.errors
        );
    }

    // -----------------------------------------------------------------------
    // Validation endpoint logic
    // -----------------------------------------------------------------------

    #[test]
    fn test_convert_validation_error_no_start() {
        let err = validation::ValidationError::NoStartNode;
        let response = convert_validation_error(&err);
        assert_eq!(response.code, "NO_START_NODE");
        assert_eq!(response.severity, "error");
        assert!(response.node_id.is_none());
    }

    #[test]
    fn test_convert_validation_error_with_node_id() {
        let err = validation::ValidationError::OrphanNode("node_42".to_string());
        let response = convert_validation_error(&err);
        assert_eq!(response.code, "ORPHAN_NODE");
        assert_eq!(response.node_id, Some("node_42".to_string()));
    }

    #[test]
    fn test_convert_validation_error_missing_activity_name() {
        let err = validation::ValidationError::MissingActivityName {
            node_id: "my_activity".to_string(),
        };
        let response = convert_validation_error(&err);
        assert_eq!(response.code, "MISSING_ACTIVITY_NAME");
        assert_eq!(response.node_id, Some("my_activity".to_string()));
    }

    // -----------------------------------------------------------------------
    // Error response format
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
    fn test_workflow_error_unauthorized() {
        let err = WorkflowError::unauthorized();
        assert_eq!(err.status, StatusCode::UNAUTHORIZED);
        assert_eq!(err.code, "UNAUTHORIZED");
    }

    #[test]
    fn test_workflow_error_bad_request() {
        let err = WorkflowError::bad_request("invalid input");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.code, "BAD_REQUEST");
        assert_eq!(err.message, "invalid input");
    }

    #[test]
    fn test_workflow_error_not_found() {
        let err = WorkflowError::not_found("Workflow 'abc' not found");
        assert_eq!(err.status, StatusCode::NOT_FOUND);
        assert_eq!(err.code, "NOT_FOUND");
    }

    // -----------------------------------------------------------------------
    // Auth extraction (require_auth is now async + validates against Supabase,
    // so token-presence tests are covered by auth::extract_bearer_token tests
    // in auth.rs. The integration path is tested via ignored Supabase tests.)
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Integration tests that need Supabase (marked #[ignore])
    // -----------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_create_workflow_integration() {
        // This test would POST a YAML workflow to a real Supabase instance.
        // It is skipped in CI unless Supabase is available.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_list_workflows_integration() {
        // This test would list workflows from a real Supabase instance.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_get_workflow_integration() {
        // This test would fetch a single workflow by ID from Supabase.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_update_workflow_integration() {
        // This test would update a workflow in Supabase.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_delete_workflow_integration() {
        // This test would delete a workflow from Supabase.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_validate_workflow_integration() {
        // This test would validate a stored workflow from Supabase.
    }

    // -----------------------------------------------------------------------
    // Catalog types serialization / deserialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_catalog_list_response_serialization() {
        let response = CatalogListResponse {
            services: vec![WorkflowSummary {
                id: "svc-1".to_string(),
                name: "Public Service".to_string(),
                description: Some("A shared service".to_string()),
                status_id: DRAFT_STATUS_ID.to_string(),
                version: "1.0.0".to_string(),
                created_at: "2025-01-01T00:00:00Z".to_string(),
            }],
            total: 1,
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["total"], 1);
        assert_eq!(json["services"][0]["name"], "Public Service");
        assert_eq!(json["services"][0]["description"], "A shared service");
    }

    #[test]
    fn test_publish_response_serialization() {
        let response = PublishResponse {
            status: "published".to_string(),
            message: "Service 'abc' is now public in the catalog".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["status"], "published");
        assert!(json["message"].as_str().unwrap().contains("public"));
    }

    #[test]
    fn test_import_request_deserialization() {
        let json_str = r#"{"project_id": "proj-123"}"#;
        let req: ImportServiceRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(req.project_id, "proj-123");
    }

    #[test]
    fn test_visibility_constants() {
        // Verify the UUID format and distinct values.
        assert_eq!(
            PRIVATE_VISIBILITY_ID,
            "00000000-0000-0000-0000-000000000001"
        );
        assert_eq!(
            TEAM_VISIBILITY_ID,
            "00000000-0000-0000-0000-000000000002"
        );
        assert_eq!(
            PUBLIC_VISIBILITY_ID,
            "00000000-0000-0000-0000-000000000003"
        );

        // Private and default should be the same.
        assert_eq!(PRIVATE_VISIBILITY_ID, DEFAULT_VISIBILITY_ID);

        // All three should be distinct.
        assert_ne!(PRIVATE_VISIBILITY_ID, TEAM_VISIBILITY_ID);
        assert_ne!(TEAM_VISIBILITY_ID, PUBLIC_VISIBILITY_ID);
        assert_ne!(PRIVATE_VISIBILITY_ID, PUBLIC_VISIBILITY_ID);
    }

    // -----------------------------------------------------------------------
    // Catalog integration tests (marked #[ignore])
    // -----------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_list_catalog_integration() {
        // This test would browse the public catalog from Supabase.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_publish_service_integration() {
        // This test would publish a service and verify visibility change.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_unpublish_service_integration() {
        // This test would unpublish a service and verify it becomes private.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_import_service_integration() {
        // This test would import a public service into a project and verify
        // lineage via parent_workflow_id.
    }
}
