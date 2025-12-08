//! Tool catalog aggregation with conflict resolution.
//!
//! This module aggregates tool definitions from multiple upstream servers
//! and handles name conflicts using configurable resolution strategies.

use crate::mcp::proxy::types::{ConflictStrategy, ToolCatalog as ToolCatalogTrait};
use crate::mcp::proxy::upstream_pool::UpstreamPool;
use crate::mcp::{McpTool, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Default implementation of tool catalog aggregation.
pub struct DefaultToolCatalog {
    /// Map of registered tool names to tool definitions.
    tools: Arc<RwLock<HashMap<String, McpTool>>>,
    /// Map of registered tool names to their source upstream.
    tool_sources: Arc<RwLock<HashMap<String, String>>>,
    /// Map of registered tool names to original tool names.
    original_names: Arc<RwLock<HashMap<String, String>>>,
    /// Conflict resolution strategy.
    conflict_strategy: ConflictStrategy,
    /// Map of upstream names to their priorities.
    upstream_priorities: HashMap<String, u32>,
}

impl DefaultToolCatalog {
    /// Create a new tool catalog.
    ///
    /// # Arguments
    ///
    /// * `strategy` - Conflict resolution strategy
    /// * `priorities` - Map of upstream names to priorities
    pub fn new(strategy: ConflictStrategy, priorities: HashMap<String, u32>) -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            tool_sources: Arc::new(RwLock::new(HashMap::new())),
            original_names: Arc::new(RwLock::new(HashMap::new())),
            conflict_strategy: strategy,
            upstream_priorities: priorities,
        }
    }

    /// Add tools from an upstream server.
    ///
    /// # Arguments
    ///
    /// * `upstream_name` - Name of the upstream server
    /// * `tools` - List of tools from this upstream
    pub async fn add_tools(&self, upstream_name: String, tools: Vec<McpTool>) {
        let mut tools_map = self.tools.write().await;
        let mut sources_map = self.tool_sources.write().await;
        let mut original_map = self.original_names.write().await;

        for tool in tools {
            let original_name = tool.name.clone();
            let registered_name = self.resolve_tool_name(
                &original_name,
                &upstream_name,
                &tools_map,
                &sources_map,
            );

            // Check if we should register this tool based on conflict strategy
            let should_register = match self.conflict_strategy {
                ConflictStrategy::AutoPrefix => true, // Always register, may be prefixed
                ConflictStrategy::Reject => {
                    // Only register if name doesn't exist
                    !tools_map.contains_key(&original_name)
                }
                ConflictStrategy::PriorityOverride => {
                    // Register if name doesn't exist, or if our priority is higher
                    if let Some(existing_source) = sources_map.get(&original_name) {
                        let existing_priority = self
                            .upstream_priorities
                            .get(existing_source)
                            .copied()
                            .unwrap_or(u32::MAX);
                        let new_priority = self
                            .upstream_priorities
                            .get(&upstream_name)
                            .copied()
                            .unwrap_or(u32::MAX);

                        // Lower number = higher priority
                        new_priority < existing_priority
                    } else {
                        true
                    }
                }
            };

            if should_register {
                // Remove existing tool if we're overriding
                if tools_map.contains_key(&registered_name) && registered_name != original_name {
                    // This is a conflict, remove the old one if PriorityOverride
                    if matches!(self.conflict_strategy, ConflictStrategy::PriorityOverride) {
                        tools_map.remove(&registered_name);
                        sources_map.remove(&registered_name);
                        original_map.remove(&registered_name);
                    }
                }

                tools_map.insert(registered_name.clone(), tool);
                sources_map.insert(registered_name.clone(), upstream_name.clone());
                original_map.insert(registered_name.clone(), original_name);
            }
        }
    }

    /// Resolve tool name based on conflict strategy.
    fn resolve_tool_name(
        &self,
        original_name: &str,
        upstream_name: &str,
        tools_map: &HashMap<String, McpTool>,
        sources_map: &HashMap<String, String>,
    ) -> String {
        match self.conflict_strategy {
            ConflictStrategy::AutoPrefix => {
                if tools_map.contains_key(original_name) {
                    format!("{}:{}", upstream_name, original_name)
                } else {
                    original_name.to_string()
                }
            }
            ConflictStrategy::Reject | ConflictStrategy::PriorityOverride => {
                original_name.to_string()
            }
        }
    }

    /// Rebuild the catalog by querying all upstreams.
    ///
    /// # Arguments
    ///
    /// * `pool` - Upstream pool to query for tools
    ///
    /// # Errors
    ///
    /// Returns an error if tool discovery fails
    pub async fn rebuild_catalog(&self, pool: &UpstreamPool) -> Result<()> {
        // Clear existing catalog
        {
            let mut tools = self.tools.write().await;
            let mut sources = self.tool_sources.write().await;
            let mut original = self.original_names.write().await;
            tools.clear();
            sources.clear();
            original.clear();
        }

        // Query all upstreams
        let upstream_names = pool.list_upstreams().await;
        for upstream_name in upstream_names {
            if let Some(client) = pool.get_upstream(&upstream_name).await {
                let client_guard = client.lock().await;
                match client_guard.discover_tools().await {
                    Ok(tools) => {
                        self.add_tools(upstream_name, tools).await;
                    }
                    Err(e) => {
                        tracing::warn!(
                            upstream_name = %upstream_name,
                            error = %e,
                            "Failed to discover tools from upstream"
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl ToolCatalogTrait for DefaultToolCatalog {
    async fn get_all_tools(&self) -> Vec<McpTool> {
        let tools = self.tools.read().await;
        tools.values().cloned().collect()
    }

    async fn get_tool_source(&self, registered_name: &str) -> Option<String> {
        let sources = self.tool_sources.read().await;
        sources.get(registered_name).cloned()
    }

    async fn get_original_name(&self, registered_name: &str) -> Option<String> {
        let original = self.original_names.read().await;
        original.get(registered_name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_autoprefix_conflict_resolution() {
        let priorities = HashMap::new();
        let catalog = DefaultToolCatalog::new(ConflictStrategy::AutoPrefix, priorities);

        let tool1 = McpTool {
            name: "test_tool".to_string(),
            description: Some("Tool from upstream1".to_string()),
            input_schema: None,
        };

        let tool2 = McpTool {
            name: "test_tool".to_string(),
            description: Some("Tool from upstream2".to_string()),
            input_schema: None,
        };

        catalog.add_tools("upstream1".to_string(), vec![tool1]).await;
        catalog.add_tools("upstream2".to_string(), vec![tool2]).await;

        let tools = catalog.get_all_tools().await;
        assert_eq!(tools.len(), 2);

        // First tool should have original name, second should be prefixed
        let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
        assert!(tool_names.contains(&"test_tool".to_string()));
        assert!(tool_names.contains(&"upstream2:test_tool".to_string()));
    }

    #[tokio::test]
    async fn test_reject_conflict_resolution() {
        let priorities = HashMap::new();
        let catalog = DefaultToolCatalog::new(ConflictStrategy::Reject, priorities);

        let tool1 = McpTool {
            name: "test_tool".to_string(),
            description: Some("Tool from upstream1".to_string()),
            input_schema: None,
        };

        let tool2 = McpTool {
            name: "test_tool".to_string(),
            description: Some("Tool from upstream2".to_string()),
            input_schema: None,
        };

        catalog.add_tools("upstream1".to_string(), vec![tool1]).await;
        catalog.add_tools("upstream2".to_string(), vec![tool2]).await;

        let tools = catalog.get_all_tools().await;
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "test_tool");
        assert_eq!(
            catalog.get_tool_source("test_tool").await,
            Some("upstream1".to_string())
        );
    }

    #[tokio::test]
    async fn test_priority_override_conflict_resolution() {
        let mut priorities = HashMap::new();
        priorities.insert("upstream1".to_string(), 2); // Lower priority
        priorities.insert("upstream2".to_string(), 1); // Higher priority

        let catalog = DefaultToolCatalog::new(ConflictStrategy::PriorityOverride, priorities);

        let tool1 = McpTool {
            name: "test_tool".to_string(),
            description: Some("Tool from upstream1".to_string()),
            input_schema: None,
        };

        let tool2 = McpTool {
            name: "test_tool".to_string(),
            description: Some("Tool from upstream2".to_string()),
            input_schema: None,
        };

        catalog.add_tools("upstream1".to_string(), vec![tool1]).await;
        catalog.add_tools("upstream2".to_string(), vec![tool2]).await;

        let tools = catalog.get_all_tools().await;
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "test_tool");
        assert_eq!(
            catalog.get_tool_source("test_tool").await,
            Some("upstream2".to_string())
        );
    }

    #[tokio::test]
    async fn test_tool_source_tracking() {
        let priorities = HashMap::new();
        let catalog = DefaultToolCatalog::new(ConflictStrategy::AutoPrefix, priorities);

        let tool = McpTool {
            name: "test_tool".to_string(),
            description: None,
            input_schema: None,
        };

        catalog.add_tools("upstream1".to_string(), vec![tool]).await;

        assert_eq!(
            catalog.get_tool_source("test_tool").await,
            Some("upstream1".to_string())
        );
    }

    #[tokio::test]
    async fn test_get_original_name() {
        let priorities = HashMap::new();
        let catalog = DefaultToolCatalog::new(ConflictStrategy::AutoPrefix, priorities);

        let tool1 = McpTool {
            name: "test_tool".to_string(),
            description: None,
            input_schema: None,
        };

        let tool2 = McpTool {
            name: "test_tool".to_string(),
            description: None,
            input_schema: None,
        };

        catalog.add_tools("upstream1".to_string(), vec![tool1]).await;
        catalog.add_tools("upstream2".to_string(), vec![tool2]).await;

        assert_eq!(
            catalog.get_original_name("test_tool").await,
            Some("test_tool".to_string())
        );
        assert_eq!(
            catalog.get_original_name("upstream2:test_tool").await,
            Some("test_tool".to_string())
        );
    }
}
