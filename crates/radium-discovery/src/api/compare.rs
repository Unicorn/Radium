//! Compare endpoint — side-by-side evaluation of multiple items

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::graph;
use crate::state::AppState;

/// Query parameters for the compare endpoint
#[derive(Debug, Deserialize)]
pub struct CompareQuery {
    /// Comma-separated item IDs
    pub ids: String,
}

/// Response wrapper for the compare endpoint
#[derive(Debug, Serialize)]
pub struct CompareResponse {
    pub items: Vec<CompareItem>,
}

/// A single item in the comparison result with enrichment data
#[derive(Debug, Serialize)]
pub struct CompareItem {
    #[serde(flatten)]
    pub node: graph::DiscoveryNode,
    pub co_used_with: Vec<String>,
    pub dependency_count: i64,
}

/// Error type for compare operations
#[derive(Debug)]
pub(crate) struct CompareError {
    status: StatusCode,
    code: String,
    message: String,
}

impl CompareError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "BAD_REQUEST".to_string(),
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

impl IntoResponse for CompareError {
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

/// GET /v1/discover/compare?ids=a,b,c
///
/// Fetch multiple items and return them side-by-side with enrichment data
/// including co-usage relationships and dependency counts.
pub async fn compare(
    State(state): State<AppState>,
    Query(params): Query<CompareQuery>,
) -> Result<impl IntoResponse, CompareError> {
    let ids: Vec<&str> = params
        .ids
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    if ids.is_empty() {
        return Err(CompareError::bad_request("ids parameter is required"));
    }
    if ids.len() > 10 {
        return Err(CompareError::bad_request(
            "Maximum 10 items can be compared",
        ));
    }

    let mut items = Vec::new();

    for id in &ids {
        let node = match graph::client::get_node(&state.graph, id).await {
            Ok(n) => n,
            Err(graph::GraphError::NotFound { .. }) => continue, // Skip missing items
            Err(e) => return Err(CompareError::internal(e.to_string())),
        };

        // Get co-used items
        let co_used = get_co_used_ids(&state.graph, id).await;

        // Get dependency count
        let dep_count = get_dependency_count(&state.graph, id).await;

        items.push(CompareItem {
            node,
            co_used_with: co_used,
            dependency_count: dep_count,
        });
    }

    Ok(Json(CompareResponse { items }))
}

/// Fetch IDs of items frequently used alongside the given item
async fn get_co_used_ids(graph: &neo4rs::Graph, id: &str) -> Vec<String> {
    let cypher = "MATCH (n {id: $id})-[:CO_USED_WITH]-(other) \
                  RETURN other.id AS other_id \
                  ORDER BY other.usage_count DESC \
                  LIMIT 5";
    match graph
        .execute(neo4rs::query(cypher).param("id", id))
        .await
    {
        Ok(mut result) => {
            let mut ids = Vec::new();
            while let Ok(Some(row)) = result.next().await {
                if let Ok(other_id) = row.get::<String>("other_id") {
                    ids.push(other_id);
                }
            }
            ids
        }
        Err(e) => {
            tracing::warn!("Failed to fetch co-used items for {id}: {e}");
            Vec::new()
        }
    }
}

/// Fetch the number of outgoing dependency relationships for an item
async fn get_dependency_count(graph: &neo4rs::Graph, id: &str) -> i64 {
    let cypher = "MATCH (n {id: $id})-[:USES|DEPENDS_ON]->(dep) RETURN count(dep) AS count";
    match graph
        .execute(neo4rs::query(cypher).param("id", id))
        .await
    {
        Ok(mut result) => {
            if let Ok(Some(row)) = result.next().await {
                row.get::<i64>("count").unwrap_or(0)
            } else {
                0
            }
        }
        Err(e) => {
            tracing::warn!("Failed to fetch dependency count for {id}: {e}");
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_query_deserialization() {
        let json = r#"{"ids":"abc,def,ghi"}"#;
        let params: CompareQuery = serde_json::from_str(json).unwrap();
        assert_eq!(params.ids, "abc,def,ghi");

        let ids: Vec<&str> = params
            .ids
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        assert_eq!(ids, vec!["abc", "def", "ghi"]);
    }

    #[test]
    fn test_compare_item_serialization() {
        let item = CompareItem {
            node: graph::DiscoveryNode {
                id: "test-1".to_string(),
                kind: "component".to_string(),
                name: "send-email".to_string(),
                description: "Sends emails".to_string(),
                category: "communication".to_string(),
                visibility: "public".to_string(),
                owner_id: "user-1".to_string(),
                usage_count: 42,
                tags: vec!["email".to_string()],
                created_at: "2026-01-01T00:00:00Z".to_string(),
                updated_at: "2026-01-01T00:00:00Z".to_string(),
                input_schema: None,
                output_schema: None,
            },
            co_used_with: vec!["test-2".to_string(), "test-3".to_string()],
            dependency_count: 5,
        };

        let json = serde_json::to_value(&item).unwrap();

        // Verify #[serde(flatten)] promotes node fields to top level
        assert_eq!(json["id"], "test-1");
        assert_eq!(json["kind"], "component");
        assert_eq!(json["name"], "send-email");
        assert_eq!(json["usage_count"], 42);

        // Verify enrichment fields are present alongside node fields
        assert_eq!(json["co_used_with"], serde_json::json!(["test-2", "test-3"]));
        assert_eq!(json["dependency_count"], 5);

        // Verify there is no nested "node" key (flatten works correctly)
        assert!(json.get("node").is_none());
    }

    #[test]
    fn test_compare_error_bad_request() {
        let err = CompareError::bad_request("ids parameter is required");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.code, "BAD_REQUEST");
        assert_eq!(err.message, "ids parameter is required");
    }

    #[test]
    fn test_compare_error_internal() {
        let err = CompareError::internal("something went wrong");
        assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.code, "INTERNAL_ERROR");
        assert_eq!(err.message, "something went wrong");
    }

    #[test]
    fn test_compare_response_serialization() {
        let response = CompareResponse {
            items: vec![CompareItem {
                node: graph::DiscoveryNode {
                    id: "a".to_string(),
                    kind: "service".to_string(),
                    name: "auth-service".to_string(),
                    description: "Authentication".to_string(),
                    category: "security".to_string(),
                    visibility: "public".to_string(),
                    owner_id: "user-1".to_string(),
                    usage_count: 100,
                    tags: vec![],
                    created_at: "2026-01-01T00:00:00Z".to_string(),
                    updated_at: "2026-01-01T00:00:00Z".to_string(),
                    input_schema: None,
                    output_schema: None,
                },
                co_used_with: vec![],
                dependency_count: 3,
            }],
        };

        let json = serde_json::to_value(&response).unwrap();
        let items = json["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["id"], "a");
        assert_eq!(items[0]["dependency_count"], 3);
    }

    #[test]
    fn test_compare_error_body_serialization() {
        let body = ErrorBody {
            error: ErrorDetail {
                code: "BAD_REQUEST".to_string(),
                message: "ids parameter is required".to_string(),
            },
        };
        let json = serde_json::to_value(&body).unwrap();
        assert_eq!(json["error"]["code"], "BAD_REQUEST");
        assert_eq!(json["error"]["message"], "ids parameter is required");
    }

    #[test]
    fn test_compare_query_empty_ids_parsing() {
        // Verify that empty string and whitespace-only entries are filtered
        let ids_str = "a,,b, ,c";
        let ids: Vec<&str> = ids_str
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        assert_eq!(ids, vec!["a", "b", "c"]);
    }
}
