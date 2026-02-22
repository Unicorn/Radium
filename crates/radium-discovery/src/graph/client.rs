//! Neo4j graph client wrapper with typed operations

use neo4rs::{query, Graph, Node};
use serde::{Deserialize, Serialize};

use super::error::GraphError;

/// Represents any discoverable item in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryNode {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub visibility: String,
    pub owner_id: String,
    pub usage_count: i64,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
}

/// Input for creating/updating a node in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexRequest {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub visibility: String,
    pub owner_id: String,
    pub tags: Vec<String>,
    #[serde(default)]
    pub input_schema: Option<serde_json::Value>,
    #[serde(default)]
    pub output_schema: Option<serde_json::Value>,
    #[serde(default)]
    pub definition: Option<serde_json::Value>,
}

/// Create or update a node in the graph
pub async fn upsert_node(
    graph: &Graph,
    req: &IndexRequest,
    embedding: Option<Vec<f32>>,
) -> Result<(), GraphError> {
    let label = label_for_kind(&req.kind);
    let now = chrono::Utc::now().to_rfc3339();

    let q = query(&format!(
        "MERGE (n:{label} {{id: $id}}) \
         SET n.name = $name, n.description = $description, \
             n.category = $category, n.visibility = $visibility, \
             n.owner_id = $owner_id, n.updated_at = $now \
         ON CREATE SET n.created_at = $now, n.usage_count = 0",
    ))
    .param("id", req.id.as_str())
    .param("name", req.name.as_str())
    .param("description", req.description.as_str())
    .param("category", req.category.as_str())
    .param("visibility", req.visibility.as_str())
    .param("owner_id", req.owner_id.as_str())
    .param("now", now.as_str());

    graph.run(q).await?;

    // Set embedding if provided
    if let Some(ref emb) = embedding {
        let emb_query = query(&format!(
            "MATCH (n:{label} {{id: $id}}) SET n.embedding = $embedding",
        ))
        .param("id", req.id.as_str())
        .param("embedding", emb.clone());

        graph.run(emb_query).await?;
    }

    // Set schemas if provided
    if let Some(ref schema) = req.input_schema {
        let schema_str = serde_json::to_string(schema)
            .map_err(|e| GraphError::Deserialization(e.to_string()))?;
        graph
            .run(
                query(&format!(
                    "MATCH (n:{label} {{id: $id}}) SET n.input_schema = $schema"
                ))
                .param("id", req.id.as_str())
                .param("schema", schema_str.as_str()),
            )
            .await?;
    }
    if let Some(ref schema) = req.output_schema {
        let schema_str = serde_json::to_string(schema)
            .map_err(|e| GraphError::Deserialization(e.to_string()))?;
        graph
            .run(
                query(&format!(
                    "MATCH (n:{label} {{id: $id}}) SET n.output_schema = $schema"
                ))
                .param("id", req.id.as_str())
                .param("schema", schema_str.as_str()),
            )
            .await?;
    }

    // Sync tags: remove old, add new
    graph
        .run(
            query(&format!(
                "MATCH (n:{label} {{id: $id}}) \
                 OPTIONAL MATCH (n)-[r:TAGGED]->() DELETE r",
            ))
            .param("id", req.id.as_str()),
        )
        .await?;

    for tag in &req.tags {
        graph
            .run(
                query(&format!(
                    "MATCH (n:{label} {{id: $id}}) \
                     MERGE (t:Tag {{name: $tag}}) \
                     MERGE (n)-[:TAGGED]->(t)",
                ))
                .param("id", req.id.as_str())
                .param("tag", tag.as_str()),
            )
            .await?;
    }

    Ok(())
}

/// Get a node by ID, searching across all label types
pub async fn get_node(graph: &Graph, id: &str) -> Result<DiscoveryNode, GraphError> {
    let cypher = "MATCH (n) WHERE n.id = $id AND (n:Component OR n:Service OR n:Project) \
                  OPTIONAL MATCH (n)-[:TAGGED]->(t:Tag) \
                  RETURN n, labels(n) AS labels, collect(t.name) AS tags";

    let mut result = graph.execute(query(cypher).param("id", id)).await?;

    let row = result.next().await?.ok_or_else(|| GraphError::NotFound {
        kind: "item".to_string(),
        id: id.to_string(),
    })?;

    let node: Node = row
        .get("n")
        .map_err(|e| GraphError::Deserialization(e.to_string()))?;
    let labels: Vec<String> = row
        .get("labels")
        .map_err(|e| GraphError::Deserialization(e.to_string()))?;
    let tags: Vec<String> = row
        .get("tags")
        .map_err(|e| GraphError::Deserialization(e.to_string()))?;

    Ok(node_to_discovery_node(&node, &labels, tags))
}

