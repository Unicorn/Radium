//! Search query builders for Neo4j
//!
//! Provides three search modes:
//! - **Semantic**: Vector similarity search across all label indexes
//! - **Structured**: Cypher-based filtering by kind, category, visibility, etc.
//! - **Combined**: Semantic search followed by in-memory filter application

use neo4rs::{query, Graph, Node};
use serde::{Deserialize, Serialize};

use super::client::{node_to_discovery_node, DiscoveryNode};
use super::error::GraphError;

/// Inbound search request from the API layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    /// Free-text query for semantic search
    pub query: Option<String>,
    /// Structured filters for field-level matching
    pub filters: Option<SearchFilters>,
    /// Scope: "all", "mine", or "marketplace"
    pub scope: Option<String>,
    /// Maximum results to return (default 10, max 100)
    pub limit: Option<i64>,
    /// Offset for pagination
    pub offset: Option<i64>,
}

/// Field-level filters for structured search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    /// Node types to include: "component", "service", "project"
    #[serde(rename = "type")]
    pub kind: Option<Vec<String>>,
    /// Category filter (exact match)
    pub category: Option<String>,
    /// Visibility filter: "public", "private", etc.
    pub visibility: Option<Vec<String>>,
    /// Filter for nodes that have an input schema defined
    pub has_input_schema: Option<String>,
    /// Minimum usage count threshold
    pub min_usage_count: Option<i64>,
}

/// Paginated search result envelope
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub results: Vec<ScoredNode>,
    pub total: usize,
}

/// A discovery node annotated with a relevance score
#[derive(Debug, Serialize)]
pub struct ScoredNode {
    #[serde(flatten)]
    pub node: DiscoveryNode,
    pub relevance_score: f64,
}

