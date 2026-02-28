//! Project CRUD endpoints.
//!
//! Provides REST API endpoints for creating, reading, updating, and deleting
//! projects stored in Supabase. Each project auto-provisions a Temporal task
//! queue on creation.

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

/// Status ID for a deployed workflow.
const DEPLOYED_STATUS_ID: &str = "00000000-0000-0000-0000-000000000003";

/// Status ID for a draft workflow.
const DRAFT_STATUS_ID: &str = "00000000-0000-0000-0000-000000000001";

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Full project response returned from GET and create/update operations.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub task_queue_name: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Summary response for listing projects.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

/// List response envelope.
#[derive(Debug, Serialize)]
pub struct ProjectListResponse {
    pub projects: Vec<ProjectSummary>,
    pub total: usize,
}

/// Response returned after deploying all services in a project.
#[derive(Debug, Serialize)]
pub struct ProjectDeployResponse {
    pub project_id: String,
    pub services_deployed: usize,
    pub services_failed: usize,
    pub results: Vec<ServiceDeployResult>,
}

/// Result of deploying a single service within a project deploy operation.
#[derive(Debug, Serialize)]
pub struct ServiceDeployResult {
    pub service_id: String,
    pub service_name: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Aggregated deployment status for a project.
#[derive(Debug, Serialize)]
pub struct ProjectStatusResponse {
    pub project_id: String,
    pub project_name: String,
    pub total_services: usize,
    pub deployed: usize,
    pub draft: usize,
    pub services: Vec<ServiceStatusSummary>,
}

/// Summary of a service's status within a project.
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceStatusSummary {
    pub id: String,
    pub name: String,
    pub status_id: String,
}

/// Response envelope for listing services in a project.
#[derive(Debug, Serialize)]
pub struct ProjectServiceListResponse {
    pub services: Vec<ServiceStatusSummary>,
    pub total: usize,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

/// Body for creating a new project.
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Body for updating an existing project.
#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: String,
    pub description: Option<String>,
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
pub struct ProjectError {
    status: StatusCode,
    code: String,
    message: String,
    details: Vec<String>,
}

impl ProjectError {
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

impl IntoResponse for ProjectError {
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

/// Body sent to Supabase for creating a task queue.
#[derive(Debug, Serialize)]
struct InsertTaskQueueRow {
    name: String,
    display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    created_by: String,
    is_default: bool,
}

/// Body sent to Supabase for creating a project.
#[derive(Debug, Serialize)]
struct InsertProjectRow {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    created_by: String,
    task_queue_name: String,
    is_active: bool,
}

/// Body sent to Supabase for updating a project.
#[derive(Debug, Serialize)]
struct UpdateProjectRow {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

/// Body sent to Supabase to update a workflow's status_id.
#[derive(Debug, Serialize)]
struct UpdateStatusRow {
    status_id: String,
}

/// Minimal response shape from task_queues insert (we only need to confirm it worked).
#[derive(Debug, Deserialize)]
struct TaskQueueInsertResponse {
    #[allow(dead_code)]
    name: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Validate the Bearer token against the Supabase `api_keys` table and check
/// the per-user rate limit.
///
/// Returns the authenticated user on success, or a `ProjectError` when the
/// token is missing, invalid, expired, revoked, or rate-limited.
async fn require_auth(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<AuthenticatedUser, ProjectError> {
    // Build minimal Parts to extract the bearer token.
    let mut request = axum::http::Request::builder()
        .uri("http://localhost/")
        .body(())
        .unwrap();
    *request.headers_mut() = headers.clone();
    let (parts, ()) = request.into_parts();

    let token =
        auth::extract_bearer_token(&parts).ok_or_else(ProjectError::unauthorized)?;

    let user = auth::validate_api_key(
        state.supabase.http_client(),
        state.supabase.url(),
        state.supabase.service_role_key(),
        &token,
    )
    .await
    .map_err(|_| ProjectError::unauthorized())?;

    // Check rate limit (keyed by user_id).
    let result = state.rate_limiter.check(&user.user_id);
    if !result.allowed {
        return Err(ProjectError {
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

/// Generate a kebab-case task queue name from a user ID prefix and project name.
///
/// Format: `{user_id[..8]}-{kebab(name)}-queue`
pub fn generate_queue_name(user_id: &str, name: &str) -> String {
    let user_prefix = &user_id[..user_id.len().min(8)];
    let kebab_name: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        // Collapse multiple consecutive hyphens into one.
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    format!("{user_prefix}-{kebab_name}-queue")
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `POST /v1/projects` -- Create a new project with an auto-provisioned task queue.
pub async fn create_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateProjectRequest>,
) -> Result<impl IntoResponse, ProjectError> {
    let user = require_auth(&headers, &state).await?;

    // Validate name is non-empty.
    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Err(ProjectError::bad_request("Project name must not be empty"));
    }

    // Generate the task queue name.
    let queue_name = generate_queue_name(&user.user_id, &name);

    // Insert the task queue first.
    let queue_row = InsertTaskQueueRow {
        name: queue_name.clone(),
        display_name: format!("{name} Queue"),
        description: body.description.as_ref().map(|d| format!("Task queue for project: {d}")),
        created_by: user.user_id.clone(),
        is_default: false,
    };

    let _queue: TaskQueueInsertResponse = state
        .supabase
        .insert("task_queues", &queue_row)
        .await
        .map_err(|e| ProjectError::from_supabase(&e))?;

    // Insert the project.
    let project_row = InsertProjectRow {
        name,
        description: body.description,
        created_by: user.user_id,
        task_queue_name: queue_name,
        is_active: true,
    };

    let created: ProjectResponse = state
        .supabase
        .insert("projects", &project_row)
        .await
        .map_err(|e| ProjectError::from_supabase(&e))?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// `GET /v1/projects` -- List all projects for the authenticated user.
pub async fn list_projects(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ProjectListResponse>, ProjectError> {
    let user = require_auth(&headers, &state).await?;

    let user_filter = format!("eq.{}", user.user_id);
    let projects: Vec<ProjectSummary> = state
        .supabase
        .select(
            "projects",
            &[
                ("select", "id,name,description,is_active,created_at"),
                ("order", "created_at.desc"),
                ("created_by", &user_filter),
            ],
        )
        .await
        .map_err(|e| ProjectError::from_supabase(&e))?;

    let total = projects.len();
    Ok(Json(ProjectListResponse { projects, total }))
}

/// `GET /v1/projects/:id` -- Get a single project by ID.
pub async fn get_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ProjectResponse>, ProjectError> {
    let user = require_auth(&headers, &state).await?;

    let user_filter = format!("eq.{}", user.user_id);
    let project: ProjectResponse = state
        .supabase
        .select_one(
            "projects",
            &[
                ("id", &format!("eq.{id}")),
                ("created_by", &user_filter),
                (
                    "select",
                    "id,name,description,task_queue_name,is_active,created_at,updated_at",
                ),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                ProjectError::not_found(format!("Project '{id}' not found"))
            }
            _ => ProjectError::from_supabase(&e),
        })?;

    Ok(Json(project))
}

/// `PUT /v1/projects/:id` -- Update a project.
pub async fn update_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectResponse>, ProjectError> {
    let user = require_auth(&headers, &state).await?;

    let update_body = UpdateProjectRow {
        name: body.name,
        description: body.description,
    };

    let user_filter = format!("eq.{}", user.user_id);
    let updated: Vec<ProjectResponse> = state
        .supabase
        .update(
            "projects",
            &[("id", &format!("eq.{id}")), ("created_by", &user_filter)],
            &update_body,
        )
        .await
        .map_err(|e| ProjectError::from_supabase(&e))?;

    let project = updated.into_iter().next().ok_or_else(|| {
        ProjectError::not_found(format!("Project '{id}' not found"))
    })?;

    Ok(Json(project))
}

/// `DELETE /v1/projects/:id` -- Delete a project.
pub async fn delete_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, ProjectError> {
    let user = require_auth(&headers, &state).await?;

    let user_filter = format!("eq.{}", user.user_id);
    state
        .supabase
        .delete(
            "projects",
            &[("id", &format!("eq.{id}")), ("created_by", &user_filter)],
        )
        .await
        .map_err(|e| ProjectError::from_supabase(&e))?;

    Ok(StatusCode::NO_CONTENT)
}

/// `POST /v1/projects/:id/deploy` -- Deploy all services in a project.
///
/// Updates the `status_id` of every workflow in the project to `DEPLOYED`.
// TODO: invoke full deploy pipeline (validation + codegen) per service
pub async fn deploy_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ProjectError> {
    let user = require_auth(&headers, &state).await?;

    // Verify project ownership.
    let user_filter = format!("eq.{}", user.user_id);
    let _project: ProjectResponse = state
        .supabase
        .select_one(
            "projects",
            &[
                ("id", &format!("eq.{id}")),
                ("created_by", &user_filter),
                (
                    "select",
                    "id,name,description,task_queue_name,is_active,created_at,updated_at",
                ),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                ProjectError::not_found(format!("Project '{id}' not found"))
            }
            _ => ProjectError::from_supabase(&e),
        })?;

    // Fetch all services in the project belonging to this user.
    let services: Vec<ServiceStatusSummary> = state
        .supabase
        .select(
            "workflows",
            &[
                ("select", "id,name,status_id"),
                ("project_id", &format!("eq.{id}")),
                ("created_by", &user_filter),
            ],
        )
        .await
        .map_err(|e| ProjectError::from_supabase(&e))?;

    let mut results = Vec::with_capacity(services.len());
    let mut deployed_count: usize = 0;
    let mut failed_count: usize = 0;

    for svc in &services {
        let update_body = UpdateStatusRow {
            status_id: DEPLOYED_STATUS_ID.to_string(),
        };

        let update_result: Result<Vec<serde_json::Value>, _> = state
            .supabase
            .update(
                "workflows",
                &[
                    ("id", &format!("eq.{}", svc.id)),
                    ("created_by", &user_filter),
                ],
                &update_body,
            )
            .await;

        match update_result {
            Ok(_) => {
                deployed_count += 1;
                results.push(ServiceDeployResult {
                    service_id: svc.id.clone(),
                    service_name: svc.name.clone(),
                    status: "deployed".to_string(),
                    error: None,
                });
            }
            Err(e) => {
                tracing::error!("Failed to deploy service '{}': {e}", svc.id);
                failed_count += 1;
                results.push(ServiceDeployResult {
                    service_id: svc.id.clone(),
                    service_name: svc.name.clone(),
                    status: "failed".to_string(),
                    error: Some(format!("Failed to update status: {e}")),
                });
            }
        }
    }

    Ok((
        StatusCode::OK,
        Json(ProjectDeployResponse {
            project_id: id,
            services_deployed: deployed_count,
            services_failed: failed_count,
            results,
        }),
    ))
}

/// `GET /v1/projects/:id/status` -- Aggregated deployment status for a project.
pub async fn project_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ProjectStatusResponse>, ProjectError> {
    let user = require_auth(&headers, &state).await?;

    // Verify project ownership and get project details.
    let user_filter = format!("eq.{}", user.user_id);
    let project: ProjectResponse = state
        .supabase
        .select_one(
            "projects",
            &[
                ("id", &format!("eq.{id}")),
                ("created_by", &user_filter),
                (
                    "select",
                    "id,name,description,task_queue_name,is_active,created_at,updated_at",
                ),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                ProjectError::not_found(format!("Project '{id}' not found"))
            }
            _ => ProjectError::from_supabase(&e),
        })?;

    // Fetch all services in the project.
    let services: Vec<ServiceStatusSummary> = state
        .supabase
        .select(
            "workflows",
            &[
                ("select", "id,name,status_id"),
                ("project_id", &format!("eq.{id}")),
                ("created_by", &user_filter),
            ],
        )
        .await
        .map_err(|e| ProjectError::from_supabase(&e))?;

    let total_services = services.len();
    let deployed = services
        .iter()
        .filter(|s| s.status_id == DEPLOYED_STATUS_ID)
        .count();
    let draft = services
        .iter()
        .filter(|s| s.status_id == DRAFT_STATUS_ID)
        .count();

    Ok(Json(ProjectStatusResponse {
        project_id: id,
        project_name: project.name,
        total_services,
        deployed,
        draft,
        services,
    }))
}

/// `GET /v1/projects/:id/services` -- List all services in a project.
pub async fn list_project_services(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ProjectServiceListResponse>, ProjectError> {
    let user = require_auth(&headers, &state).await?;

    // Verify project ownership.
    let user_filter = format!("eq.{}", user.user_id);
    let _project: ProjectResponse = state
        .supabase
        .select_one(
            "projects",
            &[
                ("id", &format!("eq.{id}")),
                ("created_by", &user_filter),
                (
                    "select",
                    "id,name,description,task_queue_name,is_active,created_at,updated_at",
                ),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                ProjectError::not_found(format!("Project '{id}' not found"))
            }
            _ => ProjectError::from_supabase(&e),
        })?;

    // Fetch all services in the project belonging to this user.
    let services: Vec<ServiceStatusSummary> = state
        .supabase
        .select(
            "workflows",
            &[
                ("select", "id,name,status_id"),
                ("project_id", &format!("eq.{id}")),
                ("created_by", &user_filter),
            ],
        )
        .await
        .map_err(|e| ProjectError::from_supabase(&e))?;

    let total = services.len();
    Ok(Json(ProjectServiceListResponse { services, total }))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Request serialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_create_project_request_serialization() {
        let json = r#"{"name": "My Project", "description": "A test project"}"#;
        let request: CreateProjectRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, "My Project");
        assert_eq!(request.description, Some("A test project".to_string()));
    }

    #[test]
    fn test_create_project_request_without_description() {
        let json = r#"{"name": "Minimal Project"}"#;
        let request: CreateProjectRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, "Minimal Project");
        assert!(request.description.is_none());
    }

    #[test]
    fn test_update_project_request_serialization() {
        let json = r#"{"name": "Updated Name", "description": "Updated desc"}"#;
        let request: UpdateProjectRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, "Updated Name");
        assert_eq!(request.description, Some("Updated desc".to_string()));
    }

    #[test]
    fn test_update_project_request_without_description() {
        let json = r#"{"name": "Just a Name"}"#;
        let request: UpdateProjectRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, "Just a Name");
        assert!(request.description.is_none());
    }

