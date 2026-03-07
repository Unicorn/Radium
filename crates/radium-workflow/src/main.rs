//! Workflow Compiler HTTP Server
//!
//! Provides REST API endpoints for workflow compilation and validation.

use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod codegen;
mod deploy_pipeline;
mod discovery;
mod kong_client;
mod monitoring;
mod schema;
mod security;
mod supabase;
mod temporal_client;
mod validation;
mod verification;
mod versioning;
mod yaml_format;

use api::state::AppState;
use monitoring::MetricsRegistry;
use security::{RateLimitConfig, SlidingWindowLimiter};
use kong_client::{KongClient, KongConfig};
use supabase::{SupabaseClient, SupabaseConfig};
use temporal_client::{TemporalClient, TemporalConfig};

#[tokio::main]
async fn main() {
    // Initialize tracing — optionally add an OTLP exporter when
    // OTEL_EXPORTER_OTLP_ENDPOINT is set (e.g. pointing at Jaeger).
    let registry = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "workflow_compiler=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer());

    if let Ok(otlp_endpoint) = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
        use opentelemetry_otlp::WithExportConfig;

        let otlp_exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(&otlp_endpoint);

        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(otlp_exporter)
            .with_trace_config(
                opentelemetry_sdk::trace::config()
                    .with_resource(opentelemetry_sdk::Resource::new(vec![
                        opentelemetry::KeyValue::new("service.name", "radium-workflow"),
                    ])),
            )
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .expect("failed to install OTLP tracer");

        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
        registry.with(otel_layer).init();
        tracing::info!(endpoint = %otlp_endpoint, "OTLP tracing enabled");
    } else {
        registry.init();
    }

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

            let kong = Some(Arc::new(KongClient::new(&KongConfig::from_env())));
            tracing::info!("Kong Admin API client initialized");

            let temporal_config = TemporalConfig::from_env();
            let temporal = Some(Arc::new(tokio::sync::Mutex::new(
                TemporalClient::new(&temporal_config),
            )));
            tracing::info!(
                address = %temporal_config.address,
                namespace = %temporal_config.namespace,
                "Temporal gRPC client initialized (lazy connection)"
            );

            Some(AppState {
                supabase: Arc::new(client),
                rate_limiter: Arc::new(SlidingWindowLimiter::new(
                    RateLimitConfig::for_api(),
                )),
                discovery,
                kong,
                temporal,
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

    // Build the metrics registry and router
    let metrics = Arc::new(MetricsRegistry::new());
    let app = api::router(app_state, metrics);

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
