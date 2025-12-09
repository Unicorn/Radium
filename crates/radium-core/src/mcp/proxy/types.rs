//! Core data structures for the MCP proxy server.
//!
//! This module defines the foundational types for the proxy server including
//! configuration structures, connection state, and trait definitions for
//! pluggable components.

use crate::mcp::{McpError, McpServerConfig, McpTool, Result, TransportType};
use crate::mcp::proxy::{DefaultSecurityLayer, DefaultToolCatalog, DefaultToolRouter, HealthChecker, ProxyServer, UpstreamPool};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Proxy server configuration.
///
/// Contains all settings for the proxy server including network settings,
/// security policies, and upstream server configurations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Whether the proxy server is enabled.
    #[serde(default = "default_enable")]
    pub enable: bool,
    /// Port to listen on for agent connections.
    #[serde(default = "default_port")]
    pub port: u16,
    /// Transport type for agent connections (SSE or HTTP).
    #[serde(default = "default_transport")]
    pub transport: ProxyTransport,
    /// Maximum number of concurrent agent connections.
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    /// Security configuration settings.
    #[serde(default)]
    pub security: SecurityConfig,
    /// Conflict resolution strategy for tool name conflicts.
    #[serde(default)]
    pub conflict_strategy: ConflictStrategy,
    /// List of upstream MCP servers to connect to.
    #[serde(default)]
    pub upstreams: Vec<UpstreamConfig>,
}

fn default_enable() -> bool {
    false
}

fn default_port() -> u16 {
    3000
}

fn default_transport() -> ProxyTransport {
    ProxyTransport::Sse
}

fn default_max_connections() -> u32 {
    100
}

/// Transport type for proxy agent connections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyTransport {
    /// Server-Sent Events transport.
    Sse,
    /// HTTP transport.
    Http,
}

impl From<ProxyTransport> for TransportType {
    fn from(transport: ProxyTransport) -> Self {
        match transport {
            ProxyTransport::Sse => TransportType::Sse,
            ProxyTransport::Http => TransportType::Http,
        }
    }
}

/// Security configuration for the proxy server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Whether to log incoming requests.
    #[serde(default = "default_true")]
    pub log_requests: bool,
    /// Whether to log outgoing responses.
    #[serde(default = "default_true")]
    pub log_responses: bool,
    /// Regex patterns for data redaction in logs.
    #[serde(default)]
    pub redact_patterns: Vec<String>,
    /// Rate limit per minute per agent/tool combination.
    #[serde(default = "default_rate_limit")]
    pub rate_limit_per_minute: u32,
}

fn default_true() -> bool {
    true
}

fn default_rate_limit() -> u32 {
    60
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            log_requests: true,
            log_responses: true,
            redact_patterns: vec![
                "api[_-]?key".to_string(),
                "password".to_string(),
                "token".to_string(),
            ],
            rate_limit_per_minute: 60,
        }
    }
}

/// Upstream server configuration.
///
/// Extends `McpServerConfig` with proxy-specific settings like priority,
/// health check interval, and optional tool filtering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamConfig {
    /// Base server configuration (name, transport, url, etc.).
    #[serde(flatten)]
    pub server: McpServerConfig,
    /// Upstream priority (lower number = higher priority).
    #[serde(default = "default_priority")]
    pub priority: u32,
    /// Health check interval in seconds.
    #[serde(default = "default_health_check_interval")]
    pub health_check_interval: u64,
    /// Optional list of tool names this upstream provides.
    /// If empty or None, all tools are assumed to be available.
    #[serde(default)]
    pub tools: Option<Vec<String>>,
}

fn default_priority() -> u32 {
    1
}

fn default_health_check_interval() -> u64 {
    30
}

/// Connection state for an upstream server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Connected and healthy.
    Connected,
    /// Disconnected.
    Disconnected,
    /// Connected but unhealthy.
    Unhealthy,
}

/// Trait for routing tool calls to appropriate upstream servers.
///
/// Implementations handle routing logic, load balancing, and failover
/// when multiple upstreams provide the same tool.
#[async_trait::async_trait]
pub trait ToolRouter: Send + Sync {
    /// Route a tool call to the appropriate upstream server.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - The tool name (may include explicit routing via `upstream:tool` syntax)
    /// * `arguments` - Tool execution arguments
    ///
    /// # Returns
    ///
    /// The tool execution result from the upstream server.
    async fn route_tool_call(
        &self,
        tool_name: &str,
        arguments: &Value,
    ) -> Result<crate::mcp::McpToolResult>;
}

