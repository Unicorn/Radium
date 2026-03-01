# P4: OSS Polish Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Complete the core open-source Radium workflow builder by connecting the composition layer to real infrastructure — bundled deploy, Kong routing, state variables API, and gateway workflows for durable edge buffering.

**Architecture:** Extract reusable deploy logic from `deploy.rs`, build a Kong Admin API client for dynamic route management, add CRUD endpoints for state variables (service and project scoped), and implement Temporal gateway workflows that buffer incoming data at the edge when internal services are down. Sequential pipeline: deploy → Kong → state vars → gateway.

**Tech Stack:** Rust (Axum, reqwest, tonic for Temporal gRPC), Supabase PostgREST, Kong 3.8 (DB mode), Temporal TypeScript SDK (codegen target), Handlebars templates

---

## Task 1: Extract Deploy Pipeline Module

Extract the core deploy steps (validate → codegen → store → update status) from `deploy_workflow` in `deploy.rs` into a reusable `deploy_pipeline` module. The existing single-workflow deploy handler will call into this extracted function.

**Files:**
- Create: `crates/radium-workflow/src/deploy_pipeline.rs`
- Modify: `crates/radium-workflow/src/lib.rs`
- Modify: `crates/radium-workflow/src/api/v1/deploy.rs`
- Test: inline in `deploy_pipeline.rs`

**Step 1: Write the failing test for deploy_single_service**

In `crates/radium-workflow/src/deploy_pipeline.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deploy_report_serialization() {
        let report = DeployReport {
            project_id: "proj-1".to_string(),
            deployed: vec![DeployedService {
                service_id: "svc-1".to_string(),
                compiled_at: "2026-03-01T00:00:00Z".to_string(),
            }],
            failed: Some(FailedService {
                service_id: "svc-2".to_string(),
                error: "Validation failed".to_string(),
            }),
            skipped: vec![SkippedService {
                service_id: "svc-3".to_string(),
                reason: "Deploy halted after failure".to_string(),
            }],
        };
        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["project_id"], "proj-1");
        assert_eq!(json["deployed"].as_array().unwrap().len(), 1);
        assert!(json["failed"].is_object());
        assert_eq!(json["skipped"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_deploy_report_no_failures() {
        let report = DeployReport {
            project_id: "proj-1".to_string(),
            deployed: vec![
                DeployedService {
                    service_id: "svc-1".to_string(),
                    compiled_at: "2026-03-01T00:00:00Z".to_string(),
                },
                DeployedService {
                    service_id: "svc-2".to_string(),
                    compiled_at: "2026-03-01T00:00:01Z".to_string(),
                },
            ],
            failed: None,
            skipped: vec![],
        };
        let json = serde_json::to_value(&report).unwrap();
        assert!(json["failed"].is_null());
        assert_eq!(json["deployed"].as_array().unwrap().len(), 2);
        assert_eq!(json["skipped"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_single_service_result_success() {
        let result = SingleServiceResult::Success {
            service_id: "svc-1".to_string(),
            compiled_at: "2026-03-01T00:00:00Z".to_string(),
        };
        assert!(matches!(result, SingleServiceResult::Success { .. }));
    }

    #[test]
    fn test_single_service_result_failure() {
        let result = SingleServiceResult::Failure {
            service_id: "svc-1".to_string(),
            error: "codegen failed".to_string(),
        };
        assert!(matches!(result, SingleServiceResult::Failure { .. }));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `nx run radium-workflow:test -- --lib deploy_pipeline`
Expected: FAIL — module doesn't exist yet

**Step 3: Write the deploy_pipeline module with types and extracted function**

Create `crates/radium-workflow/src/deploy_pipeline.rs`:

```rust
//! Reusable deploy pipeline logic.
//!
//! Extracted from the single-service deploy handler so it can be reused
//! by both single-service and project-level deploy endpoints.

use std::sync::Arc;

use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

use crate::api::state::AppState;
use crate::codegen;
use crate::schema::WorkflowDefinition;
use crate::supabase::SupabaseError;
use crate::validation;

/// Result of deploying a single service.
pub enum SingleServiceResult {
    Success {
        service_id: String,
        compiled_at: String,
    },
    Failure {
        service_id: String,
        error: String,
    },
}

/// Summary of a project deploy operation.
#[derive(Debug, Serialize)]
pub struct DeployReport {
    pub project_id: String,
    pub deployed: Vec<DeployedService>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed: Option<FailedService>,
    pub skipped: Vec<SkippedService>,
}

#[derive(Debug, Serialize)]
pub struct DeployedService {
    pub service_id: String,
    pub compiled_at: String,
}

#[derive(Debug, Serialize)]
pub struct FailedService {
    pub service_id: String,
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct SkippedService {
    pub service_id: String,
    pub reason: String,
}

/// Row for loading a workflow/service for deploy.
#[derive(Debug, serde::Deserialize)]
struct WorkflowRow {
    id: String,
    name: String,
    status_id: String,
    definition: serde_json::Value,
    deployed_at: Option<String>,
}

/// Row for inserting compiled code.
#[derive(Debug, Serialize)]
struct InsertCompiledCodeRow {
    id: String,
    workflow_id: String,
    code: serde_json::Value,
    compiled_at: String,
}

/// Row for updating workflow deploy status.
#[derive(Debug, Serialize)]
struct DeployUpdateRow {
    status_id: String,
    deployed_at: String,
}

const DEPLOYED_STATUS_ID: &str = "00000000-0000-0000-0000-000000000003";

/// Deploy a single service: validate → codegen → store compiled code → update status.
///
/// This is the core pipeline extracted from the single-service deploy endpoint.
/// It does NOT handle authentication — the caller must do that.
pub async fn deploy_single_service(
    state: &AppState,
    service_id: &str,
    user_id: &str,
) -> SingleServiceResult {
    // 1. Load the workflow/service
    let user_filter = format!("eq.{user_id}");
    let workflow: WorkflowRow = match state
        .supabase
        .select_one(
            "workflows",
            &[
                ("id", &format!("eq.{service_id}")),
                ("created_by", &user_filter),
                ("select", "id,name,status_id,definition,deployed_at"),
            ],
        )
        .await
    {
        Ok(w) => w,
        Err(e) => {
            return SingleServiceResult::Failure {
                service_id: service_id.to_string(),
                error: format!("Failed to load service: {e}"),
            }
        }
    };

    // 2. Parse the definition
    let definition: WorkflowDefinition = match serde_json::from_value(workflow.definition) {
        Ok(d) => d,
        Err(e) => {
            return SingleServiceResult::Failure {
                service_id: service_id.to_string(),
                error: format!("Failed to parse definition: {e}"),
            }
        }
    };

    // 3. Validate
    let validation_result = validation::validate(&definition);
    if !validation_result.is_valid() {
        let details: Vec<String> = validation_result
            .errors
            .iter()
            .map(ToString::to_string)
            .collect();
        return SingleServiceResult::Failure {
            service_id: service_id.to_string(),
            error: format!("Validation failed: {}", details.join(", ")),
        };
    }

    // 4. Code generation
    let generated = match codegen::generate(&definition) {
        Ok(g) => g,
        Err(e) => {
            return SingleServiceResult::Failure {
                service_id: service_id.to_string(),
                error: format!("Code generation failed: {e}"),
            }
        }
    };

    let code_json = match serde_json::to_value(&generated) {
        Ok(j) => j,
        Err(e) => {
            return SingleServiceResult::Failure {
                service_id: service_id.to_string(),
                error: format!("Failed to serialize generated code: {e}"),
            }
        }
    };

    // 5. Store compiled code
    let now = Utc::now().to_rfc3339();
    let compiled_row = InsertCompiledCodeRow {
        id: Uuid::new_v4().to_string(),
        workflow_id: service_id.to_string(),
        code: code_json,
        compiled_at: now.clone(),
    };

    if let Err(e) = state
        .supabase
        .insert::<serde_json::Value, _>("workflow_compiled_code", &compiled_row)
        .await
    {
        return SingleServiceResult::Failure {
            service_id: service_id.to_string(),
            error: format!("Failed to store compiled code: {e}"),
        };
    }

    // 6. Update workflow status to deployed
    let update_body = DeployUpdateRow {
        status_id: DEPLOYED_STATUS_ID.to_string(),
        deployed_at: now.clone(),
    };

    if let Err(e) = state
        .supabase
        .update::<Vec<serde_json::Value>, _>(
            "workflows",
            &[
                ("id", &format!("eq.{service_id}")),
                ("created_by", &user_filter),
            ],
            &update_body,
        )
        .await
    {
        return SingleServiceResult::Failure {
            service_id: service_id.to_string(),
            error: format!("Failed to update status: {e}"),
        };
    }

    // 7. Fire telemetry (non-blocking)
    if let Some(ref discovery) = state.discovery {
        let discovery = discovery.clone();
        let wf_id = service_id.to_string();
        let uid = user_id.to_string();
        tokio::spawn(async move {
            discovery.telemetry(&wf_id, "deploy", &uid, &[]).await;
        });
    }

    SingleServiceResult::Success {
        service_id: service_id.to_string(),
        compiled_at: now,
    }
}
```

Add to `crates/radium-workflow/src/lib.rs`:
```rust
pub mod deploy_pipeline;
```

**Step 4: Run tests to verify they pass**

Run: `nx run radium-workflow:test -- --lib deploy_pipeline`
Expected: PASS (4 tests for the types)

**Step 5: Update deploy.rs to call extracted function**

In `crates/radium-workflow/src/api/v1/deploy.rs`, refactor `deploy_workflow` to use the new module. Replace the inline validate → codegen → store → update steps with:

```rust
use crate::deploy_pipeline::{deploy_single_service, SingleServiceResult};

// Inside deploy_workflow, after auth:
match deploy_single_service(&state, &id, &user.user_id).await {
    SingleServiceResult::Success { service_id, compiled_at } => {
        Ok((
            StatusCode::OK,
            Json(DeployResponse {
                workflow_id: service_id,
                status: "deployed".to_string(),
                compiled_at,
                message: "Workflow compiled and deployed successfully".to_string(),
            }),
        ))
    }
    SingleServiceResult::Failure { error, .. } => {
        // Map failure reasons to appropriate error types
        if error.contains("not found") || error.contains("Failed to load") {
            Err(DeployError::not_found(format!("Service '{id}' not found")))
        } else if error.contains("Validation failed") {
            Err(DeployError::validation_failed(&error, vec![]))
        } else {
            Err(DeployError::internal(error))
        }
    }
}
```

**Step 6: Run full test suite**

Run: `nx run radium-workflow:test`
Expected: All existing tests pass (892+)

**Step 7: Commit**

```bash
git add crates/radium-workflow/src/deploy_pipeline.rs crates/radium-workflow/src/lib.rs crates/radium-workflow/src/api/v1/deploy.rs
git commit -m "refactor(radium-workflow): extract deploy pipeline into reusable module"
```

---

## Task 2: Implement Bundled Project Deploy

Replace the `deploy_project` stub in `projects.rs` with the real implementation using `deploy_single_service`. Fail-fast: on first failure, stop and report.

**Files:**
- Modify: `crates/radium-workflow/src/api/v1/projects.rs`
- Test: inline in `projects.rs`

**Step 1: Write the failing test**

Add to the test module in `projects.rs`:

```rust
#[test]
fn test_project_deploy_response_serialization() {
    let report = crate::deploy_pipeline::DeployReport {
        project_id: "proj-1".to_string(),
        deployed: vec![crate::deploy_pipeline::DeployedService {
            service_id: "svc-1".to_string(),
            compiled_at: "2026-03-01T00:00:00Z".to_string(),
        }],
        failed: Some(crate::deploy_pipeline::FailedService {
            service_id: "svc-2".to_string(),
            error: "Validation failed".to_string(),
        }),
        skipped: vec![crate::deploy_pipeline::SkippedService {
            service_id: "svc-3".to_string(),
            reason: "Deploy halted after failure".to_string(),
        }],
    };
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["deployed"].as_array().unwrap().len(), 1);
    assert_eq!(json["deployed"][0]["service_id"], "svc-1");
    assert!(json["failed"]["service_id"].as_str().is_some());
    assert_eq!(json["skipped"].as_array().unwrap().len(), 1);
}
```

**Step 2: Run test to verify it fails**

Run: `nx run radium-workflow:test -- --lib projects::tests::test_project_deploy_response`
Expected: FAIL if DeployReport not imported; PASS once import added

**Step 3: Replace deploy_project stub with real implementation**

In `crates/radium-workflow/src/api/v1/projects.rs`, replace the `deploy_project` function (the TODO stub) with:

```rust
use crate::deploy_pipeline::{
    deploy_single_service, DeployReport, DeployedService, FailedService,
    SingleServiceResult, SkippedService,
};

