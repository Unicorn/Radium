//! Shared application state.
//!
//! Holds resources that are shared across all request handlers, such as the
//! Supabase client used for database access.

use std::sync::Arc;

use crate::supabase::SupabaseClient;

/// Application-wide shared state passed to Axum handlers via `State<AppState>`.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Supabase REST API client, wrapped in `Arc` so cloning the state is cheap.
    pub supabase: Arc<SupabaseClient>,
}