/// Trait for security policy enforcement.
///
/// Handles request validation, rate limiting, logging, and response processing.
#[async_trait::async_trait]
pub trait SecurityLayer: Send + Sync {
    /// Check if a request should be allowed to proceed.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - The tool being called
    /// * `arguments` - Tool execution arguments
    /// * `agent_id` - Identifier for the requesting agent
    ///
    /// # Returns
    ///
    /// Ok(()) if the request is allowed, or an error if blocked (e.g., rate limit).
    async fn check_request(
        &self,
        tool_name: &str,
        arguments: &Value,
        agent_id: &str,
    ) -> Result<()>;

    /// Process a response (for logging, auditing, etc.).
    ///
    /// # Arguments
    ///
    /// * `tool_name` - The tool that was called
    /// * `result` - The tool execution result
    /// * `agent_id` - Identifier for the requesting agent
    async fn check_response(
        &self,
        tool_name: &str,
        result: &crate::mcp::McpToolResult,
        agent_id: &str,
    ) -> Result<()>;
}

/// Trait for aggregating and managing tool catalogs from multiple upstreams.
///
/// Handles tool discovery, conflict resolution, and catalog queries.
#[async_trait::async_trait]
pub trait ToolCatalog: Send + Sync {
    /// Get all aggregated tools from all upstream servers.
    ///
    /// # Returns
    ///
    /// A vector of all available tools (with conflict resolution applied).
    async fn get_all_tools(&self) -> Vec<McpTool>;

    /// Get the upstream server name that provides a specific tool.
    ///
    /// # Arguments
    ///
    /// * `registered_name` - The registered tool name (may be prefixed)
    ///
    /// # Returns
    ///
    /// The upstream server name if the tool exists, None otherwise.
    async fn get_tool_source(&self, registered_name: &str) -> Option<String>;

    /// Get the original tool name (before prefixing) for a registered tool.
    ///
    /// # Arguments
    ///
    /// * `registered_name` - The registered tool name (may be prefixed)
    ///
    /// # Returns
    ///
    /// The original tool name if found, None otherwise.
    async fn get_original_name(&self, registered_name: &str) -> Option<String>;
}

/// Conflict resolution strategy for tool name conflicts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictStrategy {
    /// Automatically prefix conflicting tools with upstream name.
    AutoPrefix,
    /// Reject duplicate tool names, keeping the first one.
    Reject,
    /// Keep the tool from the highest priority upstream.
    PriorityOverride,
}

impl Default for ConflictStrategy {
    fn default() -> Self {
        ConflictStrategy::AutoPrefix
    }
}

/// MCP Proxy Server main struct.
///
/// This is a wrapper that coordinates the ProxyServer with upstream pool
/// and health checking. The actual server implementation is in proxy::server.
#[derive(Debug)]
pub struct McpProxyServer {
    /// Proxy server configuration.
    pub config: ProxyConfig,
    /// The underlying proxy server.
    pub server: Option<ProxyServer>,
    /// Pool of upstream connections.
    pub pool: Arc<UpstreamPool>,
    /// Health checker for upstreams.
    pub health_checker: Arc<HealthChecker>,
    /// Shutdown signal sender.
    shutdown_tx: broadcast::Sender<()>,
    /// Shutdown signal receiver (kept to prevent channel from closing).
    _shutdown_rx: broadcast::Receiver<()>,
}

impl McpProxyServer {
    /// Create a new proxy server instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the proxy cannot be initialized.
    pub async fn new(config: ProxyConfig) -> Result<Self> {
        let pool = Arc::new(UpstreamPool::new());

        // Add upstreams to pool
        for upstream_config in &config.upstreams {
            if let Err(e) = pool.add_upstream(upstream_config.clone()).await {
                tracing::warn!(
                    upstream_name = %upstream_config.server.name,
                    error = %e,
                    "Failed to connect to upstream during proxy initialization"
                );
            }
        }

        // Build upstream priorities map for catalog
        let mut priorities = std::collections::HashMap::new();
        for upstream in &config.upstreams {
            priorities.insert(upstream.server.name.clone(), upstream.priority);
        }

        let router: Arc<dyn ToolRouter> = Arc::new(DefaultToolRouter::new(Arc::clone(&pool)));
        let catalog: Arc<dyn ToolCatalog> = Arc::new(DefaultToolCatalog::new(
            config.conflict_strategy,
            priorities,
        ));
        let security: Arc<dyn SecurityLayer> = Arc::new(DefaultSecurityLayer::new(config.security.clone())?);
        let health_checker = Arc::new(HealthChecker::new(Arc::clone(&pool)));

        // Rebuild catalog initially
        if let Ok(default_catalog) = Arc::try_unwrap(catalog.clone()) {
            default_catalog.rebuild_catalog(&pool).await?;
        } else {
            // Catalog is shared, need to use trait method or downcast
            // For now, we'll skip initial rebuild - it will happen on first tools/list
        }

        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

        Ok(Self {
            config,
            server: None,
            pool,
            health_checker,
            shutdown_tx,
            _shutdown_rx: shutdown_rx,
        })
    }

