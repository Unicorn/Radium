//! Search endpoint handler
//!
//! Dispatches to semantic, structured, or combined search based on the
//! presence of `query` and `filters` fields in the request body.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::graph::queries::{self, SearchRequest, SearchResult};
use crate::state::AppState;

/// Error type for search operations
#[derive(Debug)]
pub(crate) struct SearchError {
    status: StatusCode,
    code: String,
    message: String,
}

impl SearchError {
    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "INTERNAL_ERROR".to_string(),
            message: message.into(),
        }
    }

    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "BAD_REQUEST".to_string(),
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

impl IntoResponse for SearchError {
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

/// POST /v1/discover/search
///
/// Dispatches to the appropriate search mode based on the request:
/// - `query` only -> semantic search (vector similarity)
/// - `filters` only -> structured search (Cypher WHERE clauses)
/// - `query` + `filters` -> combined (semantic + post-filter)
/// - neither -> empty results
pub async fn search(
    State(state): State<AppState>,
    Json(req): Json<SearchRequest>,
) -> Result<impl IntoResponse, SearchError> {
    let limit = req.limit.unwrap_or(10).min(100);
    let offset = req.offset.unwrap_or(0);

    if limit < 0 {
        return Err(SearchError::bad_request("limit must be non-negative"));
    }
    if offset < 0 {
        return Err(SearchError::bad_request("offset must be non-negative"));
    }

    let results = match (&req.query, &req.filters) {
        // Semantic search only
        (Some(query_text), None) => {
            if query_text.trim().is_empty() {
                return Err(SearchError::bad_request("query must not be empty"));
            }
            let embedding = state
                .embeddings
                .embed(query_text)
                .await
                .map_err(|e| SearchError::internal(format!("Embedding generation failed: {e}")))?;

            queries::semantic_search(&state.graph, &embedding, limit, offset)
                .await
                .map_err(|e| SearchError::internal(e.to_string()))?
        }

        // Structured search only
        (None, Some(filters)) => {
            queries::structured_search(
                &state.graph,
                filters,
                req.scope.as_deref(),
                None, // user_id would come from auth middleware in the future
                limit,
                offset,
            )
            .await
            .map_err(|e| SearchError::internal(e.to_string()))?
        }

        // Combined: semantic + structured filters
        (Some(query_text), Some(filters)) => {
            if query_text.trim().is_empty() {
                return Err(SearchError::bad_request("query must not be empty"));
            }
            let embedding = state
                .embeddings
                .embed(query_text)
                .await
                .map_err(|e| SearchError::internal(format!("Embedding generation failed: {e}")))?;

            queries::combined_search(
                &state.graph,
                &embedding,
                filters,
                req.scope.as_deref(),
                None,
                limit,
                offset,
            )
            .await
            .map_err(|e| SearchError::internal(e.to_string()))?
        }

        // Neither query nor filters provided
        (None, None) => Vec::new(),
    };

    let total = results.len();
    Ok(Json(SearchResult { results, total }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_error_internal() {
        let err = SearchError::internal("something went wrong");
        assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.code, "INTERNAL_ERROR");
        assert_eq!(err.message, "something went wrong");
    }

    #[test]
    fn test_search_error_bad_request() {
        let err = SearchError::bad_request("invalid input");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.code, "BAD_REQUEST");
        assert_eq!(err.message, "invalid input");
    }

    #[test]
    fn test_error_body_serialization() {
        let body = ErrorBody {
            error: ErrorDetail {
                code: "BAD_REQUEST".to_string(),
                message: "test error".to_string(),
            },
        };
        let json = serde_json::to_value(&body).unwrap();
        assert_eq!(json["error"]["code"], "BAD_REQUEST");
        assert_eq!(json["error"]["message"], "test error");
    }
}
