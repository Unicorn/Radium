//! Shared application state.
//!
//! Holds resources that are shared across all request handlers, such as the
//! Supabase client used for database access and the API rate limiter.

use std::sync::Arc;

use crate::security::SlidingWindowLimiter;
use crate::supabase::SupabaseClient;

/// Application-wide shared state passed to Axum handlers via `State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    /// Supabase REST API client, wrapped in `Arc` so cloning the state is cheap.
    pub supabase: Arc<SupabaseClient>,
    /// Per-client rate limiter for API requests.
    pub rate_limiter: Arc<SlidingWindowLimiter>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("supabase", &self.supabase)
            .field("rate_limiter", &"SlidingWindowLimiter { .. }")
            .finish()
    }
}