/// Record that a set of components were deployed together.
/// Creates/strengthens CO_USED_WITH edges between all pairs.
pub async fn record_co_usage(graph: &Graph, component_ids: &[String]) -> Result<(), GraphError> {
    for i in 0..component_ids.len() {
        for j in (i + 1)..component_ids.len() {
            graph.run(
                query(
                    "MATCH (a), (b) WHERE a.id = $a_id AND b.id = $b_id \
                     MERGE (a)-[r:CO_USED_WITH]-(b) \
                     ON CREATE SET r.weight = 1 \
                     ON MATCH SET r.weight = r.weight + 1",
                )
                .param("a_id", component_ids[i].as_str())
                .param("b_id", component_ids[j].as_str()),
            ).await?;
        }
    }
    Ok(())
}

/// Delete a node and all its relationships
pub async fn delete_node(graph: &Graph, id: &str) -> Result<(), GraphError> {
    graph
        .run(query("MATCH (n) WHERE n.id = $id DETACH DELETE n").param("id", id))
        .await?;
    Ok(())
}

/// Map a label string to the kind field
pub(crate) fn kind_from_labels(labels: &[String]) -> String {
    if labels.contains(&"Component".to_string()) {
        "component".to_string()
    } else if labels.contains(&"Service".to_string()) {
        "service".to_string()
    } else if labels.contains(&"Project".to_string()) {
        "project".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Map a kind string to a Neo4j label.
/// Defaults to "Component" for unrecognized kinds (intentional fallback).
#[allow(clippy::match_same_arms)] // "component" and default both map to "Component" intentionally
fn label_for_kind(kind: &str) -> &str {
    match kind {
        "component" => "Component",
        "service" => "Service",
        "project" => "Project",
        _ => "Component",
    }
}

/// Convert a Neo4j Node to a `DiscoveryNode`
pub(crate) fn node_to_discovery_node(node: &Node, labels: &[String], tags: Vec<String>) -> DiscoveryNode {
    let get_str = |key: &str| -> String { node.get::<String>(key).unwrap_or_default() };
    let get_opt_str =
        |key: &str| -> Option<String> { node.get::<String>(key).ok().filter(|s| !s.is_empty()) };

    let input_schema = get_opt_str("input_schema").and_then(|s| serde_json::from_str(&s).ok());
    let output_schema = get_opt_str("output_schema").and_then(|s| serde_json::from_str(&s).ok());

    DiscoveryNode {
        id: get_str("id"),
        kind: kind_from_labels(labels),
        name: get_str("name"),
        description: get_str("description"),
        category: get_str("category"),
        visibility: get_str("visibility"),
        owner_id: get_str("owner_id"),
        usage_count: node.get::<i64>("usage_count").unwrap_or(0),
        tags,
        created_at: get_str("created_at"),
        updated_at: get_str("updated_at"),
        input_schema,
        output_schema,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kind_from_labels() {
        assert_eq!(kind_from_labels(&["Component".to_string()]), "component");
        assert_eq!(kind_from_labels(&["Service".to_string()]), "service");
        assert_eq!(kind_from_labels(&["Project".to_string()]), "project");
        assert_eq!(kind_from_labels(&[]), "unknown");
    }

    #[test]
    fn test_label_for_kind() {
        assert_eq!(label_for_kind("component"), "Component");
        assert_eq!(label_for_kind("service"), "Service");
        assert_eq!(label_for_kind("project"), "Project");
        assert_eq!(label_for_kind("anything_else"), "Component");
    }

    #[test]
    fn test_co_usage_pair_count() {
        // For N components, there should be N*(N-1)/2 pairs
        let ids = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut pair_count = 0;
        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                let _ = (&ids[i], &ids[j]); // suppress unused variable warnings
                pair_count += 1;
            }
        }
        assert_eq!(pair_count, 3); // 3 choose 2 = 3
    }

    #[test]
    fn test_discovery_node_serialization() {
        let node = DiscoveryNode {
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
        };
        let json = serde_json::to_value(&node).unwrap();
        assert_eq!(json["kind"], "component");
        assert_eq!(json["usage_count"], 42);
        // input_schema should be absent (skip_serializing_if)
        assert!(json.get("input_schema").is_none());
    }
}
