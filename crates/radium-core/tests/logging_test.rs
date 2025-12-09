#![cfg(feature = "server")]

use http::{Request, Response};
use radium_core::server::logging::RequestLoggerLayer;
use std::convert::Infallible;
use tower::{ServiceBuilder, ServiceExt};

#[tokio::test]
async fn test_request_logger_adds_request_id() {
    let service = ServiceBuilder::new().layer(RequestLoggerLayer).service_fn(
        |req: Request<String>| async move {
            let request_id = req.headers().get("x-request-id").cloned();
            Ok::<_, Infallible>(Response::new(request_id))
        },
    );

    let request = Request::builder().uri("/").body("test".to_string()).unwrap();

    let response = service.oneshot(request).await.unwrap();
    let request_id_header = response.body();

    assert!(request_id_header.is_some(), "Request ID should be added if missing");
}

#[tokio::test]
async fn test_request_logger_preserves_existing_request_id() {
    let service = ServiceBuilder::new().layer(RequestLoggerLayer).service_fn(
        |req: Request<String>| async move {
            let request_id =
                req.headers().get("x-request-id").unwrap().to_str().unwrap().to_string();
            Ok::<_, Infallible>(Response::new(request_id))
        },
    );

    let existing_id = "existing-id-123";
    let request = Request::builder()
        .uri("/")
        .header("x-request-id", existing_id)
        .body("test".to_string())
        .unwrap();

    let response = service.oneshot(request).await.unwrap();
    let request_id = response.body();

    assert_eq!(request_id, existing_id, "Existing request ID should be preserved");
}
