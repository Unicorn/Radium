//! Radium Core Server - Entry Point
//!
//! This binary starts the Radium gRPC server.

use tracing::error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use radium_core::{config::Config, server};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "radium_core=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    // Note: unwrap_or_else is acceptable here as it provides a sensible default

    // Load configuration
    let config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            error!(error = %e, "Failed to load configuration");
            std::process::exit(1);
        }
    };

    // Start server
    if let Err(e) = server::run(&config).await {
        error!(error = %e, "Server error");
        std::process::exit(1);
    }
}
