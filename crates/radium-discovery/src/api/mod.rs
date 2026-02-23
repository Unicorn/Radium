//! Discovery service API router

mod compare;
mod index;
mod related;
mod search;
mod telemetry;

use axum::{
    routing::{get, post, put},
    Json, Router,
};
use serde::Serialize;
use std::time::Duration;
use tower_http::{
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use crate::state::AppState;

/// Build the router with the given allowed CORS origins.
///
/// Pass the `allowed_origins` from `DiscoveryConfig`. To allow any origin
/// (development only), include `"*"` in the list.
pub fn router(state: AppState, allowed_origins: &[String]) -> Router {
    let cors = build_cors_layer(allowed_origins);
    build_router(state, cors)
}

fn build_cors_layer(allowed_origins: &[String]) -> CorsLayer {
    let methods = [
        axum::http::Method::GET,
        axum::http::Method::POST,
        axum::http::Method::PUT,
        axum::http::Method::DELETE,
        axum::http::Method::OPTIONS,
    ];
    let headers = [
        axum::http::header::AUTHORIZATION,
        axum::http::header::CONTENT_TYPE,
        axum::http::header::ACCEPT,
    ];

    // Allow any origin only when explicitly configured with "*"
    if allowed_origins.iter().any(|o| o == "*") {
        return CorsLayer::new()
            .allow_methods(methods)
            .allow_headers(headers)
            .allow_origin(Any);
    }

    let parsed: Vec<axum::http::HeaderValue> = allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    if parsed.is_empty() {
        // Fallback: allow nothing (restrictive default)
        CorsLayer::new()
            .allow_methods(methods)
            .allow_headers(headers)
    } else {
        CorsLayer::new()
            .allow_methods(methods)
            .allow_headers(headers)
            .allow_origin(parsed)
    }
}

fn build_router(state: AppState, cors: CorsLayer) -> Router {

    Router::new()
        .route("/health", get(health))
        .route("/v1/discover/index", post(index::create_index))
        .route(
            "/v1/discover/index/{id}",
            put(index::update_index).delete(index::delete_index),
        )
        .route(
            "/v1/discover/index/{id}/telemetry",
            post(telemetry::record_telemetry),
        )
        .route("/v1/discover/compare", get(compare::compare))
        .route("/v1/discover/search", post(search::search))
        .route(
            "/v1/discover/{id}/related",
            get(related::get_related),
        )
        .route(
            "/v1/discover/{id}/dependencies",
            get(related::get_dependencies),
        )
        .route(
            "/v1/discover/{id}/dependents",
            get(related::get_dependents),
        )
        .with_state(state)
        .layer(cors)
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(TraceLayer::new_for_http())
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    service: String,
    version: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        service: "radium-discovery".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_endpoint() {
        let Json(response) = health().await;
        assert_eq!(response.status, "ok");
        assert_eq!(response.service, "radium-discovery");
    }
}