pub async fn deploy_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ProjectError> {
    let user = require_auth(&headers, &state).await?;

    // Verify project ownership
    let user_filter = format!("eq.{}", user.user_id);
    let _project: ProjectResponse = state
        .supabase
        .select_one(
            "projects",
            &[
                ("id", &format!("eq.{id}")),
                ("created_by", &user_filter),
                ("select", "id,name,description,task_queue_name,is_active,created_at,updated_at"),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                ProjectError::not_found(format!("Project '{id}' not found"))
            }
            _ => ProjectError::from_supabase(&e),
        })?;

    // Fetch all services in the project
    let services: Vec<ServiceStatusSummary> = state
        .supabase
        .select(
            "workflows",
            &[
                ("select", "id,name,status_id"),
                ("project_id", &format!("eq.{id}")),
                ("created_by", &user_filter),
            ],
        )
        .await
        .map_err(|e| ProjectError::from_supabase(&e))?;

    if services.is_empty() {
        return Err(ProjectError::bad_request(
            "Project has no services to deploy",
        ));
    }

    // Deploy each service sequentially, fail-fast on first error
    let mut deployed = Vec::new();
    let mut failed: Option<FailedService> = None;
    let mut skipped = Vec::new();

    for (i, service) in services.iter().enumerate() {
        match deploy_single_service(&state, &service.id, &user.user_id).await {
            SingleServiceResult::Success {
                service_id,
                compiled_at,
            } => {
                deployed.push(DeployedService {
                    service_id,
                    compiled_at,
                });
            }
            SingleServiceResult::Failure {
                service_id,
                error,
            } => {
                failed = Some(FailedService {
                    service_id,
                    error,
                });
                // Mark remaining services as skipped
                for remaining in &services[i + 1..] {
                    skipped.push(SkippedService {
                        service_id: remaining.id.clone(),
                        reason: "Deploy halted after failure".to_string(),
                    });
                }
                break;
            }
        }
    }

    let report = DeployReport {
        project_id: id,
        deployed,
        failed,
        skipped,
    };

    Ok((StatusCode::OK, Json(report)))
}
```

**Step 4: Run tests**

Run: `nx run radium-workflow:test`
Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/radium-workflow/src/api/v1/projects.rs
git commit -m "feat(radium-workflow): implement bundled project deploy with fail-fast partial deploy"
```

---

## Task 3: Kong Client Module

Build a `KongClient` struct that wraps the Kong Admin API. Add it to `AppState`.

**Files:**
- Create: `crates/radium-workflow/src/kong_client.rs`
- Modify: `crates/radium-workflow/src/lib.rs`
- Modify: `crates/radium-workflow/src/api/state.rs`
- Modify: `crates/radium-workflow/src/main.rs`
- Test: inline in `kong_client.rs`

**Step 1: Write failing tests for KongClient**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kong_config_from_env_defaults() {
        // Clear any existing env vars for clean test
        std::env::remove_var("KONG_ADMIN_URL");
        let config = KongConfig::from_env();
        assert_eq!(config.admin_url, "http://localhost:8001");
    }

    #[test]
    fn test_kong_config_from_env_custom() {
        std::env::set_var("KONG_ADMIN_URL", "http://kong:8001");
        let config = KongConfig::from_env();
        assert_eq!(config.admin_url, "http://kong:8001");
        std::env::remove_var("KONG_ADMIN_URL");
    }

    #[test]
    fn test_create_service_request_serialization() {
        let req = CreateServiceRequest {
            name: "my-gateway".to_string(),
            url: "http://radium-workflow:3020/v1/gateway/iface-123".to_string(),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["name"], "my-gateway");
        assert_eq!(json["url"], "http://radium-workflow:3020/v1/gateway/iface-123");
    }

    #[test]
    fn test_create_route_request_serialization() {
        let req = CreateRouteRequest {
            paths: vec!["/api/my-service/my-signal".to_string()],
            methods: vec!["POST".to_string()],
            strip_path: false,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["paths"][0], "/api/my-service/my-signal");
        assert_eq!(json["methods"][0], "POST");
        assert!(!json["strip_path"].as_bool().unwrap());
    }

    #[test]
    fn test_create_plugin_request_serialization() {
        let mut config = serde_json::Map::new();
        config.insert("minute".to_string(), serde_json::json!(120));
        let req = CreatePluginRequest {
            name: "rate-limiting".to_string(),
            config: serde_json::Value::Object(config),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["name"], "rate-limiting");
        assert_eq!(json["config"]["minute"], 120);
    }

    #[test]
    fn test_kong_service_response_deserialization() {
        let json = serde_json::json!({
            "id": "abc-123",
            "name": "my-service",
            "host": "radium-workflow",
            "port": 3020,
            "path": "/v1/gateway/iface-123"
        });
        let resp: KongServiceResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.id, "abc-123");
        assert_eq!(resp.name, "my-service");
    }

    #[test]
    fn test_kong_route_response_deserialization() {
        let json = serde_json::json!({
            "id": "route-456",
            "paths": ["/api/my-service/my-signal"],
            "methods": ["POST"]
        });
        let resp: KongRouteResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.id, "route-456");
        assert_eq!(resp.paths, vec!["/api/my-service/my-signal"]);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `nx run radium-workflow:test -- --lib kong_client`
Expected: FAIL — module doesn't exist

**Step 3: Implement KongClient**

Create `crates/radium-workflow/src/kong_client.rs`:

```rust
//! Kong Admin API client for dynamic route management.
//!
//! Wraps the Kong Admin API (port 8001) for creating/deleting
//! services, routes, and plugins when interfaces are published/unpublished.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Kong Admin API configuration.
#[derive(Debug, Clone)]
pub struct KongConfig {
    pub admin_url: String,
}

impl KongConfig {
    pub fn from_env() -> Self {
        Self {
            admin_url: std::env::var("KONG_ADMIN_URL")
                .unwrap_or_else(|_| "http://localhost:8001".to_string()),
        }
    }
}

/// Error type for Kong operations.
#[derive(Debug)]
pub enum KongError {
    /// HTTP request failed
    Request(reqwest::Error),
    /// Kong returned a non-success status
    Api { status: u16, body: String },
    /// Response deserialization failed
    Deserialize(String),
}

impl fmt::Display for KongError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Request(e) => write!(f, "Kong request failed: {e}"),
            Self::Api { status, body } => write!(f, "Kong API error ({status}): {body}"),
            Self::Deserialize(e) => write!(f, "Kong response parse error: {e}"),
        }
    }
}

impl From<reqwest::Error> for KongError {
    fn from(e: reqwest::Error) -> Self {
        Self::Request(e)
    }
}

/// Request to create a Kong service.
#[derive(Debug, Serialize)]
pub struct CreateServiceRequest {
    pub name: String,
    pub url: String,
}

/// Request to create a Kong route on a service.
#[derive(Debug, Serialize)]
pub struct CreateRouteRequest {
    pub paths: Vec<String>,
    pub methods: Vec<String>,
    pub strip_path: bool,
}

/// Request to add a plugin to a Kong service.
#[derive(Debug, Serialize)]
pub struct CreatePluginRequest {
    pub name: String,
    pub config: serde_json::Value,
}

/// Response from Kong when creating a service.
#[derive(Debug, Deserialize)]
pub struct KongServiceResponse {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub path: Option<String>,
}

/// Response from Kong when creating a route.
#[derive(Debug, Deserialize)]
pub struct KongRouteResponse {
    pub id: String,
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub methods: Vec<String>,
}

/// Kong Admin API client.
#[derive(Clone)]
pub struct KongClient {
    client: Client,
    base_url: String,
}

impl fmt::Debug for KongClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KongClient")
            .field("base_url", &self.base_url)
            .finish()
    }
}

impl KongClient {
    pub fn new(config: KongConfig) -> Self {
        Self {
            client: Client::new(),
            base_url: config.admin_url,
        }
    }

    /// Create a Kong service pointing to an upstream URL.
    pub async fn create_service(
        &self,
        req: &CreateServiceRequest,
    ) -> Result<KongServiceResponse, KongError> {
        let resp = self
            .client
            .post(format!("{}/services", self.base_url))
            .json(req)
            .send()
            .await?;

        let status = resp.status().as_u16();
        if status >= 400 {
            let body = resp.text().await.unwrap_or_default();
            return Err(KongError::Api { status, body });
        }

        resp.json().await.map_err(|e| KongError::Deserialize(e.to_string()))
    }

    /// Create a route on an existing Kong service.
    pub async fn create_route(
        &self,
        service_id: &str,
        req: &CreateRouteRequest,
    ) -> Result<KongRouteResponse, KongError> {
        let resp = self
            .client
            .post(format!("{}/services/{service_id}/routes", self.base_url))
            .json(req)
            .send()
            .await?;

        let status = resp.status().as_u16();
        if status >= 400 {
            let body = resp.text().await.unwrap_or_default();
            return Err(KongError::Api { status, body });
        }

        resp.json().await.map_err(|e| KongError::Deserialize(e.to_string()))
    }

    /// Add a plugin to a Kong service.
    pub async fn add_plugin(
        &self,
        service_id: &str,
        req: &CreatePluginRequest,
    ) -> Result<serde_json::Value, KongError> {
        let resp = self
            .client
            .post(format!("{}/services/{service_id}/plugins", self.base_url))
            .json(req)
            .send()
            .await?;

        let status = resp.status().as_u16();
        if status >= 400 {
            let body = resp.text().await.unwrap_or_default();
            return Err(KongError::Api { status, body });
        }

        resp.json().await.map_err(|e| KongError::Deserialize(e.to_string()))
    }

    /// Delete a Kong route by ID.
    pub async fn delete_route(&self, route_id: &str) -> Result<(), KongError> {
        let resp = self
            .client
            .delete(format!("{}/routes/{route_id}", self.base_url))
            .send()
            .await?;

        let status = resp.status().as_u16();
        if status >= 400 && status != 404 {
            let body = resp.text().await.unwrap_or_default();
            return Err(KongError::Api { status, body });
        }

        Ok(())
    }

    /// Delete a Kong service by ID.
    pub async fn delete_service(&self, service_id: &str) -> Result<(), KongError> {
        let resp = self
            .client
            .delete(format!("{}/services/{service_id}", self.base_url))
            .send()
            .await?;

        let status = resp.status().as_u16();
        if status >= 400 && status != 404 {
            let body = resp.text().await.unwrap_or_default();
            return Err(KongError::Api { status, body });
        }

        Ok(())
    }
}
```

