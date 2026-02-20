//! Version 1 API routes.
//!
//! All authenticated endpoints live under `/v1`. This module re-exports the
//! sub-router that is nested into the top-level application router.

pub mod components;
pub mod workflows;

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use super::state::AppState;

/// Build the `/v1` sub-router. All routes in this router receive `AppState`.
pub fn router() -> Router<AppState> {
    Router::new()
        // Component registry
        .route("/components", get(components::list_components))
        .route("/components/{component_type}", get(components::get_component))
        // Workflow CRUD
        .route(
            "/workflows",
            post(workflows::create_workflow).get(workflows::list_workflows),
        )
        .route(
            "/workflows/{id}",
            get(workflows::get_workflow)
                .put(workflows::update_workflow)
                .delete(workflows::delete_workflow),
        )
        .route(
            "/workflows/{id}/validate",
            post(workflows::validate_workflow),
        )
}
