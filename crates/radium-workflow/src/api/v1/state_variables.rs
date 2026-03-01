//! State variable CRUD endpoints (service-scoped and project-scoped).
//!
//! Provides REST API endpoints for creating, reading, updating, and deleting
//! state variables stored in Supabase. Service-scoped variables live in the
//! `workflow_state_variables` table; project-scoped variables live in the
//! `project_state_variables` table.

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

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Allowed values for the `type` field on a state variable.
const VALID_VARIABLE_TYPES: &[&str] = &["string", "number", "boolean", "object", "array"];

/// Allowed values for the `storage_type` field on a state variable.
const VALID_STORAGE_TYPES: &[&str] = &["database", "cache"];

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
pub struct StateVarError {
    status: StatusCode,
    code: String,
    message: String,
    details: Vec<String>,
}

impl StateVarError {
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

impl IntoResponse for StateVarError {
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
// Request types
// ---------------------------------------------------------------------------

/// Body for creating a new state variable.
#[derive(Debug, Deserialize)]
pub struct CreateVariableRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub storage_type: String,
    pub schema: Option<serde_json::Value>,
    pub storage_config: Option<serde_json::Value>,
}

/// Body for updating an existing state variable.
#[derive(Debug, Deserialize)]
pub struct UpdateVariableRequest {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub r#type: Option<String>,
    pub storage_type: Option<String>,
    pub schema: Option<serde_json::Value>,
    pub storage_config: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Supabase row types (what we INSERT / UPDATE)
// ---------------------------------------------------------------------------

/// Body sent to Supabase for creating a service-scoped state variable.
#[derive(Debug, Serialize)]
struct InsertServiceVariableRow {
    workflow_id: String,
    name: String,
    #[serde(rename = "type")]
    r#type: String,
    storage_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    storage_config: Option<serde_json::Value>,
}

/// Body sent to Supabase for creating a project-scoped state variable.
#[derive(Debug, Serialize)]
struct InsertProjectVariableRow {
    project_id: String,
    name: String,
    #[serde(rename = "type")]
    r#type: String,
    storage_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    storage_config: Option<serde_json::Value>,
}

/// Body sent to Supabase for updating a state variable (service or project).
#[derive(Debug, Serialize)]
struct UpdateVariableRow {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    var_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    storage_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    storage_config: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Full service-scoped variable response.
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceVariableResponse {
    pub id: String,
    pub workflow_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub storage_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_config: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

/// Full project-scoped variable response.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectVariableResponse {
    pub id: String,
    pub project_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub storage_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_config: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

/// Generic list response envelope.
#[derive(Debug, Serialize)]
pub struct VariableListResponse<T> {
    pub variables: Vec<T>,
    pub total: usize,
}

/// Minimal response shape from workflows select (for ownership verification).
#[derive(Debug, Deserialize)]
struct WorkflowOwnerRow {
    #[allow(dead_code)]
    id: String,
}

/// Minimal response shape from projects select (for ownership verification).
#[derive(Debug, Deserialize)]
struct ProjectOwnerRow {
    #[allow(dead_code)]
    id: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Validate the Bearer token against the Supabase `api_keys` table and check
/// the per-user rate limit.
async fn require_auth(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<AuthenticatedUser, StateVarError> {
    let mut request = axum::http::Request::builder()
        .uri("http://localhost/")
        .body(())
        .unwrap();
    *request.headers_mut() = headers.clone();
    let (parts, ()) = request.into_parts();

    let token =
        auth::extract_bearer_token(&parts).ok_or_else(StateVarError::unauthorized)?;

    let user = auth::validate_api_key(
        state.supabase.http_client(),
        state.supabase.url(),
        state.supabase.service_role_key(),
        &token,
    )
    .await
    .map_err(|_| StateVarError::unauthorized())?;

    let result = state.rate_limiter.check(&user.user_id);
    if !result.allowed {
        return Err(StateVarError {
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

/// Verify the authenticated user owns the given service (workflow).
async fn verify_service_ownership(
    state: &AppState,
    service_id: &str,
    user_id: &str,
) -> Result<(), StateVarError> {
    let user_filter = format!("eq.{user_id}");
    let _service: WorkflowOwnerRow = state
        .supabase
        .select_one(
            "workflows",
            &[
                ("id", &format!("eq.{service_id}")),
                ("created_by", &user_filter),
                ("select", "id"),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                StateVarError::not_found(format!("Service '{service_id}' not found"))
            }
            _ => StateVarError::from_supabase(&e),
        })?;

    Ok(())
}

/// Verify the authenticated user owns the given project.
async fn verify_project_ownership(
    state: &AppState,
    project_id: &str,
    user_id: &str,
) -> Result<(), StateVarError> {
    let user_filter = format!("eq.{user_id}");
    let _project: ProjectOwnerRow = state
        .supabase
        .select_one(
            "projects",
            &[
                ("id", &format!("eq.{project_id}")),
                ("created_by", &user_filter),
                ("select", "id"),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                StateVarError::not_found(format!("Project '{project_id}' not found"))
            }
            _ => StateVarError::from_supabase(&e),
        })?;

    Ok(())
}

/// Validate a create-variable request body.
fn validate_variable_request(req: &CreateVariableRequest) -> Result<(), StateVarError> {
    if req.name.trim().is_empty() {
        return Err(StateVarError::bad_request(
            "Variable name must not be empty",
        ));
    }

    if !VALID_VARIABLE_TYPES.contains(&req.r#type.as_str()) {
        return Err(StateVarError::bad_request(format!(
            "Invalid type '{}'. Must be one of: {}",
            req.r#type,
            VALID_VARIABLE_TYPES.join(", ")
        )));
    }

    if !VALID_STORAGE_TYPES.contains(&req.storage_type.as_str()) {
        return Err(StateVarError::bad_request(format!(
            "Invalid storage_type '{}'. Must be one of: {}",
            req.storage_type,
            VALID_STORAGE_TYPES.join(", ")
        )));
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Service-scoped handlers
// ---------------------------------------------------------------------------

/// Column selection for `workflow_state_variables`.
const SVC_VAR_SELECT: &str =
    "id,workflow_id,name,type,storage_type,schema,storage_config,created_at,updated_at";

/// `POST /v1/services/{id}/variables` -- Create a service-scoped state variable.
pub async fn create_service_variable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<CreateVariableRequest>,
) -> Result<impl IntoResponse, StateVarError> {
    let user = require_auth(&headers, &state).await?;
    verify_service_ownership(&state, &id, &user.user_id).await?;
    validate_variable_request(&body)?;

    let row = InsertServiceVariableRow {
        workflow_id: id,
        name: body.name.trim().to_string(),
        r#type: body.r#type,
        storage_type: body.storage_type,
        schema: body.schema,
        storage_config: body.storage_config,
    };

    let created: ServiceVariableResponse = state
        .supabase
        .insert("workflow_state_variables", &row)
        .await
        .map_err(|e| StateVarError::from_supabase(&e))?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// `GET /v1/services/{id}/variables` -- List service-scoped state variables.
pub async fn list_service_variables(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<VariableListResponse<ServiceVariableResponse>>, StateVarError> {
    let user = require_auth(&headers, &state).await?;
    verify_service_ownership(&state, &id, &user.user_id).await?;

    let workflow_filter = format!("eq.{id}");
    let variables: Vec<ServiceVariableResponse> = state
        .supabase
        .select(
            "workflow_state_variables",
            &[
                ("workflow_id", &workflow_filter),
                ("select", SVC_VAR_SELECT),
                ("order", "created_at.desc"),
            ],
        )
        .await
        .map_err(|e| StateVarError::from_supabase(&e))?;

    let total = variables.len();
    Ok(Json(VariableListResponse { variables, total }))
}

/// `GET /v1/services/{id}/variables/{var_id}` -- Get a single service variable.
pub async fn get_service_variable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((id, var_id)): Path<(String, String)>,
) -> Result<Json<ServiceVariableResponse>, StateVarError> {
    let user = require_auth(&headers, &state).await?;
    verify_service_ownership(&state, &id, &user.user_id).await?;

    let variable: ServiceVariableResponse = state
        .supabase
        .select_one(
            "workflow_state_variables",
            &[
                ("id", &format!("eq.{var_id}")),
                ("workflow_id", &format!("eq.{id}")),
                ("select", SVC_VAR_SELECT),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                StateVarError::not_found(format!("Variable '{var_id}' not found"))
            }
            _ => StateVarError::from_supabase(&e),
        })?;

    Ok(Json(variable))
}

/// `PUT /v1/services/{id}/variables/{var_id}` -- Update a service variable.
pub async fn update_service_variable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((id, var_id)): Path<(String, String)>,
    Json(body): Json<UpdateVariableRequest>,
) -> Result<Json<ServiceVariableResponse>, StateVarError> {
    let user = require_auth(&headers, &state).await?;
    verify_service_ownership(&state, &id, &user.user_id).await?;

    // Validate name if provided.
    if let Some(ref name) = body.name {
        if name.trim().is_empty() {
            return Err(StateVarError::bad_request(
                "Variable name must not be empty",
            ));
        }
    }

    // Validate type if provided.
    if let Some(ref var_type) = body.r#type {
        if !VALID_VARIABLE_TYPES.contains(&var_type.as_str()) {
            return Err(StateVarError::bad_request(format!(
                "Invalid type '{}'. Must be one of: {}",
                var_type,
                VALID_VARIABLE_TYPES.join(", ")
            )));
        }
    }

    // Validate storage_type if provided.
    if let Some(ref storage_type) = body.storage_type {
        if !VALID_STORAGE_TYPES.contains(&storage_type.as_str()) {
            return Err(StateVarError::bad_request(format!(
                "Invalid storage_type '{}'. Must be one of: {}",
                storage_type,
                VALID_STORAGE_TYPES.join(", ")
            )));
        }
    }

    let update_body = UpdateVariableRow {
        name: body.name,
        var_type: body.r#type,
        storage_type: body.storage_type,
        schema: body.schema,
        storage_config: body.storage_config,
    };

    let updated: Vec<ServiceVariableResponse> = state
        .supabase
        .update(
            "workflow_state_variables",
            &[
                ("id", &format!("eq.{var_id}")),
                ("workflow_id", &format!("eq.{id}")),
            ],
            &update_body,
        )
        .await
        .map_err(|e| StateVarError::from_supabase(&e))?;

    let variable = updated.into_iter().next().ok_or_else(|| {
        StateVarError::not_found(format!("Variable '{var_id}' not found"))
    })?;

    Ok(Json(variable))
}

/// `DELETE /v1/services/{id}/variables/{var_id}` -- Delete a service variable.
pub async fn delete_service_variable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((id, var_id)): Path<(String, String)>,
) -> Result<StatusCode, StateVarError> {
    let user = require_auth(&headers, &state).await?;
    verify_service_ownership(&state, &id, &user.user_id).await?;

    state
        .supabase
        .delete(
            "workflow_state_variables",
            &[
                ("id", &format!("eq.{var_id}")),
                ("workflow_id", &format!("eq.{id}")),
            ],
        )
        .await
        .map_err(|e| StateVarError::from_supabase(&e))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Project-scoped handlers
// ---------------------------------------------------------------------------

/// Column selection for `project_state_variables`.
const PROJ_VAR_SELECT: &str =
    "id,project_id,name,type,storage_type,schema,storage_config,created_at,updated_at";

/// `POST /v1/projects/{id}/variables` -- Create a project-scoped state variable.
pub async fn create_project_variable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<CreateVariableRequest>,
) -> Result<impl IntoResponse, StateVarError> {
    let user = require_auth(&headers, &state).await?;
    verify_project_ownership(&state, &id, &user.user_id).await?;
    validate_variable_request(&body)?;

    let row = InsertProjectVariableRow {
        project_id: id,
        name: body.name.trim().to_string(),
        r#type: body.r#type,
        storage_type: body.storage_type,
        schema: body.schema,
        storage_config: body.storage_config,
    };

    let created: ProjectVariableResponse = state
        .supabase
        .insert("project_state_variables", &row)
        .await
        .map_err(|e| StateVarError::from_supabase(&e))?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// `GET /v1/projects/{id}/variables` -- List project-scoped state variables.
pub async fn list_project_variables(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<VariableListResponse<ProjectVariableResponse>>, StateVarError> {
    let user = require_auth(&headers, &state).await?;
    verify_project_ownership(&state, &id, &user.user_id).await?;

    let project_filter = format!("eq.{id}");
    let variables: Vec<ProjectVariableResponse> = state
        .supabase
        .select(
            "project_state_variables",
            &[
                ("project_id", &project_filter),
                ("select", PROJ_VAR_SELECT),
                ("order", "created_at.desc"),
            ],
        )
        .await
        .map_err(|e| StateVarError::from_supabase(&e))?;

    let total = variables.len();
    Ok(Json(VariableListResponse { variables, total }))
}

/// `GET /v1/projects/{id}/variables/{var_id}` -- Get a single project variable.
pub async fn get_project_variable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((id, var_id)): Path<(String, String)>,
) -> Result<Json<ProjectVariableResponse>, StateVarError> {
    let user = require_auth(&headers, &state).await?;
    verify_project_ownership(&state, &id, &user.user_id).await?;

    let variable: ProjectVariableResponse = state
        .supabase
        .select_one(
            "project_state_variables",
            &[
                ("id", &format!("eq.{var_id}")),
                ("project_id", &format!("eq.{id}")),
                ("select", PROJ_VAR_SELECT),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                StateVarError::not_found(format!("Variable '{var_id}' not found"))
            }
            _ => StateVarError::from_supabase(&e),
        })?;

    Ok(Json(variable))
}

/// `PUT /v1/projects/{id}/variables/{var_id}` -- Update a project variable.
pub async fn update_project_variable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((id, var_id)): Path<(String, String)>,
    Json(body): Json<UpdateVariableRequest>,
) -> Result<Json<ProjectVariableResponse>, StateVarError> {
    let user = require_auth(&headers, &state).await?;
    verify_project_ownership(&state, &id, &user.user_id).await?;

    // Validate name if provided.
    if let Some(ref name) = body.name {
        if name.trim().is_empty() {
            return Err(StateVarError::bad_request(
                "Variable name must not be empty",
            ));
        }
    }

    // Validate type if provided.
    if let Some(ref var_type) = body.r#type {
        if !VALID_VARIABLE_TYPES.contains(&var_type.as_str()) {
            return Err(StateVarError::bad_request(format!(
                "Invalid type '{}'. Must be one of: {}",
                var_type,
                VALID_VARIABLE_TYPES.join(", ")
            )));
        }
    }

    // Validate storage_type if provided.
    if let Some(ref storage_type) = body.storage_type {
        if !VALID_STORAGE_TYPES.contains(&storage_type.as_str()) {
            return Err(StateVarError::bad_request(format!(
                "Invalid storage_type '{}'. Must be one of: {}",
                storage_type,
                VALID_STORAGE_TYPES.join(", ")
            )));
        }
    }

    let update_body = UpdateVariableRow {
        name: body.name,
        var_type: body.r#type,
        storage_type: body.storage_type,
        schema: body.schema,
        storage_config: body.storage_config,
    };

    let updated: Vec<ProjectVariableResponse> = state
        .supabase
        .update(
            "project_state_variables",
            &[
                ("id", &format!("eq.{var_id}")),
                ("project_id", &format!("eq.{id}")),
            ],
            &update_body,
        )
        .await
        .map_err(|e| StateVarError::from_supabase(&e))?;

    let variable = updated.into_iter().next().ok_or_else(|| {
        StateVarError::not_found(format!("Variable '{var_id}' not found"))
    })?;

    Ok(Json(variable))
}

/// `DELETE /v1/projects/{id}/variables/{var_id}` -- Delete a project variable.
pub async fn delete_project_variable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((id, var_id)): Path<(String, String)>,
) -> Result<StatusCode, StateVarError> {
    let user = require_auth(&headers, &state).await?;
    verify_project_ownership(&state, &id, &user.user_id).await?;

    state
        .supabase
        .delete(
            "project_state_variables",
            &[
                ("id", &format!("eq.{var_id}")),
                ("project_id", &format!("eq.{id}")),
            ],
        )
        .await
        .map_err(|e| StateVarError::from_supabase(&e))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Constants
    // -----------------------------------------------------------------------

    #[test]
    fn test_valid_variable_types() {
        assert_eq!(VALID_VARIABLE_TYPES.len(), 5);
        assert!(VALID_VARIABLE_TYPES.contains(&"string"));
        assert!(VALID_VARIABLE_TYPES.contains(&"number"));
        assert!(VALID_VARIABLE_TYPES.contains(&"boolean"));
        assert!(VALID_VARIABLE_TYPES.contains(&"object"));
        assert!(VALID_VARIABLE_TYPES.contains(&"array"));
        assert!(!VALID_VARIABLE_TYPES.contains(&"integer"));
        assert!(!VALID_VARIABLE_TYPES.contains(&"map"));
    }

    #[test]
    fn test_valid_storage_types() {
        assert_eq!(VALID_STORAGE_TYPES.len(), 2);
        assert!(VALID_STORAGE_TYPES.contains(&"database"));
        assert!(VALID_STORAGE_TYPES.contains(&"cache"));
        assert!(!VALID_STORAGE_TYPES.contains(&"memory"));
        assert!(!VALID_STORAGE_TYPES.contains(&"file"));
    }

    // -----------------------------------------------------------------------
    // Request deserialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_create_request_deserialization() {
        let json = r#"{
            "name": "counter",
            "type": "number",
            "storage_type": "database",
            "schema": {"minimum": 0},
            "storage_config": {"ttl": 3600}
        }"#;
        let req: CreateVariableRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "counter");
        assert_eq!(req.r#type, "number");
        assert_eq!(req.storage_type, "database");
        assert!(req.schema.is_some());
        assert!(req.storage_config.is_some());
    }

