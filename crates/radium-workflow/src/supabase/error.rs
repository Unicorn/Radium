//! Supabase REST client error types.

use std::fmt;

/// Errors that can occur when interacting with the Supabase REST API.
#[derive(Debug)]
pub enum SupabaseError {
    /// An HTTP request to Supabase failed at the transport level.
    RequestFailed(reqwest::Error),

    /// Supabase returned a non-success status code.
    ApiError {
        status: u16,
        message: String,
    },

    /// A query expected to find a resource returned nothing.
    NotFound {
        resource: String,
        key: String,
        value: String,
    },

    /// Missing or invalid configuration (e.g. environment variables).
    ConfigError(String),

    /// Response body could not be deserialized into the expected type.
    DeserializationError(String),
}

impl fmt::Display for SupabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SupabaseError::RequestFailed(err) => {
                write!(f, "Supabase request failed: {err}")
            }
            SupabaseError::ApiError { status, message } => {
                write!(f, "Supabase API error (HTTP {status}): {message}")
            }
            SupabaseError::NotFound {
                resource,
                key,
                value,
            } => {
                write!(f, "Not found: {resource} where {key} = {value}")
            }
            SupabaseError::ConfigError(msg) => {
                write!(f, "Supabase configuration error: {msg}")
            }
            SupabaseError::DeserializationError(msg) => {
                write!(f, "Deserialization error: {msg}")
            }
        }
    }
}

impl std::error::Error for SupabaseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SupabaseError::RequestFailed(err) => Some(err),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for SupabaseError {
    fn from(err: reqwest::Error) -> Self {
        SupabaseError::RequestFailed(err)
    }
}
