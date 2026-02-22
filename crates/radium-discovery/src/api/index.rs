//! Index API endpoints — create, update, delete items in the discovery graph

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::graph;
use crate::state::AppState;

/// Error type for index operations
#[derive(Debug)]
pub(crate) struct IndexError {
    status: StatusCode,
    code: String,
    message: String,
}

impl IndexError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "BAD_REQUEST".to_string(),
            message: message.into(),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "NOT_FOUND".to_string(),
            message: message.into(),
        }
    }

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

impl IntoResponse for IndexError {
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

impl From<graph::GraphError> for IndexError {
    fn from(e: graph::GraphError) -> Self {
        match e {
            graph::GraphError::NotFound { kind, id } => {
                Self::not_found(format!("{kind} with id={id} not found"))
            }
            other => Self::internal(other.to_string()),
        }
    }
}

/// POST /v1/discover/index — Index a new item
pub async fn create_index(
    State(state): State<AppState>,
    Json(req): Json<graph::IndexRequest>,
) -> Result<impl IntoResponse, IndexError> {
    // Validate required fields
    if req.name.is_empty() {
        return Err(IndexError::bad_request("name is required"));
    }
    if req.kind.is_empty() {
        return Err(IndexError::bad_request("kind is required"));
    }

    // Generate embedding from name + description + category
    let text = format!("{} {} {}", req.name, req.description, req.category);
    let embedding = match state.embeddings.embed(&text).await {
        Ok(emb) => Some(emb),
        Err(e) => {
            tracing::warn!("Failed to generate embedding: {e}");
            None
        }
    };

    graph::client::upsert_node(&state.graph, &req, embedding).await?;

    let node = graph::client::get_node(&state.graph, &req.id).await?;

    Ok((StatusCode::CREATED, Json(node)))
}

/// PUT /v1/discover/index/:id — Update an existing item
pub async fn update_index(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(mut req): Json<graph::IndexRequest>,
) -> Result<impl IntoResponse, IndexError> {
    // Ensure the path ID matches the body ID
    req.id = id;

    // Generate embedding from name + description + category
    let text = format!("{} {} {}", req.name, req.description, req.category);
    let embedding = match state.embeddings.embed(&text).await {
        Ok(emb) => Some(emb),
        Err(e) => {
            tracing::warn!("Failed to generate embedding: {e}");
            None
        }
    };

    graph::client::upsert_node(&state.graph, &req, embedding).await?;

    let node = graph::client::get_node(&state.graph, &req.id).await?;

    Ok(Json(node))
}

/// DELETE /v1/discover/index/:id — Remove an item from the graph
pub async fn delete_index(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, IndexError> {
    graph::client::delete_node(&state.graph, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_error_bad_request() {
        let err = IndexError::bad_request("test error");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.code, "BAD_REQUEST");
    }

    #[test]
    fn test_index_error_not_found() {
        let err = IndexError::not_found("not found");
        assert_eq!(err.status, StatusCode::NOT_FOUND);
        assert_eq!(err.code, "NOT_FOUND");
    }

    #[test]
    fn test_index_error_from_graph_not_found() {
        let graph_err = graph::GraphError::NotFound {
            kind: "component".to_string(),
            id: "test-1".to_string(),
        };
        let index_err = IndexError::from(graph_err);
        assert_eq!(index_err.status, StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_index_error_from_graph_other() {
        let graph_err = graph::GraphError::Deserialization("bad data".to_string());
        let index_err = IndexError::from(graph_err);
        assert_eq!(index_err.status, StatusCode::INTERNAL_SERVER_ERROR);
    }
}
