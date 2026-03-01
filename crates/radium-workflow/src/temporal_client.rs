//! Temporal gRPC client for gateway workflow management.
//!
//! Wraps the Temporal gRPC API to start, signal, query, and terminate gateway
//! workflows. The client uses lazy connection establishment -- the gRPC channel
//! is created on first use and reused for subsequent calls.
//!
//! # Current Status
//!
//! The methods are implemented as stubs that log their operations and return
//! success. The actual Temporal proto integration will be added when the proto
//! files are available. This gives downstream tasks (gateway handler, lifecycle
//! wiring) a stable interface to code against.

use serde::Serialize;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for connecting to the Temporal gRPC server.
#[derive(Debug, Clone)]
pub struct TemporalConfig {
    /// gRPC address of the Temporal frontend service (e.g. `http://localhost:7233`).
    pub address: String,
    /// Temporal namespace to use for all workflow operations.
    pub namespace: String,
}

impl TemporalConfig {
    /// Build configuration from environment variables.
    ///
    /// | Variable              | Default                    |
    /// |-----------------------|----------------------------|
    /// | `TEMPORAL_ADDRESS`    | `http://localhost:7233`    |
    /// | `TEMPORAL_NAMESPACE`  | `default`                  |
    pub fn from_env() -> Self {
        Self {
            address: std::env::var("TEMPORAL_ADDRESS")
                .unwrap_or_else(|_| "http://localhost:7233".to_string()),
            namespace: std::env::var("TEMPORAL_NAMESPACE")
                .unwrap_or_else(|_| "default".to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur when interacting with the Temporal gRPC API.
#[derive(Debug)]
pub enum TemporalError {
    /// Failed to establish a gRPC connection.
    Connection(String),
    /// A gRPC call returned an error status.
    Rpc(String),
    /// Failed to serialize or deserialize a payload.
    Serialization(String),
}

impl std::fmt::Display for TemporalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connection(msg) => write!(f, "Temporal connection error: {msg}"),
            Self::Rpc(msg) => write!(f, "Temporal RPC error: {msg}"),
            Self::Serialization(msg) => write!(f, "Temporal serialization error: {msg}"),
        }
    }
}

impl std::error::Error for TemporalError {}

// ---------------------------------------------------------------------------
// Signal payload
// ---------------------------------------------------------------------------

/// Payload sent when signaling a gateway workflow with an incoming request.
#[derive(Debug, Serialize)]
pub struct SignalPayload {
    /// The actual request data to deliver to the workflow.
    pub data: serde_json::Value,
    /// ISO 8601 timestamp of when the signal was received.
    pub received_at: String,
    /// Unique identifier for this request, used for deduplication and tracing.
    pub request_id: String,
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Generate the deterministic workflow ID for a gateway interface.
///
/// All gateway workflows use the pattern `gateway-{interface_id}` so that
/// we can signal or terminate a specific workflow by its interface ID alone.
pub fn gateway_workflow_id(interface_id: &str) -> String {
    format!("gateway-{interface_id}")
}

/// Generate the task queue name for a gateway interface.
///
/// Each gateway interface gets its own task queue so that its worker(s) only
/// pick up work for that specific interface.
pub fn gateway_task_queue(interface_id: &str) -> String {
    format!("gateway-{interface_id}-queue")
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// gRPC client for the Temporal workflow service.
///
/// Provides methods to start, signal, query, and terminate gateway workflows.
/// The underlying gRPC channel is lazily established on first use.
pub struct TemporalClient {
    config: TemporalConfig,
    channel: Option<tonic::transport::Channel>,
}

impl Clone for TemporalClient {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            channel: self.channel.clone(),
        }
    }
}

impl std::fmt::Debug for TemporalClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TemporalClient")
            .field("address", &self.config.address)
            .field("namespace", &self.config.namespace)
            .field("connected", &self.channel.is_some())
            .finish()
    }
}

impl TemporalClient {
    /// Create a new client with the given configuration.
    ///
    /// The gRPC channel is **not** established until the first method call.
    pub fn new(config: &TemporalConfig) -> Self {
        Self {
            config: config.clone(),
            channel: None,
        }
    }

