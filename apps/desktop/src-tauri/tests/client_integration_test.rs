//! Integration tests for the desktop gRPC client.
//!
//! These tests verify that the ClientManager can connect to a running Radium server
//! and perform gRPC operations.

use radium_core::{config::Config, server};
use radium_desktop_lib::client::ClientManager;
use std::time::Duration;
use tokio::time;

/// Start a test server on a random available port
async fn start_test_server() -> u16 {
    use std::net::TcpListener;
    
    // Find an available port
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    
    let mut config = Config::default();
    config.server.address = format!("127.0.0.1:{}", port).parse().unwrap();
    config.server.enable_grpc_web = false;

    tokio::spawn(async move {
        server::run(&config).await.expect("Server failed to run");
    });
    
    // Give the server a moment to start
    time::sleep(Duration::from_millis(500)).await;
    
    port
}

/// Test that ClientManager can connect to a running server and make a Ping call
#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test client_integration_test -- --ignored"]
async fn test_client_manager_ping() {
    let port = start_test_server().await;
    let server_address = format!("http://127.0.0.1:{}", port);
    
    let manager = ClientManager::with_address(server_address);
    
    // Get client and make a ping call
    let mut client = manager.get_client().await.expect("Should connect to server");
    
    let request = tonic::Request::new(radium_core::proto::PingRequest {
        message: "Hello from test".to_string(),
    });
    
    let response = client
        .ping(request)
        .await
        .expect("Ping should succeed");
    
    let ping_response = response.into_inner();
    assert_eq!(ping_response.message, "Pong! Hello from test");
}

/// Test that ClientManager reuses connections
#[tokio::test]
#[ignore = "Requires running server - run manually with: cargo test --test client_integration_test -- --ignored"]
async fn test_client_manager_connection_reuse() {
    let port = start_test_server().await;
    let server_address = format!("http://127.0.0.1:{}", port);
    
    let manager = ClientManager::with_address(server_address);
    
    // Get client twice - should reuse the connection
    let mut client1 = manager.get_client().await.expect("Should connect");
    let mut client2 = manager.get_client().await.expect("Should reuse connection");
    
    // Both clients should work
    let request1 = tonic::Request::new(radium_core::proto::PingRequest {
        message: "Test1".to_string(),
    });
    
    let request2 = tonic::Request::new(radium_core::proto::PingRequest {
        message: "Test2".to_string(),
    });
    
    let _response1 = client1
        .ping(request1)
        .await
        .expect("First client should work");
    
    let _response2 = client2
        .ping(request2)
        .await
        .expect("Second client should work");
}

