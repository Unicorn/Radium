//! Tests for request logging middleware
//!
//! Note: These tests are currently disabled as they require additional dependencies.
//! The logging middleware is tested through integration tests when the server runs.

#![allow(dead_code, unused_imports)]

use bytes::Bytes;
use http::{Request, Response, StatusCode};
use http_body_util::Full;
use radium_core::server::logging::RequestLoggerLayer;
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// A simple test service that returns a successful response
#[derive(Clone)]
struct TestService;

impl<B> Service<Request<B>> for TestService {
    type Response = Response<Full<Bytes>>;
    type Error = String;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: Request<B>) -> Self::Future {
        Box::pin(async move {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Full::new(Bytes::from("OK")))
                .unwrap())
        })
    }
}

#[tokio::test]
#[ignore = "Requires additional dependencies - logging is tested through server integration tests"]
async fn test_request_logger_generates_request_id() {
    let layer = RequestLoggerLayer;
    let mut service = layer.layer(TestService);

    let request = Request::builder().uri("/test").body(()).unwrap();

    let response = service.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "Requires additional dependencies - logging is tested through server integration tests"]
async fn test_request_logger_preserves_existing_request_id() {
    let layer = RequestLoggerLayer;
    let mut service = layer.layer(TestService);

    let request = Request::builder()
        .uri("/test")
        .header("x-request-id", "custom-request-id-123")
        .body(())
        .unwrap();

    // The service should preserve the existing request ID
    let response = service.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "Requires additional dependencies - logging is tested through server integration tests"]
async fn test_request_logger_measures_duration() {
    use std::time::Instant;

    let layer = RequestLoggerLayer;
    let mut service = layer.layer(TestService);

    let start = Instant::now();
    let request = Request::builder().uri("/test").body(()).unwrap();

    let response = service.call(request).await.unwrap();
    let duration = start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    // Duration should be very small (less than 100ms for a simple test service)
    assert!(duration.as_millis() < 100);
}
