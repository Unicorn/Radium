# Composition Layer Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the REST API and CLI for composing components into services, grouping services into projects, managing service interfaces, and publishing services to a catalog.

**Architecture:** Rename `/v1/workflows` to `/v1/services` (no users, clean break). Add project CRUD with auto-provisioned task queues. Add service interface CRUD. Add a service catalog for cross-project reuse. Restructure CLI from flat workflow commands to `radium service` and `radium project` subcommands.

**Tech Stack:** Rust (Axum + Serde + Tokio), Supabase (PostgREST via existing `SupabaseClient`), clap (CLI), existing DB schema unchanged

**Working directory:** `/Users/mattbernier/projects/unicorn/Radium/.worktrees/composition-layer`

**Test command:** `cargo test -p radium-workflow --lib` (currently 809 passed)

**Clippy command:** `cargo clippy -p radium-workflow -p radium-cli` (ignore pre-existing warnings in other modules)

---

## Task 1: Rename workflows.rs to services.rs (API routes)

Rename the API module from `workflows` to `services` and update all routes from `/v1/workflows` to `/v1/services`. The handlers stay functionally identical — this is purely a rename pass.

**Files:**
- Rename: `crates/radium-workflow/src/api/v1/workflows.rs` -> `crates/radium-workflow/src/api/v1/services.rs`
- Modify: `crates/radium-workflow/src/api/v1/mod.rs`
- Modify: `crates/radium-workflow/src/api/v1/deploy.rs` (if it references `workflows` module)
- Modify: `crates/radium-workflow/src/main.rs` (if it references `workflows`)

**What to do:**

1. Copy `workflows.rs` to `services.rs` (exact same content)
2. Delete `workflows.rs`
3. In `mod.rs`, change `pub mod workflows;` to `pub mod services;` and update all route paths from `/workflows` to `/services`:
   ```rust
   // Old
   .route("/workflows", post(workflows::create_workflow).get(workflows::list_workflows))
   .route("/workflows/{id}", get(workflows::get_workflow)...)
   // New
   .route("/services", post(services::create_workflow).get(services::list_workflows))
   .route("/services/{id}", get(services::get_workflow)...)
   .route("/services/{id}/validate", post(services::validate_workflow))
   .route("/services/{id}/deploy", post(deploy::deploy_workflow))
   .route("/services/{id}/undeploy", post(deploy::undeploy_workflow))
   .route("/services/{id}/status", get(deploy::workflow_status))
   ```
4. The handler function names can stay as `create_workflow`, `list_workflows`, etc. for now (renaming function names is cosmetic and can happen later)
5. Run tests: `cargo test -p radium-workflow --lib` — should still be 809 passed
6. Commit: `refactor(radium-workflow): rename /v1/workflows routes to /v1/services`

**Important:** Do NOT rename the `workflows` DB table or the Supabase queries inside the handlers. Only the route paths and module name change.

---

## Task 2: Add project_id to service creation + Project CRUD API

Add project management endpoints and require `project_id` when creating services.

**Files:**
- Create: `crates/radium-workflow/src/api/v1/projects.rs`
- Modify: `crates/radium-workflow/src/api/v1/services.rs` (add `project_id` to create)
- Modify: `crates/radium-workflow/src/api/v1/mod.rs` (add project routes)

**What to build in `projects.rs`:**

Follow the exact same patterns as `services.rs` (formerly `workflows.rs`):
- Same `require_auth()` pattern (copy from services.rs or extract to shared helper)
- Same `ProjectError` type (mirrors `WorkflowError`: unauthorized, bad_request, not_found, internal, from_supabase)
- Same `ErrorEnvelope { error: ErrorBody { code, message, details } }` response shape

**Request/Response types:**

```rust
#[derive(Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub task_queue_name: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct ProjectListResponse {
    pub projects: Vec<ProjectSummary>,
    pub total: usize,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}
```

**Supabase insert rows:**

```rust
#[derive(Serialize)]
struct InsertProjectRow {
    name: String,
    description: Option<String>,
    created_by: String,
    task_queue_name: String,
    is_active: bool,
}

#[derive(Serialize)]
struct InsertTaskQueueRow {
    name: String,
    display_name: String,
    description: Option<String>,
    created_by: String,
    is_default: bool,
}
```

**Handlers:**

- `create_project` — POST `/v1/projects`
  1. Validate `name` is non-empty
  2. Generate queue name: `{user_id[..8]}-{kebab(name)}-queue`
  3. Insert into `task_queues` table first
  4. Insert into `projects` table with the queue name
  5. Return 201 with ProjectResponse