    #[test]
    fn test_create_request_minimal() {
        let json = r#"{
            "name": "flag",
            "type": "boolean",
            "storage_type": "cache"
        }"#;
        let req: CreateVariableRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "flag");
        assert_eq!(req.r#type, "boolean");
        assert_eq!(req.storage_type, "cache");
        assert!(req.schema.is_none());
        assert!(req.storage_config.is_none());
    }

    // -----------------------------------------------------------------------
    // Insert row serialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_create_request_serialization() {
        let row = InsertServiceVariableRow {
            workflow_id: "wf-123".to_string(),
            name: "counter".to_string(),
            r#type: "number".to_string(),
            storage_type: "database".to_string(),
            schema: Some(serde_json::json!({"minimum": 0})),
            storage_config: None,
        };

        let json = serde_json::to_value(&row).unwrap();
        assert_eq!(json["workflow_id"], "wf-123");
        assert_eq!(json["name"], "counter");
        assert_eq!(json["type"], "number");
        assert_eq!(json["storage_type"], "database");
        assert!(json.get("schema").is_some());
        // storage_config is None -> should be omitted
        assert!(
            json.get("storage_config").is_none(),
            "storage_config should be omitted when None"
        );
    }

    #[test]
    fn test_insert_project_variable_row_serialization() {
        let row = InsertProjectVariableRow {
            project_id: "proj-1".to_string(),
            name: "settings".to_string(),
            r#type: "object".to_string(),
            storage_type: "database".to_string(),
            schema: None,
            storage_config: Some(serde_json::json!({"compress": true})),
        };

        let json = serde_json::to_value(&row).unwrap();
        assert_eq!(json["project_id"], "proj-1");
        assert_eq!(json["name"], "settings");
        assert_eq!(json["type"], "object");
        assert!(json.get("schema").is_none(), "schema should be omitted when None");
        assert!(json.get("storage_config").is_some());
    }

    // -----------------------------------------------------------------------
    // Update row serialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_update_variable_row_serialization() {
        let row = UpdateVariableRow {
            name: Some("renamed".to_string()),
            var_type: Some("string".to_string()),
            storage_type: None,
            schema: None,
            storage_config: None,
        };

        let json = serde_json::to_value(&row).unwrap();
        assert_eq!(json["name"], "renamed");
        assert_eq!(json["type"], "string");
        assert!(
            json.get("storage_type").is_none(),
            "storage_type should be omitted when None"
        );
        assert!(json.get("schema").is_none(), "schema should be omitted when None");
        assert!(
            json.get("storage_config").is_none(),
            "storage_config should be omitted when None"
        );
    }

    #[test]
    fn test_update_variable_row_all_none() {
        let row = UpdateVariableRow {
            name: None,
            var_type: None,
            storage_type: None,
            schema: None,
            storage_config: None,
        };

        let json = serde_json::to_value(&row).unwrap();
        let obj = json.as_object().unwrap();
        assert!(obj.is_empty(), "All-None update row should serialize to empty object");
    }

    // -----------------------------------------------------------------------
    // Response deserialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_variable_response_deserialization() {
        let json = serde_json::json!({
            "id": "var-1",
            "workflow_id": "wf-1",
            "name": "counter",
            "type": "number",
            "storage_type": "database",
            "schema": {"minimum": 0},
            "storage_config": {"ttl": 3600},
            "created_at": "2026-03-01T00:00:00Z",
            "updated_at": "2026-03-01T00:00:00Z"
        });
        let resp: ServiceVariableResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.id, "var-1");
        assert_eq!(resp.workflow_id, "wf-1");
        assert_eq!(resp.name, "counter");
        assert_eq!(resp.r#type, "number");
        assert_eq!(resp.storage_type, "database");
        assert!(resp.schema.is_some());
        assert!(resp.storage_config.is_some());
    }

    #[test]
    fn test_variable_response_optional_fields_omitted() {
        let resp = ServiceVariableResponse {
            id: "var-1".to_string(),
            workflow_id: "wf-1".to_string(),
            name: "flag".to_string(),
            r#type: "boolean".to_string(),
            storage_type: "cache".to_string(),
            schema: None,
            storage_config: None,
            created_at: "2026-03-01T00:00:00Z".to_string(),
            updated_at: "2026-03-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_value(&resp).unwrap();
        assert!(
            json.get("schema").is_none(),
            "schema should be omitted when None"
        );
        assert!(
            json.get("storage_config").is_none(),
            "storage_config should be omitted when None"
        );
        // Required fields present
        assert_eq!(json["id"], "var-1");
        assert_eq!(json["type"], "boolean");
        assert_eq!(json["storage_type"], "cache");
    }

    #[test]
    fn test_project_variable_response_deserialization() {
        let json = serde_json::json!({
            "id": "pvar-1",
            "project_id": "proj-1",
            "name": "config",
            "type": "object",
            "storage_type": "database",
            "created_at": "2026-03-01T00:00:00Z",
            "updated_at": "2026-03-01T00:00:00Z"
        });
        let resp: ProjectVariableResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.id, "pvar-1");
        assert_eq!(resp.project_id, "proj-1");
        assert_eq!(resp.r#type, "object");
        assert!(resp.schema.is_none());
    }

    // -----------------------------------------------------------------------
    // List response serialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_variable_list_response_serialization() {
        let response = VariableListResponse {
            variables: vec![
                ServiceVariableResponse {
                    id: "v1".to_string(),
                    workflow_id: "wf-1".to_string(),
                    name: "counter".to_string(),
                    r#type: "number".to_string(),
                    storage_type: "database".to_string(),
                    schema: None,
                    storage_config: None,
                    created_at: "2026-03-01T00:00:00Z".to_string(),
                    updated_at: "2026-03-01T00:00:00Z".to_string(),
                },
                ServiceVariableResponse {
                    id: "v2".to_string(),
                    workflow_id: "wf-1".to_string(),
                    name: "items".to_string(),
                    r#type: "array".to_string(),
                    storage_type: "cache".to_string(),
                    schema: Some(serde_json::json!({"items": {"type": "string"}})),
                    storage_config: None,
                    created_at: "2026-03-02T00:00:00Z".to_string(),
                    updated_at: "2026-03-02T00:00:00Z".to_string(),
                },
            ],
            total: 2,
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["total"], 2);
        assert_eq!(json["variables"].as_array().unwrap().len(), 2);
        assert_eq!(json["variables"][0]["name"], "counter");
        assert_eq!(json["variables"][1]["name"], "items");
        assert!(json["variables"][0].get("schema").is_none());
        assert!(json["variables"][1].get("schema").is_some());
    }

    // -----------------------------------------------------------------------
    // Error constructors
    // -----------------------------------------------------------------------

    #[test]
    fn test_state_var_error_unauthorized() {
        let err = StateVarError::unauthorized();
        assert_eq!(err.status, StatusCode::UNAUTHORIZED);
        assert_eq!(err.code, "UNAUTHORIZED");
        assert_eq!(
            err.message,
            "Authorization header with Bearer token is required"
        );
    }

    #[test]
    fn test_state_var_error_bad_request() {
        let err = StateVarError::bad_request("name is required");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.code, "BAD_REQUEST");
        assert_eq!(err.message, "name is required");
    }

    #[test]
    fn test_state_var_error_not_found() {
        let err = StateVarError::not_found("Variable 'xyz' not found");
        assert_eq!(err.status, StatusCode::NOT_FOUND);
        assert_eq!(err.code, "NOT_FOUND");
        assert_eq!(err.message, "Variable 'xyz' not found");
    }

    #[test]
    fn test_state_var_error_internal() {
        let err = StateVarError::internal("something broke");
        assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.code, "INTERNAL_ERROR");
        assert_eq!(err.message, "something broke");
    }

    // -----------------------------------------------------------------------
    // Validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_empty_name_fails() {
        let req = CreateVariableRequest {
            name: "  ".to_string(),
            r#type: "string".to_string(),
            storage_type: "database".to_string(),
            schema: None,
            storage_config: None,
        };
        let result = validate_variable_request(&req);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "BAD_REQUEST");
        assert!(err.message.contains("name"));
    }

    #[test]
    fn test_validate_invalid_type_fails() {
        let req = CreateVariableRequest {
            name: "counter".to_string(),
            r#type: "integer".to_string(),
            storage_type: "database".to_string(),
            schema: None,
            storage_config: None,
        };
        let result = validate_variable_request(&req);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "BAD_REQUEST");
        assert!(err.message.contains("integer"));
    }

    #[test]
    fn test_validate_invalid_storage_type_fails() {
        let req = CreateVariableRequest {
            name: "counter".to_string(),
            r#type: "number".to_string(),
            storage_type: "memory".to_string(),
            schema: None,
            storage_config: None,
        };
        let result = validate_variable_request(&req);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "BAD_REQUEST");
        assert!(err.message.contains("memory"));
    }

    #[test]
    fn test_validate_valid_request_passes() {
        let req = CreateVariableRequest {
            name: "counter".to_string(),
            r#type: "number".to_string(),
            storage_type: "database".to_string(),
            schema: Some(serde_json::json!({"minimum": 0})),
            storage_config: None,
        };
        assert!(validate_variable_request(&req).is_ok());
    }

    // -----------------------------------------------------------------------
    // Error envelope serialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_error_envelope_serialization() {
        let envelope = ErrorEnvelope {
            error: ErrorBody {
                code: "BAD_REQUEST".to_string(),
                message: "Invalid input".to_string(),
                details: vec!["detail 1".to_string()],
            },
        };

        let json = serde_json::to_value(&envelope).unwrap();
        assert_eq!(json["error"]["code"], "BAD_REQUEST");
        assert_eq!(json["error"]["message"], "Invalid input");
        assert_eq!(json["error"]["details"][0], "detail 1");
    }

    // -----------------------------------------------------------------------
    // Integration tests that need Supabase (marked #[ignore])
    // -----------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_create_service_variable_integration() {
        // This test would POST a variable to a real Supabase instance.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_list_service_variables_integration() {
        // This test would list variables from a real Supabase instance.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_create_project_variable_integration() {
        // This test would POST a project variable to a real Supabase instance.
    }
}
