//! MCP protocol message types and JSON-RPC 2.0 handling.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// JSON-RPC 2.0 request message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (should be "2.0").
    pub jsonrpc: String,
    /// Method name.
    pub method: String,
    /// Method parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    /// Request ID.
    pub id: Option<Value>,
}

/// JSON-RPC 2.0 response message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version (should be "2.0").
    pub jsonrpc: String,
    /// Result (on success).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error (on failure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    /// Request ID (matches the request).
    pub id: Option<Value>,
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code.
    pub code: i32,
    /// Error message.
    pub message: String,
    /// Optional error data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// JSON-RPC 2.0 notification message (no response expected).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    /// JSON-RPC version (should be "2.0").
    pub jsonrpc: String,
    /// Method name.
    pub method: String,
    /// Method parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// MCP protocol message (can be request, response, or notification).
#[derive(Debug, Clone)]
pub enum McpMessage {
    /// Request message.
    Request(JsonRpcRequest),
    /// Response message.
    Response(JsonRpcResponse),
    /// Notification message.
    Notification(JsonRpcNotification),
}

impl McpMessage {
    /// Parse a message from JSON bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the message cannot be parsed.
    pub fn from_bytes(bytes: &[u8]) -> crate::mcp::Result<Self> {
        let value: Value = serde_json::from_slice(bytes)?;

        // Check if it's a request (has "method" and optionally "id")
        if value.get("method").is_some() {
            if value.get("id").is_some() {
                let request: JsonRpcRequest = serde_json::from_value(value)?;
                return Ok(McpMessage::Request(request));
            } else {
                let notification: JsonRpcNotification = serde_json::from_value(value)?;
                return Ok(McpMessage::Notification(notification));
            }
        }

        // Check if it's a response (has "result" or "error")
        if value.get("result").is_some() || value.get("error").is_some() {
            let response: JsonRpcResponse = serde_json::from_value(value)?;
            return Ok(McpMessage::Response(response));
        }

        Err(crate::mcp::McpError::Protocol(
            "Invalid JSON-RPC message format".to_string(),
        ))
    }

    /// Serialize the message to JSON bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the message cannot be serialized.
    pub fn to_bytes(&self) -> crate::mcp::Result<Vec<u8>> {
        let json = match self {
            McpMessage::Request(req) => serde_json::to_vec(req)?,
            McpMessage::Response(resp) => serde_json::to_vec(resp)?,
            McpMessage::Notification(notif) => serde_json::to_vec(notif)?,
        };
        Ok(json)
    }

    /// Get the message ID if it's a request or response.
    pub fn id(&self) -> Option<&Value> {
        match self {
            McpMessage::Request(req) => req.id.as_ref(),
            McpMessage::Response(resp) => resp.id.as_ref(),
            McpMessage::Notification(_) => None,
        }
    }
}

/// MCP initialize request parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    /// Protocol version.
    pub protocol_version: String,
    /// Client capabilities.
    pub capabilities: ClientCapabilities,
    /// Client information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_info: Option<ClientInfo>,
}

/// Client capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Experimental capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, Value>>,
}

/// Client information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Client name.
    pub name: String,
    /// Client version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// MCP initialize result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    /// Protocol version.
    pub protocol_version: String,
    /// Server capabilities.
    pub capabilities: ServerCapabilities,
    /// Server information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_info: Option<ServerInfo>,
}

/// Server capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Experimental capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, Value>>,
}

