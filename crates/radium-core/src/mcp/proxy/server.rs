//! MCP protocol server for agent connections.
//!
//! This module implements the server side of the MCP protocol, accepting
//! connections from agents and routing requests to upstream servers.

use crate::mcp::messages::{JsonRpcRequest, JsonRpcResponse, JsonRpcError};
use crate::mcp::proxy::types::{
    ProxyConfig, SecurityLayer as SecurityLayerTrait,
    ToolCatalog as ToolCatalogTrait, ToolRouter as ToolRouterTrait,
};
use crate::mcp::{McpError, Result};
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde_json::{json, Value};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use uuid::Uuid;

/// MCP proxy server that handles agent connections.
pub struct ProxyServer {
    /// Proxy configuration.
    config: ProxyConfig,
    /// Tool router for routing requests.
    router: Arc<dyn ToolRouterTrait>,
    /// Tool catalog for aggregating tools.
    catalog: Arc<dyn ToolCatalogTrait>,
    /// Security layer for policy enforcement.
    security: Arc<dyn SecurityLayerTrait>,
    /// TCP listener (when server is running).
    listener: Option<TcpListener>,
    /// Shutdown signal sender.
    shutdown_tx: Option<broadcast::Sender<()>>,
    /// Active connection tasks.
    connection_tasks: Arc<tokio::sync::Mutex<Vec<JoinHandle<()>>>>,
}

impl std::fmt::Debug for ProxyServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProxyServer")
            .field("config", &self.config)
            .field("router", &"<ToolRouterTrait>")
            .field("catalog", &"<ToolCatalogTrait>")
            .field("security", &"<SecurityLayerTrait>")
            .field("listener", &self.listener.is_some())
            .field("shutdown_tx", &self.shutdown_tx.is_some())
            .field("connection_tasks", &"<Mutex<Vec<JoinHandle>>>")
            .finish()
    }
}

