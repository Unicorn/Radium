//! HTTP API module
//!
//! Provides REST API endpoints for workflow compilation and validation.

mod errors;
mod handlers;
mod middleware;

use axum::{
    routing::{get, post},
    Router,
};
use std::time::Duration;
use tower_http::{
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

pub use handlers::{CompileRequest, CompileResponse, ValidateResponse};

/// Create the API router
pub fn router() -> Router {
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);

    let timeout = TimeoutLayer::new(Duration::from_secs(30));

    Router::new()
        .route("/compile", post(handlers::compile))
        .route("/validate", post(handlers::validate))
        .route("/health", get(handlers::health))
        .layer(cors)
        .layer(timeout)
        .layer(TraceLayer::new_for_http())
}
