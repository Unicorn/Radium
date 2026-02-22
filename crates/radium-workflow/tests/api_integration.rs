//! API integration tests for the workflow CRUD endpoints.
//!
//! These tests spin up a real Axum server on a random port and exercise the
//! full request path including Supabase persistence. They require:
//!
//! - `SUPABASE_URL` env var pointing to a running Supabase instance
//! - `SUPABASE_SERVICE_ROLE_KEY` env var with the service-role key
//!
//! All tests are marked `#[ignore]` so they never run in CI without the
//! infrastructure in place. Run them explicitly with:
//!
//! ```sh
//! cargo test -p radium-workflow --test api_integration -- --ignored
//! ```

use std::sync::Arc;

use radium_workflow::api::router;
use radium_workflow::api::state::AppState;
use radium_workflow::supabase::{SupabaseClient, SupabaseConfig};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Test fixture: valid YAML workflow
// ---------------------------------------------------------------------------

const TEST_WORKFLOW_YAML: &str = r#"
name: Test Integration Workflow
description: Simple test workflow

components:
  - id: start
    type: trigger
    config:
      trigger_type: webhook

  - id: process
    type: activity
    config:
      name: processData
      timeout: 30s

  - id: done
    type: stop

connections:
  - from: start
    to: process
  - from: process
    to: done
"#;

/// Updated YAML used for the PUT /v1/workflows/:id test.
const UPDATED_WORKFLOW_YAML: &str = r#"
name: Updated Integration Workflow
description: Updated description

components:
  - id: start
    type: trigger
    config:
      trigger_type: webhook

  - id: process
    type: activity
    config:
      name: processDataUpdated
      timeout: 60s

  - id: done
    type: stop

connections:
  - from: start
    to: process
  - from: process
    to: done
"#;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read required env vars and panic with a clear message if missing.
fn require_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| {
        panic!(
            "Integration test requires the `{name}` environment variable to be set"
        )
    })
}

/// Build an `AppState` from environment variables.
fn build_app_state() -> AppState {
    let config = SupabaseConfig {
        url: require_env("SUPABASE_URL"),
        service_role_key: require_env("SUPABASE_SERVICE_ROLE_KEY"),
    };
    AppState {
        supabase: Arc::new(SupabaseClient::new(config)),
        rate_limiter: Arc::new(radium_workflow::security::SlidingWindowLimiter::new(
            radium_workflow::security::RateLimitConfig::unlimited(),
        )),
        discovery: None,
    }
}

/// Start an Axum server on a random port and return the base URL.
///
/// The server runs in a background Tokio task and is automatically dropped
/// when the runtime shuts down at the end of the test.
async fn start_test_server() -> String {
    let state = build_app_state();
    let app = router(Some(state));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port");
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });

    format!("http://{addr}")
}

/// Build a reqwest client with the Bearer token header pre-configured.
fn http_client() -> reqwest::Client {
    let api_key = require_env("SUPABASE_SERVICE_ROLE_KEY");
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        format!("Bearer {api_key}").parse().unwrap(),
    );
    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap()
}

/// Create a workflow via the API and return its `id`. Panics on failure.
async fn create_workflow(base_url: &str, client: &reqwest::Client) -> String {
    let resp = client
        .post(format!("{base_url}/v1/workflows"))
        .header("content-type", "application/x-yaml")
        .body(TEST_WORKFLOW_YAML)
        .send()
        .await
        .expect("POST /v1/workflows request failed");

    assert_eq!(
        resp.status().as_u16(),
        201,
        "Expected 201 Created, got {}",
        resp.status()
    );

    let body: Value = resp.json().await.expect("Failed to parse create response");
    body["id"]
        .as_str()
        .expect("Response missing `id` field")
        .to_string()
}

/// Delete a workflow, swallowing errors (best-effort cleanup).
async fn cleanup_workflow(base_url: &str, client: &reqwest::Client, id: &str) {
    let _ = client
        .delete(format!("{base_url}/v1/workflows/{id}"))
        .send()
        .await;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "Requires running Supabase instance (SUPABASE_URL, SUPABASE_SERVICE_ROLE_KEY)"]
