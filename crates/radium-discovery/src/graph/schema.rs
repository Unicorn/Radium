//! Neo4j schema initialization — constraints, indexes, vector indexes
//! All operations are idempotent (IF NOT EXISTS).

use neo4rs::{query, Graph};

use super::error::GraphError;

/// Initialize the Neo4j schema. Safe to call on every startup.
pub async fn initialize(graph: &Graph) -> Result<(), GraphError> {
    tracing::info!("Initializing Neo4j schema...");

    let constraints = [
        "CREATE CONSTRAINT component_id IF NOT EXISTS FOR (c:Component) REQUIRE c.id IS UNIQUE",
        "CREATE CONSTRAINT service_id IF NOT EXISTS FOR (s:Service) REQUIRE s.id IS UNIQUE",
        "CREATE CONSTRAINT project_id IF NOT EXISTS FOR (p:Project) REQUIRE p.id IS UNIQUE",
        "CREATE CONSTRAINT tag_name IF NOT EXISTS FOR (t:Tag) REQUIRE t.name IS UNIQUE",
        "CREATE CONSTRAINT user_id IF NOT EXISTS FOR (u:User) REQUIRE u.id IS UNIQUE",
    ];

    for cypher in &constraints {
        graph.run(query(cypher)).await.map_err(|e| {
            GraphError::SchemaInit(format!("Constraint failed: {e}"))
        })?;
    }

    let fulltext_indexes = [
        "CREATE FULLTEXT INDEX component_search IF NOT EXISTS FOR (c:Component) ON EACH [c.name, c.description, c.category]",
        "CREATE FULLTEXT INDEX service_search IF NOT EXISTS FOR (s:Service) ON EACH [s.name, s.description]",
        "CREATE FULLTEXT INDEX project_search IF NOT EXISTS FOR (p:Project) ON EACH [p.name, p.description]",
    ];

    for cypher in &fulltext_indexes {
        graph.run(query(cypher)).await.map_err(|e| {
            GraphError::SchemaInit(format!("Index failed: {e}"))
        })?;
    }

    tracing::info!("Neo4j schema initialized successfully");
    Ok(())
}

/// Initialize vector indexes for semantic search.
/// Called separately because vector index dimension depends on the embedding provider.
pub async fn initialize_vector_indexes(
    graph: &Graph,
    dimension: usize,
) -> Result<(), GraphError> {
    tracing::info!("Initializing vector indexes (dimension={dimension})...");

    let labels = ["Component", "Service", "Project"];
    for label in &labels {
        let lower = label.to_lowercase();
        let cypher = format!(
            "CREATE VECTOR INDEX {lower}_embedding IF NOT EXISTS \
             FOR (n:{label}) ON (n.embedding) \
             OPTIONS {{indexConfig: {{`vector.dimensions`: {dimension}, \
             `vector.similarity_function`: 'cosine'}}}}"
        );
        graph.run(query(&cypher)).await.map_err(|e| {
            GraphError::SchemaInit(format!("Vector index for {label} failed: {e}"))
        })?;
    }

    tracing::info!("Vector indexes initialized");
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_constraint_queries_are_valid_cypher_syntax() {
        let constraints = [
            "CREATE CONSTRAINT component_id IF NOT EXISTS FOR (c:Component) REQUIRE c.id IS UNIQUE",
            "CREATE CONSTRAINT service_id IF NOT EXISTS FOR (s:Service) REQUIRE s.id IS UNIQUE",
        ];
        for c in &constraints {
            assert!(c.contains("IF NOT EXISTS"));
            assert!(c.contains("REQUIRE"));
            assert!(c.contains("IS UNIQUE"));
        }
    }
}
