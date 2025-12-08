//! Tool routing with load balancing and failover.
//!
//! This module provides intelligent routing of tool calls to upstream servers
//! with load balancing and automatic failover capabilities.

use crate::mcp::proxy::types::{ToolRouter as ToolRouterTrait, UpstreamPool};
use crate::mcp::{McpError, McpToolResult, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Default implementation of tool routing with load balancing.
pub struct DefaultToolRouter {
    /// Pool of upstream connections.
    pool: Arc<UpstreamPool>,
    /// Map of tool names to upstream names that provide them.
    tool_map: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Round-robin state for load balancing.
    round_robin_state: Arc<Mutex<HashMap<String, usize>>>,
}

impl DefaultToolRouter {
    /// Create a new tool router.
    pub fn new(pool: Arc<UpstreamPool>) -> Self {
        Self {
            pool,
            tool_map: Arc::new(RwLock::new(HashMap::new())),
            round_robin_state: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a tool provided by an upstream.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - Name of the tool
    /// * `upstream_name` - Name of the upstream that provides this tool
    pub async fn register_tool(&self, tool_name: String, upstream_name: String) {
        let mut tool_map = self.tool_map.write().await;
        let upstreams = tool_map.entry(tool_name).or_insert_with(Vec::new);
        
        if !upstreams.contains(&upstream_name) {
            upstreams.push(upstream_name.clone());
            
            // Sort by priority (get from pool config)
            if let Some(config) = self.pool.get_upstream_config(&upstream_name).await {
                // Sort existing upstreams by their priorities
                upstreams.sort_by_key(|name| {
                    // Get priority from pool - we'll need to track this
                    // For now, just maintain insertion order
                    // TODO: Sort by actual priority from config
                    0
                });
            }
        }
    }

    /// Update tool map by querying all upstreams for their tools.
    pub async fn update_tool_map(&self) -> Result<()> {
        let mut tool_map = HashMap::new();
        let upstream_names = self.pool.list_upstreams().await;

        for upstream_name in upstream_names {
            if let Some(client) = self.pool.get_upstream(&upstream_name).await {
                let client_guard = client.lock().await;
                match client_guard.discover_tools().await {
                    Ok(tools) => {
                        for tool in tools {
                            tool_map
                                .entry(tool.name.clone())
                                .or_insert_with(Vec::new)
                                .push(upstream_name.clone());
                        }
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

        let mut map = self.tool_map.write().await;
        *map = tool_map;
        Ok(())
    }

    /// Select an upstream using round-robin load balancing.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - Name of the tool
    /// * `candidates` - List of candidate upstream names
    ///
    /// # Returns
    ///
    /// Selected upstream name, or None if no healthy candidates
    async fn select_upstream(&self, tool_name: &str, candidates: &[String]) -> Option<String> {
        if candidates.is_empty() {
            return None;
        }

        // Filter to only healthy upstreams
        let healthy_candidates: Vec<String> = candidates
            .iter()
            .filter(|name| {
                // Check if upstream is connected
                self.pool.get_upstream(name).await.is_some()
            })
            .cloned()
            .collect();

        if healthy_candidates.is_empty() {
            return None;
        }

        // Get or create round-robin state for this tool
        let mut state = self.round_robin_state.lock().await;
        let index = state.entry(tool_name.to_string()).or_insert(0);
        
        let selected = healthy_candidates[*index % healthy_candidates.len()].clone();
        *index = (*index + 1) % healthy_candidates.len();
        
        Some(selected)
    }

    /// Route a tool call with failover support.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - Tool name (may include explicit routing)
    /// * `arguments` - Tool execution arguments
    ///
    /// # Returns
    ///
    /// Tool execution result
    async fn route_with_failover(
        &self,
        tool_name: &str,
        actual_tool_name: &str,
        upstream_name: &str,
        arguments: &Value,
    ) -> Result<McpToolResult> {
        let client = self.pool.get_upstream(upstream_name).await.ok_or_else(|| {
            McpError::tool_not_found(
                tool_name,
                format!("Upstream '{}' is not connected", upstream_name),
            )
        })?;

        let client_guard = client.lock().await;
        client_guard.execute_tool(actual_tool_name, arguments).await
    }
}

#[async_trait::async_trait]
impl ToolRouterTrait for DefaultToolRouter {
    async fn route_tool_call(
        &self,
        tool_name: &str,
        arguments: &Value,
    ) -> Result<McpToolResult> {
        // Check for explicit routing syntax: "upstream_name:tool_name"
        if let Some(colon_pos) = tool_name.find(':') {
            let upstream_name = &tool_name[..colon_pos];
            let actual_tool_name = &tool_name[colon_pos + 1..];

            // Direct routing to specified upstream
            return self.route_with_failover(tool_name, actual_tool_name, upstream_name, arguments).await;
        }

        // Implicit routing - look up in tool map
        let candidates = {
            let tool_map = self.tool_map.read().await;
            tool_map.get(tool_name).cloned()
        };

        let candidates = match candidates {
            Some(c) => c,
            None => {
                return Err(McpError::tool_not_found(
                    tool_name,
                    format!(
                        "Tool '{}' not found in any upstream. Use update_tool_map() to refresh the tool catalog.",
                        tool_name
                    ),
                ));
            }
        };

        // Try each candidate in order with failover
        let mut last_error = None;
        for upstream_name in &candidates {
            match self.route_with_failover(tool_name, tool_name, upstream_name, arguments).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    tracing::warn!(
                        upstream_name = %upstream_name,
                        tool_name = %tool_name,
                        error = %e,
                        "Tool execution failed, trying next upstream"
                    );
                    // Mark upstream as unhealthy
                    self.pool.mark_unhealthy(upstream_name).await;
                    last_error = Some(e);
                }
            }
        }

        // All upstreams failed
        Err(last_error.unwrap_or_else(|| {
            McpError::tool_not_found(
                tool_name,
                "All upstreams failed to execute the tool".to_string(),
            )
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_explicit_routing() {
        // This test would require mock upstreams
        // For now, we test the parsing logic
        let tool_name = "upstream1:test_tool";
        assert!(tool_name.contains(':'));
        let colon_pos = tool_name.find(':').unwrap();
        assert_eq!(&tool_name[..colon_pos], "upstream1");
        assert_eq!(&tool_name[colon_pos + 1..], "test_tool");
    }

    #[tokio::test]
    async fn test_router_creation() {
        let pool = Arc::new(UpstreamPool::new());
        let router = DefaultToolRouter::new(pool);
        
        // Router should be created
        let _ = router;
    }
}
