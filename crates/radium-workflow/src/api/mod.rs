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
mod middleware;
pub mod state;
pub mod v1;

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

use state::AppState;

pub use handlers::{CompileRequest, CompileResponse, ValidateResponse};

/// Create the API router.
///
/// When `app_state` is `Some`, the v1 routes (which require a Supabase
/// connection) are mounted under `/v1`. When it is `None` (e.g. because
/// Supabase env vars are not configured), only the core compilation and
/// health endpoints are available.
pub fn router(app_state: Option<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);

    let timeout = TimeoutLayer::new(Duration::from_secs(30));

    let mut app = Router::new()
        .route("/compile", post(handlers::compile))
        .route("/validate", post(handlers::validate))
        .route("/health", get(handlers::health));

    if let Some(state) = app_state {
        let v1 = v1::router().with_state(state);
        app = app.nest("/v1", v1);
    }

    app.layer(cors)
        .layer(timeout)
        .layer(TraceLayer::new_for_http())
}