    // -----------------------------------------------------------------------
    // Response serialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_project_response_serialization() {
        let response = ProjectResponse {
            id: "abc-123".to_string(),
            name: "Test Project".to_string(),
            description: Some("A description".to_string()),
            task_queue_name: "abc12345-test-project-queue".to_string(),
            is_active: true,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["id"], "abc-123");
        assert_eq!(json["name"], "Test Project");
        assert_eq!(json["description"], "A description");
        assert_eq!(json["task_queue_name"], "abc12345-test-project-queue");
        assert_eq!(json["is_active"], true);
    }

    #[test]
    fn test_project_response_skips_none_description() {
        let response = ProjectResponse {
            id: "abc-123".to_string(),
            name: "No Desc".to_string(),
            description: None,
            task_queue_name: "abc12345-no-desc-queue".to_string(),
            is_active: true,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert!(json.get("description").is_none(), "description should be omitted when None");
    }

    #[test]
    fn test_project_list_response_serialization() {
        let response = ProjectListResponse {
            projects: vec![
                ProjectSummary {
                    id: "p1".to_string(),
                    name: "Project One".to_string(),
                    description: None,
                    is_active: true,
                    created_at: "2024-01-01T00:00:00Z".to_string(),
                },
                ProjectSummary {
                    id: "p2".to_string(),
                    name: "Project Two".to_string(),
                    description: Some("Second project".to_string()),
                    is_active: false,
                    created_at: "2024-01-02T00:00:00Z".to_string(),
                },
            ],
            total: 2,
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["total"], 2);
        assert_eq!(json["projects"].as_array().unwrap().len(), 2);
        assert_eq!(json["projects"][0]["name"], "Project One");
        // First project has no description -- should be omitted
        assert!(json["projects"][0].get("description").is_none());
        // Second project has a description
        assert_eq!(json["projects"][1]["description"], "Second project");
    }

    // -----------------------------------------------------------------------
    // Queue name generation
    // -----------------------------------------------------------------------

    #[test]
    fn test_queue_name_generation() {
        let name = generate_queue_name("abcdefgh-1234-5678", "My Cool Project");
        assert_eq!(name, "abcdefgh-my-cool-project-queue");
    }

    #[test]
    fn test_queue_name_generation_with_special_chars() {
        let name = generate_queue_name("12345678-abcd", "Hello World! @#$%");
        assert_eq!(name, "12345678-hello-world-queue");
    }

    #[test]
    fn test_queue_name_generation_short_user_id() {
        let name = generate_queue_name("abc", "Test");
        assert_eq!(name, "abc-test-queue");
    }

    #[test]
    fn test_queue_name_generation_preserves_numbers() {
        let name = generate_queue_name("user1234-rest", "Project 42");
        assert_eq!(name, "user1234-project-42-queue");
    }

    // -----------------------------------------------------------------------
    // Error constructors
    // -----------------------------------------------------------------------

    #[test]
    fn test_project_error_unauthorized() {
        let err = ProjectError::unauthorized();
        assert_eq!(err.status, StatusCode::UNAUTHORIZED);
        assert_eq!(err.code, "UNAUTHORIZED");
    }

    #[test]
    fn test_project_error_bad_request() {
        let err = ProjectError::bad_request("name is required");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.code, "BAD_REQUEST");
        assert_eq!(err.message, "name is required");
    }