impl ProxyServer {
    /// Create a new proxy server.
    ///
    /// # Arguments
    ///
    /// * `config` - Proxy configuration
    /// * `router` - Tool router instance
    /// * `catalog` - Tool catalog instance
    /// * `security` - Security layer instance
    pub fn new(
        config: ProxyConfig,
        router: Arc<dyn ToolRouterTrait>,
        catalog: Arc<dyn ToolCatalogTrait>,
        security: Arc<dyn SecurityLayerTrait>,
    ) -> Self {
        let (shutdown_tx, _) = broadcast::channel(16);
        Self {
            config,
            router,
            catalog,
            security,
            listener: None,
            shutdown_tx: Some(shutdown_tx),
            connection_tasks: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    /// Start the proxy server.
    ///
    /// # Errors
    ///
    /// Returns an error if the server cannot bind to the configured port.
    pub async fn start(&mut self) -> Result<()> {
        let addr = SocketAddr::from(([0, 0, 0, 0], self.config.port));
        let listener = TcpListener::bind(addr).await.map_err(|e| {
            McpError::connection(
                format!("Failed to bind to {}: {}", addr, e),
                format!(
                    "Cannot bind to port {}. Common causes:\n  - Port already in use\n  - Insufficient permissions\n  - Invalid port number\n\nTry:\n  - Check if another process is using port {}\n  - Use a different port in your config\n  - Run with appropriate permissions if using ports < 1024",
                    self.config.port, self.config.port
                ),
            )
        })?;

        tracing::info!(port = self.config.port, "MCP proxy server started");

        let mut shutdown_rx = self.shutdown_tx.as_ref().unwrap().subscribe();
        let router = Arc::clone(&self.router);
        let catalog = Arc::clone(&self.catalog);
        let security = Arc::clone(&self.security);
        let connection_tasks = Arc::clone(&self.connection_tasks);
        let _listener_addr = listener.local_addr().unwrap();

        // Spawn accept loop
        let accept_handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, addr)) => {
                                tracing::debug!(%addr, "New agent connection");
                                let router_clone = Arc::clone(&router);
                                let catalog_clone = Arc::clone(&catalog);
                                let security_clone = Arc::clone(&security);
                                let task = tokio::spawn(async move {
                                    if let Err(e) = Self::handle_connection(
                                        stream,
                                        router_clone,
                                        catalog_clone,
                                        security_clone,
                                    ).await {
                                        tracing::warn!(%addr, error = %e, "Connection handler error");
                                    }
                                });
                                let mut tasks = connection_tasks.lock().await;
                                tasks.push(task);
                            }
                            Err(e) => {
                                tracing::error!(error = %e, "Error accepting connection");
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        tracing::info!("Shutdown signal received, stopping accept loop");
                        break;
                    }
                }
            }
        });

        // Note: listener is moved into the async block above
        // Store accept handle for cleanup
        let mut tasks = self.connection_tasks.lock().await;
        tasks.push(accept_handle);

        Ok(())
    }

    /// Handle a single agent connection using HTTP/1.1.
    async fn handle_connection(
        stream: tokio::net::TcpStream,
        router: Arc<dyn ToolRouterTrait>,
        catalog: Arc<dyn ToolCatalogTrait>,
        security: Arc<dyn SecurityLayerTrait>,
    ) -> Result<()> {
        let io = TokioIo::new(stream);
        let router_clone = Arc::clone(&router);
        let catalog_clone = Arc::clone(&catalog);
        let security_clone = Arc::clone(&security);

        let service = service_fn(move |req: Request<hyper::body::Incoming>| {
            let router = Arc::clone(&router_clone);
            let catalog = Arc::clone(&catalog_clone);
            let security = Arc::clone(&security_clone);
            Self::handle_http_request(req, router, catalog, security)
        });

        if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
            tracing::error!(error = %e, "Error serving connection");
        }

        Ok(())
    }

    /// Handle an HTTP request.
    async fn handle_http_request(
        req: Request<hyper::body::Incoming>,
        router: Arc<dyn ToolRouterTrait>,
        catalog: Arc<dyn ToolCatalogTrait>,
        security: Arc<dyn SecurityLayerTrait>,
    ) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
        // Generate agent ID from request (or use connection ID)
        let agent_id = Uuid::new_v4().to_string();

        // Only handle POST requests for JSON-RPC
        if req.method() != Method::POST {
            let mut response = Response::new(Full::new(Bytes::from(
                "Method not allowed. Use POST for JSON-RPC requests.",
            )));
            *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
            return Ok(response);
        }

        // Read request body
        let body = req.into_body();
        let bytes = body.collect().await.map(|b| b.to_bytes()).unwrap_or_default();
        
        // Parse JSON-RPC request
        let request: JsonRpcRequest = match serde_json::from_slice(&bytes) {
            Ok(req) => req,
            Err(e) => {
                let error_response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: "Parse error".to_string(),
                        data: Some(json!({"error": e.to_string()})),
                    }),
                    id: None,
                };
                let response_bytes = serde_json::to_vec(&error_response).unwrap_or_default();
                let mut response = Response::new(Full::new(Bytes::from(response_bytes)));
                *response.status_mut() = StatusCode::BAD_REQUEST;
                response.headers_mut().insert(
                    hyper::header::CONTENT_TYPE,
                    "application/json".parse().unwrap(),
                );
                return Ok(response);
            }
        };

        // Handle request
        let response = Self::handle_request(
            &request,
            &agent_id,
            &*router,
            &*catalog,
            &*security,
        ).await;

        // Serialize response
        let response_bytes = serde_json::to_vec(&response).unwrap_or_default();
        let mut http_response = Response::new(Full::new(Bytes::from(response_bytes)));
        http_response.headers_mut().insert(
            hyper::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        *http_response.status_mut() = StatusCode::OK;

        Ok(http_response)
    }

    /// Handle a JSON-RPC request.
    async fn handle_request(
        request: &JsonRpcRequest,
        agent_id: &str,
        router: &dyn ToolRouterTrait,
        catalog: &dyn ToolCatalogTrait,
        security: &dyn SecurityLayerTrait,
    ) -> JsonRpcResponse {
        let request_id = request.id.clone();

        let result = match request.method.as_str() {
            "initialize" => {
                // Handle initialization
                let init_result = json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "radium-mcp-proxy",
                        "version": "0.1.0"
                    }
                });
                Ok(Some(init_result))
            }
            "tools/list" => {
                let tools = catalog.get_all_tools().await;
                let tools_json: Vec<Value> = tools
                    .iter()
                    .map(|tool| {
                        json!({
                            "name": tool.name,
                            "description": tool.description,
                            "inputSchema": tool.input_schema
                        })
                    })
                    .collect();
                Ok(Some(json!({"tools": tools_json})))
            }
            "tools/call" => {
                let params = request.params.as_ref().and_then(|p| p.as_object());
                let tool_name = match params
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                {
                    Some(name) => name,
                    None => {
                        return JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32602,
                                message: "Missing 'name' parameter in tools/call".to_string(),
                                data: Some(json!({
                                    "hint": "The tools/call request must include a 'name' parameter specifying the tool to execute."
                                })),
                            }),
                            id: request_id,
                        };
                    }
                };
                let default_args = json!({});
                let arguments = params
                    .and_then(|p| p.get("arguments"))
                    .unwrap_or(&default_args);

                // Check security
                match security.check_request(tool_name, arguments, agent_id).await {
                    Ok(_) => {}
                    Err(e) => {
                        return JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32000,
                                message: e.to_string(),
                                data: None,
                            }),
                            id: request_id,
                        };
                    }
                }

                // Route and execute
                match router.route_tool_call(tool_name, arguments).await {
                    Ok(result) => {
                        // Log response
                        let _ = security.check_response(tool_name, &result, agent_id).await;
                        
                        let content_json: Vec<Value> = result
                            .content
                            .iter()
                            .map(|c| match c {
                                crate::mcp::McpContent::Text { text } => {
                                    json!({"type": "text", "text": text})
                                }
                                crate::mcp::McpContent::Image { data, mime_type } => {
                                    json!({"type": "image", "data": data, "mimeType": mime_type})
                                }
                                crate::mcp::McpContent::Audio { data, mime_type } => {
                                    json!({"type": "audio", "data": data, "mimeType": mime_type})
                                }
                            })
                            .collect();
                        
                        Ok(Some(json!({
                            "content": content_json,
                            "isError": result.is_error
                        })))
                    }
                    Err(e) => {
                        Err(JsonRpcError {
                            code: -32603,
                            message: format!("Tool execution failed: {}", e),
                            data: Some(json!({"error": e.to_string()})),
                        })
                    }
                }
            }
            "prompts/list" => {
                // TODO: Implement prompts aggregation (similar to tools/list)
                Ok(Some(json!({"prompts": []})))
            }
            "prompts/get" => {
                // TODO: Implement prompt retrieval
                Err(JsonRpcError {
                    code: -32601,
                    message: "Method not implemented".to_string(),
                    data: None,
                })
            }
            _ => {
                Err(JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                })
            }
        };

        match result {
            Ok(Some(result_value)) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(result_value),
                error: None,
                id: request_id,
            },
            Ok(None) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: None,
                id: request_id,
            },
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(error),
                id: request_id,
            },
        }
    }

    /// Stop the proxy server.
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown fails.
    pub async fn stop(&mut self) -> Result<()> {
        // Send shutdown signal
        if let Some(shutdown_tx) = &self.shutdown_tx {
            let _ = shutdown_tx.send(());
        }

        // Wait for connection tasks to complete (with timeout)
        let tasks = std::mem::take(&mut *self.connection_tasks.lock().await);
        for task in tasks {
            task.abort();
        }

        // Close listener
        self.listener = None;

        tracing::info!("MCP proxy server stopped");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::proxy::{ConflictStrategy, ProxyTransport, SecurityConfig};

    #[tokio::test]
    async fn test_proxy_server_creation() {
        let config = ProxyConfig {
            enable: true,
            port: 0, // Use 0 for test (OS will assign port)
            transport: ProxyTransport::Http,
            max_connections: 100,
            security: SecurityConfig::default(),
            upstreams: vec![],
            conflict_strategy: ConflictStrategy::AutoPrefix,
        };

        let pool = Arc::new(UpstreamPool::new());
        let router: Arc<dyn ToolRouterTrait> = Arc::new(DefaultToolRouter::new(pool.clone()));
        
        let priorities = HashMap::new();
        let catalog: Arc<dyn ToolCatalogTrait> = Arc::new(DefaultToolCatalog::new(
            ConflictStrategy::AutoPrefix,
            priorities,
        ));
        
        let security: Arc<dyn SecurityLayerTrait> = Arc::new(
            DefaultSecurityLayer::new(SecurityConfig::default()).unwrap(),
        );

        let server = ProxyServer::new(config, router, catalog, security);
        // Server should be created
        let _ = server;
    }
}