- `list_projects` — GET `/v1/projects`
  1. Select from `projects` where `created_by = eq.{user_id}`, order by `created_at.desc`

- `get_project` — GET `/v1/projects/{id}`
  1. Select one from `projects` where `id = eq.{id}` and `created_by = eq.{user_id}`

- `update_project` — PUT `/v1/projects/{id}`
  1. Accept `{ name, description }` body
  2. Update `projects` where `id` and `created_by` match

- `delete_project` — DELETE `/v1/projects/{id}`
  1. Delete from `projects` where `id` and `created_by` match (cascades workflows)

**Route wiring in `mod.rs`:**
```rust
.route("/projects", post(projects::create_project).get(projects::list_projects))
.route("/projects/{id}", get(projects::get_project).put(projects::update_project).delete(projects::delete_project))
```

**Update `services.rs` (create handler):**

Add `project_id` to `InsertWorkflowRow`. In `create_workflow`, extract `project_id` from the request body or a query param. Look up the project to get its `task_queue_id`, then include both in the insert.

Actually — the YAML workflow body doesn't have `project_id`. Add it as a query parameter: `POST /v1/services?project_id=<uuid>`. Extract with `axum::extract::Query`.

**Validation tests** (in `projects.rs` `#[cfg(test)] mod tests`):
- `test_create_project_request_serialization`
- `test_project_response_serialization`
- `test_queue_name_generation` (test the `{user_id[..8]}-{kebab(name)}-queue` logic)
- `test_project_error_unauthorized`
- `test_project_error_bad_request`
- `test_project_error_not_found`
- Integration stubs (`#[ignore]`) for CRUD operations

**Run tests:** `cargo test -p radium-workflow --lib`
**Commit:** `feat(radium-workflow): add project CRUD API with auto-provisioned task queues`

---

## Task 3: Service Interfaces API

Add CRUD endpoints for service interfaces (signal, query, update, mcp, graphql).

**Files:**
- Create: `crates/radium-workflow/src/api/v1/interfaces.rs`
- Modify: `crates/radium-workflow/src/api/v1/mod.rs` (add interface routes)

**What to build in `interfaces.rs`:**

Same auth + error patterns as projects.rs.

**Request/Response types:**

```rust
const VALID_INTERFACE_TYPES: &[&str] = &["signal", "query", "update", "mcp", "graphql"];

#[derive(Deserialize)]
pub struct CreateInterfaceRequest {
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub interface_type: String,       // must be in VALID_INTERFACE_TYPES
    pub callable_name: Option<String>,
    pub input_schema: Option<serde_json::Value>,
    pub output_schema: Option<serde_json::Value>,
    pub is_public: Option<bool>,      // defaults to false
}

#[derive(Serialize, Deserialize)]
pub struct InterfaceResponse {
    pub id: String,
    pub workflow_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub interface_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callable_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
    pub is_public: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct InterfaceListResponse {
    pub interfaces: Vec<InterfaceResponse>,
    pub total: usize,
}
```

**Supabase operations:**
- INSERT into `service_interfaces` with `workflow_id` from path param
- SELECT from `service_interfaces` where `workflow_id = eq.{id}` (verify workflow ownership first by checking `workflows` table `created_by`)
- UPDATE `service_interfaces` by id
- DELETE `service_interfaces` by id

**Handlers:**

- `create_interface` — POST `/v1/services/{id}/interfaces`
  1. Auth + verify service ownership (`workflows` table, `created_by = user_id`)
  2. Validate `interface_type` is in VALID_INTERFACE_TYPES
  3. Validate `name` is non-empty
  4. Insert into `service_interfaces`
  5. Return 201

- `list_interfaces` — GET `/v1/services/{id}/interfaces`
  1. Auth + verify service ownership
  2. Select from `service_interfaces` where `workflow_id = eq.{id}`

- `get_interface` — GET `/v1/services/{id}/interfaces/{iid}`
- `update_interface` — PUT `/v1/services/{id}/interfaces/{iid}`
- `delete_interface` — DELETE `/v1/services/{id}/interfaces/{iid}`

- `publish_interface` — POST `/v1/services/{id}/interfaces/{iid}/publish`
  1. Insert into `public_interfaces` with a generated `route_path`
  2. Kong integration is a stub (just records to DB, does not call Kong API)

- `unpublish_interface` — POST `/v1/services/{id}/interfaces/{iid}/unpublish`
  1. Delete from `public_interfaces` where `service_interface_id = eq.{iid}`

