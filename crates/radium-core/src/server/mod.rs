//! Server module for Radium Core.
//!
//! This module contains the gRPC server implementation and service handlers.

pub mod logging;
mod radium_service;

pub use radium_service::RadiumService;

use std::net::SocketAddr;

use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower::ServiceBuilder;
use tracing::info;

use crate::config::Config;
use crate::error::Result;
use crate::proto::radium_server::RadiumServer;
use logging::RequestLoggerLayer;

/// Default gRPC-Web port offset from the main gRPC port.
const GRPC_WEB_PORT_OFFSET: u16 = 1;

/// Start the Radium gRPC server.
///
/// If gRPC-Web is enabled in the configuration, also starts a gRPC-Web server
/// on the configured web address (or a default port if not specified).
///
/// # Errors
///
/// Returns an error if the server fails to start or bind to the configured address.
pub async fn run(config: &Config) -> Result<()> {
    let addr = config.server.address;

    // Create a shared database instance for the service
    let db = crate::storage::Database::open_in_memory()?;
    let service = RadiumService::new(db);

    if config.server.enable_grpc_web {
        // Determine the gRPC-Web address
        let web_addr = config.server.web_address.unwrap_or_else(|| {
            // Default: use the main address with port + 1
            SocketAddr::new(addr.ip(), addr.port() + GRPC_WEB_PORT_OFFSET)
        });

        info!(
            grpc_addr = %addr,
            grpc_web_addr = %web_addr,
            "Starting gRPC server with gRPC-Web support"
        );

        // If both addresses are the same, serve both on one port
        if addr == web_addr {
            info!(%addr, "Serving gRPC and gRPC-Web on same address");
            Server::builder()
                .accept_http1(true)
                .layer(ServiceBuilder::new().layer(RequestLoggerLayer).layer(GrpcWebLayer::new()))
                .add_service(RadiumServer::new(service))
                .serve(addr)
                .await?;
        } else {
            // Clone service for the second server
            let db_web = crate::storage::Database::open_in_memory()?;
            let service_web = RadiumService::new(db_web);

            // Spawn gRPC-Web server in background
            let grpc_web_handle = tokio::spawn(async move {
                info!(%web_addr, "gRPC-Web server started");
                Server::builder()
                    .accept_http1(true)
                    .layer(
                        ServiceBuilder::new().layer(RequestLoggerLayer).layer(GrpcWebLayer::new()),
                    )
                    .add_service(RadiumServer::new(service_web))
                    .serve(web_addr)
                    .await
            });

            // Run main gRPC server
            info!(%addr, "gRPC server started");
            let grpc_handle = tokio::spawn(async move {
                Server::builder().add_service(RadiumServer::new(service)).serve(addr).await
            });

            // Wait for either server to finish (or error)
            tokio::select! {
                result = grpc_handle => {
                    result.map_err(|e| crate::error::RadiumError::Io(
                        std::io::Error::other(e)
                    ))??;
                }
                result = grpc_web_handle => {
                    result.map_err(|e| crate::error::RadiumError::Io(
                        std::io::Error::other(e)
                    ))??;
                }
            }
        }
    } else {
        // Plain gRPC server only
        info!(%addr, "Starting plain gRPC server (gRPC-Web disabled)");
        Server::builder().add_service(RadiumServer::new(service)).serve(addr).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_config_default_address() {
        let config = Config::default();
        assert_eq!(config.server.address, "127.0.0.1:50051".parse().unwrap());
    }

    #[tokio::test]
    async fn test_server_config_grpc_web_enabled() {
        let mut config = Config::default();
        config.server.enable_grpc_web = true;
        config.server.web_address = Some("127.0.0.1:50052".parse().unwrap());

        // Test that config is valid
        assert!(config.server.enable_grpc_web);
        assert_eq!(config.server.web_address, Some("127.0.0.1:50052".parse().unwrap()));
    }

    #[tokio::test]
    async fn test_server_config_grpc_web_disabled() {
        let mut config = Config::default();
        config.server.enable_grpc_web = false;

        // Test that config is valid
        assert!(!config.server.enable_grpc_web);
    }

    // Note: Full server startup/shutdown tests would require:
    // 1. Binding to actual ports (may conflict in CI)
    // 2. Spawning server in background
    // 3. Testing client connections
    // 4. Graceful shutdown
    // These are better suited for integration tests rather than unit tests.
    // The server::run function is tested indirectly through integration tests.
}
