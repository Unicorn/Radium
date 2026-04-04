//! HTTP API module
//!
//! Provides REST API endpoints for workflow compilation, validation, and the
//! versioned v1 API for component discovery and workflow management.

// API types are part of the public interface
#![allow(dead_code)]
#![allow(unused_imports)]

pub mod auth;
mod errors;
mod handlers;
pub mod metrics_middleware;
mod middleware;
pub mod state;
pub mod v1;

use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Extension, Router,
};
use tower_http::{
    cors::CorsLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use crate::monitoring::MetricsRegistry;
use state::AppState;

pub use handlers::{CompileRequest, CompileResponse, ValidateResponse};

/// Maximum request body size (1 MiB).
const MAX_BODY_SIZE: usize = 1_048_576;

/// Create the API router.
///
/// When `app_state` is `Some`, the v1 routes (which require a Supabase
/// connection) are mounted under `/v1`. When it is `None` (e.g. because
/// Supabase env vars are not configured), only the core compilation and
/// health endpoints are available.
///
/// The `metrics` parameter provides a shared metrics registry for the
/// `/metrics` endpoint and the request metrics middleware.
pub fn router(app_state: Option<AppState>, metrics: Arc<MetricsRegistry>) -> Router {
    // Restrict CORS to the methods and headers actually used by clients.
    let cors = CorsLayer::new()
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ])
        .allow_origin(tower_http::cors::Any);

    let timeout = TimeoutLayer::new(Duration::from_secs(30));

    let mut app = Router::new()
        .route("/compile", post(handlers::compile))
        .route("/validate", post(handlers::validate))
        .route("/health", get(handlers::health))
        .route("/metrics", get(handlers::metrics));

    if let Some(state) = app_state {
        let v1 = v1::router().with_state(state);
        app = app.nest("/v1", v1);
    }

    app.layer(axum::middleware::from_fn(metrics_middleware::track_requests))
        .layer(Extension(metrics))
        .layer(DefaultBodyLimit::max(MAX_BODY_SIZE))
        .layer(cors)
        .layer(timeout)
        .layer(TraceLayer::new_for_http())
}