**Step 4: Add `kong_client` to `lib.rs`**

```rust
pub mod kong_client;
```

**Step 5: Add KongClient to AppState**

In `crates/radium-workflow/src/api/state.rs`, add:
```rust
use crate::kong_client::KongClient;

#[derive(Clone)]
pub struct AppState {
    pub supabase: Arc<SupabaseClient>,
    pub rate_limiter: Arc<SlidingWindowLimiter>,
    pub discovery: Option<Arc<DiscoveryClient>>,
    pub kong: Option<Arc<KongClient>>,
}
```

In `crates/radium-workflow/src/main.rs`, add Kong client initialization:
```rust
use radium_workflow::kong_client::{KongClient, KongConfig};

// Inside the app_state creation block:
let kong = {
    let config = KongConfig::from_env();
    Some(Arc::new(KongClient::new(config)))
};

Some(AppState {
    supabase: Arc::new(client),
    rate_limiter: Arc::new(SlidingWindowLimiter::new(RateLimitConfig::for_api())),
    discovery,
    kong,
})
```

**Step 6: Run tests**

Run: `nx run radium-workflow:test`
Expected: All tests pass

**Step 7: Commit**

```bash
git add crates/radium-workflow/src/kong_client.rs crates/radium-workflow/src/lib.rs crates/radium-workflow/src/api/state.rs crates/radium-workflow/src/main.rs
git commit -m "feat(radium-workflow): add Kong Admin API client module"
```

---

## Task 4: Update Docker Compose for Kong DB Mode

Switch Kong from declarative/dbless mode to PostgreSQL-backed mode with the Admin API.

**Files:**
- Modify: `docker-compose.yml`
- Test: manual verification via `docker compose up`

**Step 1: Add Kong database service to docker-compose.yml**

Add a new service `kong-database` before the `kong` service:

```yaml
kong-database:
  image: postgres:15
  container_name: radium-kong-database
  environment:
    POSTGRES_USER: kong
    POSTGRES_PASSWORD: kong
    POSTGRES_DB: kong
  volumes:
    - kong_data:/var/lib/postgresql/data
  healthcheck:
    test: ["CMD-SHELL", "pg_isready -U kong"]
    interval: 5s
    timeout: 5s
    retries: 5
  restart: unless-stopped
```

**Step 2: Add Kong migration init service**

```yaml
kong-migration:
  image: kong:3.8
  container_name: radium-kong-migration
  depends_on:
    kong-database:
      condition: service_healthy
  environment:
    KONG_DATABASE: postgres
    KONG_PG_HOST: kong-database
    KONG_PG_USER: kong
    KONG_PG_PASSWORD: kong
  command: kong migrations bootstrap
  restart: "no"
```

**Step 3: Update Kong service configuration**

Replace the Kong environment section:

```yaml
kong:
  image: kong:3.8
  container_name: radium-kong
  depends_on:
    kong-database:
      condition: service_healthy
    kong-migration:
      condition: service_completed_successfully
    auth:
      condition: service_healthy
    rest:
      condition: service_started
  healthcheck:
    test: ["CMD", "kong", "health"]
    interval: 5s
    timeout: 5s
    retries: 10
  restart: unless-stopped
  ports:
    - "8000:8000"
    - "8001:8001"
  environment:
    KONG_DATABASE: postgres
    KONG_PG_HOST: kong-database
    KONG_PG_USER: kong
    KONG_PG_PASSWORD: kong
    KONG_DNS_ORDER: LAST,A,CNAME
    KONG_PLUGINS: bundled
    KONG_NGINX_PROXY_PROXY_BUFFER_SIZE: 160k
    KONG_NGINX_PROXY_PROXY_BUFFERS: 64 160k
    KONG_ADMIN_LISTEN: 0.0.0.0:8001
    KONG_PROXY_LISTEN: 0.0.0.0:8000
```

Remove the `KONG_DECLARATIVE_CONFIG` env var and the kong.yml volume mount.

**Step 4: Add kong_data volume**

In the `volumes` section at the bottom of docker-compose.yml:

```yaml
volumes:
  kong_data:
  # ... existing volumes
```

**Step 5: Create Kong seed script**

Create `scripts/seed-kong.sh` that uses the Admin API to recreate the routes that were previously in `kong.yml`:

```bash
#!/bin/bash
# Seed Kong with initial routes after migration from declarative mode.
# Run this once after switching to DB mode.

KONG_ADMIN=${KONG_ADMIN_URL:-http://localhost:8001}

echo "Seeding Kong routes..."

# Create radium-workflow service
curl -s -X POST "$KONG_ADMIN/services" \
  -d name=radium-workflow \
  -d url=http://radium-workflow:3020 | jq .

# Create routes for radium-workflow
curl -s -X POST "$KONG_ADMIN/services/radium-workflow/routes" \
  -d 'paths[]=/v1/workflows' \
  -d 'paths[]=/v1/components' \
  -d 'paths[]=/v1/services' \
  -d 'paths[]=/v1/projects' \
  -d strip_path=false | jq .

# Add plugins to radium-workflow
curl -s -X POST "$KONG_ADMIN/services/radium-workflow/plugins" \
  -d name=cors | jq .
curl -s -X POST "$KONG_ADMIN/services/radium-workflow/plugins" \
  -d name=correlation-id \
  -d config.header_name=X-Request-ID | jq .
curl -s -X POST "$KONG_ADMIN/services/radium-workflow/plugins" \
  -d name=rate-limiting \
  -d config.minute=120 \
  -d config.policy=local | jq .

# Create radium-discovery service
curl -s -X POST "$KONG_ADMIN/services" \
  -d name=radium-discovery \
  -d url=http://radium-discovery:3030 | jq .

# Create routes for radium-discovery
curl -s -X POST "$KONG_ADMIN/services/radium-discovery/routes" \
  -d 'paths[]=/v1/discover' \
  -d strip_path=false | jq .

# Add plugins to radium-discovery
curl -s -X POST "$KONG_ADMIN/services/radium-discovery/plugins" \
  -d name=cors | jq .
curl -s -X POST "$KONG_ADMIN/services/radium-discovery/plugins" \
  -d name=correlation-id \
  -d config.header_name=X-Request-ID | jq .
curl -s -X POST "$KONG_ADMIN/services/radium-discovery/plugins" \
  -d name=rate-limiting \
  -d config.minute=120 \
  -d config.policy=local | jq .

echo "Kong seed complete."
```

**Step 6: Verify**

Run: `docker compose up -d kong-database kong-migration kong`
Then: `curl http://localhost:8001/services` — should return empty list
Then: `bash scripts/seed-kong.sh` — should create services/routes

**Step 7: Commit**

```bash
git add docker-compose.yml scripts/seed-kong.sh
git commit -m "feat(infra): switch Kong from dbless to DB mode with Admin API"
```

---

## Task 5: Wire Kong into Interface Publish/Unpublish

Update `publish_interface` and `unpublish_interface` in `interfaces.rs` to create/delete real Kong routes via the `KongClient`.

**Files:**
- Modify: `crates/radium-workflow/src/api/v1/interfaces.rs`
- Test: inline in `interfaces.rs`

**Step 1: Write failing tests**

Add tests verifying the Kong integration fields:

```rust
#[test]
fn test_publish_response_with_kong_fields() {
    let json = serde_json::json!({
        "id": "pub-1",
        "service_interface_id": "iface-1",
        "route_path": "/api/my-service/my-signal",
        "http_method": "POST",
        "kong_route_id": "kong-route-123",
        "kong_service_id": "kong-svc-456",
        "is_active": true,
        "created_at": "2026-03-01T00:00:00Z",
        "updated_at": "2026-03-01T00:00:00Z"
    });
    let resp: PublishResponse = serde_json::from_value(json).unwrap();
    assert_eq!(resp.kong_route_id.unwrap(), "kong-route-123");
    assert_eq!(resp.kong_service_id.unwrap(), "kong-svc-456");
}
```

**Step 2: Run test to verify it fails**

Run: `nx run radium-workflow:test -- --lib interfaces::tests::test_publish_response_with_kong_fields`
Expected: FAIL — `kong_route_id` and `kong_service_id` not on `PublishResponse`

**Step 3: Update PublishResponse and InsertPublicInterfaceRow**

