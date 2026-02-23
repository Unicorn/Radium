//! Workflow Compiler HTTP Server
//!
//! Provides REST API endpoints for workflow compilation and validation.

use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod codegen;
mod discovery;
mod schema;
mod security;
mod supabase;
mod validation;
mod verification;
mod versioning;
mod yaml_format;

use api::state::AppState;
use security::{RateLimitConfig, SlidingWindowLimiter};
use supabase::{SupabaseClient, SupabaseConfig};

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

    // Attempt to build Supabase configuration from environment variables.
    // If the vars are missing the core endpoints (/compile, /validate, /health)
    // will still work -- only the /v1 routes are disabled.
    let app_state = match SupabaseConfig::from_env() {
        Ok(config) => {
            tracing::info!("Supabase configuration loaded -- v1 API routes enabled");
            let client = SupabaseClient::new(config);

            let discovery = discovery::client::DiscoveryClient::from_env().map(Arc::new);
            if discovery.is_some() {
                tracing::info!("Discovery service integration enabled");
            }

            Some(AppState {
                supabase: Arc::new(client),
                rate_limiter: Arc::new(SlidingWindowLimiter::new(
                    RateLimitConfig::for_api(),
                )),
                discovery,
            })
        }
        Err(e) => {
            tracing::warn!(
                "Supabase configuration not available ({e}). \
                 The /v1 API routes will NOT be mounted. \
                 Set SUPABASE_URL and SUPABASE_SERVICE_ROLE_KEY to enable them."
            );
            None
        }
    };

    // Build the router
    let app = api::router(app_state);

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