async fn test_create_workflow() {
    let base_url = start_test_server().await;
    let client = http_client();

    let id = create_workflow(&base_url, &client).await;

    // Verify response shape by re-fetching.
    let resp = client
        .get(format!("{base_url}/v1/workflows/{id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["name"], "Test Integration Workflow");
    assert!(body["definition"].is_object(), "definition should be a JSON object");

    // Cleanup.
    cleanup_workflow(&base_url, &client, &id).await;
}

#[tokio::test]
#[ignore = "Requires running Supabase instance (SUPABASE_URL, SUPABASE_SERVICE_ROLE_KEY)"]
async fn test_list_workflows() {
    let base_url = start_test_server().await;
    let client = http_client();

    // Create a workflow so the list is non-empty.
    let id = create_workflow(&base_url, &client).await;

    let resp = client
        .get(format!("{base_url}/v1/workflows"))
        .send()
        .await
        .expect("GET /v1/workflows request failed");

    assert_eq!(resp.status().as_u16(), 200);

    let body: Value = resp.json().await.unwrap();
    assert!(body["workflows"].is_array(), "Response should contain `workflows` array");
    assert!(
        body["total"].as_u64().unwrap_or(0) >= 1,
        "Total should be at least 1 after creating a workflow"
    );

    // Verify the created workflow appears in the list.
    let workflows = body["workflows"].as_array().unwrap();
    let found = workflows.iter().any(|w| w["id"].as_str() == Some(&id));
    assert!(found, "Created workflow should appear in the list");

    // Cleanup.
    cleanup_workflow(&base_url, &client, &id).await;
}

#[tokio::test]
#[ignore = "Requires running Supabase instance (SUPABASE_URL, SUPABASE_SERVICE_ROLE_KEY)"]
async fn test_get_workflow() {
    let base_url = start_test_server().await;
    let client = http_client();

    let id = create_workflow(&base_url, &client).await;

    let resp = client
        .get(format!("{base_url}/v1/workflows/{id}"))
        .send()
        .await
        .expect("GET /v1/workflows/:id request failed");

    assert_eq!(resp.status().as_u16(), 200);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["id"].as_str().unwrap(), id);
    assert_eq!(body["name"], "Test Integration Workflow");
    assert_eq!(
        body["description"],
        "Simple test workflow"
    );
    assert!(body["definition"].is_object());
    assert!(body["created_at"].is_string());
    assert!(body["updated_at"].is_string());

    // Cleanup.
    cleanup_workflow(&base_url, &client, &id).await;
}

#[tokio::test]
#[ignore = "Requires running Supabase instance (SUPABASE_URL, SUPABASE_SERVICE_ROLE_KEY)"]
async fn test_get_workflow_not_found() {
    let base_url = start_test_server().await;
    let client = http_client();

    let fake_id = "00000000-0000-0000-0000-000000000000";
    let resp = client
        .get(format!("{base_url}/v1/workflows/{fake_id}"))
        .send()
        .await
        .expect("GET /v1/workflows/:id request failed");

    // Should return 404 or 500 depending on Supabase response.
    assert!(
        resp.status().as_u16() == 404 || resp.status().as_u16() == 500,
        "Expected 404 or 500 for non-existent workflow, got {}",
        resp.status()
    );
}

#[tokio::test]
#[ignore = "Requires running Supabase instance (SUPABASE_URL, SUPABASE_SERVICE_ROLE_KEY)"]
async fn test_validate_workflow() {
    let base_url = start_test_server().await;
    let client = http_client();

    let id = create_workflow(&base_url, &client).await;

    let resp = client
        .post(format!("{base_url}/v1/workflows/{id}/validate"))
        .send()
        .await
        .expect("POST /v1/workflows/:id/validate request failed");

    assert_eq!(resp.status().as_u16(), 200);

    let body: Value = resp.json().await.unwrap();
    // The test workflow is well-formed, so it should be valid.
    assert_eq!(
        body["valid"], true,
        "Test workflow should be valid, got errors: {:?}",
        body["errors"]
    );
    assert!(body["errors"].is_array());
    assert!(body["warnings"].is_array());
    assert!(body["suggestions"].is_array());

    // Cleanup.
    cleanup_workflow(&base_url, &client, &id).await;
}

#[tokio::test]
#[ignore = "Requires running Supabase instance (SUPABASE_URL, SUPABASE_SERVICE_ROLE_KEY)"]
async fn test_update_workflow() {
    let base_url = start_test_server().await;
    let client = http_client();

    let id = create_workflow(&base_url, &client).await;

    let resp = client
        .put(format!("{base_url}/v1/workflows/{id}"))
        .header("content-type", "application/x-yaml")
        .body(UPDATED_WORKFLOW_YAML)
        .send()
        .await
        .expect("PUT /v1/workflows/:id request failed");

    assert_eq!(
        resp.status().as_u16(),
        200,
        "Expected 200 OK, got {}",
        resp.status()
    );

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["id"].as_str().unwrap(), id);
    assert_eq!(body["name"], "Updated Integration Workflow");
    assert_eq!(body["description"], "Updated description");

    // Verify the update persisted by re-fetching.
    let get_resp = client
        .get(format!("{base_url}/v1/workflows/{id}"))
        .send()
        .await
        .unwrap();
    let get_body: Value = get_resp.json().await.unwrap();
    assert_eq!(get_body["name"], "Updated Integration Workflow");

    // Cleanup.
    cleanup_workflow(&base_url, &client, &id).await;
}

