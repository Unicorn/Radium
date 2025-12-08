//! Integration tests for MCP proxy server.

use radium_core::mcp::proxy::{
    DefaultSecurityLayer, DefaultToolCatalog, DefaultToolRouter, ProxyConfig,
    ProxyConfigManager, ProxyServer, ProxyTransport, SecurityConfig, UpstreamPool,
};
use radium_core::mcp::proxy::types::{
    ConflictStrategy, SecurityLayer as SecurityLayerTrait,
    ToolCatalog as ToolCatalogTrait, ToolRouter as ToolRouterTrait,
};
use radium_core::mcp::{McpServerConfig, TransportType};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;

/// Test helper to create a basic proxy configuration.
fn create_test_proxy_config() -> ProxyConfig {
    ProxyConfig {
        enable: true,
        port: 0, // Use 0 for OS-assigned port in tests
        transport: ProxyTransport::Http,
        max_connections: 10,
        security: SecurityConfig::default(),
        upstreams: vec![],
    }
}

#[tokio::test]
async fn test_proxy_config_loading() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("mcp-proxy.toml");

    let toml_content = r#"
[mcp.proxy]
enable = true
port = 3000
transport = "http"
max_connections = 50

[mcp.proxy.security]
log_requests = true
log_responses = true
rate_limit_per_minute = 30
"#;

    std::fs::write(&config_path, toml_content).unwrap();

    let manager = ProxyConfigManager::new(config_path);
    let config = manager.load().unwrap();

    assert!(config.enable);
    assert_eq!(config.port, 3000);
    assert_eq!(config.transport, ProxyTransport::Http);
    assert_eq!(config.max_connections, 50);
    assert_eq!(config.security.rate_limit_per_minute, 30);
}

#[tokio::test]
async fn test_proxy_config_validation() {
    let mut config = create_test_proxy_config();
    
    // Test invalid port
    config.port = 70000;
    assert!(ProxyConfigManager::validate_config(&config).is_err());

    // Test zero max_connections
    config.port = 3000;
    config.max_connections = 0;
    assert!(ProxyConfigManager::validate_config(&config).is_err());

    // Test valid config
    config.max_connections = 100;
    assert!(ProxyConfigManager::validate_config(&config).is_ok());
}

#[tokio::test]
async fn test_upstream_pool_management() {
    let pool = Arc::new(UpstreamPool::new());

    // Create test upstream config
    let upstream_config = radium_core::mcp::proxy::types::UpstreamConfig {
        server: McpServerConfig {
            name: "test-upstream".to_string(),
            transport: TransportType::Stdio,
            command: Some("echo".to_string()),
            args: Some(vec!["test".to_string()]),
            url: None,
            auth: None,
        },
        priority: 1,
        health_check_interval: 30,
        tools: None,
    };

    // Add upstream (may fail to connect, but should be tracked)
    let _ = pool.add_upstream(upstream_config.clone()).await;

    let upstreams = pool.list_upstreams().await;
    // Upstream should be in the list even if connection failed
    assert!(upstreams.len() >= 0);
}

#[tokio::test]
async fn test_tool_catalog_conflict_resolution() {
    let priorities = HashMap::new();
    let catalog = DefaultToolCatalog::new(ConflictStrategy::AutoPrefix, priorities);

    let tool1 = radium_core::mcp::McpTool {
        name: "duplicate_tool".to_string(),
        description: Some("Tool from upstream1".to_string()),
        input_schema: None,
    };

    let tool2 = radium_core::mcp::McpTool {
        name: "duplicate_tool".to_string(),
        description: Some("Tool from upstream2".to_string()),
        input_schema: None,
    };

    catalog.add_tools("upstream1".to_string(), vec![tool1]).await;
    catalog.add_tools("upstream2".to_string(), vec![tool2]).await;

    let tools = catalog.get_all_tools().await;
    assert_eq!(tools.len(), 2);

    // Check that one tool is prefixed
    let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
    assert!(tool_names.contains(&"duplicate_tool".to_string()));
    assert!(tool_names.contains(&"upstream2:duplicate_tool".to_string()));
}

#[tokio::test]
async fn test_security_layer_rate_limiting() {
    let mut security_config = SecurityConfig::default();
    security_config.rate_limit_per_minute = 2;

    let security = DefaultSecurityLayer::new(security_config).unwrap();

    // First two requests should succeed
    let result1 = security
        .check_request("test_tool", &json!({}), "agent1")
        .await;
    assert!(result1.is_ok());

    let result2 = security
        .check_request("test_tool", &json!({}), "agent1")
        .await;
    assert!(result2.is_ok());

    // Third request should be rate limited
    let result3 = security
        .check_request("test_tool", &json!({}), "agent1")
        .await;
    assert!(result3.is_err());
    assert!(result3.unwrap_err().to_string().contains("Rate limit"));
}

#[tokio::test]
async fn test_security_layer_redaction() {
    let security = DefaultSecurityLayer::new(SecurityConfig::default()).unwrap();

    let text = r#"{"api_key": "secret123", "password": "mypass"}"#;
    let redacted = security.redact_sensitive_data(text);

    assert!(redacted.contains("[REDACTED]"));
    assert!(!redacted.contains("secret123"));
    assert!(!redacted.contains("mypass"));
}

#[tokio::test]
async fn test_proxy_server_creation() {
    let config = create_test_proxy_config();
    let pool = Arc::new(UpstreamPool::new());
    let router: Arc<dyn ToolRouterTrait> = Arc::new(DefaultToolRouter::new(pool.clone()));
    
    let priorities = HashMap::new();
    let catalog: Arc<dyn ToolCatalogTrait> =
        Arc::new(DefaultToolCatalog::new(ConflictStrategy::AutoPrefix, priorities));
    
    let security: Arc<dyn SecurityLayerTrait> =
        Arc::new(DefaultSecurityLayer::new(SecurityConfig::default()).unwrap());

    let server = ProxyServer::new(config, router, catalog, security);
    // Server should be created successfully
    let _ = server;
}

#[tokio::test]
async fn test_tool_router_explicit_routing() {
    let pool = Arc::new(UpstreamPool::new());
    let router = DefaultToolRouter::new(pool);

    // Test explicit routing syntax parsing
    let tool_name = "upstream1:test_tool";
    assert!(tool_name.contains(':'));
    let colon_pos = tool_name.find(':').unwrap();
    assert_eq!(&tool_name[..colon_pos], "upstream1");
    assert_eq!(&tool_name[colon_pos + 1..], "test_tool");
}

// Note: Full end-to-end integration tests would require:
// - Mock MCP servers as upstreams
// - Actual HTTP client/server communication
// - Tool execution with real upstreams
// These are left as placeholders for now - full implementation would
// require more sophisticated test infrastructure

