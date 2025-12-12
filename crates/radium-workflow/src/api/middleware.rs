//! API middleware

use axum::http::Request;
use tower_http::request_id::{MakeRequestId, RequestId};
use uuid::Uuid;

/// Generate unique request IDs
#[derive(Clone, Copy)]
pub struct RequestIdGenerator;

impl MakeRequestId for RequestIdGenerator {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let id = Uuid::new_v4().to_string();
        Some(RequestId::new(id.parse().unwrap()))
    }
}
