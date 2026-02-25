//! Version 1 API routes.
//!
//! All authenticated endpoints live under `/v1`. This module re-exports the
//! sub-router that is nested into the top-level application router.

pub mod components;
pub mod deploy;
pub mod workflows;

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use super::state::AppState;

/// Build the `/v1` sub-router. All routes in this router receive `AppState`.
pub fn router() -> Router<AppState> {
    Router::new()
        // Component registry (built-in + custom CRUD)
        .route(
            "/components",
            get(components::list_components).post(components::create_component),
        )
        .route(
            "/components/custom",
            get(components::list_custom_components),
        )
        .route(
            "/components/custom/{name}",
            delete(components::delete_custom_component),
        )
        .route(
            "/components/{component_type}",
            get(components::get_component),
        )
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
        // Deploy pipeline
        .route(
            "/workflows/{id}/deploy",
            post(deploy::deploy_workflow),
        )
        .route(
            "/workflows/{id}/undeploy",
            post(deploy::undeploy_workflow),
        )
        .route(
            "/workflows/{id}/status",
            get(deploy::workflow_status),
        )
}
