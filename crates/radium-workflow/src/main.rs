//! Workflow Compiler HTTP Server
//!
//! Provides REST API endpoints for workflow compilation and validation.

use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod codegen;
mod schema;
mod validation;
mod verification;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "workflow_compiler=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Build the router
    let app = api::router();

    // Get port from environment or default to 3020
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3020);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Workflow Compiler listening on {}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
