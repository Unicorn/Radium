//! Integration tests for gRPC-Web support.
//!
//! RAD-039: This test needs to be completed with proper gRPC-Web framing.

use radium_core::{config::Config, server};
use std::time::Duration;
use tokio::time;

async fn start_test_server_on_port(port: u16) {
    let mut config = Config::default();
    config.server.address = format!("127.0.0.1:{}", port).parse().unwrap();
    config.server.enable_grpc_web = true;
    config.server.web_address = Some(format!("127.0.0.1:{}", port + 1).parse().unwrap());

    tokio::spawn(async move {
        server::run(&config).await.expect("Server failed to run");
    });
    // Give the server a moment to start
    time::sleep(Duration::from_millis(200)).await;
}

/// Test gRPC-Web ping endpoint.
///
/// This test verifies that the gRPC-Web endpoint is accessible and responds correctly.
///
/// # Note
/// gRPC-Web requires a specific frame format:
/// - Request: 1 byte compressed flag + 4 bytes length + message bytes
/// - Response: Same format, possibly with trailing metadata frame
///
/// This test implements proper gRPC-Web framing and verifies the endpoint works.
#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test grpc_web_test -- --ignored"]
async fn test_grpc_web_ping() {
    use prost::Message;
    use radium_core::proto::{PingRequest, PingResponse};

    start_test_server_on_port(50066).await;

    let client = reqwest::Client::new();
    let request = PingRequest { message: "Hello gRPC-Web".to_string() };

    // Create gRPC-Web frame: 1 byte compressed flag + 4 bytes length + message
    let mut message_buf = Vec::new();
    request.encode(&mut message_buf).unwrap();

    let mut frame = Vec::with_capacity(5 + message_buf.len());
    frame.push(0x00); // Uncompressed
    frame.extend_from_slice(&(message_buf.len() as u32).to_be_bytes());
    frame.extend_from_slice(&message_buf);

    let response = client
        .post("http://127.0.0.1:50067/radium.Radium/Ping")
        .header("Content-Type", "application/grpc-web+proto")
        .header("Accept", "application/grpc-web+proto")
        .body(frame)
        .send()
        .await
        .expect("gRPC-Web request failed");

    assert_eq!(response.status(), 200);

    let body = response.bytes().await.unwrap();

    // gRPC-Web responses have a 1-byte prefix for the compression status and then 4 bytes for the length.
    // After that, it's the actual protobuf message.
    assert!(!body.is_empty(), "Response body should not be empty");
    assert!(body.len() >= 5, "Response should have at least 5 bytes for frame header");

    assert_eq!(body[0], 0x00, "First byte should be 0x00 (uncompressed)");
    let message_length = u32::from_be_bytes(body[1..5].try_into().unwrap());
    let message_bytes = &body[5..5 + message_length as usize];

    let ping_response = PingResponse::decode(message_bytes).unwrap();
    assert_eq!(ping_response.message, "Pong! Received: Hello gRPC-Web");
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test grpc_web_test -- --ignored"]
async fn test_grpc_web_cors_headers() {
    start_test_server_on_port(50068).await;

    let client = reqwest::Client::new();
    let request = client
        .request(reqwest::Method::OPTIONS, "http://127.0.0.1:50069/radium.Radium/Ping")
        .header("Origin", "http://localhost:3000")
        .header("Access-Control-Request-Method", "POST")
        .header("Access-Control-Request-Headers", "content-type");

    let response = request.send().await.expect("CORS preflight request failed");

    // gRPC-Web should support CORS
    // Note: Actual CORS headers depend on server configuration
    assert_eq!(response.status(), 200);
}

#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test grpc_web_test -- --ignored"]
async fn test_grpc_web_error_response() {
    use prost::Message;
    use radium_core::proto::GetAgentRequest;

    start_test_server_on_port(50070).await;

    let client = reqwest::Client::new();
    let request = GetAgentRequest { agent_id: "nonexistent".to_string() };

    // Create gRPC-Web frame
    let mut message_buf = Vec::new();
    request.encode(&mut message_buf).unwrap();

    let mut frame = Vec::with_capacity(5 + message_buf.len());
    frame.push(0x00); // Uncompressed
    frame.extend_from_slice(&(message_buf.len() as u32).to_be_bytes());
    frame.extend_from_slice(&message_buf);

    let response = client
        .post("http://127.0.0.1:50071/radium.Radium/GetAgent")
        .header("Content-Type", "application/grpc-web+proto")
        .header("Accept", "application/grpc-web+proto")
        .body(frame)
        .send()
        .await
        .expect("gRPC-Web request failed");

    // Should return error status (404 for not found)
    // gRPC-Web errors are typically returned as HTTP 200 with error in the frame
    assert_eq!(response.status(), 200);

    let body = response.bytes().await.unwrap();
    // Error responses in gRPC-Web still use the frame format
    assert!(!body.is_empty());
}