    /// Ensure we have an active gRPC channel, creating one if necessary.
    ///
    /// The channel is cached for subsequent calls. Returns a clone of the
    /// channel (tonic channels are cheap to clone).
    pub async fn connect(&mut self) -> Result<tonic::transport::Channel, TemporalError> {
        if let Some(ref channel) = self.channel {
            return Ok(channel.clone());
        }

        tracing::info!(
            address = %self.config.address,
            namespace = %self.config.namespace,
            "Establishing Temporal gRPC connection"
        );

        let channel = tonic::transport::Channel::from_shared(self.config.address.clone())
            .map_err(|e| TemporalError::Connection(format!("Invalid Temporal address: {e}")))?
            .connect_lazy();

        self.channel = Some(channel.clone());
        Ok(channel)
    }

    /// Start a new gateway workflow for the given interface.
    ///
    /// Returns the workflow run ID on success.
    ///
    /// # TODO
    ///
    /// When Temporal proto files are integrated, this will:
    /// 1. Build a `StartWorkflowExecutionRequest` with the gateway workflow type
    /// 2. Set the workflow ID to `gateway-{interface_id}`
    /// 3. Set the task queue to `gateway-{interface_id}-queue`
    /// 4. Call `workflow_service_client.start_workflow_execution(request)`
    /// 5. Return the `run_id` from the response
    pub async fn start_gateway_workflow(
        &mut self,
        interface_id: &str,
        task_queue: &str,
    ) -> Result<String, TemporalError> {
        let _channel = self.connect().await?;

        let workflow_id = gateway_workflow_id(interface_id);

        tracing::info!(
            workflow_id = %workflow_id,
            task_queue = %task_queue,
            namespace = %self.config.namespace,
            "Starting gateway workflow (stub)"
        );

        // TODO: Replace with actual gRPC call:
        //
        // let mut client = WorkflowServiceClient::new(channel);
        // let request = StartWorkflowExecutionRequest {
        //     namespace: self.config.namespace.clone(),
        //     workflow_id: workflow_id.clone(),
        //     workflow_type: Some(WorkflowType { name: "GatewayWorkflow".into() }),
        //     task_queue: Some(TaskQueue { name: task_queue.into(), kind: 0 }),
        //     ..Default::default()
        // };
        // let response = client.start_workflow_execution(request).await
        //     .map_err(|e| TemporalError::Rpc(e.message().to_string()))?;
        // Ok(response.into_inner().run_id)

        Ok(workflow_id)
    }

    /// Signal a running gateway workflow with an incoming request payload.
    ///
    /// The signal delivers the request data to the workflow so it can process
    /// it. The workflow buffers signals and processes them in order.
    ///
    /// # TODO
    ///
    /// When Temporal proto files are integrated, this will:
    /// 1. Serialize the `SignalPayload` to a Temporal `Payload`
    /// 2. Build a `SignalWorkflowExecutionRequest`
    /// 3. Call `workflow_service_client.signal_workflow_execution(request)`
    pub async fn signal_gateway_workflow(
        &mut self,
        interface_id: &str,
        payload: &SignalPayload,
    ) -> Result<(), TemporalError> {
        let _channel = self.connect().await?;

        let workflow_id = gateway_workflow_id(interface_id);

        tracing::info!(
            workflow_id = %workflow_id,
            request_id = %payload.request_id,
            namespace = %self.config.namespace,
            "Signaling gateway workflow (stub)"
        );

        // TODO: Replace with actual gRPC call:
        //
        // let mut client = WorkflowServiceClient::new(channel);
        // let payload_bytes = serde_json::to_vec(payload)
        //     .map_err(|e| TemporalError::Serialization(e.to_string()))?;
        // let request = SignalWorkflowExecutionRequest {
        //     namespace: self.config.namespace.clone(),
        //     workflow_execution: Some(WorkflowExecution {
        //         workflow_id: workflow_id.clone(),
        //         run_id: String::new(),
        //     }),
        //     signal_name: "gateway_request".into(),
        //     input: Some(Payloads {
        //         payloads: vec![Payload { data: payload_bytes, ..Default::default() }],
        //     }),
        //     ..Default::default()
        // };
        // client.signal_workflow_execution(request).await
        //     .map_err(|e| TemporalError::Rpc(e.message().to_string()))?;

        Ok(())
    }

