//! Gateway HTTP handler for published interface traffic.
//!
//! This handler is the public-facing endpoint that Kong routes to when external
//! traffic hits a published interface path (e.g., `/api/my-service/my-signal`).
//! Kong forwards the request to `/v1/gateway/{interface_id}`, where this handler
//! signals the corresponding Temporal gateway workflow and returns 202 Accepted
//! immediately.
//!
//! **Authentication**: This endpoint does NOT require Bearer token authentication.
//! It is the public-facing gateway endpoint. Interface-level API key
//! authentication will be added as a future feature.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::state::AppState;
use crate::temporal_client::SignalPayload;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Structured error for gateway handler failures.
///
/// Returns a JSON envelope: `{ "error": { "code": "...", "message": "..." } }`
#[derive(Debug)]
pub struct GatewayError {
    pub status: StatusCode,
    code: String,
    message: String,
}

impl GatewayError {
    /// The requested interface was not found or is not active.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "NOT_FOUND".to_string(),
            message: message.into(),
        }
    }

    /// The gateway workflow engine is not available.
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "SERVICE_UNAVAILABLE".to_string(),
            message: message.into(),
        }
    }

    /// An unexpected internal error occurred.
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "INTERNAL_ERROR".to_string(),
            message: message.into(),
        }
    }
}

/// JSON envelope for gateway error responses.
#[derive(Debug, Serialize)]
struct GatewayErrorEnvelope {
    error: GatewayErrorBody,
}

/// Inner body of the gateway error envelope.
#[derive(Debug, Serialize)]
struct GatewayErrorBody {
    code: String,
    message: String,
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let envelope = GatewayErrorEnvelope {
            error: GatewayErrorBody {
                code: self.code,
                message: self.message,
            },
        };
        (self.status, Json(envelope)).into_response()
    }
}

// ---------------------------------------------------------------------------
// Response type
// ---------------------------------------------------------------------------

/// Response returned when a gateway request is successfully accepted.
#[derive(Debug, Serialize, Deserialize)]
pub struct GatewayAcceptedResponse {
    /// Always `"accepted"` for successful gateway requests.
    pub status: String,
    /// Unique identifier for tracking this request through the system.
    pub request_id: String,
    /// Human-readable message describing the outcome.
    pub message: String,
}

// ---------------------------------------------------------------------------
// Supabase row type for public_interfaces lookup
// ---------------------------------------------------------------------------

/// Minimal row shape for verifying a public interface is active.
#[derive(Debug, Deserialize)]
struct PublicInterfaceRow {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    service_interface_id: String,
    #[allow(dead_code)]
    is_active: bool,
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// `POST /v1/gateway/{interface_id}` -- Accept an incoming request for a
/// published interface and signal the corresponding Temporal gateway workflow.
///
/// This handler:
/// 1. Extracts or generates a request ID from the `X-Request-ID` header
/// 2. Parses the request body as JSON (empty body defaults to `{}`)
/// 3. Verifies the interface is published and active in `public_interfaces`
/// 4. Signals the Temporal gateway workflow with the request payload
/// 5. Returns 202 Accepted immediately
pub async fn handle_gateway_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(interface_id): Path<String>,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse, GatewayError> {
    // 1. Extract or generate request ID.
    let request_id = headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map_or_else(|| Uuid::new_v4().to_string(), str::to_string);

    tracing::info!(
        interface_id = %interface_id,
        request_id = %request_id,
        "Gateway request received"
    );

    // 2. Parse request body as JSON (empty body → `{}`).
    let data: serde_json::Value = if body.is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_slice(&body).map_err(|e| {
            GatewayError::internal(format!("Invalid JSON body: {e}"))
        })?
    };

    // 3. Verify the interface is published and active.
    let interface_filter = format!("eq.{interface_id}");
    let _record: PublicInterfaceRow = state
        .supabase
        .select_one(
            "public_interfaces",
            &[
                ("service_interface_id", &interface_filter),
                ("is_active", "eq.true"),
                ("select", "id,service_interface_id,is_active"),
            ],
        )
        .await
        .map_err(|_| {
            GatewayError::not_found(format!(
                "Interface '{interface_id}' is not published or not active"
            ))
        })?;

    // 4. Get the Temporal client (return 503 if not available).
    let temporal = state.temporal.as_ref().ok_or_else(|| {
        GatewayError::unavailable("Gateway workflow engine is not available")
    })?;

    // 5. Build the signal payload and signal the workflow.
    let payload = SignalPayload {
        data,
        received_at: Utc::now().to_rfc3339(),
        request_id: request_id.clone(),
    };

    let mut client = temporal.lock().await;
    client
        .signal_gateway_workflow(&interface_id, &payload)
        .await
        .map_err(|e| {
            tracing::error!(
                interface_id = %interface_id,
                request_id = %request_id,
                error = %e,
                "Failed to signal gateway workflow"
            );
            GatewayError::internal("Failed to deliver request to gateway workflow")
        })?;

    // 6. Return 202 Accepted.
    let response = GatewayAcceptedResponse {
        status: "accepted".to_string(),
        request_id,
        message: "Request queued for processing".to_string(),
    };

    Ok((StatusCode::ACCEPTED, Json(response)))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_response_accepted() {
        let resp = GatewayAcceptedResponse {
            status: "accepted".to_string(),
            request_id: "req-123".to_string(),
            message: "Request queued for processing".to_string(),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["status"], "accepted");
        assert_eq!(json["request_id"], "req-123");
        assert_eq!(json["message"], "Request queued for processing");
    }

    #[test]
    fn test_gateway_response_deserialization() {
        let json = serde_json::json!({
            "status": "accepted",
            "request_id": "req-456",
            "message": "Queued"
        });
        let resp: GatewayAcceptedResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.status, "accepted");
        assert_eq!(resp.request_id, "req-456");
        assert_eq!(resp.message, "Queued");
    }

    #[test]
    fn test_gateway_error_not_found() {
        let err = GatewayError::not_found("Interface not found");
        assert_eq!(err.status, StatusCode::NOT_FOUND);
        assert_eq!(err.code, "NOT_FOUND");
        assert_eq!(err.message, "Interface not found");
    }

    #[test]
    fn test_gateway_error_unavailable() {
        let err = GatewayError::unavailable("Gateway workflow not running");
        assert_eq!(err.status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(err.code, "SERVICE_UNAVAILABLE");
        assert_eq!(err.message, "Gateway workflow not running");
    }

    #[test]
    fn test_gateway_error_internal() {
        let err = GatewayError::internal("Something went wrong");
        assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.code, "INTERNAL_ERROR");
        assert_eq!(err.message, "Something went wrong");
    }

    #[test]
    fn test_gateway_error_into_response() {
        let err = GatewayError::not_found("Test error");
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_gateway_error_envelope_serialization() {
        let envelope = GatewayErrorEnvelope {
            error: GatewayErrorBody {
                code: "NOT_FOUND".to_string(),
                message: "Interface not found".to_string(),
            },
        };
        let json = serde_json::to_value(&envelope).unwrap();
        assert_eq!(json["error"]["code"], "NOT_FOUND");
        assert_eq!(json["error"]["message"], "Interface not found");
    }
}