    #[test]
    fn test_project_error_not_found() {
        let err = ProjectError::not_found("Project 'abc' not found");
        assert_eq!(err.status, StatusCode::NOT_FOUND);
        assert_eq!(err.code, "NOT_FOUND");
        assert_eq!(err.message, "Project 'abc' not found");
    }

    #[test]
    fn test_project_error_internal() {
        let err = ProjectError::internal("something broke");
        assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.code, "INTERNAL_ERROR");
        assert_eq!(err.message, "something broke");
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
    // Deploy / status response serialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_project_deploy_response_serialization() {
        let response = ProjectDeployResponse {
            project_id: "proj-1".to_string(),
            services_deployed: 2,
            services_failed: 1,
            results: vec![
                ServiceDeployResult {
                    service_id: "svc-1".to_string(),
                    service_name: "Service One".to_string(),
                    status: "deployed".to_string(),
                    error: None,
                },
                ServiceDeployResult {
                    service_id: "svc-2".to_string(),
                    service_name: "Service Two".to_string(),
                    status: "deployed".to_string(),
                    error: None,
                },
                ServiceDeployResult {
                    service_id: "svc-3".to_string(),
                    service_name: "Service Three".to_string(),
                    status: "failed".to_string(),
                    error: Some("Database error".to_string()),
                },
            ],
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["project_id"], "proj-1");
        assert_eq!(json["services_deployed"], 2);
        assert_eq!(json["services_failed"], 1);
        assert_eq!(json["results"].as_array().unwrap().len(), 3);
        assert_eq!(json["results"][0]["status"], "deployed");
        // Successful result should omit error field
        assert!(json["results"][0].get("error").is_none());
        // Failed result should include error
        assert_eq!(json["results"][2]["status"], "failed");
        assert_eq!(json["results"][2]["error"], "Database error");
    }