    /// Terminate a running gateway workflow.
    ///
    /// Used during undeploy to stop the gateway from accepting new requests.
    ///
    /// # TODO
    ///
    /// When Temporal proto files are integrated, this will:
    /// 1. Build a `TerminateWorkflowExecutionRequest`
    /// 2. Call `workflow_service_client.terminate_workflow_execution(request)`
    pub async fn terminate_gateway_workflow(
        &mut self,
        interface_id: &str,
    ) -> Result<(), TemporalError> {
        let _channel = self.connect().await?;

        let workflow_id = gateway_workflow_id(interface_id);

        tracing::info!(
            workflow_id = %workflow_id,
            namespace = %self.config.namespace,
            "Terminating gateway workflow (stub)"
        );

        // TODO: Replace with actual gRPC call:
        //
        // let mut client = WorkflowServiceClient::new(channel);
        // let request = TerminateWorkflowExecutionRequest {
        //     namespace: self.config.namespace.clone(),
        //     workflow_execution: Some(WorkflowExecution {
        //         workflow_id: workflow_id.clone(),
        //         run_id: String::new(),
        //     }),
        //     reason: "Gateway undeployed".into(),
        //     ..Default::default()
        // };
        // client.terminate_workflow_execution(request).await
        //     .map_err(|e| TemporalError::Rpc(e.message().to_string()))?;

        Ok(())
    }

    /// Query the buffer depth of a running gateway workflow.
    ///
    /// Returns the number of signals currently queued in the workflow's buffer.
    /// This is useful for monitoring and backpressure decisions.
    ///
    /// # TODO
    ///
    /// When Temporal proto files are integrated, this will:
    /// 1. Build a `QueryWorkflowRequest` with query type `"buffer_depth"`
    /// 2. Call `workflow_service_client.query_workflow(request)`
    /// 3. Deserialize the response payload as `u64`
    pub async fn query_gateway_buffer_depth(
        &mut self,
        interface_id: &str,
    ) -> Result<u64, TemporalError> {
        let _channel = self.connect().await?;

        let workflow_id = gateway_workflow_id(interface_id);

        tracing::info!(
            workflow_id = %workflow_id,
            namespace = %self.config.namespace,
            "Querying gateway buffer depth (stub)"
        );

        // TODO: Replace with actual gRPC call:
        //
        // let mut client = WorkflowServiceClient::new(channel);
        // let request = QueryWorkflowRequest {
        //     namespace: self.config.namespace.clone(),
        //     execution: Some(WorkflowExecution {
        //         workflow_id: workflow_id.clone(),
        //         run_id: String::new(),
        //     }),
        //     query: Some(WorkflowQuery {
        //         query_type: "buffer_depth".into(),
        //         ..Default::default()
        //     }),
        //     ..Default::default()
        // };
        // let response = client.query_workflow(request).await
        //     .map_err(|e| TemporalError::Rpc(e.message().to_string()))?;
        // // Deserialize the query result payload as u64
        // let depth = deserialize_query_result(&response.into_inner())?;
        // Ok(depth)

        Ok(0)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial(temporal_env)]
    fn test_temporal_config_defaults() {
        std::env::remove_var("TEMPORAL_ADDRESS");
        std::env::remove_var("TEMPORAL_NAMESPACE");

        let config = TemporalConfig::from_env();
        assert_eq!(config.address, "http://localhost:7233");
        assert_eq!(config.namespace, "default");
    }

    #[test]
    #[serial(temporal_env)]
    fn test_temporal_config_custom() {
        std::env::set_var("TEMPORAL_ADDRESS", "http://temporal.internal:7233");
        std::env::set_var("TEMPORAL_NAMESPACE", "production");

        let config = TemporalConfig::from_env();
        assert_eq!(config.address, "http://temporal.internal:7233");
        assert_eq!(config.namespace, "production");

        // Clean up.
        std::env::remove_var("TEMPORAL_ADDRESS");
        std::env::remove_var("TEMPORAL_NAMESPACE");
    }