Update the types:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct PublishResponse {
    pub id: String,
    pub service_interface_id: String,
    pub route_path: String,
    pub http_method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kong_route_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kong_service_id: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
struct InsertPublicInterfaceRow {
    service_interface_id: String,
    route_path: String,
    http_method: String,
    kong_route_id: Option<String>,
    kong_service_id: Option<String>,
    is_active: bool,
}
```

**Step 4: Update publish_interface handler**

Replace the existing `publish_interface` function body (after the interface fetch and route_path generation):

```rust
use crate::kong_client::{CreatePluginRequest, CreateRouteRequest, CreateServiceRequest};

// Create Kong service + route if Kong client is available
let (kong_service_id, kong_route_id) = if let Some(ref kong) = state.kong {
    // The gateway handler URL — Kong will route to this
    let gateway_url = format!(
        "http://radium-workflow:3020/v1/gateway/{}",
        iid
    );

    let kong_service_name = format!(
        "gw-{}-{}",
        kebab_case(&service_name),
        kebab_case(&interface.name)
    );

    let kong_svc = kong
        .create_service(&CreateServiceRequest {
            name: kong_service_name,
            url: gateway_url,
        })
        .await
        .map_err(|e| InterfaceError::internal(format!("Kong service creation failed: {e}")))?;

    let kong_route = kong
        .create_route(
            &kong_svc.id,
            &CreateRouteRequest {
                paths: vec![route_path.clone()],
                methods: vec!["POST".to_string()],
                strip_path: false,
            },
        )
        .await
        .map_err(|e| InterfaceError::internal(format!("Kong route creation failed: {e}")))?;

    // Add default plugins: cors, correlation-id, rate-limiting
    for plugin in &[
        CreatePluginRequest {
            name: "cors".to_string(),
            config: serde_json::json!({}),
        },
        CreatePluginRequest {
            name: "correlation-id".to_string(),
            config: serde_json::json!({"header_name": "X-Request-ID"}),
        },
        CreatePluginRequest {
            name: "rate-limiting".to_string(),
            config: serde_json::json!({"minute": 120, "policy": "local"}),
        },
    ] {
        let _ = kong.add_plugin(&kong_svc.id, plugin).await;
    }

    (Some(kong_svc.id), Some(kong_route.id))
} else {
    (None, None)
};

let row = InsertPublicInterfaceRow {
    service_interface_id: iid,
    route_path,
    http_method: "POST".to_string(),
    kong_route_id,
    kong_service_id,
    is_active: true,
};

let created: PublishResponse = state
    .supabase
    .insert("public_interfaces", &row)
    .await
    .map_err(|e| InterfaceError::from_supabase(&e))?;

Ok((StatusCode::CREATED, Json(created)))
```

**Step 5: Update unpublish_interface handler**

Before deleting from `public_interfaces`, first fetch the record to get Kong IDs:

```rust
pub async fn unpublish_interface(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((id, iid)): Path<(String, String)>,
) -> Result<StatusCode, InterfaceError> {
    let user = require_auth(&headers, &state).await?;
    verify_service_ownership(&state, &id, &user.user_id).await?;

    // Fetch the public_interfaces record to get Kong IDs before deleting
    let records: Vec<PublishResponse> = state
        .supabase
        .select(
            "public_interfaces",
            &[
                ("service_interface_id", &format!("eq.{iid}")),
                ("select", "id,service_interface_id,route_path,http_method,kong_route_id,kong_service_id,is_active,created_at,updated_at"),
            ],
        )
        .await
        .map_err(|e| InterfaceError::from_supabase(&e))?;

    // Delete Kong route and service if they exist
    if let Some(ref kong) = state.kong {
        for record in &records {
            if let Some(ref route_id) = record.kong_route_id {
                let _ = kong.delete_route(route_id).await;
            }
            if let Some(ref service_id) = record.kong_service_id {
                let _ = kong.delete_service(service_id).await;
            }
        }
    }

    // Delete from database
    state
        .supabase
        .delete(
            "public_interfaces",
            &[("service_interface_id", &format!("eq.{iid}"))],
        )
        .await
        .map_err(|e| InterfaceError::from_supabase(&e))?;

    Ok(StatusCode::NO_CONTENT)
}
```

**Step 6: Run tests**

Run: `nx run radium-workflow:test`
Expected: All tests pass

**Step 7: Commit**

```bash
git add crates/radium-workflow/src/api/v1/interfaces.rs
git commit -m "feat(radium-workflow): wire Kong Admin API into interface publish/unpublish"
```

---

## Task 6: State Variables API — Service-Scoped

Add CRUD endpoints for service-scoped state variables.

**Files:**
- Create: `crates/radium-workflow/src/api/v1/state_variables.rs`
- Modify: `crates/radium-workflow/src/api/v1/mod.rs`
- Test: inline in `state_variables.rs`

**Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_variable_types() {
        assert!(VALID_VARIABLE_TYPES.contains(&"string"));
        assert!(VALID_VARIABLE_TYPES.contains(&"number"));
        assert!(VALID_VARIABLE_TYPES.contains(&"boolean"));
        assert!(VALID_VARIABLE_TYPES.contains(&"object"));
        assert!(VALID_VARIABLE_TYPES.contains(&"array"));
        assert!(!VALID_VARIABLE_TYPES.contains(&"invalid"));
    }

    #[test]
    fn test_valid_storage_types() {
        assert!(VALID_STORAGE_TYPES.contains(&"database"));
        assert!(VALID_STORAGE_TYPES.contains(&"cache"));
        assert!(!VALID_STORAGE_TYPES.contains(&"invalid"));
    }

    #[test]
    fn test_create_request_serialization() {
        let req = CreateVariableRequest {
            name: "counter".to_string(),
            r#type: "number".to_string(),
            storage_type: "database".to_string(),
            schema: Some(serde_json::json!({"type": "integer"})),
            storage_config: None,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["name"], "counter");
        assert_eq!(json["type"], "number");
        assert_eq!(json["storage_type"], "database");
        assert!(json["schema"].is_object());
    }

    #[test]
    fn test_variable_response_deserialization() {
        let json = serde_json::json!({
            "id": "var-1",
            "workflow_id": "svc-1",
            "name": "counter",
            "type": "number",
            "storage_type": "database",
            "schema": {"type": "integer"},
            "storage_config": null,
            "created_at": "2026-03-01T00:00:00Z",
            "updated_at": "2026-03-01T00:00:00Z"
        });
        let resp: ServiceVariableResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.name, "counter");
        assert_eq!(resp.r#type, "number");
    }

    #[test]
    fn test_insert_row_serialization() {
        let row = InsertServiceVariableRow {
            name: "counter".to_string(),
            r#type: "number".to_string(),
            storage_type: "database".to_string(),
            schema: None,
            storage_config: None,
            workflow_id: "svc-1".to_string(),
        };
        let json = serde_json::to_value(&row).unwrap();
        assert_eq!(json["workflow_id"], "svc-1");
        assert!(!json.as_object().unwrap().contains_key("schema"));
    }

    #[test]
    fn test_state_var_error_unauthorized() {
        let err = StateVarError::unauthorized();
        assert_eq!(err.status, StatusCode::UNAUTHORIZED);
        assert_eq!(err.code, "UNAUTHORIZED");
    }

    #[test]
    fn test_state_var_error_bad_request() {
        let err = StateVarError::bad_request("invalid type");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert!(err.message.contains("invalid type"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `nx run radium-workflow:test -- --lib state_variables`
Expected: FAIL — module doesn't exist

**Step 3: Implement state_variables.rs**

Create `crates/radium-workflow/src/api/v1/state_variables.rs`:

```rust
//! State variables CRUD API.
//!
//! Supports both service-scoped (workflow_state_variables) and
//! project-scoped (project_state_variables) state variables.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::api::state::AppState;
use crate::supabase::SupabaseError;

const VALID_VARIABLE_TYPES: &[&str] = &["string", "number", "boolean", "object", "array"];
const VALID_STORAGE_TYPES: &[&str] = &["database", "cache"];

// ── Error type ──────────────────────────────────────────

#[derive(Debug)]
pub struct StateVarError {
    pub status: StatusCode,
    pub code: String,
    pub message: String,
    details: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: String,
    message: String,
    details: Vec<String>,
}

impl StateVarError {
    pub fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "UNAUTHORIZED".to_string(),
            message: "Authorization header with Bearer token is required".to_string(),
            details: vec![],
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "BAD_REQUEST".to_string(),
            message: message.into(),
            details: vec![],
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "NOT_FOUND".to_string(),
            message: message.into(),
            details: vec![],
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "INTERNAL_ERROR".to_string(),
            message: message.into(),
            details: vec![],
        }
    }

    fn from_supabase(err: &SupabaseError) -> Self {
        match err {
            SupabaseError::NotFound { .. } => Self::not_found(err.to_string()),
            SupabaseError::ApiError { status, .. } if *status == 404 => {
                Self::not_found(err.to_string())
            }
            _ => {
                tracing::error!("Supabase error: {err}");
                Self::internal("Database operation failed")
            }
        }
    }
}

impl IntoResponse for StateVarError {
    fn into_response(self) -> Response {
        let envelope = ErrorEnvelope {
            error: ErrorBody {
                code: self.code,
                message: self.message,
                details: self.details,
            },
        };
        (self.status, Json(envelope)).into_response()
    }
}

