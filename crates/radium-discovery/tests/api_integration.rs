//! Integration tests for the Discovery Service API.
//!
//! These tests require a running Neo4j instance and an embedding provider.
//! Run with:
//! ```sh
//! cargo test -p radium-discovery --test api_integration -- --ignored
//! ```
//!
//! Required environment:
//! - `NEO4J_URI` (default: `bolt://localhost:7687`)
//! - `NEO4J_USER` (default: `neo4j`)
//! - `NEO4J_PASSWORD` (default: `radium-dev`)
//! - `ANTHROPIC_API_KEY` or `OPENAI_API_KEY` for embedding generation

use std::net::SocketAddr;
use std::sync::Arc;

use reqwest::Client;
use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Start the discovery service on a random port against real Neo4j.
///
/// Returns the bound address and a task handle for the background server.
/// The caller is responsible for cleanup of test data.
#[allow(clippy::disallowed_methods)] // Integration test setup requires env var access
async fn start_test_server() -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let neo4j_uri =
        std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let neo4j_user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let neo4j_password =
        std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "radium-dev".to_string());

    let graph = neo4rs::Graph::new(&neo4j_uri, &neo4j_user, &neo4j_password)
        .await
        .expect("Failed to connect to Neo4j for integration tests");

    // Initialize schema (idempotent)
    radium_discovery::graph::schema::initialize(&graph)
        .await
        .expect("Failed to initialize Neo4j schema");

    // Create embedding provider from environment
    let embeddings = radium_discovery::embeddings::create_provider()
        .expect("Integration tests require an embedding provider (set ANTHROPIC_API_KEY or OPENAI_API_KEY)");

    // Initialize vector indexes with the provider's dimension
    radium_discovery::graph::schema::initialize_vector_indexes(&graph, embeddings.dimension())
        .await
        .expect("Failed to initialize vector indexes");

    let state = radium_discovery::state::AppState {
        graph,
        embeddings: Arc::from(embeddings),
    };

    // Allow any origin in integration tests for convenience
    let app = radium_discovery::api::router(state, &["*".to_string()]);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port");
    let addr = listener.local_addr().unwrap();

    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });

    // Give the server a moment to start accepting connections
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    (addr, handle)
}

fn client() -> Client {
    Client::new()
}

fn base_url(addr: &SocketAddr) -> String {
    format!("http://{addr}")
}

/// Best-effort cleanup: delete a node by ID via the API.
async fn cleanup_node(addr: &SocketAddr, id: &str) {
    let _ = client()
        .delete(format!("{}/v1/discover/index/{id}", base_url(addr)))
        .send()
        .await;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "Requires running Neo4j instance"]
async fn test_health_endpoint() {
    let (addr, _handle) = start_test_server().await;

    let resp = client()
        .get(format!("{}/health", base_url(&addr)))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["service"], "radium-discovery");
}

#[tokio::test]
#[ignore = "Requires running Neo4j instance"]
async fn test_index_and_retrieve() {
    let (addr, _handle) = start_test_server().await;
    let test_id = format!("test-index-{}", uuid::Uuid::new_v4());

    let index_req = json!({
        "id": test_id,
        "kind": "component",
        "name": "send-email",
        "description": "Sends transactional emails",
        "category": "communication",
        "visibility": "public",
        "owner_id": "user-1",
        "tags": ["email", "notification"]
    });

    let resp = client()
        .post(format!("{}/v1/discover/index", base_url(&addr)))
        .json(&index_req)
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        201,
        "Expected 201 Created for index operation"
    );
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["id"], test_id);
    assert_eq!(body["kind"], "component");
    assert_eq!(body["name"], "send-email");

    // Cleanup
    cleanup_node(&addr, &test_id).await;
}

