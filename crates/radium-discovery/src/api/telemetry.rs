//! Telemetry endpoint — record usage events

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use neo4rs::query;
use serde::{Deserialize, Serialize};

use crate::state::AppState;

/// A usage event to record
#[derive(Debug, Deserialize)]
pub struct TelemetryEvent {
    pub event: String,
    pub user_id: String,
    #[serde(default)]
    pub component_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
struct TelemetryResponse {
    recorded: bool,
}

/// Error type for telemetry operations
#[derive(Debug)]
pub(crate) struct TelemetryError {
    status: StatusCode,
    code: String,
    message: String,
}

impl TelemetryError {
    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "INTERNAL_ERROR".to_string(),
            message: message.into(),
        }
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: ErrorDetail,
}

#[derive(Serialize)]
struct ErrorDetail {
    code: String,
    message: String,
}

impl IntoResponse for TelemetryError {
    fn into_response(self) -> axum::response::Response {
        let body = ErrorBody {
            error: ErrorDetail {
                code: self.code,
                message: self.message,
            },
        };
        (self.status, Json(body)).into_response()
    }
}

/// POST /v1/discover/index/:id/telemetry — Record a usage event
pub async fn record_telemetry(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(event): Json<TelemetryEvent>,
) -> Result<impl IntoResponse, TelemetryError> {
    // Increment usage_count on the node
    state
        .graph
        .run(
            query(
                "MATCH (n) WHERE n.id = $id SET n.usage_count = coalesce(n.usage_count, 0) + 1",
            )
            .param("id", id.as_str()),
        )
        .await
        .map_err(|e| TelemetryError::internal(e.to_string()))?;

    // Create/update user relationship
    state
        .graph
        .run(
            query(
                "MATCH (n) WHERE n.id = $id \
                 MERGE (u:User {id: $user_id}) \
                 MERGE (u)-[r:DEPLOYED]->(n) \
                 ON CREATE SET r.count = 1, r.first_at = datetime() \
                 ON MATCH SET r.count = r.count + 1, r.last_at = datetime()",
            )
            .param("id", id.as_str())
            .param("user_id", event.user_id.as_str()),
        )
        .await
        .map_err(|e| TelemetryError::internal(e.to_string()))?;

    // Record co-usage edges when multiple components are deployed together
    if event.event == "deploy" && !event.component_ids.is_empty() {
        if let Err(e) = crate::graph::client::record_co_usage(&state.graph, &event.component_ids).await {
            tracing::warn!("Failed to record co-usage: {e}");
        }
    }

    Ok(Json(TelemetryResponse { recorded: true }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_event_deserialization() {
        let json = r#"{"event": "deploy", "user_id": "user-1"}"#;
        let event: TelemetryEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event, "deploy");
        assert_eq!(event.user_id, "user-1");
        assert!(event.component_ids.is_empty());
    }

    #[test]
    fn test_telemetry_event_with_component_ids() {
        let json =
            r#"{"event": "deploy", "user_id": "user-1", "component_ids": ["comp-1", "comp-2"]}"#;
        let event: TelemetryEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.component_ids.len(), 2);
    }

    #[test]
    fn test_telemetry_event_deploy_with_components() {
        let json = r#"{"event": "deploy", "user_id": "user-1", "component_ids": ["comp-1", "comp-2", "comp-3"]}"#;
        let event: TelemetryEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event, "deploy");
        assert_eq!(event.component_ids.len(), 3);
    }

    #[test]
    fn test_telemetry_response_serialization() {
        let resp = TelemetryResponse { recorded: true };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["recorded"], true);
    }
}
