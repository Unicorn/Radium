//! Logging middleware for gRPC requests with request IDs and timing.

use std::task::{Context, Poll};
use std::time::Instant;

use http::Request;
use tower::{Layer, Service};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Header name for request ID
const REQUEST_ID_HEADER: &str = "x-request-id";

/// A `Layer` that adds logging with request IDs and timing to requests.
#[derive(Debug, Clone)]
pub struct RequestLoggerLayer;

impl<S> Layer<S> for RequestLoggerLayer {
    type Service = RequestLoggerService<S>;

    fn layer(&self, service: S) -> Self::Service {
        RequestLoggerService { service }
    }
}

/// A `Service` that logs request information with request IDs and timing.
#[derive(Debug, Clone)]
pub struct RequestLoggerService<S> {
    service: S,
}

impl<S, B> Service<Request<B>> for RequestLoggerService<S>
where
    S: Service<Request<B>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<B>) -> Self::Future {
        // Generate or extract request ID
        let request_id = request
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .map_or_else(|| Uuid::new_v4().to_string(), ToString::to_string);

        // Add request ID to headers if not present
        if !request.headers().contains_key(REQUEST_ID_HEADER) {
            if let Ok(header_value) = http::HeaderValue::from_str(&request_id) {
                request.headers_mut().insert(REQUEST_ID_HEADER, header_value);
            }
        }

        let method = request.method().clone();
        let uri = request.uri().path().to_string();
        let start_time = Instant::now();

        // Create a tracing span with request ID
        let span = tracing::span!(
            tracing::Level::INFO,
            "request",
            request_id = %request_id,
            method = %method,
            uri = %uri
        );
        let _enter = span.enter();

        info!(
            request_id = %request_id,
            method = %method,
            uri = %uri,
            "Received request"
        );

        for (name, value) in request.headers() {
            debug!(
                request_id = %request_id,
                header = %name,
                value = %value.to_str().unwrap_or("<?>"),
                "Request header"
            );
        }

        let future = self.service.call(request);
        Box::pin(async move {
            let result = future.await;
            let duration = start_time.elapsed();

            match &result {
                Ok(_) => {
                    info!(
                        request_id = %request_id,
                        method = %method,
                        uri = %uri,
                        duration_ms = duration.as_millis(),
                        "Request completed successfully"
                    );
                }
                Err(_) => {
                    warn!(
                        request_id = %request_id,
                        method = %method,
                        uri = %uri,
                        duration_ms = duration.as_millis(),
                        "Request failed"
                    );
                }
            }

            result
        })
    }
}
