//! Discovery service API router

mod index;
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

pub fn router(state: AppState) -> Router {
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
        // TODO: Restrict CORS origins for production deployment
        .allow_origin(Any);

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