#[tokio::test]
#[ignore = "Requires running Neo4j instance"]
async fn test_search_structured() {
    let (addr, _handle) = start_test_server().await;
    let id1 = format!("test-search-1-{}", uuid::Uuid::new_v4());
    let id2 = format!("test-search-2-{}", uuid::Uuid::new_v4());

    // Index two items with different categories
    let resp1 = client()
        .post(format!("{}/v1/discover/index", base_url(&addr)))
        .json(&json!({
            "id": id1,
            "kind": "component",
            "name": "email-sender",
            "description": "Sends emails via SMTP",
            "category": "communication",
            "visibility": "public",
            "owner_id": "user-1",
            "tags": ["email"]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp1.status(), 201, "Failed to index first item");

    let resp2 = client()
        .post(format!("{}/v1/discover/index", base_url(&addr)))
        .json(&json!({
            "id": id2,
            "kind": "component",
            "name": "db-query",
            "description": "Queries a SQL database",
            "category": "data",
            "visibility": "public",
            "owner_id": "user-1",
            "tags": ["database"]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp2.status(), 201, "Failed to index second item");

    // Search by category
    let resp = client()
        .post(format!("{}/v1/discover/search", base_url(&addr)))
        .json(&json!({
            "filters": { "category": "communication" },
            "limit": 10
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let results = body["results"].as_array().unwrap();
    // Should find at least the communication item
    assert!(
        results.iter().any(|r| r["name"] == "email-sender"),
        "Expected email-sender in search results, got: {body}"
    );

    // Cleanup
    cleanup_node(&addr, &id1).await;
    cleanup_node(&addr, &id2).await;
}

#[tokio::test]
#[ignore = "Requires running Neo4j instance"]
async fn test_telemetry_usage_count() {
    let (addr, _handle) = start_test_server().await;
    let test_id = format!("test-telemetry-{}", uuid::Uuid::new_v4());

    // Index an item
    let resp = client()
        .post(format!("{}/v1/discover/index", base_url(&addr)))
        .json(&json!({
            "id": test_id,
            "kind": "component",
            "name": "test-component",
            "description": "A component for telemetry testing",
            "category": "test",
            "visibility": "public",
            "owner_id": "user-1",
            "tags": []
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "Failed to index item for telemetry test");

    // Send telemetry event
    let resp = client()
        .post(format!(
            "{}/v1/discover/index/{test_id}/telemetry",
            base_url(&addr)
        ))
        .json(&json!({
            "event": "deploy",
            "user_id": "user-1"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["recorded"], true);

    // Cleanup
    cleanup_node(&addr, &test_id).await;
}

#[tokio::test]
#[ignore = "Requires running Neo4j instance"]
async fn test_delete() {
    let (addr, _handle) = start_test_server().await;
    let test_id = format!("test-delete-{}", uuid::Uuid::new_v4());

    // Index an item
    let resp = client()
        .post(format!("{}/v1/discover/index", base_url(&addr)))
        .json(&json!({
            "id": test_id,
            "kind": "component",
            "name": "to-delete",
            "description": "Will be deleted",
            "category": "test",
            "visibility": "public",
            "owner_id": "user-1",
            "tags": []
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "Failed to index item for delete test");

    // Delete it
    let resp = client()
        .delete(format!(
            "{}/v1/discover/index/{test_id}",
            base_url(&addr)
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 204, "Expected 204 No Content for deletion");
}

#[tokio::test]
#[ignore = "Requires running Neo4j instance"]
async fn test_compare() {
    let (addr, _handle) = start_test_server().await;
    let id1 = format!("test-compare-1-{}", uuid::Uuid::new_v4());
    let id2 = format!("test-compare-2-{}", uuid::Uuid::new_v4());

    // Index two items
    for (id, name) in [(&id1, "comp-a"), (&id2, "comp-b")] {
        let resp = client()
            .post(format!("{}/v1/discover/index", base_url(&addr)))
            .json(&json!({
                "id": id,
                "kind": "component",
                "name": name,
                "description": "Test component for comparison",
                "category": "test",
                "visibility": "public",
                "owner_id": "user-1",
                "tags": []
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 201, "Failed to index {name}");
    }

    let resp = client()
        .get(format!(
            "{}/v1/discover/compare?ids={},{}", base_url(&addr), id1, id2
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 2, "Expected 2 items in comparison result");

    // Cleanup
    cleanup_node(&addr, &id1).await;
    cleanup_node(&addr, &id2).await;
}

#[tokio::test]
#[ignore = "Requires running Neo4j instance"]
async fn test_update_index() {
    let (addr, _handle) = start_test_server().await;
    let test_id = format!("test-update-{}", uuid::Uuid::new_v4());

    // Create
    let resp = client()
        .post(format!("{}/v1/discover/index", base_url(&addr)))
        .json(&json!({
            "id": test_id,
            "kind": "component",
            "name": "original-name",
            "description": "Original description",
            "category": "test",
            "visibility": "public",
            "owner_id": "user-1",
            "tags": ["original"]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);

    // Update via PUT
    let resp = client()
        .put(format!(
            "{}/v1/discover/index/{test_id}",
            base_url(&addr)
        ))
        .json(&json!({
            "id": test_id,
            "kind": "component",
            "name": "updated-name",
            "description": "Updated description",
            "category": "test",
            "visibility": "public",
            "owner_id": "user-1",
            "tags": ["updated"]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["name"], "updated-name");
    assert_eq!(body["description"], "Updated description");

    // Cleanup
    cleanup_node(&addr, &test_id).await;
}

#[tokio::test]
#[ignore = "Requires running Neo4j instance"]
async fn test_search_empty_results() {
    let (addr, _handle) = start_test_server().await;

    // Search with no query and no filters should return empty
    let resp = client()
        .post(format!("{}/v1/discover/search", base_url(&addr)))
        .json(&json!({}))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let results = body["results"].as_array().unwrap();
    assert!(results.is_empty(), "No-filter search should return empty");
    assert_eq!(body["total"], 0);
}

#[tokio::test]
#[ignore = "Requires running Neo4j instance"]
async fn test_compare_empty_ids() {
    let (addr, _handle) = start_test_server().await;

    let resp = client()
        .get(format!("{}/v1/discover/compare?ids=", base_url(&addr)))
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        400,
        "Compare with empty ids should return 400"
    );
}

#[tokio::test]
#[ignore = "Requires running Neo4j instance"]
async fn test_full_lifecycle() {
    let (addr, _handle) = start_test_server().await;
    let test_id = format!("test-lifecycle-{}", uuid::Uuid::new_v4());

    // 1. Create
    let resp = client()
        .post(format!("{}/v1/discover/index", base_url(&addr)))
        .json(&json!({
            "id": test_id,
            "kind": "component",
            "name": "lifecycle-test",
            "description": "Full lifecycle test component",
            "category": "testing",
            "visibility": "public",
            "owner_id": "user-1",
            "tags": ["test"]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "Step 1 (Create) failed");

    // 2. Update
    let resp = client()
        .put(format!(
            "{}/v1/discover/index/{test_id}",
            base_url(&addr)
        ))
        .json(&json!({
            "id": test_id,
            "kind": "component",
            "name": "lifecycle-test-updated",
            "description": "Updated lifecycle description",
            "category": "testing",
            "visibility": "public",
            "owner_id": "user-1",
            "tags": ["test", "updated"]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "Step 2 (Update) failed");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["name"], "lifecycle-test-updated");

    // 3. Search (structured)
    let resp = client()
        .post(format!("{}/v1/discover/search", base_url(&addr)))
        .json(&json!({ "filters": { "category": "testing" } }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "Step 3 (Search) failed");

    // 4. Telemetry
    let resp = client()
        .post(format!(
            "{}/v1/discover/index/{test_id}/telemetry",
            base_url(&addr)
        ))
        .json(&json!({ "event": "deploy", "user_id": "user-1" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "Step 4 (Telemetry) failed");

    // 5. Compare (single item is valid)
    let resp = client()
        .get(format!(
            "{}/v1/discover/compare?ids={test_id}",
            base_url(&addr)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "Step 5 (Compare) failed");
    let body: Value = resp.json().await.unwrap();
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);

    // 6. Delete
    let resp = client()
        .delete(format!(
            "{}/v1/discover/index/{test_id}",
            base_url(&addr)
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204, "Step 6 (Delete) failed");
}
