//! Relationship inference pipeline
//!
//! Automatically creates edges in the graph based on:
//! - Component references in workflow definitions
//! - Schema similarity between items

pub mod definition_parser;
pub mod schema_similarity;

use neo4rs::{query, Graph};

use crate::graph::GraphError;

/// Run inference on a newly indexed item.
/// Creates USES/DEPENDS_ON edges from definition parsing,
/// and SIMILAR_SCHEMA edges from schema comparison.
pub async fn infer_relationships(
    graph: &Graph,
    id: &str,
    kind: &str,
    definition: Option<&serde_json::Value>,
    input_schema: Option<&serde_json::Value>,
) -> Result<(), GraphError> {
    // 1. Parse definition for component references
    if let Some(def) = definition {
        let rels = definition_parser::extract_relationships(def);

        // Create USES edges for components
        for comp_id in &rels.component_ids {
            graph
                .run(
                    query(
                        "MATCH (a), (b) WHERE a.id = $src_id AND b.id = $target_id \
                         MERGE (a)-[:USES]->(b)",
                    )
                    .param("src_id", id)
                    .param("target_id", comp_id.as_str()),
                )
                .await
                .map_err(|e| GraphError::SchemaInit(format!("USES edge failed: {e}")))?;
        }

        // Create DEPENDS_ON edges for child workflows
        for wf_id in &rels.child_workflow_ids {
            graph
                .run(
                    query(
                        "MATCH (a), (b) WHERE a.id = $src_id AND b.id = $target_id \
                         MERGE (a)-[:DEPENDS_ON]->(b)",
                    )
                    .param("src_id", id)
                    .param("target_id", wf_id.as_str()),
                )
                .await
                .map_err(|e| GraphError::SchemaInit(format!("DEPENDS_ON edge failed: {e}")))?;
        }
    }

    // 2. Find schema-similar items (only for components)
    if kind == "component" {
        if let Some(schema) = input_schema {
            find_and_link_similar(graph, id, schema).await?;
        }
    }

    Ok(())
}

/// Find existing items with similar schemas and create SIMILAR_SCHEMA edges
async fn find_and_link_similar(
    graph: &Graph,
    id: &str,
    schema: &serde_json::Value,
) -> Result<(), GraphError> {
    // Get all components with input schemas
    let cypher = "MATCH (n:Component) WHERE n.id <> $id AND n.input_schema IS NOT NULL \
                  RETURN n.id AS other_id, n.input_schema AS other_schema";

    let mut result = graph.execute(query(cypher).param("id", id)).await?;

    while let Some(row) = result.next().await? {
        let other_id: String = row
            .get("other_id")
            .map_err(|e| GraphError::Deserialization(e.to_string()))?;
        let other_schema_str: String = row
            .get("other_schema")
            .map_err(|e| GraphError::Deserialization(e.to_string()))?;

        if let Ok(other_schema) = serde_json::from_str::<serde_json::Value>(&other_schema_str) {
            let similarity = schema_similarity::schema_overlap(schema, &other_schema);

            if similarity > 0.5 {
                graph
                    .run(
                        query(
                            "MATCH (a {id: $a_id}), (b {id: $b_id}) \
                             MERGE (a)-[r:SIMILAR_SCHEMA]-(b) \
                             SET r.similarity = $similarity",
                        )
                        .param("a_id", id)
                        .param("b_id", other_id.as_str())
                        .param("similarity", similarity),
                    )
                    .await
                    .map_err(|e| {
                        GraphError::SchemaInit(format!("SIMILAR_SCHEMA edge failed: {e}"))
                    })?;
            }
        }
    }

    Ok(())
}
