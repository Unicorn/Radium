//! Version 1 API routes.
//!
//! All authenticated endpoints live under `/v1`. This module re-exports the
//! sub-router that is nested into the top-level application router.

pub mod components;
pub mod deploy;
pub mod interfaces;
pub mod projects;
pub mod services;
pub mod state_variables;

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
        // Project CRUD
        .route(
            "/projects",
            post(projects::create_project).get(projects::list_projects),
        )
        .route(
            "/projects/{id}",
            get(projects::get_project)
                .put(projects::update_project)
                .delete(projects::delete_project),
        )
        .route(
            "/projects/{id}/deploy",
            post(projects::deploy_project),
        )
        .route(
            "/projects/{id}/status",
            get(projects::project_status),
        )
        .route(
            "/projects/{id}/services",
            get(projects::list_project_services),
        )
        // Project state variables
        .route(
            "/projects/{id}/variables",
            post(state_variables::create_project_variable)
                .get(state_variables::list_project_variables),
        )
        .route(
            "/projects/{id}/variables/{var_id}",
            get(state_variables::get_project_variable)
                .put(state_variables::update_project_variable)
                .delete(state_variables::delete_project_variable),
        )
        // Service CRUD
        .route(
            "/services",
            post(services::create_workflow).get(services::list_workflows),
        )
        // Service catalog (must be before /services/{id} to avoid capture)
        .route("/services/catalog", get(services::list_catalog))
        .route(
            "/services/catalog/{source_id}/import",
            post(services::import_service),
        )
        .route(
            "/services/{id}",
            get(services::get_workflow)
                .put(services::update_workflow)
                .delete(services::delete_workflow),
        )
        .route(
            "/services/{id}/validate",
            post(services::validate_workflow),
        )
        .route(
            "/services/{id}/publish",
            post(services::publish_service),
        )
        .route(
            "/services/{id}/unpublish",
            post(services::unpublish_service),
        )
        // Service state variables
        .route(
            "/services/{id}/variables",
            post(state_variables::create_service_variable)
                .get(state_variables::list_service_variables),
        )
        .route(
            "/services/{id}/variables/{var_id}",
            get(state_variables::get_service_variable)
                .put(state_variables::update_service_variable)
                .delete(state_variables::delete_service_variable),
        )
        // Service interfaces
        .route(
            "/services/{id}/interfaces",
            post(interfaces::create_interface).get(interfaces::list_interfaces),
        )
        .route(
            "/services/{id}/interfaces/{iid}",
            get(interfaces::get_interface)
                .put(interfaces::update_interface)
                .delete(interfaces::delete_interface),
        )
        .route(
            "/services/{id}/interfaces/{iid}/publish",
            post(interfaces::publish_interface),
        )
        .route(
            "/services/{id}/interfaces/{iid}/unpublish",
            post(interfaces::unpublish_interface),
        )
        // Deploy pipeline
        .route(
            "/services/{id}/deploy",
            post(deploy::deploy_workflow),
        )
        .route(
            "/services/{id}/undeploy",
            post(deploy::undeploy_workflow),
        )
        .route(
            "/services/{id}/status",
            get(deploy::workflow_status),
        )
}