    #[test]
    fn test_signal_payload_serialization() {
        let payload = SignalPayload {
            data: serde_json::json!({"key": "value", "count": 42}),
            received_at: "2025-01-15T10:30:00Z".to_string(),
            request_id: "req-abc-123".to_string(),
        };

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["data"]["key"], "value");
        assert_eq!(json["data"]["count"], 42);
        assert_eq!(json["received_at"], "2025-01-15T10:30:00Z");
        assert_eq!(json["request_id"], "req-abc-123");
    }

    #[test]
    fn test_gateway_workflow_id_generation() {
        assert_eq!(gateway_workflow_id("webhook-1"), "gateway-webhook-1");
        assert_eq!(gateway_workflow_id("trigger-abc"), "gateway-trigger-abc");
        assert_eq!(
            gateway_workflow_id("550e8400-e29b-41d4-a716-446655440000"),
            "gateway-550e8400-e29b-41d4-a716-446655440000"
        );
    }

    #[test]
    fn test_gateway_task_queue_generation() {
        assert_eq!(
            gateway_task_queue("webhook-1"),
            "gateway-webhook-1-queue"
        );
        assert_eq!(
            gateway_task_queue("trigger-abc"),
            "gateway-trigger-abc-queue"
        );
    }

    #[test]
    fn test_temporal_error_display() {
        let err = TemporalError::Connection("refused".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("connection"));
        assert!(msg.contains("refused"));

        let err = TemporalError::Rpc("deadline exceeded".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("RPC"));
        assert!(msg.contains("deadline exceeded"));

        let err = TemporalError::Serialization("invalid utf-8".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("serialization"));
        assert!(msg.contains("invalid utf-8"));
    }

    #[test]
    fn test_temporal_client_new_and_debug() {
        let config = TemporalConfig {
            address: "http://temporal:7233".to_string(),
            namespace: "test-ns".to_string(),
        };
        let client = TemporalClient::new(&config);

        let debug = format!("{client:?}");
        assert!(debug.contains("http://temporal:7233"));
        assert!(debug.contains("test-ns"));
        assert!(debug.contains("connected: false"));
    }

    #[test]
    fn test_temporal_client_clone() {
        let config = TemporalConfig {
            address: "http://temporal:7233".to_string(),
            namespace: "clone-test".to_string(),
        };
        let client = TemporalClient::new(&config);
        let cloned = client.clone();

        assert_eq!(
            format!("{client:?}"),
            format!("{cloned:?}")
        );
    }

    #[tokio::test]
    async fn test_start_gateway_workflow_stub() {
        let config = TemporalConfig {
            address: "http://localhost:7233".to_string(),
            namespace: "default".to_string(),
        };
        let mut client = TemporalClient::new(&config);

        let result = client
            .start_gateway_workflow("test-interface", "test-queue")
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "gateway-test-interface");
    }

    #[tokio::test]
    async fn test_signal_gateway_workflow_stub() {
        let config = TemporalConfig {
            address: "http://localhost:7233".to_string(),
            namespace: "default".to_string(),
        };
        let mut client = TemporalClient::new(&config);

        let payload = SignalPayload {
            data: serde_json::json!({"test": true}),
            received_at: "2025-01-15T10:30:00Z".to_string(),
            request_id: "req-test-1".to_string(),
        };

        let result = client
            .signal_gateway_workflow("test-interface", &payload)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_terminate_gateway_workflow_stub() {
        let config = TemporalConfig {
            address: "http://localhost:7233".to_string(),
            namespace: "default".to_string(),
        };
        let mut client = TemporalClient::new(&config);

        let result = client.terminate_gateway_workflow("test-interface").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_gateway_buffer_depth_stub() {
        let config = TemporalConfig {
            address: "http://localhost:7233".to_string(),
            namespace: "default".to_string(),
        };
        let mut client = TemporalClient::new(&config);

        let result = client.query_gateway_buffer_depth("test-interface").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
