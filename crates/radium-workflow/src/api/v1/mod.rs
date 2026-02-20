//! Version 1 API routes.
//!
//! All authenticated endpoints live under `/v1`. This module re-exports the
//! sub-router that is nested into the top-level application router.

pub mod components;

use axum::{routing::get, Router};

use super::state::AppState;

/// Build the `/v1` sub-router. All routes in this router receive `AppState`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/components", get(components::list_components))
        .route("/components/{component_type}", get(components::get_component))
}
