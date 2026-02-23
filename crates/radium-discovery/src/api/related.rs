//! Related items endpoint handlers
//!
//! Provides three relationship-based endpoints for a given item:
//! - related: co-used items (CO_USED_WITH edges)
//! - dependencies: items this node depends on (USES/DEPENDS_ON outgoing)
//! - dependents: items that depend on this node (USES/DEPENDS_ON incoming)

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::graph;
use crate::state::AppState;

/// Response for relationship queries
#[derive(Debug, Serialize)]
pub struct RelatedResponse {
    pub items: Vec<graph::DiscoveryNode>,
    pub total: usize,
}

/// Error type for related operations
#[derive(Debug)]
pub(crate) struct RelatedError {
    status: StatusCode,
    code: String,
    message: String,
}

impl RelatedError {
    fn not_found(id: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "NOT_FOUND".to_string(),
            message: format!("Item '{id}' not found"),
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

impl IntoResponse for RelatedError {
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

/// GET /v1/discover/{id}/related
///
/// Returns items that are frequently co-used with the given item
/// (CO_USED_WITH edges in both directions), ordered by usage count descending.
pub async fn get_related(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, RelatedError> {
    match graph::client::get_node(&state.graph, &id).await {
        Ok(_) => {}
        Err(graph::GraphError::NotFound { .. }) => return Err(RelatedError::not_found(&id)),
        Err(e) => return Err(RelatedError::internal(e.to_string())),
    }

    let ids = fetch_related_ids(
        &state.graph,
        "MATCH (n {id: $id})-[:CO_USED_WITH]-(other) \
         RETURN other.id AS related_id \
         ORDER BY other.usage_count DESC \
         LIMIT 20",
        &id,
        "related_id",
    )
    .await?;

    let items = resolve_nodes(&state.graph, ids).await;
    let total = items.len();
    Ok(Json(RelatedResponse { items, total }))
}

/// GET /v1/discover/{id}/dependencies
///
/// Returns items that the given item depends on (outgoing USES/DEPENDS_ON edges).
pub async fn get_dependencies(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, RelatedError> {
    match graph::client::get_node(&state.graph, &id).await {
        Ok(_) => {}
        Err(graph::GraphError::NotFound { .. }) => return Err(RelatedError::not_found(&id)),
        Err(e) => return Err(RelatedError::internal(e.to_string())),
    }

    let ids = fetch_related_ids(
        &state.graph,
        "MATCH (n {id: $id})-[:USES|DEPENDS_ON]->(dep) \
         RETURN dep.id AS related_id \
         ORDER BY dep.name ASC \
         LIMIT 100",
        &id,
        "related_id",
    )
    .await?;

    let items = resolve_nodes(&state.graph, ids).await;
    let total = items.len();
    Ok(Json(RelatedResponse { items, total }))
}

/// GET /v1/discover/{id}/dependents
///
/// Returns items that depend on the given item (incoming USES/DEPENDS_ON edges).
pub async fn get_dependents(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, RelatedError> {
    match graph::client::get_node(&state.graph, &id).await {
        Ok(_) => {}
        Err(graph::GraphError::NotFound { .. }) => return Err(RelatedError::not_found(&id)),
        Err(e) => return Err(RelatedError::internal(e.to_string())),
    }

    let ids = fetch_related_ids(
        &state.graph,
        "MATCH (dependent)-[:USES|DEPENDS_ON]->(n {id: $id}) \
         RETURN dependent.id AS related_id \
         ORDER BY dependent.name ASC \
         LIMIT 100",
        &id,
        "related_id",
    )
    .await?;

    let items = resolve_nodes(&state.graph, ids).await;
    let total = items.len();
    Ok(Json(RelatedResponse { items, total }))
}

/// Run a Cypher query and collect node IDs from the named return column.
async fn fetch_related_ids(
    graph: &neo4rs::Graph,
    cypher: &str,
    id: &str,
    column: &str,
) -> Result<Vec<String>, RelatedError> {
    let mut result = graph
        .execute(neo4rs::query(cypher).param("id", id))
        .await
        .map_err(|e| RelatedError::internal(format!("Graph query failed: {e}")))?;

    let mut ids = Vec::new();
    while let Ok(Some(row)) = result.next().await {
        if let Ok(related_id) = row.get::<String>(column) {
            ids.push(related_id);
        }
    }
    Ok(ids)
}

/// Resolve a list of IDs to full DiscoveryNode values, skipping any that fail.
async fn resolve_nodes(graph: &neo4rs::Graph, ids: Vec<String>) -> Vec<graph::DiscoveryNode> {
    let mut nodes = Vec::new();
    for id in &ids {
        match graph::client::get_node(graph, id).await {
            Ok(node) => nodes.push(node),
            Err(e) => {
                tracing::warn!("Failed to resolve related node {id}: {e}");
            }
        }
    }
    nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_related_error_not_found() {
        let err = RelatedError::not_found("item-123");
        assert_eq!(err.status, StatusCode::NOT_FOUND);
        assert_eq!(err.code, "NOT_FOUND");
        assert!(err.message.contains("item-123"));
    }

    #[test]
    fn test_related_error_internal() {
        let err = RelatedError::internal("graph unreachable");
        assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.code, "INTERNAL_ERROR");
        assert_eq!(err.message, "graph unreachable");
    }

    #[test]
    fn test_related_response_serialization() {
        let response = RelatedResponse {
            items: vec![],
            total: 0,
        };
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["total"], 0);
        assert!(json["items"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_error_body_serialization() {
        let body = ErrorBody {
            error: ErrorDetail {
                code: "NOT_FOUND".to_string(),
                message: "Item 'x' not found".to_string(),
            },
        };
        let json = serde_json::to_value(&body).unwrap();
        assert_eq!(json["error"]["code"], "NOT_FOUND");
        assert_eq!(json["error"]["message"], "Item 'x' not found");
    }
}
