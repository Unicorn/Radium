//! Shared test utilities for Radium Core integration tests.
//!
//! This module provides common helper functions used across multiple test files
//! to reduce code duplication and improve maintainability.

use radium_core::{config::Config, proto::radium_client::RadiumClient, server};
use std::net::TcpListener;
use std::time::Duration;
use tokio::time;

/// Finds an available port by binding to port 0 and returning the allocated port.
///
/// # Returns
/// An available port number.
fn find_available_port() -> u16 {
    // Try to bind to port 0, which will allocate an available port
    // We drop the listener immediately - the port will be available when we bind again
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port 0");
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    // Small delay to ensure port is released
    std::thread::sleep(Duration::from_millis(10));
    port
}

/// Starts a test server on an available port.
///
/// # Returns
/// The port number the server is running on.
///
/// # Panics
/// Panics if the server fails to start or if the address parsing fails.
pub async fn start_test_server() -> u16 {
    let port = find_available_port();
    start_test_server_on_port(port).await;
    port
}

/// Starts a test server on the specified port.
///
/// # Arguments
/// * `port` - The port number to start the server on
///
/// # Panics
/// Panics if the server fails to start or if the address parsing fails.
pub async fn start_test_server_on_port(port: u16) {
    let mut config = Config::default();
    config.server.address = format!("127.0.0.1:{}", port).parse().unwrap();
    config.server.enable_grpc_web = false;

    tokio::spawn(async move {
        server::run(&config).await.expect("Server failed to run");
    });
    // Give the server a moment to start
    time::sleep(Duration::from_millis(200)).await;
}

/// Creates a connected gRPC client for testing.
///
/// # Arguments
/// * `port` - The port number where the server is running
///
/// # Returns
/// A connected `RadiumClient` ready for use in tests.
///
/// # Panics
/// Panics if the client fails to connect to the server.
pub async fn create_test_client(port: u16) -> RadiumClient<tonic::transport::Channel> {
    let endpoint = tonic::transport::Endpoint::from_shared(format!("http://127.0.0.1:{}", port))
        .expect("Invalid endpoint URI");
    let channel = endpoint.connect().await.expect("Failed to connect to server");
    RadiumClient::new(channel)
}