    /// Get a reference to the underlying ProxyServer.
    /// Returns None if the server hasn't been started yet.
    pub fn server(&self) -> Option<&ProxyServer> {
        self.server.as_ref()
    }

    /// Start the proxy server.
    ///
    /// Begins accepting agent connections and initializes upstream connections.
    ///
    /// # Errors
    ///
    /// Returns an error if the server cannot be started.
    pub async fn start(&mut self) -> Result<()> {
        if self.server.is_some() {
            return Err(McpError::config(
                "Proxy server already started",
                "The proxy server is already running. Call stop() before starting again.",
            ));
        }

        // Start health checks for all upstreams
        for upstream_config in &self.config.upstreams {
            self.health_checker
                .start_health_check(
                    upstream_config.server.name.clone(),
                    upstream_config.health_check_interval,
                )
                .await;
        }

        // Build router and catalog for server
        let mut priorities = std::collections::HashMap::new();
        for upstream in &self.config.upstreams {
            priorities.insert(upstream.server.name.clone(), upstream.priority);
        }

        let router: Arc<dyn ToolRouter> = Arc::new(DefaultToolRouter::new(Arc::clone(&self.pool)));
        let catalog: Arc<dyn ToolCatalog> = Arc::new(DefaultToolCatalog::new(
            self.config.conflict_strategy,
            priorities,
        ));
        let security: Arc<dyn SecurityLayer> = Arc::new(DefaultSecurityLayer::new(self.config.security.clone())?);

        let mut server = ProxyServer::new(
            self.config.clone(),
            router,
            catalog,
            security,
        );

        server.start().await?;
        self.server = Some(server);

        Ok(())
    }

    /// Stop the proxy server.
    ///
    /// Gracefully shuts down all connections and cleans up resources.
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown fails.
    pub async fn stop(&mut self) -> Result<()> {
        if self.server.is_none() {
            return Err(McpError::config(
                "Proxy server not running",
                "The proxy server is not running. Call start() before stopping.",
            ));
        }

        // Send shutdown signal
        let _ = self.shutdown_tx.send(());

        // Stop server
        if let Some(mut server) = self.server.take() {
            server.stop().await?;
        }

        // Stop all health check tasks
        self.health_checker.stop_all().await;

        // Disconnect all upstreams
        let upstream_names = self.pool.list_upstreams().await;
        for name in upstream_names {
            if let Err(e) = self.pool.remove_upstream(&name).await {
                tracing::warn!(
                    upstream_name = %name,
                    error = %e,
                    "Error disconnecting upstream during proxy shutdown"
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_config_defaults() {
        let config = ProxyConfig {
            enable: false,
            port: 3000,
            transport: ProxyTransport::Sse,
            max_connections: 100,
            security: SecurityConfig::default(),
            conflict_strategy: ConflictStrategy::default(),
            upstreams: Vec::new(),
        };

        assert!(!config.enable);
        assert_eq!(config.port, 3000);
        assert_eq!(config.max_connections, 100);
    }

    #[test]
    fn test_upstream_config_defaults() {
        let upstream = UpstreamConfig {
            server: McpServerConfig {
                name: "test".to_string(),
                transport: TransportType::Stdio,
                command: None,
                args: None,
                url: None,
                auth: None,
            },
            priority: 1,
            health_check_interval: 30,
            tools: None,
        };

        assert_eq!(upstream.priority, 1);
        assert_eq!(upstream.health_check_interval, 30);
    }

    #[test]
    fn test_connection_state() {
        assert_eq!(ConnectionState::Connected, ConnectionState::Connected);
        assert_ne!(ConnectionState::Connected, ConnectionState::Disconnected);
    }

    #[test]
    fn test_proxy_transport_conversion() {
        assert_eq!(
            TransportType::from(ProxyTransport::Sse),
            TransportType::Sse
        );
        assert_eq!(
            TransportType::from(ProxyTransport::Http),
            TransportType::Http
        );
    }

    #[test]
    fn test_conflict_strategy_default() {
        assert_eq!(ConflictStrategy::default(), ConflictStrategy::AutoPrefix);
    }
}