#[tokio::test]
#[ignore = "Requires running Supabase instance (SUPABASE_URL, SUPABASE_SERVICE_ROLE_KEY)"]
async fn test_delete_workflow() {
    let base_url = start_test_server().await;
    let client = http_client();

    let id = create_workflow(&base_url, &client).await;

    // Delete the workflow.
    let resp = client
        .delete(format!("{base_url}/v1/workflows/{id}"))
        .send()
        .await
        .expect("DELETE /v1/workflows/:id request failed");

    assert_eq!(
        resp.status().as_u16(),
        204,
        "Expected 204 No Content, got {}",
        resp.status()
    );

    // Verify it is gone.
    let get_resp = client
        .get(format!("{base_url}/v1/workflows/{id}"))
        .send()
        .await
        .unwrap();

    assert!(
        get_resp.status().as_u16() == 404 || get_resp.status().as_u16() == 500,
        "Expected 404 or 500 after deletion, got {}",
        get_resp.status()
    );
}

#[tokio::test]
#[ignore = "Requires running Supabase instance (SUPABASE_URL, SUPABASE_SERVICE_ROLE_KEY)"]
async fn test_create_workflow_unauthorized() {
    let base_url = start_test_server().await;

    // Use a client with NO Authorization header.
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/v1/workflows"))
        .header("content-type", "application/x-yaml")
        .body(TEST_WORKFLOW_YAML)
        .send()
        .await
        .expect("POST /v1/workflows request failed");

    assert_eq!(
        resp.status().as_u16(),
        401,
        "Expected 401 Unauthorized without Bearer token, got {}",
        resp.status()
    );

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
}

#[tokio::test]
#[ignore = "Requires running Supabase instance (SUPABASE_URL, SUPABASE_SERVICE_ROLE_KEY)"]
async fn test_create_workflow_invalid_yaml() {
    let base_url = start_test_server().await;
    let client = http_client();

    let resp = client
        .post(format!("{base_url}/v1/workflows"))
        .header("content-type", "application/x-yaml")
        .body("this is: [not valid: yaml: {{{}}")
        .send()
        .await
        .expect("POST /v1/workflows request failed");

    assert_eq!(
        resp.status().as_u16(),
        400,
        "Expected 400 Bad Request for invalid YAML, got {}",
        resp.status()
    );
}

#[tokio::test]
#[ignore = "Requires running Supabase instance (SUPABASE_URL, SUPABASE_SERVICE_ROLE_KEY)"]
async fn test_full_crud_lifecycle() {
    let base_url = start_test_server().await;
    let client = http_client();

    // 1. CREATE
    let id = create_workflow(&base_url, &client).await;

    // 2. READ (GET single)
    let get_resp = client
        .get(format!("{base_url}/v1/workflows/{id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(get_resp.status().as_u16(), 200);
    let get_body: Value = get_resp.json().await.unwrap();
    assert_eq!(get_body["name"], "Test Integration Workflow");

    // 3. LIST (should contain the new workflow)
    let list_resp = client
        .get(format!("{base_url}/v1/workflows"))
        .send()
        .await
        .unwrap();
    assert_eq!(list_resp.status().as_u16(), 200);
    let list_body: Value = list_resp.json().await.unwrap();
    let found = list_body["workflows"]
        .as_array()
        .unwrap()
        .iter()
        .any(|w| w["id"].as_str() == Some(&id));
    assert!(found, "Workflow should be in the list");

    // 4. VALIDATE
    let validate_resp = client
        .post(format!("{base_url}/v1/workflows/{id}/validate"))
        .send()
        .await
        .unwrap();
    assert_eq!(validate_resp.status().as_u16(), 200);
    let validate_body: Value = validate_resp.json().await.unwrap();
    assert_eq!(validate_body["valid"], true);

    // 5. UPDATE
    let update_resp = client
        .put(format!("{base_url}/v1/workflows/{id}"))
        .header("content-type", "application/x-yaml")
        .body(UPDATED_WORKFLOW_YAML)
        .send()
        .await
        .unwrap();
    assert_eq!(update_resp.status().as_u16(), 200);
    let update_body: Value = update_resp.json().await.unwrap();
    assert_eq!(update_body["name"], "Updated Integration Workflow");

    // 6. DELETE
    let delete_resp = client
        .delete(format!("{base_url}/v1/workflows/{id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(delete_resp.status().as_u16(), 204);

    // 7. Verify deletion
    let gone_resp = client
        .get(format!("{base_url}/v1/workflows/{id}"))
        .send()
        .await
        .unwrap();
    assert!(
        gone_resp.status().as_u16() == 404 || gone_resp.status().as_u16() == 500,
        "Workflow should be gone after deletion"
    );
}
