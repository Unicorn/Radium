//! Shared application state.
//!
//! Holds resources that are shared across all request handlers, such as the
//! Supabase client used for database access and the API rate limiter.

use std::sync::Arc;

use crate::discovery::client::DiscoveryClient;
use crate::kong_client::KongClient;
use crate::security::SlidingWindowLimiter;
use crate::supabase::SupabaseClient;
use crate::temporal_client::TemporalClient;

/// Application-wide shared state passed to Axum handlers via `State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    /// Supabase REST API client, wrapped in `Arc` so cloning the state is cheap.
    pub supabase: Arc<SupabaseClient>,
    /// Per-client rate limiter for API requests.
    pub rate_limiter: Arc<SlidingWindowLimiter>,
    /// Optional discovery service client for indexing and telemetry.
    pub discovery: Option<Arc<DiscoveryClient>>,
    /// Optional Kong Admin API client for dynamic route management.
    pub kong: Option<Arc<KongClient>>,
    /// Optional Temporal gRPC client for gateway workflow management.
    ///
    /// Wrapped in a `Mutex` because `TemporalClient` has mutable state
    /// (the lazily-established gRPC channel).
    pub temporal: Option<Arc<tokio::sync::Mutex<TemporalClient>>>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("supabase", &self.supabase)
            .field("rate_limiter", &"SlidingWindowLimiter { .. }")
            .field("discovery", &self.discovery)
            .field("kong", &self.kong)
            .field("temporal", &self.temporal.as_ref().map(|_| "TemporalClient { .. }"))
            .finish()
    }
}