/// Server information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server name.
    pub name: String,
    /// Server version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "test/method".to_string(),
            params: Some(serde_json::json!({"key": "value"})),
            id: Some(serde_json::json!(1)),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("2.0"));
        assert!(json.contains("test/method"));
        assert!(json.contains("key"));
    }

    #[test]
    fn test_jsonrpc_response_serialization() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({"success": true})),
            error: None,
            id: Some(serde_json::json!(1)),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("2.0"));
        assert!(json.contains("success"));
    }

    #[test]
    fn test_jsonrpc_error_serialization() {
        let error = JsonRpcError {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: None,
        };

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("-32600"));
        assert!(json.contains("Invalid Request"));
    }

    #[test]
    fn test_mcp_message_from_bytes_request() {
        let json = r#"{"jsonrpc":"2.0","method":"test/method","params":{"key":"value"},"id":1}"#;
        let message = McpMessage::from_bytes(json.as_bytes()).unwrap();

        match message {
            McpMessage::Request(req) => {
                assert_eq!(req.method, "test/method");
                assert_eq!(req.id, Some(serde_json::json!(1)));
            }
            _ => panic!("Expected request"),
        }
    }

    #[test]
    fn test_mcp_message_from_bytes_response() {
        let json = r#"{"jsonrpc":"2.0","result":{"success":true},"id":1}"#;
        let message = McpMessage::from_bytes(json.as_bytes()).unwrap();

        match message {
            McpMessage::Response(resp) => {
                assert!(resp.result.is_some());
                assert_eq!(resp.id, Some(serde_json::json!(1)));
            }
            _ => panic!("Expected response"),
        }
    }

    #[test]
    fn test_mcp_message_from_bytes_notification() {
        let json = r#"{"jsonrpc":"2.0","method":"test/notify","params":{"key":"value"}}"#;
        let message = McpMessage::from_bytes(json.as_bytes()).unwrap();

        match message {
            McpMessage::Notification(notif) => {
                assert_eq!(notif.method, "test/notify");
            }
            _ => panic!("Expected notification"),
        }
    }

    #[test]
    fn test_mcp_message_to_bytes() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "test/method".to_string(),
            params: None,
            id: Some(serde_json::json!(1)),
        };
        let message = McpMessage::Request(request);

        let bytes = message.to_bytes().unwrap();
        let parsed = McpMessage::from_bytes(&bytes).unwrap();

        match parsed {
            McpMessage::Request(req) => {
                assert_eq!(req.method, "test/method");
            }
            _ => panic!("Expected request"),
        }
    }

    #[test]
    fn test_mcp_message_id() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "test/method".to_string(),
            params: None,
            id: Some(serde_json::json!(42)),
        };
        let message = McpMessage::Request(request);

        assert_eq!(message.id(), Some(&serde_json::json!(42)));
    }

    #[test]
    fn test_initialize_params() {
        let params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities {
                experimental: None,
            },
            client_info: Some(ClientInfo {
                name: "radium".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("2024-11-05"));
        assert!(json.contains("radium"));
    }

    #[test]
    fn test_initialize_params_serialization() {
        let params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities {
                experimental: None,
            },
            client_info: Some(ClientInfo {
                name: "radium".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        };

        let json = serde_json::to_string(&params).unwrap();
        let parsed: InitializeParams = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.protocol_version, "2024-11-05");
        assert_eq!(parsed.client_info.as_ref().unwrap().name, "radium");
    }

    #[test]
    fn test_initialize_result() {
        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                experimental: None,
            },
            server_info: Some(ServerInfo {
                name: "test-server".to_string(),
                version: Some("1.0.0".to_string()),
            }),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("2024-11-05"));
        assert!(json.contains("test-server"));
    }

    #[test]
    fn test_initialize_result_serialization() {
        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                experimental: None,
            },
            server_info: Some(ServerInfo {
                name: "test-server".to_string(),
                version: Some("1.0.0".to_string()),
            }),
        };

        let json = serde_json::to_string(&result).unwrap();
        let parsed: InitializeResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.protocol_version, "2024-11-05");
        assert_eq!(parsed.server_info.as_ref().unwrap().name, "test-server");
    }

    #[test]
    fn test_jsonrpc_request_without_params() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "test/method".to_string(),
            params: None,
            id: Some(serde_json::json!(1)),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("2.0"));
        assert!(json.contains("test/method"));
    }

    #[test]
    fn test_jsonrpc_response_with_error() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: -32600,
                message: "Invalid Request".to_string(),
                data: None,
            }),
            id: Some(serde_json::json!(1)),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("error"));
        assert!(json.contains("-32600"));
    }

    #[test]
    fn test_jsonrpc_error_with_data() {
        let error = JsonRpcError {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: Some(serde_json::json!({"details": "missing field"})),
        };

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("details"));
    }

    #[test]
    fn test_jsonrpc_notification_serialization() {
        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "test/notify".to_string(),
            params: Some(serde_json::json!({"key": "value"})),
        };

        let json = serde_json::to_string(&notification).unwrap();
        assert!(json.contains("2.0"));
        assert!(json.contains("test/notify"));
    }

    #[test]
    fn test_mcp_message_from_bytes_invalid() {
        let invalid_json = b"not valid json";
        let result = McpMessage::from_bytes(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_message_from_bytes_missing_fields() {
        let invalid_json = r#"{"jsonrpc":"2.0"}"#;
        let result = McpMessage::from_bytes(invalid_json.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_message_response_with_error() {
        let json = r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid Request"},"id":1}"#;
        let message = McpMessage::from_bytes(json.as_bytes()).unwrap();

        match message {
            McpMessage::Response(resp) => {
                assert!(resp.error.is_some());
                assert_eq!(resp.error.as_ref().unwrap().code, -32600);
            }
            _ => panic!("Expected response with error"),
        }
    }

    #[test]
    fn test_mcp_message_id_none() {
        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "test/notify".to_string(),
            params: None,
        };
        let message = McpMessage::Notification(notification);

        assert_eq!(message.id(), None);
    }

    #[test]
    fn test_mcp_message_response_id() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({"success": true})),
            error: None,
            id: Some(serde_json::json!("test-id")),
        };
        let message = McpMessage::Response(response);

        assert_eq!(message.id(), Some(&serde_json::json!("test-id")));
    }

    #[test]
    fn test_client_capabilities_serialization() {
        let mut experimental = HashMap::new();
        experimental.insert("feature1".to_string(), serde_json::json!(true));
        let caps = ClientCapabilities {
            experimental: Some(experimental),
        };

        let json = serde_json::to_string(&caps).unwrap();
        assert!(json.contains("experimental"));
    }

    #[test]
    fn test_server_capabilities_serialization() {
        let caps = ServerCapabilities {
            experimental: None,
        };

        let json = serde_json::to_string(&caps).unwrap();
        let parsed: ServerCapabilities = serde_json::from_str(&json).unwrap();
        assert!(parsed.experimental.is_none());
    }

    #[test]
    fn test_client_info_serialization() {
        let info = ClientInfo {
            name: "radium".to_string(),
            version: Some("0.1.0".to_string()),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("radium"));
        assert!(json.contains("0.1.0"));
    }

    #[test]
    fn test_client_info_without_version() {
        let info = ClientInfo {
            name: "radium".to_string(),
            version: None,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("radium"));
        // Version should be omitted when None
    }

    #[test]
    fn test_server_info_serialization() {
        let info = ServerInfo {
            name: "test-server".to_string(),
            version: Some("1.0.0".to_string()),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("test-server"));
        assert!(json.contains("1.0.0"));
    }
}