/// Semantic search using vector similarity across all label indexes.
///
/// Queries each vector index (`component_embedding`, `service_embedding`,
/// `project_embedding`) separately, merges results, deduplicates by ID,
/// and returns sorted by descending score.
pub async fn semantic_search(
    graph: &Graph,
    embedding: &[f32],
    limit: i64,
    offset: i64,
) -> Result<Vec<ScoredNode>, GraphError> {
    let mut all_results = Vec::new();

    let indexes = [
        "component_embedding",
        "service_embedding",
        "project_embedding",
    ];

    for index_name in &indexes {
        let cypher = format!(
            "CALL db.index.vector.queryNodes('{index_name}', $k, $embedding) \
             YIELD node, score \
             OPTIONAL MATCH (node)-[:TAGGED]->(t:Tag) \
             RETURN node, labels(node) AS labels, collect(t.name) AS tags, score \
             ORDER BY score DESC"
        );

        let result = graph
            .execute(
                query(&cypher)
                    .param("k", limit + offset)
                    .param("embedding", embedding.to_vec()),
            )
            .await;

        match result {
            Ok(mut rows) => {
                while let Ok(Some(row)) = rows.next().await {
                    let node: Node = match row.get("node") {
                        Ok(n) => n,
                        Err(_) => continue,
                    };
                    let labels: Vec<String> = row.get("labels").unwrap_or_default();
                    let tags: Vec<String> = row.get("tags").unwrap_or_default();
                    let score: f64 = row.get("score").unwrap_or(0.0);

                    all_results.push(ScoredNode {
                        node: node_to_discovery_node(&node, &labels, tags),
                        relevance_score: score,
                    });
                }
            }
            Err(e) => {
                // Index might not exist yet — log and continue to next index
                tracing::warn!("Vector search on {index_name} failed: {e}");
            }
        }
    }

    // Sort by score descending
    all_results.sort_by(|a, b| {
        b.relevance_score
            .partial_cmp(&a.relevance_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Deduplicate by node ID (keep highest score)
    let mut seen = std::collections::HashSet::new();
    all_results.retain(|item| seen.insert(item.node.id.clone()));

    // Apply offset and limit
    let start = (offset as usize).min(all_results.len());
    let end = (start + limit as usize).min(all_results.len());
    all_results = all_results[start..end].to_vec();

    Ok(all_results)
}

/// Structured search using dynamic Cypher WHERE clauses.
///
/// Builds filters from the provided `SearchFilters`, applies scope-based
/// access control, and orders by `usage_count` descending.
pub async fn structured_search(
    graph: &Graph,
    filters: &SearchFilters,
    scope: Option<&str>,
    user_id: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<ScoredNode>, GraphError> {
    let mut where_clauses = Vec::new();

    // Build label filter from kind list
    let label_filter = build_label_filter(filters.kind.as_ref());

    if let Some(ref cat) = filters.category {
        where_clauses.push(format!("n.category = '{}'", escape_cypher(cat)));
    }

    if let Some(ref vis_list) = filters.visibility {
        let vis_values: Vec<String> = vis_list.iter().map(|v| escape_cypher(v)).collect();
        where_clauses.push(format!("n.visibility IN [{}]", quote_list(&vis_values)));
    }

    if let Some(ref has_schema) = filters.has_input_schema {
        match has_schema.as_str() {
            "true" => where_clauses.push("n.input_schema IS NOT NULL".to_string()),
            "false" => where_clauses.push("n.input_schema IS NULL".to_string()),
            _ => {}
        }
    }

    if let Some(min) = filters.min_usage_count {
        where_clauses.push(format!("n.usage_count >= {min}"));
    }

    // Scope filtering
    match scope {
        Some("mine") => {
            if let Some(uid) = user_id {
                where_clauses.push(format!("n.owner_id = '{}'", escape_cypher(uid)));
            }
        }
        Some("marketplace") => {
            where_clauses.push("n.visibility = 'public'".to_string());
        }
        _ => {}
    }

    let where_str = if where_clauses.is_empty() {
        String::new()
    } else {
        format!(" AND {}", where_clauses.join(" AND "))
    };

    let cypher = format!(
        "MATCH (n) WHERE ({label_filter}){where_str} \
         OPTIONAL MATCH (n)-[:TAGGED]->(t:Tag) \
         RETURN n, labels(n) AS labels, collect(t.name) AS tags \
         ORDER BY n.usage_count DESC \
         SKIP {offset} LIMIT {limit}"
    );

    let mut results = Vec::new();
    let mut rows = graph.execute(query(&cypher)).await?;

    while let Ok(Some(row)) = rows.next().await {
        let node: Node = match row.get("n") {
            Ok(n) => n,
            Err(_) => continue,
        };
        let labels: Vec<String> = row.get("labels").unwrap_or_default();
        let tags: Vec<String> = row.get("tags").unwrap_or_default();

        let usage_count = node.get::<i64>("usage_count").unwrap_or(0);

        results.push(ScoredNode {
            node: node_to_discovery_node(&node, &labels, tags),
            // For structured search, use normalized usage_count as relevance
            // Precision loss is acceptable: usage_count is a relevance heuristic, not exact
            #[allow(clippy::cast_precision_loss)]
            relevance_score: usage_count as f64,
        });
    }

    Ok(results)
}

/// Combined search: semantic vector search first, then filter results in-memory.
///
/// Fetches a larger set via semantic search (3x the requested limit) and
/// then applies structured filters to the results before truncating.
pub async fn combined_search(
    graph: &Graph,
    embedding: &[f32],
    filters: &SearchFilters,
    scope: Option<&str>,
    user_id: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<ScoredNode>, GraphError> {
    // Fetch a wider result set from semantic search to allow for post-filtering
    let over_fetch_limit = (limit + offset) * 3;
    let mut candidates = semantic_search(graph, embedding, over_fetch_limit, 0).await?;

    // Apply structured filters in-memory
    candidates.retain(|item| matches_filters(&item.node, filters, scope, user_id));

    // Apply offset and limit
    let start = (offset as usize).min(candidates.len());
    let end = (start + limit as usize).min(candidates.len());
    candidates = candidates[start..end].to_vec();

    Ok(candidates)
}

/// Check if a node matches the given structured filters
fn matches_filters(
    node: &DiscoveryNode,
    filters: &SearchFilters,
    scope: Option<&str>,
    user_id: Option<&str>,
) -> bool {
    if let Some(ref kinds) = filters.kind {
        if !kinds.is_empty() && !kinds.contains(&node.kind) {
            return false;
        }
    }

    if let Some(ref cat) = filters.category {
        if node.category != *cat {
            return false;
        }
    }

    if let Some(ref vis_list) = filters.visibility {
        if !vis_list.is_empty() && !vis_list.contains(&node.visibility) {
            return false;
        }
    }

    if let Some(ref has_schema) = filters.has_input_schema {
        match has_schema.as_str() {
            "true" => {
                if node.input_schema.is_none() {
                    return false;
                }
            }
            "false" => {
                if node.input_schema.is_some() {
                    return false;
                }
            }
            _ => {}
        }
    }

    if let Some(min) = filters.min_usage_count {
        if node.usage_count < min {
            return false;
        }
    }

    // Scope filtering
    match scope {
        Some("mine") => {
            if let Some(uid) = user_id {
                if node.owner_id != uid {
                    return false;
                }
            }
        }
        Some("marketplace") => {
            if node.visibility != "public" {
                return false;
            }
        }
        _ => {}
    }

    true
}

/// Build a Cypher label filter expression from an optional list of kinds
fn build_label_filter(kinds: Option<&Vec<String>>) -> String {
    match kinds {
        Some(kinds) if !kinds.is_empty() => {
            let labels: Vec<String> = kinds
                .iter()
                .map(|k| {
                    // "component" and unknown kinds both map to Component intentionally
                    #[allow(clippy::match_same_arms)]
                    match k.as_str() {
                        "component" => "n:Component".to_string(),
                        "service" => "n:Service".to_string(),
                        "project" => "n:Project".to_string(),
                        _ => "n:Component".to_string(),
                    }
                })
                .collect();
            format!("({})", labels.join(" OR "))
        }
        _ => "(n:Component OR n:Service OR n:Project)".to_string(),
    }
}

/// Escape single quotes in Cypher string values to prevent injection
fn escape_cypher(s: &str) -> String {
    s.replace('\'', "\\'")
}

/// Format a list of strings as Cypher quoted values: 'a', 'b', 'c'
fn quote_list(items: &[String]) -> String {
    items
        .iter()
        .map(|s| format!("'{s}'"))
        .collect::<Vec<_>>()
        .join(", ")
}

// ScoredNode needs Clone for the slice/truncation operations above
impl Clone for ScoredNode {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
            relevance_score: self.relevance_score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_request_deserialization_full() {
        let json = serde_json::json!({
            "query": "send email",
            "filters": {
                "type": ["component", "service"],
                "category": "communication",
                "visibility": ["public"],
                "has_input_schema": "true",
                "min_usage_count": 5
            },
            "scope": "marketplace",
            "limit": 20,
            "offset": 10
        });

        let req: SearchRequest = serde_json::from_value(json).unwrap();
        assert_eq!(req.query.as_deref(), Some("send email"));
        assert_eq!(req.scope.as_deref(), Some("marketplace"));
        assert_eq!(req.limit, Some(20));
        assert_eq!(req.offset, Some(10));

        let filters = req.filters.unwrap();
        assert_eq!(filters.kind.as_ref().unwrap().len(), 2);
        assert_eq!(filters.category.as_deref(), Some("communication"));
        assert_eq!(filters.visibility.as_ref().unwrap(), &["public"]);
        assert_eq!(filters.has_input_schema.as_deref(), Some("true"));
        assert_eq!(filters.min_usage_count, Some(5));
    }

    #[test]
    fn test_search_request_deserialization_minimal() {
        let json = serde_json::json!({});
        let req: SearchRequest = serde_json::from_value(json).unwrap();
        assert!(req.query.is_none());
        assert!(req.filters.is_none());
        assert!(req.scope.is_none());
        assert!(req.limit.is_none());
        assert!(req.offset.is_none());
    }

    #[test]
    fn test_search_request_semantic_only() {
        let json = serde_json::json!({
            "query": "email notification"
        });
        let req: SearchRequest = serde_json::from_value(json).unwrap();
        assert_eq!(req.query.as_deref(), Some("email notification"));
        assert!(req.filters.is_none());
    }

    #[test]
    fn test_search_request_structured_only() {
        let json = serde_json::json!({
            "filters": {
                "type": ["component"],
                "category": "data"
            }
        });
        let req: SearchRequest = serde_json::from_value(json).unwrap();
        assert!(req.query.is_none());
        let filters = req.filters.unwrap();
        assert_eq!(filters.kind.as_ref().unwrap(), &["component"]);
        assert_eq!(filters.category.as_deref(), Some("data"));
    }

    #[test]
    fn test_search_filters_empty() {
        let json = serde_json::json!({});
        let filters: SearchFilters = serde_json::from_value(json).unwrap();
        assert!(filters.kind.is_none());
        assert!(filters.category.is_none());
        assert!(filters.visibility.is_none());
        assert!(filters.has_input_schema.is_none());
        assert!(filters.min_usage_count.is_none());
    }

    #[test]
    fn test_scored_node_serialization_flattens_node() {
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
        let scored = ScoredNode {
            node,
            relevance_score: 0.95,
        };
        let json = serde_json::to_value(&scored).unwrap();

        // Flattened fields from DiscoveryNode should be at top level
        assert_eq!(json["id"], "test-1");
        assert_eq!(json["kind"], "component");
        assert_eq!(json["name"], "send-email");
        assert_eq!(json["relevance_score"], 0.95);
        // No nested "node" key
        assert!(json.get("node").is_none());
    }

    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            results: vec![],
            total: 0,
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["total"], 0);
        assert!(json["results"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_build_label_filter_all() {
        let filter = build_label_filter(None);
        assert_eq!(filter, "(n:Component OR n:Service OR n:Project)");
    }

    #[test]
    fn test_build_label_filter_empty_vec() {
        let kinds = vec![];
        let filter = build_label_filter(Some(&kinds));
        assert_eq!(filter, "(n:Component OR n:Service OR n:Project)");
    }

    #[test]
    fn test_build_label_filter_single() {
        let kinds = vec!["service".to_string()];
        let filter = build_label_filter(Some(&kinds));
        assert_eq!(filter, "(n:Service)");
    }

    #[test]
    fn test_build_label_filter_multiple() {
        let kinds = vec!["component".to_string(), "project".to_string()];
        let filter = build_label_filter(Some(&kinds));
        assert_eq!(filter, "(n:Component OR n:Project)");
    }

    #[test]
    fn test_build_label_filter_unknown_defaults_to_component() {
        let kinds = vec!["widget".to_string()];
        let filter = build_label_filter(Some(&kinds));
        assert_eq!(filter, "(n:Component)");
    }

    #[test]
    fn test_escape_cypher() {
        assert_eq!(escape_cypher("hello"), "hello");
        assert_eq!(escape_cypher("it's"), "it\\'s");
        assert_eq!(escape_cypher("a'b'c"), "a\\'b\\'c");
    }

    #[test]
    fn test_quote_list() {
        let items = vec!["public".to_string(), "private".to_string()];
        assert_eq!(quote_list(&items), "'public', 'private'");
    }

    #[test]
    fn test_quote_list_empty() {
        let items: Vec<String> = vec![];
        assert_eq!(quote_list(&items), "");
    }

    #[test]
    fn test_matches_filters_all_pass() {
        let node = make_test_node("component", "communication", "public", 10, true);
        let filters = SearchFilters {
            kind: Some(vec!["component".to_string()]),
            category: Some("communication".to_string()),
            visibility: Some(vec!["public".to_string()]),
            has_input_schema: Some("true".to_string()),
            min_usage_count: Some(5),
        };
        assert!(matches_filters(&node, &filters, None, None));
    }

    #[test]
    fn test_matches_filters_kind_mismatch() {
        let node = make_test_node("service", "data", "public", 10, false);
        let filters = SearchFilters {
            kind: Some(vec!["component".to_string()]),
            category: None,
            visibility: None,
            has_input_schema: None,
            min_usage_count: None,
        };
        assert!(!matches_filters(&node, &filters, None, None));
    }

    #[test]
    fn test_matches_filters_category_mismatch() {
        let node = make_test_node("component", "data", "public", 10, false);
        let filters = SearchFilters {
            kind: None,
            category: Some("communication".to_string()),
            visibility: None,
            has_input_schema: None,
            min_usage_count: None,
        };
        assert!(!matches_filters(&node, &filters, None, None));
    }

    #[test]
    fn test_matches_filters_usage_count_below_min() {
        let node = make_test_node("component", "data", "public", 3, false);
        let filters = SearchFilters {
            kind: None,
            category: None,
            visibility: None,
            has_input_schema: None,
            min_usage_count: Some(5),
        };
        assert!(!matches_filters(&node, &filters, None, None));
    }

    #[test]
    fn test_matches_filters_scope_mine() {
        let node = make_test_node("component", "data", "public", 10, false);
        let filters = SearchFilters {
            kind: None,
            category: None,
            visibility: None,
            has_input_schema: None,
            min_usage_count: None,
        };
        // Owner matches
        assert!(matches_filters(&node, &filters, Some("mine"), Some("user-1")));
        // Owner does not match
        assert!(!matches_filters(&node, &filters, Some("mine"), Some("user-2")));
    }

    #[test]
    fn test_matches_filters_scope_marketplace() {
        let public_node = make_test_node("component", "data", "public", 10, false);
        let private_node = make_test_node("component", "data", "private", 10, false);
        let filters = SearchFilters {
            kind: None,
            category: None,
            visibility: None,
            has_input_schema: None,
            min_usage_count: None,
        };
        assert!(matches_filters(&public_node, &filters, Some("marketplace"), None));
        assert!(!matches_filters(&private_node, &filters, Some("marketplace"), None));
    }

    #[test]
    fn test_matches_filters_has_input_schema_true() {
        let with_schema = make_test_node("component", "data", "public", 10, true);
        let without_schema = make_test_node("component", "data", "public", 10, false);
        let filters = SearchFilters {
            kind: None,
            category: None,
            visibility: None,
            has_input_schema: Some("true".to_string()),
            min_usage_count: None,
        };
        assert!(matches_filters(&with_schema, &filters, None, None));
        assert!(!matches_filters(&without_schema, &filters, None, None));
    }

    #[test]
    fn test_matches_filters_has_input_schema_false() {
        let with_schema = make_test_node("component", "data", "public", 10, true);
        let without_schema = make_test_node("component", "data", "public", 10, false);
        let filters = SearchFilters {
            kind: None,
            category: None,
            visibility: None,
            has_input_schema: Some("false".to_string()),
            min_usage_count: None,
        };
        assert!(!matches_filters(&with_schema, &filters, None, None));
        assert!(matches_filters(&without_schema, &filters, None, None));
    }

    #[test]
    fn test_matches_filters_no_filters() {
        let node = make_test_node("component", "data", "public", 10, false);
        let filters = SearchFilters {
            kind: None,
            category: None,
            visibility: None,
            has_input_schema: None,
            min_usage_count: None,
        };
        assert!(matches_filters(&node, &filters, None, None));
    }

    #[test]
    fn test_scored_node_clone() {
        let scored = ScoredNode {
            node: make_test_node("component", "data", "public", 5, false),
            relevance_score: 0.88,
        };
        let cloned = scored.clone();
        assert_eq!(cloned.node.id, scored.node.id);
        assert_eq!(cloned.relevance_score, scored.relevance_score);
    }

    /// Helper to construct a `DiscoveryNode` for filter tests
    fn make_test_node(
        kind: &str,
        category: &str,
        visibility: &str,
        usage_count: i64,
        has_schema: bool,
    ) -> DiscoveryNode {
        DiscoveryNode {
            id: "test-1".to_string(),
            kind: kind.to_string(),
            name: "test-node".to_string(),
            description: "A test node".to_string(),
            category: category.to_string(),
            visibility: visibility.to_string(),
            owner_id: "user-1".to_string(),
            usage_count,
            tags: vec![],
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            input_schema: if has_schema {
                Some(serde_json::json!({"type": "object"}))
            } else {
                None
            },
            output_schema: None,
        }
    }
}