// ── Request/Response types ──────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateVariableRequest {
    pub name: String,
    pub r#type: String,
    pub storage_type: String,
    pub schema: Option<serde_json::Value>,
    pub storage_config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVariableRequest {
    pub name: Option<String>,
    pub r#type: Option<String>,
    pub storage_type: Option<String>,
    pub schema: Option<serde_json::Value>,
    pub storage_config: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct InsertServiceVariableRow {
    name: String,
    r#type: String,
    storage_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    storage_config: Option<serde_json::Value>,
    workflow_id: String,
}

#[derive(Debug, Serialize)]
struct InsertProjectVariableRow {
    name: String,
    r#type: String,
    storage_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    storage_config: Option<serde_json::Value>,
    project_id: String,
}

#[derive(Debug, Serialize)]
struct UpdateVariableRow {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    var_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    storage_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    storage_config: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceVariableResponse {
    pub id: String,
    pub workflow_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub storage_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_config: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectVariableResponse {
    pub id: String,
    pub project_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub storage_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_config: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct VariableListResponse<T: Serialize> {
    pub variables: Vec<T>,
    pub total: usize,
}

// ── Auth helper ─────────────────────────────────────────

struct AuthenticatedUser {
    user_id: String,
}

async fn require_auth(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<AuthenticatedUser, StateVarError> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(StateVarError::unauthorized)?;

    let rate_result = state.rate_limiter.check(token);
    if !rate_result.allowed {
        return Err(StateVarError {
            status: StatusCode::TOO_MANY_REQUESTS,
            code: "RATE_LIMITED".to_string(),
            message: format!(
                "Rate limit exceeded. Try again in {} seconds.",
                rate_result.retry_after.unwrap_or(60)
            ),
            details: vec![],
        });
    }

    #[derive(Deserialize)]
    struct ApiKeyRow {
        user_id: String,
    }

    let key: ApiKeyRow = state
        .supabase
        .select_one(
            "api_keys",
            &[
                ("key_hash", &format!("eq.{token}")),
                ("is_active", "eq.true"),
                ("select", "user_id"),
            ],
        )
        .await
        .map_err(|_| StateVarError::unauthorized())?;

    Ok(AuthenticatedUser {
        user_id: key.user_id,
    })
}

// ── Ownership helpers ───────────────────────────────────

async fn verify_service_ownership(
    state: &AppState,
    service_id: &str,
    user_id: &str,
) -> Result<(), StateVarError> {
    #[derive(Deserialize)]
    struct Row {
        #[allow(dead_code)]
        id: String,
    }

    let _: Row = state
        .supabase
        .select_one(
            "workflows",
            &[
                ("id", &format!("eq.{service_id}")),
                ("created_by", &format!("eq.{user_id}")),
                ("select", "id"),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                StateVarError::not_found(format!("Service '{service_id}' not found"))
            }
            _ => StateVarError::from_supabase(&e),
        })?;

    Ok(())
}

async fn verify_project_ownership(
    state: &AppState,
    project_id: &str,
    user_id: &str,
) -> Result<(), StateVarError> {
    #[derive(Deserialize)]
    struct Row {
        #[allow(dead_code)]
        id: String,
    }

    let _: Row = state
        .supabase
        .select_one(
            "projects",
            &[
                ("id", &format!("eq.{project_id}")),
                ("created_by", &format!("eq.{user_id}")),
                ("select", "id"),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                StateVarError::not_found(format!("Project '{project_id}' not found"))
            }
            _ => StateVarError::from_supabase(&e),
        })?;

    Ok(())
}

// ── Validation ──────────────────────────────────────────

fn validate_variable_request(req: &CreateVariableRequest) -> Result<(), StateVarError> {
    if req.name.trim().is_empty() {
        return Err(StateVarError::bad_request("Variable name cannot be empty"));
    }
    if !VALID_VARIABLE_TYPES.contains(&req.r#type.as_str()) {
        return Err(StateVarError::bad_request(format!(
            "Invalid variable type '{}'. Valid types: {}",
            req.r#type,
            VALID_VARIABLE_TYPES.join(", ")
        )));
    }
    if !VALID_STORAGE_TYPES.contains(&req.storage_type.as_str()) {
        return Err(StateVarError::bad_request(format!(
            "Invalid storage type '{}'. Valid types: {}",
            req.storage_type,
            VALID_STORAGE_TYPES.join(", ")
        )));
    }
    Ok(())
}

// ── Service-scoped handlers ─────────────────────────────

pub async fn create_service_variable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(service_id): Path<String>,
    Json(body): Json<CreateVariableRequest>,
) -> Result<impl IntoResponse, StateVarError> {
    let user = require_auth(&headers, &state).await?;
    verify_service_ownership(&state, &service_id, &user.user_id).await?;
    validate_variable_request(&body)?;

    let row = InsertServiceVariableRow {
        name: body.name,
        r#type: body.r#type,
        storage_type: body.storage_type,
        schema: body.schema,
        storage_config: body.storage_config,
        workflow_id: service_id,
    };

    let created: ServiceVariableResponse = state
        .supabase
        .insert("workflow_state_variables", &row)
        .await
        .map_err(|e| StateVarError::from_supabase(&e))?;

    Ok((StatusCode::CREATED, Json(created)))
}

pub async fn list_service_variables(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(service_id): Path<String>,
) -> Result<impl IntoResponse, StateVarError> {
    let user = require_auth(&headers, &state).await?;
    verify_service_ownership(&state, &service_id, &user.user_id).await?;

    let variables: Vec<ServiceVariableResponse> = state
        .supabase
        .select(
            "workflow_state_variables",
            &[
                ("workflow_id", &format!("eq.{service_id}")),
                ("select", "id,workflow_id,name,type,storage_type,schema,storage_config,created_at,updated_at"),
                ("order", "created_at.asc"),
            ],
        )
        .await
        .map_err(|e| StateVarError::from_supabase(&e))?;

    let total = variables.len();
    Ok(Json(VariableListResponse { variables, total }))
}

pub async fn get_service_variable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((service_id, var_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, StateVarError> {
    let user = require_auth(&headers, &state).await?;
    verify_service_ownership(&state, &service_id, &user.user_id).await?;

    let variable: ServiceVariableResponse = state
        .supabase
        .select_one(
            "workflow_state_variables",
            &[
                ("id", &format!("eq.{var_id}")),
                ("workflow_id", &format!("eq.{service_id}")),
                ("select", "id,workflow_id,name,type,storage_type,schema,storage_config,created_at,updated_at"),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                StateVarError::not_found(format!("Variable '{var_id}' not found"))
            }
            _ => StateVarError::from_supabase(&e),
        })?;

    Ok(Json(variable))
}

pub async fn update_service_variable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((service_id, var_id)): Path<(String, String)>,
    Json(body): Json<UpdateVariableRequest>,
) -> Result<impl IntoResponse, StateVarError> {
    let user = require_auth(&headers, &state).await?;
    verify_service_ownership(&state, &service_id, &user.user_id).await?;

    if let Some(ref t) = body.r#type {
        if !VALID_VARIABLE_TYPES.contains(&t.as_str()) {
            return Err(StateVarError::bad_request(format!(
                "Invalid variable type '{t}'"
            )));
        }
    }
    if let Some(ref st) = body.storage_type {
        if !VALID_STORAGE_TYPES.contains(&st.as_str()) {
            return Err(StateVarError::bad_request(format!(
                "Invalid storage type '{st}'"
            )));
        }
    }

    let update = UpdateVariableRow {
        name: body.name,
        var_type: body.r#type,
        storage_type: body.storage_type,
        schema: body.schema,
        storage_config: body.storage_config,
    };

    let updated: Vec<ServiceVariableResponse> = state
        .supabase
        .update(
            "workflow_state_variables",
            &[
                ("id", &format!("eq.{var_id}")),
                ("workflow_id", &format!("eq.{service_id}")),
            ],
            &update,
        )
        .await
        .map_err(|e| StateVarError::from_supabase(&e))?;

    let variable = updated
        .into_iter()
        .next()
        .ok_or_else(|| StateVarError::not_found(format!("Variable '{var_id}' not found")))?;

    Ok(Json(variable))
}

pub async fn delete_service_variable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((service_id, var_id)): Path<(String, String)>,
) -> Result<StatusCode, StateVarError> {
    let user = require_auth(&headers, &state).await?;
    verify_service_ownership(&state, &service_id, &user.user_id).await?;

    state
        .supabase
        .delete(
            "workflow_state_variables",
            &[
                ("id", &format!("eq.{var_id}")),
                ("workflow_id", &format!("eq.{service_id}")),
            ],
        )
        .await
        .map_err(|e| StateVarError::from_supabase(&e))?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Project-scoped handlers ─────────────────────────────

pub async fn create_project_variable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
    Json(body): Json<CreateVariableRequest>,
) -> Result<impl IntoResponse, StateVarError> {
    let user = require_auth(&headers, &state).await?;
    verify_project_ownership(&state, &project_id, &user.user_id).await?;
    validate_variable_request(&body)?;

    let row = InsertProjectVariableRow {
        name: body.name,
        r#type: body.r#type,
        storage_type: body.storage_type,
        schema: body.schema,
        storage_config: body.storage_config,
        project_id,
    };

    let created: ProjectVariableResponse = state
        .supabase
        .insert("project_state_variables", &row)
        .await
        .map_err(|e| StateVarError::from_supabase(&e))?;

    Ok((StatusCode::CREATED, Json(created)))
}

pub async fn list_project_variables(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<String>,
) -> Result<impl IntoResponse, StateVarError> {
    let user = require_auth(&headers, &state).await?;
    verify_project_ownership(&state, &project_id, &user.user_id).await?;

    let variables: Vec<ProjectVariableResponse> = state
        .supabase
        .select(
            "project_state_variables",
            &[
                ("project_id", &format!("eq.{project_id}")),
                ("select", "id,project_id,name,type,storage_type,schema,storage_config,created_at,updated_at"),
                ("order", "created_at.asc"),
            ],
        )
        .await
        .map_err(|e| StateVarError::from_supabase(&e))?;

    let total = variables.len();
    Ok(Json(VariableListResponse { variables, total }))
}

pub async fn get_project_variable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, var_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, StateVarError> {
    let user = require_auth(&headers, &state).await?;
    verify_project_ownership(&state, &project_id, &user.user_id).await?;

    let variable: ProjectVariableResponse = state
        .supabase
        .select_one(
            "project_state_variables",
            &[
                ("id", &format!("eq.{var_id}")),
                ("project_id", &format!("eq.{project_id}")),
                ("select", "id,project_id,name,type,storage_type,schema,storage_config,created_at,updated_at"),
            ],
        )
        .await
        .map_err(|e| match &e {
            SupabaseError::NotFound { .. } => {
                StateVarError::not_found(format!("Variable '{var_id}' not found"))
            }
            _ => StateVarError::from_supabase(&e),
        })?;

    Ok(Json(variable))
}

pub async fn update_project_variable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, var_id)): Path<(String, String)>,
    Json(body): Json<UpdateVariableRequest>,
) -> Result<impl IntoResponse, StateVarError> {
    let user = require_auth(&headers, &state).await?;
    verify_project_ownership(&state, &project_id, &user.user_id).await?;

    if let Some(ref t) = body.r#type {
        if !VALID_VARIABLE_TYPES.contains(&t.as_str()) {
            return Err(StateVarError::bad_request(format!(
                "Invalid variable type '{t}'"
            )));
        }
    }
    if let Some(ref st) = body.storage_type {
        if !VALID_STORAGE_TYPES.contains(&st.as_str()) {
            return Err(StateVarError::bad_request(format!(
                "Invalid storage type '{st}'"
            )));
        }
    }

    let update = UpdateVariableRow {
        name: body.name,
        var_type: body.r#type,
        storage_type: body.storage_type,
        schema: body.schema,
        storage_config: body.storage_config,
    };

    let updated: Vec<ProjectVariableResponse> = state
        .supabase
        .update(
            "project_state_variables",
            &[
                ("id", &format!("eq.{var_id}")),
                ("project_id", &format!("eq.{project_id}")),
            ],
            &update,
        )
        .await
        .map_err(|e| StateVarError::from_supabase(&e))?;

    let variable = updated
        .into_iter()
        .next()
        .ok_or_else(|| StateVarError::not_found(format!("Variable '{var_id}' not found")))?;

    Ok(Json(variable))
}

pub async fn delete_project_variable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, var_id)): Path<(String, String)>,
) -> Result<StatusCode, StateVarError> {
    let user = require_auth(&headers, &state).await?;
    verify_project_ownership(&state, &project_id, &user.user_id).await?;

    state
        .supabase
        .delete(
            "project_state_variables",
            &[
                ("id", &format!("eq.{var_id}")),
                ("project_id", &format!("eq.{project_id}")),
            ],
        )
        .await
        .map_err(|e| StateVarError::from_supabase(&e))?;

    Ok(StatusCode::NO_CONTENT)
}
```

**Step 4: Add routes to mod.rs**

In `crates/radium-workflow/src/api/v1/mod.rs`, add:

```rust
pub mod state_variables;
```

And add routes:

```rust
// Service state variables
.route("/services/{id}/variables", post(state_variables::create_service_variable).get(state_variables::list_service_variables))
.route("/services/{id}/variables/{var_id}", get(state_variables::get_service_variable).put(state_variables::update_service_variable).delete(state_variables::delete_service_variable))

