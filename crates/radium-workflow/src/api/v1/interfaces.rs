//! Service interface CRUD endpoints.
//!
//! Provides REST API endpoints for creating, reading, updating, and deleting
//! service interfaces (signal, query, update, mcp, graphql) stored in Supabase.
//! Includes publish/unpublish endpoints that record public interface routes
//! to the `public_interfaces` table (Kong integration is stubbed).

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

/// Allowed values for the `interface_type` field.
const VALID_INTERFACE_TYPES: &[&str] = &["signal", "query", "update", "mcp", "graphql"];

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

/// Body for creating a new service interface.
#[derive(Debug, Deserialize)]
pub struct CreateInterfaceRequest {
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub interface_type: String,
    pub callable_name: Option<String>,
    pub input_schema: Option<serde_json::Value>,
    pub output_schema: Option<serde_json::Value>,
    pub is_public: Option<bool>,
}

/// Body for updating an existing service interface.
#[derive(Debug, Deserialize)]
pub struct UpdateInterfaceRequest {
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub interface_type: Option<String>,
    pub callable_name: Option<serde_json::Value>,
    pub input_schema: Option<serde_json::Value>,
    pub output_schema: Option<serde_json::Value>,
    pub is_public: Option<bool>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Full interface response returned from GET and create/update operations.
#[derive(Debug, Serialize, Deserialize)]
pub struct InterfaceResponse {
    pub id: String,
    pub workflow_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub interface_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callable_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
    pub is_public: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// List response envelope.
#[derive(Debug, Serialize)]
pub struct InterfaceListResponse {
    pub interfaces: Vec<InterfaceResponse>,
    pub total: usize,
}

/// Response returned after a successful publish.
#[derive(Debug, Serialize, Deserialize)]
pub struct PublishResponse {
    pub id: String,
    pub service_interface_id: String,
    pub route_path: String,
    pub http_method: String,
    pub created_at: String,
    pub updated_at: String,
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
pub struct InterfaceError {
    status: StatusCode,
    code: String,
    message: String,
    details: Vec<String>,
}

impl InterfaceError {
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

impl IntoResponse for InterfaceError {
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

/// Body sent to Supabase for creating a service interface.
#[derive(Debug, Serialize)]
struct InsertInterfaceRow {
    workflow_id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    interface_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    callable_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_schema: Option<serde_json::Value>,
    is_public: bool,
}

/// Body sent to Supabase for updating a service interface.
#[derive(Debug, Serialize)]
struct UpdateInterfaceRow {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    interface_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    callable_name: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_public: Option<bool>,
}

/// Body sent to Supabase for creating a public interface record.
#[derive(Debug, Serialize)]
struct InsertPublicInterfaceRow {
    service_interface_id: String,
    route_path: String,
    http_method: String,
}

/// Minimal response shape from workflows select (for ownership verification).
#[derive(Debug, Deserialize)]
struct WorkflowOwnerRow {
    #[allow(dead_code)]
    id: String,
    name: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Validate the Bearer token against the Supabase `api_keys` table and check
/// the per-user rate limit.
async fn require_auth(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<AuthenticatedUser, InterfaceError> {
    let mut request = axum::http::Request::builder()
        .uri("http://localhost/")
        .body(())
        .unwrap();
    *request.headers_mut() = headers.clone();
    let (parts, ()) = request.into_parts();

    let token =
        auth::extract_bearer_token(&parts).ok_or_else(InterfaceError::unauthorized)?;

    let user = auth::validate_api_key(
        state.supabase.http_client(),
        state.supabase.url(),
        state.supabase.service_role_key(),
        &token,
    )
    .await
    .map_err(|_| InterfaceError::unauthorized())?;

    let result = state.rate_limiter.check(&user.user_id);
    if !result.allowed {
        return Err(InterfaceError {
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
/// Returns the service name on success (needed for publish route generation).
async fn verify_service_ownership(
    state: &AppState,
    service_id: &str,
    user_id: &str,
) -> Result<String, InterfaceError> {
    let user_filter = format!("eq.{user_id}");
    let service: WorkflowOwnerRow = state
        .supabase
        .select_one(
            "workflows",
            &[
                ("id", &format!("eq.{service_id}")),
                ("created_by", &user_filter),
                ("select", "id,name"),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                InterfaceError::not_found(format!("Service '{service_id}' not found"))
            }
            _ => InterfaceError::from_supabase(&e),
        })?;

    Ok(service.name)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `POST /v1/services/{id}/interfaces` -- Create a new interface on a service.
pub async fn create_interface(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<CreateInterfaceRequest>,
) -> Result<impl IntoResponse, InterfaceError> {
    let user = require_auth(&headers, &state).await?;
    verify_service_ownership(&state, &id, &user.user_id).await?;

    // Validate name is non-empty.
    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Err(InterfaceError::bad_request(
            "Interface name must not be empty",
        ));
    }

    // Validate interface_type.
    if !VALID_INTERFACE_TYPES.contains(&body.interface_type.as_str()) {
        return Err(InterfaceError::bad_request(format!(
            "Invalid interface_type '{}'. Must be one of: {}",
            body.interface_type,
            VALID_INTERFACE_TYPES.join(", ")
        )));
    }

    let row = InsertInterfaceRow {
        workflow_id: id,
        name,
        display_name: body.display_name,
        description: body.description,
        interface_type: body.interface_type,
        callable_name: body.callable_name,
        input_schema: body.input_schema,
        output_schema: body.output_schema,
        is_public: body.is_public.unwrap_or(false),
    };

    let created: InterfaceResponse = state
        .supabase
        .insert("service_interfaces", &row)
        .await
        .map_err(|e| InterfaceError::from_supabase(&e))?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// `GET /v1/services/{id}/interfaces` -- List all interfaces for a service.
pub async fn list_interfaces(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<InterfaceListResponse>, InterfaceError> {
    let user = require_auth(&headers, &state).await?;
    verify_service_ownership(&state, &id, &user.user_id).await?;

    let workflow_filter = format!("eq.{id}");
    let interfaces: Vec<InterfaceResponse> = state
        .supabase
        .select(
            "service_interfaces",
            &[
                ("workflow_id", &workflow_filter),
                (
                    "select",
                    "id,workflow_id,name,display_name,description,interface_type,callable_name,input_schema,output_schema,is_public,created_at,updated_at",
                ),
                ("order", "created_at.desc"),
            ],
        )
        .await
        .map_err(|e| InterfaceError::from_supabase(&e))?;

    let total = interfaces.len();
    Ok(Json(InterfaceListResponse { interfaces, total }))
}

/// `GET /v1/services/{id}/interfaces/{iid}` -- Get a single interface.
pub async fn get_interface(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((id, iid)): Path<(String, String)>,
) -> Result<Json<InterfaceResponse>, InterfaceError> {
    let user = require_auth(&headers, &state).await?;
    verify_service_ownership(&state, &id, &user.user_id).await?;

    let interface: InterfaceResponse = state
        .supabase
        .select_one(
            "service_interfaces",
            &[
                ("id", &format!("eq.{iid}")),
                ("workflow_id", &format!("eq.{id}")),
                (
                    "select",
                    "id,workflow_id,name,display_name,description,interface_type,callable_name,input_schema,output_schema,is_public,created_at,updated_at",
                ),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                InterfaceError::not_found(format!("Interface '{iid}' not found"))
            }
            _ => InterfaceError::from_supabase(&e),
        })?;

    Ok(Json(interface))
}

/// `PUT /v1/services/{id}/interfaces/{iid}` -- Update an interface.
pub async fn update_interface(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((id, iid)): Path<(String, String)>,
    Json(body): Json<UpdateInterfaceRequest>,
) -> Result<Json<InterfaceResponse>, InterfaceError> {
    let user = require_auth(&headers, &state).await?;
    verify_service_ownership(&state, &id, &user.user_id).await?;

    // Validate name if provided.
    if let Some(ref name) = body.name {
        if name.trim().is_empty() {
            return Err(InterfaceError::bad_request(
                "Interface name must not be empty",
            ));
        }
    }

    // Validate interface_type if provided.
    if let Some(ref interface_type) = body.interface_type {
        if !VALID_INTERFACE_TYPES.contains(&interface_type.as_str()) {
            return Err(InterfaceError::bad_request(format!(
                "Invalid interface_type '{}'. Must be one of: {}",
                interface_type,
                VALID_INTERFACE_TYPES.join(", ")
            )));
        }
    }

    let update_body = UpdateInterfaceRow {
        name: body.name,
        display_name: body.display_name,
        description: body.description,
        interface_type: body.interface_type,
        callable_name: body.callable_name,
        input_schema: body.input_schema,
        output_schema: body.output_schema,
        is_public: body.is_public,
    };

    let updated: Vec<InterfaceResponse> = state
        .supabase
        .update(
            "service_interfaces",
            &[
                ("id", &format!("eq.{iid}")),
                ("workflow_id", &format!("eq.{id}")),
            ],
            &update_body,
        )
        .await
        .map_err(|e| InterfaceError::from_supabase(&e))?;

    let interface = updated.into_iter().next().ok_or_else(|| {
        InterfaceError::not_found(format!("Interface '{iid}' not found"))
    })?;

    Ok(Json(interface))
}

/// `DELETE /v1/services/{id}/interfaces/{iid}` -- Delete an interface.
pub async fn delete_interface(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((id, iid)): Path<(String, String)>,
) -> Result<StatusCode, InterfaceError> {
    let user = require_auth(&headers, &state).await?;
    verify_service_ownership(&state, &id, &user.user_id).await?;

    state
        .supabase
        .delete(
            "service_interfaces",
            &[
                ("id", &format!("eq.{iid}")),
                ("workflow_id", &format!("eq.{id}")),
            ],
        )
        .await
        .map_err(|e| InterfaceError::from_supabase(&e))?;

    Ok(StatusCode::NO_CONTENT)
}

/// `POST /v1/services/{id}/interfaces/{iid}/publish` -- Publish an interface.
///
/// Generates a route path and records it in the `public_interfaces` table.
/// Kong integration is stubbed -- this only writes the DB record.
pub async fn publish_interface(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((id, iid)): Path<(String, String)>,
) -> Result<impl IntoResponse, InterfaceError> {
    let user = require_auth(&headers, &state).await?;
    let service_name = verify_service_ownership(&state, &id, &user.user_id).await?;

    // Fetch the interface to get its name for route generation.
    let interface: InterfaceResponse = state
        .supabase
        .select_one(
            "service_interfaces",
            &[
                ("id", &format!("eq.{iid}")),
                ("workflow_id", &format!("eq.{id}")),
                ("select", "id,workflow_id,name,display_name,description,interface_type,callable_name,input_schema,output_schema,is_public,created_at,updated_at"),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                InterfaceError::not_found(format!("Interface '{iid}' not found"))
            }
            _ => InterfaceError::from_supabase(&e),
        })?;

    // Generate route path: /api/{service_name}/{interface_name}
    let route_path = format!(
        "/api/{}/{}",
        kebab_case(&service_name),
        kebab_case(&interface.name)
    );

    let row = InsertPublicInterfaceRow {
        service_interface_id: iid,
        route_path,
        http_method: "POST".to_string(),
    };

    let created: PublishResponse = state
        .supabase
        .insert("public_interfaces", &row)
        .await
        .map_err(|e| InterfaceError::from_supabase(&e))?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// `POST /v1/services/{id}/interfaces/{iid}/unpublish` -- Unpublish an interface.
///
/// Removes the public interface record from the `public_interfaces` table.
pub async fn unpublish_interface(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((id, iid)): Path<(String, String)>,
) -> Result<StatusCode, InterfaceError> {
    let user = require_auth(&headers, &state).await?;
    verify_service_ownership(&state, &id, &user.user_id).await?;

    state
        .supabase
        .delete(
            "public_interfaces",
            &[("service_interface_id", &format!("eq.{iid}"))],
        )
        .await
        .map_err(|e| InterfaceError::from_supabase(&e))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

/// Convert a string to kebab-case for URL paths.
fn kebab_case(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
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
    fn test_create_interface_request_serialization() {
        let json = r#"{
            "name": "get_status",
            "display_name": "Get Status",
            "description": "Returns the current status",
            "interface_type": "query",
            "callable_name": "getStatus",
            "input_schema": {"type": "object"},
            "output_schema": {"type": "string"},
            "is_public": true
        }"#;
        let request: CreateInterfaceRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, "get_status");
        assert_eq!(request.display_name, Some("Get Status".to_string()));
        assert_eq!(request.description, Some("Returns the current status".to_string()));
        assert_eq!(request.interface_type, "query");
        assert_eq!(request.callable_name, Some("getStatus".to_string()));
        assert!(request.input_schema.is_some());
        assert!(request.output_schema.is_some());
        assert_eq!(request.is_public, Some(true));
    }

    // -----------------------------------------------------------------------
    // Response serialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_interface_response_serialization() {
        let response = InterfaceResponse {
            id: "iface-123".to_string(),
            workflow_id: "wf-456".to_string(),
            name: "process_order".to_string(),
            display_name: Some("Process Order".to_string()),
            description: Some("Processes an incoming order".to_string()),
            interface_type: "signal".to_string(),
            callable_name: Some("processOrder".to_string()),
            input_schema: Some(serde_json::json!({"type": "object"})),
            output_schema: Some(serde_json::json!({"type": "boolean"})),
            is_public: false,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["id"], "iface-123");
        assert_eq!(json["workflow_id"], "wf-456");
        assert_eq!(json["name"], "process_order");
        assert_eq!(json["display_name"], "Process Order");
        assert_eq!(json["interface_type"], "signal");
        assert_eq!(json["is_public"], false);
    }

    #[test]
    fn test_interface_response_optional_fields_omitted() {
        let response = InterfaceResponse {
            id: "iface-123".to_string(),
            workflow_id: "wf-456".to_string(),
            name: "minimal".to_string(),
            display_name: None,
            description: None,
            interface_type: "query".to_string(),
            callable_name: None,
            input_schema: None,
            output_schema: None,
            is_public: false,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert!(json.get("display_name").is_none(), "display_name should be omitted when None");
        assert!(json.get("description").is_none(), "description should be omitted when None");
        assert!(json.get("callable_name").is_none(), "callable_name should be omitted when None");
        assert!(json.get("input_schema").is_none(), "input_schema should be omitted when None");
        assert!(json.get("output_schema").is_none(), "output_schema should be omitted when None");
        // Required fields should always be present.
        assert_eq!(json["id"], "iface-123");
        assert_eq!(json["name"], "minimal");
        assert_eq!(json["interface_type"], "query");
        assert_eq!(json["is_public"], false);
    }

    #[test]
    fn test_interface_list_response_serialization() {
        let response = InterfaceListResponse {
            interfaces: vec![
                InterfaceResponse {
                    id: "i1".to_string(),
                    workflow_id: "wf-1".to_string(),
                    name: "signal_start".to_string(),
                    display_name: None,
                    description: None,
                    interface_type: "signal".to_string(),
                    callable_name: None,
                    input_schema: None,
                    output_schema: None,
                    is_public: true,
                    created_at: "2024-01-01T00:00:00Z".to_string(),
                    updated_at: "2024-01-01T00:00:00Z".to_string(),
                },
                InterfaceResponse {
                    id: "i2".to_string(),
                    workflow_id: "wf-1".to_string(),
                    name: "query_status".to_string(),
                    display_name: Some("Query Status".to_string()),
                    description: None,
                    interface_type: "query".to_string(),
                    callable_name: None,
                    input_schema: None,
                    output_schema: None,
                    is_public: false,
                    created_at: "2024-01-02T00:00:00Z".to_string(),
                    updated_at: "2024-01-02T00:00:00Z".to_string(),
                },
            ],
            total: 2,
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["total"], 2);
        assert_eq!(json["interfaces"].as_array().unwrap().len(), 2);
        assert_eq!(json["interfaces"][0]["name"], "signal_start");
        assert_eq!(json["interfaces"][1]["name"], "query_status");
        assert_eq!(json["interfaces"][1]["display_name"], "Query Status");
        // First interface has no display_name -- should be omitted.
        assert!(json["interfaces"][0].get("display_name").is_none());
    }

    // -----------------------------------------------------------------------
    // Valid interface types
    // -----------------------------------------------------------------------

    #[test]
    fn test_valid_interface_types() {
        assert_eq!(VALID_INTERFACE_TYPES.len(), 5);
        assert!(VALID_INTERFACE_TYPES.contains(&"signal"));
        assert!(VALID_INTERFACE_TYPES.contains(&"query"));
        assert!(VALID_INTERFACE_TYPES.contains(&"update"));
        assert!(VALID_INTERFACE_TYPES.contains(&"mcp"));
        assert!(VALID_INTERFACE_TYPES.contains(&"graphql"));
        assert!(!VALID_INTERFACE_TYPES.contains(&"rest"));
        assert!(!VALID_INTERFACE_TYPES.contains(&"grpc"));
    }

    // -----------------------------------------------------------------------
    // Error constructors
    // -----------------------------------------------------------------------

    #[test]
    fn test_interface_error_unauthorized() {
        let err = InterfaceError::unauthorized();
        assert_eq!(err.status, StatusCode::UNAUTHORIZED);
        assert_eq!(err.code, "UNAUTHORIZED");
    }

    #[test]
    fn test_interface_error_bad_request() {
        let err = InterfaceError::bad_request("name is required");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.code, "BAD_REQUEST");
        assert_eq!(err.message, "name is required");
    }

    #[test]
    fn test_interface_error_not_found() {
        let err = InterfaceError::not_found("Interface 'abc' not found");
        assert_eq!(err.status, StatusCode::NOT_FOUND);
        assert_eq!(err.code, "NOT_FOUND");
        assert_eq!(err.message, "Interface 'abc' not found");
    }

    // -----------------------------------------------------------------------
    // Insert row serialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_insert_interface_row_serialization() {
        let row = InsertInterfaceRow {
            workflow_id: "wf-123".to_string(),
            name: "my_signal".to_string(),
            display_name: Some("My Signal".to_string()),
            description: None,
            interface_type: "signal".to_string(),
            callable_name: None,
            input_schema: Some(serde_json::json!({"type": "object"})),
            output_schema: None,
            is_public: false,
        };

        let json = serde_json::to_value(&row).unwrap();
        assert_eq!(json["workflow_id"], "wf-123");
        assert_eq!(json["name"], "my_signal");
        assert_eq!(json["display_name"], "My Signal");
        assert_eq!(json["interface_type"], "signal");
        assert_eq!(json["is_public"], false);
        // Optional None fields should be omitted.
        assert!(json.get("description").is_none());
        assert!(json.get("callable_name").is_none());
        assert!(json.get("output_schema").is_none());
        // input_schema is Some, so it should be present.
        assert!(json.get("input_schema").is_some());
    }

    // -----------------------------------------------------------------------
    // Kebab case utility
    // -----------------------------------------------------------------------

    #[test]
    fn test_kebab_case_basic() {
        assert_eq!(kebab_case("My Service"), "my-service");
    }

    #[test]
    fn test_kebab_case_special_chars() {
        assert_eq!(kebab_case("Hello World! @#$%"), "hello-world");
    }

    #[test]
    fn test_kebab_case_already_kebab() {
        assert_eq!(kebab_case("already-kebab"), "already-kebab");
    }

    #[test]
    fn test_kebab_case_with_numbers() {
        assert_eq!(kebab_case("Service 42"), "service-42");
    }

    // -----------------------------------------------------------------------
    // Integration tests that need Supabase (marked #[ignore])
    // -----------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_create_interface_integration() {
        // This test would POST an interface to a real Supabase instance.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_list_interfaces_integration() {
        // This test would list interfaces from a real Supabase instance.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_get_interface_integration() {
        // This test would fetch a single interface by ID from Supabase.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_update_interface_integration() {
        // This test would update an interface in Supabase.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_delete_interface_integration() {
        // This test would delete an interface from Supabase.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_publish_interface_integration() {
        // This test would publish an interface to the public_interfaces table.
    }

    #[tokio::test]
    #[ignore = "Requires running Supabase instance"]
    async fn test_unpublish_interface_integration() {
        // This test would unpublish an interface from the public_interfaces table.
    }
}
