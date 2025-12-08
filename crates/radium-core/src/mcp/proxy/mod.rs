//! MCP Proxy Server for Centralized Tool Orchestration
//!
//! This module implements an MCP proxy server that provides a single endpoint
//! for agents to access multiple upstream MCP servers. The proxy handles:
//!
//! - **Request Routing**: Routes tool calls to appropriate upstream servers
//! - **Load Balancing**: Distributes load across multiple upstreams
//! - **Failover**: Automatically fails over to backup servers on errors
//! - **Health Checking**: Monitors upstream server health and reconnects automatically
//! - **Security**: Centralized logging, rate limiting, and data redaction
//! - **Tool Catalog Aggregation**: Merges tools from all upstreams with conflict resolution
//!
//! # Architecture Overview
//!
//! The proxy server acts as both an MCP server (to agents) and an MCP client
//! (to upstream servers):
//!
//! ```
//! Agent -> [Proxy Server] -> Upstream MCP Servers
//!         (MCP Server)      (MCP Clients)
//! ```
//!
//! ## Component Responsibilities
//!
//! - **McpProxyServer**: Main server struct managing lifecycle and connections
//! - **UpstreamPool**: Manages connections to multiple upstream MCP servers
//! - **ToolRouter**: Routes tool calls to appropriate upstreams with load balancing
//! - **ToolCatalog**: Aggregates tools from all upstreams, handles conflicts
//! - **SecurityLayer**: Enforces security policies (rate limiting, logging, redaction)
//! - **HealthChecker**: Monitors upstream health and handles reconnection
//!
//! ## Data Flow for Tool Execution
//!
//! 1. Agent connects to proxy and sends `tools/call` request
//! 2. Proxy's SecurityLayer validates request (rate limiting, logging)
//! 3. ToolRouter determines which upstream should handle the tool
//! 4. Proxy forwards request to selected upstream via McpClient
//! 5. Upstream executes tool and returns result
//! 6. Proxy's SecurityLayer processes response (logging, redaction)
//! 7. Proxy returns result to agent
//!
//! ## Connection Lifecycle
//!
//! - **Agent Connections**: Accept connections via SSE/HTTP transport, handle MCP protocol
//! - **Upstream Connections**: Maintain persistent connections using McpClient
//! - **Health Monitoring**: Periodically check upstream health, reconnect on failure
//! - **Graceful Shutdown**: Close all connections cleanly on stop
//!
//! ## Error Handling Strategy
//!
//! - **Upstream Failures**: Automatically fail over to backup upstreams
//! - **Rate Limit Exceeded**: Return clear error to agent without forwarding
//! - **Invalid Requests**: Validate and return protocol errors
//! - **Connection Errors**: Log and mark upstream as unhealthy, retry with backoff
//!
//! ## Thread Safety
//!
//! All shared state uses `Arc<Mutex<>>` or `Arc<RwLock<>>` for thread-safe access:
//! - UpstreamPool: RwLock for concurrent reads, Mutex for writes
//! - ToolCatalog: RwLock for concurrent tool queries
//! - SecurityLayer: Internal synchronization for rate limiting
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use radium_core::mcp::proxy::{ProxyConfig, McpProxyServer};
//!
//! # async fn example() -> radium_core::mcp::Result<()> {
//! let config = ProxyConfig {
//!     enable: true,
//!     port: 3000,
//!     transport: ProxyTransport::Sse,
//!     max_connections: 100,
//!     security: SecurityConfig::default(),
//!     upstreams: vec![],
//! };
//!
//! let mut proxy = McpProxyServer::new(config).await?;
//! proxy.start().await?;
//! // Proxy is now accepting connections
//! # Ok(())
//! # }
//! ```

pub mod types;

// Re-export public types
pub use types::{
    ConflictStrategy, ConnectionState, McpProxyServer, ProxyConfig, ProxyTransport,
    SecurityConfig, ToolCatalog, ToolRouter, UpstreamConfig,
};

// Forward declarations for components that will be implemented in later tasks
// These will be uncommented as the components are implemented

// pub mod upstream_pool;
// pub use upstream_pool::UpstreamPool;

// pub mod router;
// pub use router::DefaultToolRouter;

// pub mod catalog;
// pub use catalog::DefaultToolCatalog;

// pub mod security;
// pub use security::DefaultSecurityLayer;

// pub mod health;
// pub use health::HealthChecker;

// pub mod server;
// pub use server::McpProxyServer;