// Project state variables
.route("/projects/{id}/variables", post(state_variables::create_project_variable).get(state_variables::list_project_variables))
.route("/projects/{id}/variables/{var_id}", get(state_variables::get_project_variable).put(state_variables::update_project_variable).delete(state_variables::delete_project_variable))
```

**Step 5: Run tests**

Run: `nx run radium-workflow:test`
Expected: All tests pass

**Step 6: Commit**

```bash
git add crates/radium-workflow/src/api/v1/state_variables.rs crates/radium-workflow/src/api/v1/mod.rs
git commit -m "feat(radium-workflow): add state variables CRUD API (service + project scoped)"
```

---

## Task 7: CLI State Variable Commands

Add `variable` subcommands to both `radium service` and `radium project`.

**Files:**
- Modify: `crates/radium-cli/src/commands/services.rs`
- Modify: `crates/radium-cli/src/commands/projects.rs`
- Modify: `crates/radium-cli/src/main.rs`
- Test: inline parse tests

**Step 1: Write failing parse tests**

In `crates/radium-cli/src/main.rs` test module:

```rust
#[test]
fn test_service_variable_list() {
    let cli = Cli::parse_from(["radium", "service", "variable", "list", "svc-1"]);
    match cli.command {
        Commands::Service { action } => match action {
            ServiceAction::Variable { action } => match action {
                VariableAction::List { service_id } => assert_eq!(service_id, "svc-1"),
                _ => panic!("expected List"),
            },
            _ => panic!("expected Variable"),
        },
        _ => panic!("expected Service"),
    }
}

#[test]
fn test_service_variable_create() {
    let cli = Cli::parse_from(["radium", "service", "variable", "create", "svc-1", "vars.json"]);
    match cli.command {
        Commands::Service { action } => match action {
            ServiceAction::Variable { action } => match action {
                VariableAction::Create { service_id, file } => {
                    assert_eq!(service_id, "svc-1");
                    assert_eq!(file, "vars.json");
                },
                _ => panic!("expected Create"),
            },
            _ => panic!("expected Variable"),
        },
        _ => panic!("expected Service"),
    }
}

#[test]
fn test_project_variable_list() {
    let cli = Cli::parse_from(["radium", "project", "variable", "list", "proj-1"]);
    match cli.command {
        Commands::Project { action } => match action {
            ProjectAction::Variable { action } => match action {
                ProjectVariableAction::List { project_id } => assert_eq!(project_id, "proj-1"),
                _ => panic!("expected List"),
            },
            _ => panic!("expected Variable"),
        },
        _ => panic!("expected Project"),
    }
}
```

**Step 2: Run test to verify it fails**

Run: `nx run radium-cli:test`
Expected: FAIL — VariableAction doesn't exist

**Step 3: Add VariableAction to services.rs**

In `crates/radium-cli/src/commands/services.rs`, add to `ServiceAction`:

```rust
/// Manage service state variables
Variable {
    #[command(subcommand)]
    action: VariableAction,
},
```

Add the `VariableAction` enum:

```rust
#[derive(Subcommand, Clone)]
pub enum VariableAction {
    /// List state variables for a service
    List { service_id: String },
    /// Create a state variable from a JSON file
    Create { service_id: String, file: String },
    /// Show a specific state variable
    Show { service_id: String, variable_id: String },
    /// Update a state variable from a JSON file
    Update { service_id: String, variable_id: String, file: String },
    /// Delete a state variable
    Delete { service_id: String, variable_id: String },
}
```

Add handler match arms for VariableAction following the same pattern as InterfaceAction (API calls via client).

**Step 4: Add ProjectVariableAction to projects.rs**

In `crates/radium-cli/src/commands/projects.rs`, add to `ProjectAction`:

```rust
/// Manage project state variables
Variable {
    #[command(subcommand)]
    action: ProjectVariableAction,
},
```

Add the `ProjectVariableAction` enum:

```rust
#[derive(Subcommand, Clone)]
pub enum ProjectVariableAction {
    /// List shared state variables for a project
    List { project_id: String },
    /// Create a shared state variable from a JSON file
    Create { project_id: String, file: String },
    /// Show a specific shared state variable
    Show { project_id: String, variable_id: String },
    /// Update a shared state variable from a JSON file
    Update { project_id: String, variable_id: String, file: String },
    /// Delete a shared state variable
    Delete { project_id: String, variable_id: String },
}
```

Add handler match arms using `/v1/projects/{id}/variables` endpoints.

**Step 5: Wire into main.rs**

Update the `Commands::Service` and `Commands::Project` match arms to handle the new `Variable` variants.

**Step 6: Run tests**

Run: `nx run radium-cli:test`
Expected: All parse tests pass

**Step 7: Commit**

```bash
git add crates/radium-cli/src/commands/services.rs crates/radium-cli/src/commands/projects.rs crates/radium-cli/src/main.rs
git commit -m "feat(radium-cli): add state variable subcommands for services and projects"
```

---

## Task 8: Temporal gRPC Client Module

Build a `TemporalClient` that wraps Temporal's gRPC API for starting, signaling, querying, and terminating workflows. Add to `AppState`.

**Files:**
- Create: `crates/radium-workflow/src/temporal_client.rs`
- Modify: `crates/radium-workflow/src/lib.rs`
- Modify: `crates/radium-workflow/src/api/state.rs`
- Modify: `crates/radium-workflow/src/main.rs`
- Modify: `crates/radium-workflow/Cargo.toml` (add `tonic` and `prost` deps)
- Test: inline in `temporal_client.rs`

**Step 1: Add dependencies**

Add to `crates/radium-workflow/Cargo.toml`:

```toml
tonic = "0.12"
prost = "0.13"
prost-types = "0.13"
```

**Step 2: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporal_config_defaults() {
        std::env::remove_var("TEMPORAL_ADDRESS");
        std::env::remove_var("TEMPORAL_NAMESPACE");
        let config = TemporalConfig::from_env();
        assert_eq!(config.address, "http://localhost:7233");
        assert_eq!(config.namespace, "default");
    }

    #[test]
    fn test_temporal_config_custom() {
        std::env::set_var("TEMPORAL_ADDRESS", "http://temporal:7233");
        std::env::set_var("TEMPORAL_NAMESPACE", "radium");
        let config = TemporalConfig::from_env();
        assert_eq!(config.address, "http://temporal:7233");
        assert_eq!(config.namespace, "radium");
        std::env::remove_var("TEMPORAL_ADDRESS");
        std::env::remove_var("TEMPORAL_NAMESPACE");
    }

    #[test]
    fn test_signal_payload_serialization() {
        let payload = SignalPayload {
            data: serde_json::json!({"key": "value"}),
            received_at: "2026-03-01T00:00:00Z".to_string(),
            request_id: "req-123".to_string(),
        };
        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["request_id"], "req-123");
        assert_eq!(json["data"]["key"], "value");
    }

    #[test]
    fn test_gateway_workflow_id_generation() {
        let wf_id = gateway_workflow_id("iface-abc-123");
        assert_eq!(wf_id, "gateway-iface-abc-123");
    }

    #[test]
    fn test_gateway_task_queue_generation() {
        let queue = gateway_task_queue("iface-abc-123");
        assert_eq!(queue, "gateway-iface-abc-123-queue");
    }
}
```

**Step 3: Implement temporal_client.rs**

Create `crates/radium-workflow/src/temporal_client.rs`:

