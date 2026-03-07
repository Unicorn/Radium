//! HTTP request metrics middleware.
//!
//! Records `http_requests_total` (counter) and
//! `http_request_duration_seconds` (histogram) for every request.

use std::sync::Arc;
use std::time::Instant;

use axum::{
    extract::{MatchedPath, Request},
    middleware::Next,
    response::Response,
    Extension,
};

use crate::monitoring::MetricsRegistry;

/// Axum middleware that records request count and latency metrics.
///
/// Uses [`MatchedPath`] when available to keep label cardinality low
/// (e.g. `/v1/services/{id}` instead of `/v1/services/abc-123`).
pub async fn track_requests(
    Extension(registry): Extension<Arc<MetricsRegistry>>,
    matched_path: Option<MatchedPath>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().to_string();
    let path = matched_path
        .map(|mp| mp.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());
    let start = Instant::now();

    let response = next.run(request).await;

    let status = response.status().as_u16();
    let duration = start.elapsed();

    // Increment per-method/status counter.
    let counter_name = format!(
        "http_requests_total{{method=\"{}\",status=\"{}\",path=\"{}\"}}",
        method, status, path,
    );
    registry.counter(&counter_name).inc();

    // Record latency in seconds.
    registry
        .histogram("http_request_duration_seconds")
        .observe(duration.as_secs_f64());

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, routing::get, Router};
    use tower::ServiceExt;

    async fn ok_handler() -> &'static str {
        "ok"
    }

    #[tokio::test]
    async fn test_middleware_increments_counter() {
        let registry = Arc::new(MetricsRegistry::new());

        let app = Router::new()
            .route("/ping", get(ok_handler))
            .layer(axum::middleware::from_fn(track_requests))
            .layer(Extension(registry.clone()));

        let req = axum::http::Request::builder()
            .uri("/ping")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 200);

        let snap = registry.snapshot();
        // MatchedPath resolves to the route pattern "/ping"
        let key = "http_requests_total{method=\"GET\",status=\"200\",path=\"/ping\"}";
        assert_eq!(snap.counters.get(key), Some(&1));
    }
}