    #[test]
    fn test_service_deploy_result_serialization() {
        // Successful result omits error
        let success = ServiceDeployResult {
            service_id: "svc-ok".to_string(),
            service_name: "Good Service".to_string(),
            status: "deployed".to_string(),
            error: None,
        };
        let json = serde_json::to_value(&success).unwrap();
        assert_eq!(json["service_id"], "svc-ok");
        assert_eq!(json["service_name"], "Good Service");
        assert_eq!(json["status"], "deployed");
        assert!(json.get("error").is_none(), "error should be omitted when None");

        // Failed result includes error
        let failure = ServiceDeployResult {
            service_id: "svc-bad".to_string(),
            service_name: "Bad Service".to_string(),
            status: "failed".to_string(),
            error: Some("update failed".to_string()),
        };
        let json = serde_json::to_value(&failure).unwrap();
        assert_eq!(json["status"], "failed");
        assert_eq!(json["error"], "update failed");
    }

    #[test]
    fn test_project_status_response_serialization() {
        let response = ProjectStatusResponse {
            project_id: "proj-1".to_string(),
            project_name: "My Project".to_string(),
            total_services: 3,
            deployed: 2,
            draft: 1,
            services: vec![
                ServiceStatusSummary {
                    id: "svc-1".to_string(),
                    name: "Service One".to_string(),
                    status_id: DEPLOYED_STATUS_ID.to_string(),
                },
                ServiceStatusSummary {
                    id: "svc-2".to_string(),
                    name: "Service Two".to_string(),
                    status_id: DEPLOYED_STATUS_ID.to_string(),
                },
                ServiceStatusSummary {
                    id: "svc-3".to_string(),
                    name: "Service Three".to_string(),
                    status_id: DRAFT_STATUS_ID.to_string(),
                },
            ],
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["project_id"], "proj-1");
        assert_eq!(json["project_name"], "My Project");
        assert_eq!(json["total_services"], 3);
        assert_eq!(json["deployed"], 2);
        assert_eq!(json["draft"], 1);
        assert_eq!(json["services"].as_array().unwrap().len(), 3);
        assert_eq!(json["services"][0]["name"], "Service One");
        assert_eq!(json["services"][2]["status_id"], DRAFT_STATUS_ID);
    }

