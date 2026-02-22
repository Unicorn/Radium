//! Graph operation errors

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("Neo4j error: {0}")]
    Neo4j(#[from] neo4rs::Error),

    #[error("Node not found: {kind} with id={id}")]
    NotFound { kind: String, id: String },

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Schema initialization failed: {0}")]
    SchemaInit(String),
}