**Route wiring in `mod.rs`:**
```rust
.route("/services/{id}/interfaces", post(interfaces::create_interface).get(interfaces::list_interfaces))
.route("/services/{id}/interfaces/{iid}", get(interfaces::get_interface).put(interfaces::update_interface).delete(interfaces::delete_interface))
.route("/services/{id}/interfaces/{iid}/publish", post(interfaces::publish_interface))
.route("/services/{id}/interfaces/{iid}/unpublish", post(interfaces::unpublish_interface))
```

**Tests:**
- Validation: interface_type must be valid, name required
- Serialization roundtrips
- Error response formatting
- Integration stubs

**Run tests:** `cargo test -p radium-workflow --lib`
**Commit:** `feat(radium-workflow): add service interfaces CRUD API`

---

## Task 4: Service Catalog (publish, browse, import)

Add catalog endpoints for publishing services and importing them into other projects.

**Files:**
- Modify: `crates/radium-workflow/src/api/v1/services.rs` (add catalog handlers)
- Modify: `crates/radium-workflow/src/api/v1/mod.rs` (add catalog routes)

**Handlers (added to `services.rs`):**

- `list_catalog` — GET `/v1/services/catalog`
  1. Auth required
  2. Select from `workflows` where `visibility_id = eq.{PUBLIC_VISIBILITY_ID}` (or team+public)
  3. Exclude user's own services (they can see those via `list`)
  4. Return service summaries

- `publish_service` — POST `/v1/services/{id}/publish`
  1. Auth + ownership check
  2. Update `workflows` set `visibility_id` to PUBLIC (`00000000-0000-0000-0000-000000000003`)
  3. Return success message

- `unpublish_service` — POST `/v1/services/{id}/unpublish`
  1. Auth + ownership check
  2. Update `workflows` set `visibility_id` to PRIVATE (`00000000-0000-0000-0000-000000000001`)

- `import_service` — POST `/v1/services/catalog/{source_id}/import`
  1. Auth required
  2. Body: `{ "project_id": "..." }`
  3. Fetch source service definition (must be public/team visibility)
  4. Create a new service in the target project, copying the definition
  5. Set `parent_workflow_id` to source service ID for lineage
  6. Return the newly created service

**Route wiring (order matters — `catalog` before `{id}`):**
```rust
.route("/services/catalog", get(services::list_catalog))
.route("/services/catalog/{source_id}/import", post(services::import_service))
// ... existing service routes ...
.route("/services/{id}/publish", post(services::publish_service))
.route("/services/{id}/unpublish", post(services::unpublish_service))
```

**Constants:**
```rust
const PUBLIC_VISIBILITY_ID: &str = "00000000-0000-0000-0000-000000000003";
const PRIVATE_VISIBILITY_ID: &str = "00000000-0000-0000-0000-000000000001";
```

**Tests:**
- Catalog visibility constants
- Import request/response serialization
- Error responses for non-existent or private source service
- Integration stubs

**Run tests:** `cargo test -p radium-workflow --lib`
**Commit:** `feat(radium-workflow): add service catalog (publish, browse, import)`

---

## Task 5: Project deploy + status (bundled deploy)

Add bundled deployment for projects.

**Files:**
- Modify: `crates/radium-workflow/src/api/v1/projects.rs` (add deploy + status handlers)
- Modify: `crates/radium-workflow/src/api/v1/mod.rs` (add routes)

**Handlers:**

- `deploy_project` — POST `/v1/projects/{id}/deploy`
  1. Auth + project ownership
  2. Select all workflows where `project_id = eq.{id}` and `created_by = eq.{user_id}`
  3. For each service, call the deploy pipeline (reuse logic from `deploy.rs`)
  4. Return aggregated result: `{ project_id, services_deployed: N, services_failed: N, results: [...] }`

- `project_status` — GET `/v1/projects/{id}/status`
  1. Auth + project ownership
  2. Select all workflows in project with their deployment status
  3. Return: `{ project_id, total_services, deployed, draft, error, services: [...] }`

- `list_project_services` — GET `/v1/projects/{id}/services`
  1. Auth + project ownership
  2. Select workflows where `project_id = eq.{id}` and `created_by = eq.{user_id}`

**Route wiring:**
```rust
.route("/projects/{id}/deploy", post(projects::deploy_project))
.route("/projects/{id}/status", get(projects::project_status))
.route("/projects/{id}/services", get(projects::list_project_services))
```