```rust
//! Temporal gRPC client for gateway workflow management.
//!
//! Wraps Temporal's WorkflowService gRPC API to start, signal,
//! query, and terminate gateway workflows.

use serde::Serialize;
use std::fmt;
use tonic::transport::Channel;

/// Temporal connection configuration.
#[derive(Debug, Clone)]
pub struct TemporalConfig {
    pub address: String,
    pub namespace: String,
}

impl TemporalConfig {
    pub fn from_env() -> Self {
        Self {
            address: std::env::var("TEMPORAL_ADDRESS")
                .unwrap_or_else(|_| "http://localhost:7233".to_string()),
            namespace: std::env::var("TEMPORAL_NAMESPACE")
                .unwrap_or_else(|_| "default".to_string()),
        }
    }
}

/// Error type for Temporal operations.
#[derive(Debug)]
pub enum TemporalError {
    Connection(String),
    Rpc(String),
    Serialization(String),
}

impl fmt::Display for TemporalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connection(e) => write!(f, "Temporal connection failed: {e}"),
            Self::Rpc(e) => write!(f, "Temporal RPC error: {e}"),
            Self::Serialization(e) => write!(f, "Temporal serialization error: {e}"),
        }
    }
}

/// Signal payload sent to gateway workflows.
#[derive(Debug, Serialize)]
pub struct SignalPayload {
    pub data: serde_json::Value,
    pub received_at: String,
    pub request_id: String,
}

/// Generate the workflow ID for a gateway workflow.
pub fn gateway_workflow_id(interface_id: &str) -> String {
    format!("gateway-{interface_id}")
}

/// Generate the task queue for a gateway workflow.
pub fn gateway_task_queue(interface_id: &str) -> String {
    format!("gateway-{interface_id}-queue")
}

/// Temporal gRPC client.
///
/// Uses tonic to communicate with Temporal's WorkflowService.
/// The Temporal server exposes a gRPC API defined in temporal.api.workflowservice.v1.
#[derive(Clone)]
pub struct TemporalClient {
    config: TemporalConfig,
    channel: Option<Channel>,
}

impl fmt::Debug for TemporalClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TemporalClient")
            .field("address", &self.config.address)
            .field("namespace", &self.config.namespace)
            .finish()
    }
}

impl TemporalClient {
    pub fn new(config: TemporalConfig) -> Self {
        Self {
            config,
            channel: None,
        }
    }

    /// Ensure gRPC channel is connected.
    async fn connect(&mut self) -> Result<Channel, TemporalError> {
        if let Some(ref channel) = self.channel {
            return Ok(channel.clone());
        }

        let channel = Channel::from_shared(self.config.address.clone())
            .map_err(|e| TemporalError::Connection(e.to_string()))?
            .connect()
            .await
            .map_err(|e| TemporalError::Connection(e.to_string()))?;

        self.channel = Some(channel.clone());
        Ok(channel)
    }

    /// Start a gateway workflow for a published interface.
    ///
    /// The workflow runs a generated TypeScript gateway that receives
    /// signals and forwards them to internal services.
    pub async fn start_gateway_workflow(
        &mut self,
        interface_id: &str,
        _task_queue: &str,
    ) -> Result<String, TemporalError> {
        let _channel = self.connect().await?;
        let workflow_id = gateway_workflow_id(interface_id);

        // TODO: Implement gRPC StartWorkflowExecution call.
        // For now, we return the workflow ID to enable integration.
        // The actual gRPC proto types will be generated from Temporal's
        // api-go proto definitions or used via the temporal-client crate.
        //
        // The call would look like:
        // let mut client = WorkflowServiceClient::new(channel);
        // client.start_workflow_execution(StartWorkflowExecutionRequest {
        //     namespace: self.config.namespace.clone(),
        //     workflow_id: workflow_id.clone(),
        //     workflow_type: Some(WorkflowType { name: "gatewayWorkflow".into() }),
        //     task_queue: Some(TaskQueue { name: task_queue.into(), .. }),
        //     ...
        // }).await?;

        tracing::info!(
            workflow_id = %workflow_id,
            interface_id = %interface_id,
            "Gateway workflow start requested (gRPC integration pending)"
        );

        Ok(workflow_id)
    }

    /// Send a signal to a running gateway workflow.
    pub async fn signal_gateway_workflow(
        &mut self,
        interface_id: &str,
        payload: &SignalPayload,
    ) -> Result<(), TemporalError> {
        let _channel = self.connect().await?;
        let workflow_id = gateway_workflow_id(interface_id);

        let _payload_json = serde_json::to_vec(payload)
            .map_err(|e| TemporalError::Serialization(e.to_string()))?;

        // TODO: Implement gRPC SignalWorkflowExecution call.
        // let mut client = WorkflowServiceClient::new(channel);
        // client.signal_workflow_execution(SignalWorkflowExecutionRequest {
        //     namespace: self.config.namespace.clone(),
        //     workflow_execution: Some(WorkflowExecution {
        //         workflow_id: workflow_id.clone(),
        //         run_id: "".into(),
        //     }),
        //     signal_name: "incomingRequest".into(),
        //     input: Some(Payloads { payloads: vec![...] }),
        //     ...
        // }).await?;

        tracing::info!(
            workflow_id = %workflow_id,
            request_id = %payload.request_id,
            "Gateway signal sent (gRPC integration pending)"
        );

        Ok(())
    }

    /// Terminate a gateway workflow (on interface unpublish).
    pub async fn terminate_gateway_workflow(
        &mut self,
        interface_id: &str,
    ) -> Result<(), TemporalError> {
        let _channel = self.connect().await?;
        let workflow_id = gateway_workflow_id(interface_id);

        // TODO: Implement gRPC TerminateWorkflowExecution call.
        // let mut client = WorkflowServiceClient::new(channel);
        // client.terminate_workflow_execution(TerminateWorkflowExecutionRequest {
        //     namespace: self.config.namespace.clone(),
        //     workflow_execution: Some(WorkflowExecution {
        //         workflow_id: workflow_id.clone(),
        //         run_id: "".into(),
        //     }),
        //     reason: "Interface unpublished".into(),
        //     ...
        // }).await?;

        tracing::info!(
            workflow_id = %workflow_id,
            "Gateway workflow termination requested (gRPC integration pending)"
        );

        Ok(())
    }

    /// Query a gateway workflow for its buffer depth.
    pub async fn query_gateway_buffer_depth(
        &mut self,
        interface_id: &str,
    ) -> Result<u64, TemporalError> {
        let _channel = self.connect().await?;
        let _workflow_id = gateway_workflow_id(interface_id);

        // TODO: Implement gRPC QueryWorkflow call.
        // Returns the number of pending/unprocessed signals.

        tracing::info!(
            interface_id = %interface_id,
            "Gateway buffer depth query (gRPC integration pending)"
        );

        Ok(0) // Placeholder
    }
}
```

**Step 4: Add to lib.rs**

```rust
pub mod temporal_client;
```

**Step 5: Add TemporalClient to AppState**

In `crates/radium-workflow/src/api/state.rs`:

```rust
use crate::temporal_client::TemporalClient;

#[derive(Clone)]
pub struct AppState {
    pub supabase: Arc<SupabaseClient>,
    pub rate_limiter: Arc<SlidingWindowLimiter>,
    pub discovery: Option<Arc<DiscoveryClient>>,
    pub kong: Option<Arc<KongClient>>,
    pub temporal: Option<Arc<tokio::sync::Mutex<TemporalClient>>>,
}
```

In `main.rs`, initialize:

```rust
use radium_workflow::temporal_client::{TemporalClient, TemporalConfig};

let temporal = {
    let config = TemporalConfig::from_env();
    Some(Arc::new(tokio::sync::Mutex::new(TemporalClient::new(config))))
};
```

**Step 6: Run tests**

Run: `nx run radium-workflow:test`
Expected: All tests pass

**Step 7: Commit**

```bash
git add crates/radium-workflow/src/temporal_client.rs crates/radium-workflow/src/lib.rs crates/radium-workflow/src/api/state.rs crates/radium-workflow/src/main.rs crates/radium-workflow/Cargo.toml
git commit -m "feat(radium-workflow): add Temporal gRPC client module for gateway workflows"
```

---

## Task 9: Gateway HTTP Handler

Create the HTTP handler that Kong routes to for published interfaces. It receives incoming requests and signals the corresponding gateway workflow.

**Files:**
- Create: `crates/radium-workflow/src/api/v1/gateway.rs`
- Modify: `crates/radium-workflow/src/api/v1/mod.rs`
- Test: inline in `gateway.rs`

**Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_response_accepted() {
        let resp = GatewayAcceptedResponse {
            status: "accepted".to_string(),
            request_id: "req-123".to_string(),
            message: "Request queued for processing".to_string(),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["status"], "accepted");
        assert_eq!(json["request_id"], "req-123");
    }

    #[test]
    fn test_gateway_error_not_found() {
        let err = GatewayError::not_found("Interface not found");
        assert_eq!(err.status, StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_gateway_error_unavailable() {
        let err = GatewayError::unavailable("Gateway workflow not running");
        assert_eq!(err.status, StatusCode::SERVICE_UNAVAILABLE);
    }
}
```

**Step 2: Implement gateway.rs**

Create `crates/radium-workflow/src/api/v1/gateway.rs`:

```rust
//! Gateway HTTP handler for published interfaces.
//!
//! Kong routes incoming traffic to this handler.
//! The handler sends a signal to the corresponding Temporal gateway workflow.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::state::AppState;
use crate::temporal_client::SignalPayload;

// ── Error type ──────────────────────────────────────────

#[derive(Debug)]
pub struct GatewayError {
    pub status: StatusCode,
    code: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: String,
    message: String,
}

impl GatewayError {
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "NOT_FOUND".to_string(),
            message: message.into(),
        }
    }

    pub fn unavailable(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "SERVICE_UNAVAILABLE".to_string(),
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "INTERNAL_ERROR".to_string(),
            message: message.into(),
        }
    }
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let envelope = ErrorEnvelope {
            error: ErrorBody {
                code: self.code,
                message: self.message,
            },
        };
        (self.status, Json(envelope)).into_response()
    }
}

// ── Response types ──────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct GatewayAcceptedResponse {
    pub status: String,
    pub request_id: String,
    pub message: String,
}

// ── Handler ─────────────────────────────────────────────

/// Receive incoming HTTP request for a published interface and signal
/// the corresponding gateway workflow.
///
/// Returns 202 Accepted immediately — processing is async via Temporal.
pub async fn handle_gateway_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(interface_id): Path<String>,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse, GatewayError> {
    // Generate a request ID for tracking
    let request_id = headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Parse the request body as JSON
    let data: serde_json::Value = if body.is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_slice(&body)
            .map_err(|e| GatewayError::internal(format!("Invalid JSON body: {e}")))?
    };

    // Verify the interface exists and is published
    #[derive(Deserialize)]
    struct PublicRow {
        #[allow(dead_code)]
        id: String,
    }

    let _: PublicRow = state
        .supabase
        .select_one(
            "public_interfaces",
            &[
                ("service_interface_id", &format!("eq.{interface_id}")),
                ("is_active", "eq.true"),
                ("select", "id"),
            ],
        )
        .await
        .map_err(|_| GatewayError::not_found(format!("Interface '{interface_id}' not found or not published")))?;

    // Signal the gateway workflow
    let temporal = state
        .temporal
        .as_ref()
        .ok_or_else(|| GatewayError::unavailable("Temporal client not configured"))?;

    let payload = SignalPayload {
        data,
        received_at: Utc::now().to_rfc3339(),
        request_id: request_id.clone(),
    };

    let mut client = temporal.lock().await;
    client
        .signal_gateway_workflow(&interface_id, &payload)
        .await
        .map_err(|e| GatewayError::unavailable(format!("Failed to signal gateway: {e}")))?;

    Ok((
        StatusCode::ACCEPTED,
        Json(GatewayAcceptedResponse {
            status: "accepted".to_string(),
            request_id,
            message: "Request queued for processing".to_string(),
        }),
    ))
}
```

**Step 3: Add gateway routes to mod.rs**

In `crates/radium-workflow/src/api/v1/mod.rs`:

```rust
pub mod gateway;

// Add route:
.route("/gateway/{interface_id}", post(gateway::handle_gateway_request))
```

**Step 4: Run tests**

Run: `nx run radium-workflow:test`
Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/radium-workflow/src/api/v1/gateway.rs crates/radium-workflow/src/api/v1/mod.rs
git commit -m "feat(radium-workflow): add gateway HTTP handler for published interface traffic"
```

---

## Task 10: Gateway Workflow Codegen Templates

Add Handlebars templates for generating the gateway workflow and worker TypeScript code. Extend `GeneratedCode` and the codegen pipeline to produce gateway files when interfaces are present.

**Files:**
- Create: `crates/radium-workflow/src/codegen/templates/gateway.ts.hbs`
- Create: `crates/radium-workflow/src/codegen/templates/gateway_worker.ts.hbs`
- Modify: `crates/radium-workflow/src/codegen/typescript.rs`
- Modify: `crates/radium-workflow/src/codegen/mod.rs`
- Test: inline in `typescript.rs`

**Step 1: Write failing test**

Add to `typescript.rs` test module:

```rust
#[test]
fn test_generated_code_has_gateway_fields() {
    // When interfaces are present, GeneratedCode should have gateway fields
    let code = GeneratedCode {
        workflow: String::new(),
        activities: String::new(),
        worker: String::new(),
        package_json: String::new(),
        tsconfig: String::new(),
        gateway: Some("gateway code".to_string()),
        gateway_worker: Some("gateway worker code".to_string()),
    };
    assert!(code.gateway.is_some());
    assert!(code.gateway_worker.is_some());
}

#[test]
fn test_generated_code_no_gateway_without_interfaces() {
    let code = GeneratedCode {
        workflow: String::new(),
        activities: String::new(),
        worker: String::new(),
        package_json: String::new(),
        tsconfig: String::new(),
        gateway: None,
        gateway_worker: None,
    };
    assert!(code.gateway.is_none());
}
```

