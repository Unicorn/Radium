//! API error types

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// API error response
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.error.as_str() {
            "NOT_FOUND" => StatusCode::NOT_FOUND,
            "BAD_REQUEST" => StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR" => StatusCode::UNPROCESSABLE_ENTITY,
            "INTERNAL_ERROR" => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(self)).into_response()
    }
}

impl ApiError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        ApiError {
            error: "BAD_REQUEST".to_string(),
            message: message.into(),
            details: None,
        }
    }

    pub fn validation_error(message: impl Into<String>) -> Self {
        ApiError {
            error: "VALIDATION_ERROR".to_string(),
            message: message.into(),
            details: None,
        }
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        ApiError {
            error: "INTERNAL_ERROR".to_string(),
            message: message.into(),
            details: None,
        }
    }
}