**Tests:**
- Deploy response serialization
- Status response aggregation logic
- Integration stubs

**Run tests:** `cargo test -p radium-workflow --lib`
**Commit:** `feat(radium-workflow): add project deploy and status endpoints`

---

## Task 6: CLI restructure (radium service + radium project)

Restructure the CLI from flat workflow commands to `radium service` and `radium project` subcommands.

**Files:**
- Create: `crates/radium-cli/src/commands/services.rs` (replaces workflows.rs)
- Create: `crates/radium-cli/src/commands/projects.rs`
- Delete: `crates/radium-cli/src/commands/workflows.rs`
- Modify: `crates/radium-cli/src/commands/mod.rs`
- Modify: `crates/radium-cli/src/main.rs`

**CLI structure:**

```rust
// main.rs
#[derive(Subcommand)]
enum Commands {
    Login { url: String, key: String },
    Components { action: Option<ComponentAction> },
    Discover { action: DiscoverAction },
    /// Service management (create, deploy, interfaces, catalog)
    Service {
        #[command(subcommand)]
        action: ServiceAction,
    },
    /// Project management (create, deploy, status)
    Project {
        #[command(subcommand)]
        action: ProjectAction,
    },
    /// Migrate workflow files to canonical component names
    Migrate { files: Vec<String>, dry_run: bool, output_dir: Option<String> },
}
```

**`commands/services.rs`:**

```rust
#[derive(Subcommand, Clone)]
pub enum ServiceAction {
    /// List services
    List {
        #[arg(long)]
        project: Option<String>,
    },
    /// Create a service from a definition file
    Create {
        file: String,
        #[arg(long, required = true)]
        project: String,
    },
    Show { id: String },
    Update { id: String, file: String },
    Delete { id: String },
    Validate { file: String },
    Deploy { id: String },
    Undeploy { id: String },
    Status { id: String },
    Publish { id: String },
    Unpublish { id: String },
    /// Browse the service catalog
    Catalog {
        #[arg(long)]
        search: Option<String>,
    },
    /// Import a service from the catalog
    Import {
        catalog_id: String,
        #[arg(long, required = true)]
        project: String,
    },
    /// Manage service interfaces
    Interface {
        #[command(subcommand)]
        action: InterfaceAction,
    },
}

#[derive(Subcommand, Clone)]
pub enum InterfaceAction {
    List { service_id: String },
    Create { service_id: String, file: String },
    Publish { service_id: String, interface_id: String },
    Unpublish { service_id: String, interface_id: String },
    Delete { service_id: String, interface_id: String },
}
```

Each handler follows the same pattern as existing `commands/workflows.rs`:
1. Load profile + build ApiClient
2. Call the API with `/v1/services/...` or `/v1/projects/...`
3. Return pretty-printed JSON

**`commands/projects.rs`:**

```rust
#[derive(Subcommand, Clone)]
pub enum ProjectAction {
    List,
    Create {
        #[arg(long, required = true)]
        name: String,
        #[arg(long)]
        description: Option<String>,
    },
    Show { id: String },
    Update {
        id: String,
        #[arg(long)]
        name: Option<String>,
    },
    Delete { id: String },
    Deploy { id: String },
    Status { id: String },
    Services { id: String },
}
```

**Update `main.rs` handler routing:**
Replace all old top-level `Create`, `List`, `Show`, `Deploy`, etc. with the new `Service { action }` and `Project { action }` match arms.

**CLI parse tests** (in main.rs `#[cfg(test)]`):
- `test_parse_service_list`
- `test_parse_service_create_with_project`
- `test_parse_service_deploy`
- `test_parse_service_interface_list`
- `test_parse_project_create`
- `test_parse_project_deploy`
- `test_parse_project_services`

**Run tests:** `cargo test -p radium-cli`
**Commit:** `feat(radium-cli): restructure CLI to radium service + radium project subcommands`

---

## Summary

| Task | Description | Depends On |
|------|-------------|------------|
| 1 | Rename workflows.rs to services.rs, update routes | — |
| 2 | Project CRUD API + project_id on service create | Task 1 |
| 3 | Service interfaces CRUD API | Task 1 |
| 4 | Service catalog (publish, browse, import) | Task 1 |
| 5 | Project deploy + status (bundled) | Task 2 |
| 6 | CLI restructure (service + project subcommands) | Tasks 1-5 |

Tasks 2, 3, and 4 can run in parallel after Task 1. Task 5 depends on Task 2. Task 6 depends on all API tasks.
