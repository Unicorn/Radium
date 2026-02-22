//! Shared application state

use std::sync::Arc;
use crate::embeddings::EmbeddingProvider;

/// Application state shared across all handlers
#[derive(Clone)]
#[allow(dead_code)] // Fields used when API endpoints are implemented (Tasks 5-8)
pub struct AppState {
    pub graph: neo4rs::Graph,
    pub embeddings: Arc<dyn EmbeddingProvider>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("graph", &"Neo4j Graph { .. }")
            .field("embeddings", &"EmbeddingProvider { .. }")
            .finish()
    }
}