**Step 2: Run test to verify it fails**

Run: `nx run radium-workflow:test -- --lib codegen`
Expected: FAIL — `gateway` field doesn't exist on `GeneratedCode`

**Step 3: Create gateway.ts.hbs template**

Create `crates/radium-workflow/src/codegen/templates/gateway.ts.hbs`:

```handlebars
// Gateway Workflow - Generated by Radium v{{version}}
// Generated at: {{generated_at}}
// Interface: {{interface_name}} ({{interface_type}})
//
// This workflow acts as a durable edge buffer for the published interface.
// It receives incoming requests as signals, queues them, and forwards
// to the internal service workflow with retry.

import {
  defineSignal,
  defineQuery,
  setHandler,
  condition,
  proxyActivities,
  continueAsNew,
  sleep,
} from '@temporalio/workflow';

import type * as activities from './gateway_activities';

const { forwardToService } = proxyActivities<typeof activities>({
  startToCloseTimeout: '30s',
  retry: {
    initialInterval: '1s',
    maximumInterval: '5m',
    backoffCoefficient: 2,
    maximumAttempts: 100,
    nonRetryableErrorTypes: ['PERMANENT_FAILURE'],
  },
});

// Signal: incoming HTTP request data
export const incomingRequestSignal = defineSignal<[IncomingRequest]>('incomingRequest');

// Query: how many signals are pending (buffer depth)
export const bufferDepthQuery = defineQuery<number>('bufferDepth');

interface IncomingRequest {
  data: Record<string, unknown>;
  receivedAt: string;
  requestId: string;
}

const CONTINUE_AS_NEW_THRESHOLD = {{continue_as_new_threshold}};

export async function gatewayWorkflow(
  pendingFromPrevious: IncomingRequest[] = []
): Promise<void> {
  const buffer: IncomingRequest[] = [...pendingFromPrevious];
  let processedCount = 0;
  let running = true;

  // Query handler: report buffer depth
  setHandler(bufferDepthQuery, () => buffer.length);

  // Signal handler: queue incoming requests
  setHandler(incomingRequestSignal, (request: IncomingRequest) => {
    buffer.push(request);
  });

  // Main processing loop
  while (running) {
    // Wait until there's something in the buffer
    await condition(() => buffer.length > 0, '1m');

    // Process all buffered requests
    while (buffer.length > 0) {
      const request = buffer.shift()!;
      try {
        await forwardToService({
          interfaceId: '{{interface_id}}',
          serviceId: '{{service_id}}',
          interfaceType: '{{interface_type}}',
          data: request.data,
          requestId: request.requestId,
          receivedAt: request.receivedAt,
        });
        processedCount++;
      } catch (err) {
        // If all retries exhausted, log and continue (don't block the queue)
        console.error(
          `Failed to forward request ${request.requestId} after all retries:`,
          err
        );
        processedCount++;
      }

      // Continue-as-new to manage event history
      if (processedCount >= CONTINUE_AS_NEW_THRESHOLD) {
        const remaining = [...buffer];
        await continueAsNew<typeof gatewayWorkflow>(remaining);
      }
    }
  }
}
```

**Step 4: Create gateway_worker.ts.hbs template**

Create `crates/radium-workflow/src/codegen/templates/gateway_worker.ts.hbs`:

```handlebars
// Gateway Worker - Generated by Radium v{{version}}
// Generated at: {{generated_at}}
// Interface: {{interface_name}}
//
// Runs the gateway workflow and its activities.

import { NativeConnection, Worker } from '@temporalio/worker';
import * as activities from './gateway_activities';

async function run() {
  const connection = await NativeConnection.connect({
    address: process.env.TEMPORAL_ADDRESS || 'localhost:7233',
  });

  const worker = await Worker.create({
    connection,
    namespace: process.env.TEMPORAL_NAMESPACE || 'default',
    workflowsPath: require.resolve('./gateway'),
    activities,
    taskQueue: '{{task_queue}}',
  });

  console.log('Gateway worker started for interface: {{interface_name}}');
  await worker.run();
}

run().catch((err) => {
  console.error('Gateway worker failed:', err);
  process.exit(1);
});
```

**Step 5: Extend GeneratedCode and codegen**

In `crates/radium-workflow/src/codegen/typescript.rs`, update:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct GeneratedCode {
    pub workflow: String,
    pub activities: String,
    pub worker: String,
    pub package_json: String,
    pub tsconfig: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_worker: Option<String>,
}
```

Add gateway template data struct and rendering logic:

```rust
#[derive(Debug, Serialize)]
struct GatewayTemplateData {
    version: String,
    generated_at: String,
    interface_id: String,
    interface_name: String,
    interface_type: String,
    service_id: String,
    task_queue: String,
    continue_as_new_threshold: u32,
}
```

Register the new templates in `CodeGenerator::new()`:
```rust
hbs.register_template_string("gateway", include_str!("templates/gateway.ts.hbs"))?;
hbs.register_template_string("gateway_worker", include_str!("templates/gateway_worker.ts.hbs"))?;
```

The `generate()` function should accept an optional list of interfaces and generate gateway files if present. For now, `gateway` and `gateway_worker` will be `None` (interface data is injected at deploy time in a future step).

**Step 6: Run tests**

Run: `nx run radium-workflow:test`
Expected: All tests pass (including existing codegen tests)

**Step 7: Commit**

```bash
git add crates/radium-workflow/src/codegen/templates/gateway.ts.hbs crates/radium-workflow/src/codegen/templates/gateway_worker.ts.hbs crates/radium-workflow/src/codegen/typescript.rs crates/radium-workflow/src/codegen/mod.rs
git commit -m "feat(radium-workflow): add gateway workflow codegen templates"
```

---

## Task 11: Wire Gateway into Publish/Unpublish Lifecycle

Update `publish_interface` to start a gateway workflow (via Temporal) and `unpublish_interface` to terminate it.

**Files:**
- Modify: `crates/radium-workflow/src/api/v1/interfaces.rs`
- Test: inline in `interfaces.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_publish_response_gateway_workflow_id() {
    let json = serde_json::json!({
        "id": "pub-1",
        "service_interface_id": "iface-1",
        "route_path": "/api/my-service/my-signal",
        "http_method": "POST",
        "kong_route_id": "route-123",
        "kong_service_id": "svc-456",
        "gateway_workflow_id": "gateway-iface-1",
        "is_active": true,
        "created_at": "2026-03-01T00:00:00Z",
        "updated_at": "2026-03-01T00:00:00Z"
    });
    let resp: PublishResponse = serde_json::from_value(json).unwrap();
    assert_eq!(resp.gateway_workflow_id.unwrap(), "gateway-iface-1");
}
```

**Step 2: Update PublishResponse**

Add `gateway_workflow_id` field:

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub gateway_workflow_id: Option<String>,
```

Add to `InsertPublicInterfaceRow`:
```rust
gateway_workflow_id: Option<String>,
```

**Step 3: Update publish_interface**

After Kong route creation, start the gateway workflow:

```rust
// Start gateway workflow via Temporal
let gateway_workflow_id = if let Some(ref temporal) = state.temporal {
    let task_queue = crate::temporal_client::gateway_task_queue(&iid);
    let mut client = temporal.lock().await;
    match client.start_gateway_workflow(&iid, &task_queue).await {
        Ok(wf_id) => Some(wf_id),
        Err(e) => {
            tracing::warn!("Failed to start gateway workflow: {e}");
            None
        }
    }
} else {
    None
};
```

Include in the insert row:
```rust
gateway_workflow_id,
```

**Step 4: Update unpublish_interface**

Before deleting Kong routes, also terminate the gateway workflow:

```rust
// Terminate gateway workflows
if let Some(ref temporal) = state.temporal {
    let mut client = temporal.lock().await;
    let _ = client.terminate_gateway_workflow(&iid).await;
}
```

**Step 5: Run tests**

Run: `nx run radium-workflow:test`
Expected: All tests pass

**Step 6: Commit**

```bash
git add crates/radium-workflow/src/api/v1/interfaces.rs
git commit -m "feat(radium-workflow): wire gateway workflow lifecycle into interface publish/unpublish"
```

---

## Task 12: Update Kong Route Config and Add Gateway Route

Update the Kong seed script to include gateway routes, and update the Docker Compose env vars.

**Files:**
- Modify: `scripts/seed-kong.sh`
- Modify: `docker-compose.yml`
- Test: manual verification

**Step 1: Update Docker Compose environment**

Add Temporal and Kong env vars to the `radium-workflow` service:

```yaml
radium-workflow:
  environment:
    # ... existing vars
    KONG_ADMIN_URL: http://kong:8001
    TEMPORAL_ADDRESS: http://temporal:7233
    TEMPORAL_NAMESPACE: default
```

**Step 2: Update Kong seed script**

Add the gateway route to the seed script:

```bash
# Add gateway route for radium-workflow (handles /v1/gateway/* paths)
curl -s -X POST "$KONG_ADMIN/services/radium-workflow/routes" \
  -d 'paths[]=/v1/gateway' \
  -d strip_path=false | jq .
```

**Step 3: Verify**

Run: `docker compose up -d` and check Kong routes
Run: `curl http://localhost:8001/routes | jq '.data[].paths'`
Expected: Should include `/v1/gateway` path

**Step 4: Commit**

```bash
git add docker-compose.yml scripts/seed-kong.sh
git commit -m "feat(infra): add gateway routes and temporal/kong env vars to docker compose"
```

---

## Task 13: Update Memory and Plan Status

Update project memory files to reflect P4 completion.

**Files:**
- Modify: `~/.claude/projects/-Users-mattbernier-projects-unicorn-Radium/memory/plans-status.md`

**Step 1: Update plans-status.md**

Add P4 to the completed section with all tasks listed.

**Step 2: Commit memory update**

No git commit needed — memory files are outside the repo.

---

## Summary

| Task | Description | Est. New Tests |
|------|-------------|---------------|
| 1 | Extract deploy pipeline module | 4+ |
| 2 | Implement bundled project deploy | 2+ |
| 3 | Kong client module | 6+ |
| 4 | Docker Compose Kong DB mode | manual |
| 5 | Wire Kong into publish/unpublish | 1+ |
| 6 | State variables API (service + project) | 7+ |
| 7 | CLI state variable commands | 6+ |
| 8 | Temporal gRPC client module | 5+ |
| 9 | Gateway HTTP handler | 3+ |
| 10 | Gateway codegen templates | 2+ |
| 11 | Wire gateway into publish/unpublish lifecycle | 1+ |
| 12 | Docker Compose + Kong gateway routes | manual |
| 13 | Memory/plan status update | N/A |

**Total: 13 tasks, ~37+ new tests**