    #[test]
    fn test_project_service_list_response_serialization() {
        let response = ProjectServiceListResponse {
            services: vec![
                ServiceStatusSummary {
                    id: "svc-a".to_string(),
                    name: "Alpha".to_string(),
                    status_id: DRAFT_STATUS_ID.to_string(),
                },
                ServiceStatusSummary {
                    id: "svc-b".to_string(),
                    name: "Beta".to_string(),
                    status_id: DEPLOYED_STATUS_ID.to_string(),
                },
            ],
            total: 2,
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["total"], 2);
        assert_eq!(json["services"].as_array().unwrap().len(), 2);
        assert_eq!(json["services"][0]["id"], "svc-a");
        assert_eq!(json["services"][1]["name"], "Beta");
    }

    #[test]
    fn test_deploy_status_constants() {
        assert_eq!(DEPLOYED_STATUS_ID, "00000000-0000-0000-0000-000000000003");
        assert_eq!(DRAFT_STATUS_ID, "00000000-0000-0000-0000-000000000001");
        assert_ne!(DEPLOYED_STATUS_ID, DRAFT_STATUS_ID);
    }

    // -----------------------------------------------------------------------
    // Integration tests that need Supabase (marked #[ignore])
    // -----------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_create_project_integration() {
        // This test would POST a project to a real Supabase instance.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_list_projects_integration() {
        // This test would list projects from a real Supabase instance.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_get_project_integration() {
        // This test would fetch a single project by ID from Supabase.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_update_project_integration() {
        // This test would update a project in Supabase.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_delete_project_integration() {
        // This test would delete a project from Supabase.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_deploy_project_integration() {
        // This test would deploy all services in a project via Supabase.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_project_status_integration() {
        // This test would check aggregated deployment status from Supabase.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_list_project_services_integration() {
        // This test would list all services in a project from Supabase.
    }
}
