# P4: OSS Polish — Deploy Pipeline, Kong Integration, State Variables, Gateway Workflows

## Overview

P4 completes the core open-source Radium workflow builder by connecting the composition layer (P3) to real infrastructure. After P4, users can define services and interfaces, deploy projects as a unit, publish interfaces with durable edge buffering, and manage state variables — all through the CLI/API.

## Goals

1. **Bundled deploy** — Deploy all services in a project with a single command, with fail-fast partial deploy and reporting
2. **Kong integration** — Dynamically create/remove API gateway routes when interfaces are published/unpublished
3. **State variables API** — CRUD endpoints for service-scoped and project-scoped state variables
4. **Gateway workflows** — Durable edge buffering via Temporal gateway workflows that accept traffic even when internal services are down

## Architecture Decisions

### Deploy Strategy: Fail-Fast Partial Deploy
When deploying a project, services are deployed sequentially. On the first failure, deployment stops. Already-deployed services remain running. A deploy report summarizes successes, the failure, and skipped services. This allows users to diagnose and fix the failing service without losing progress on successful deployments.

### Kong Mode: DB-Backed with Admin API
Kong switches from declarative/dbless mode to PostgreSQL-backed mode. This enables real-time route management via the Kong Admin API. Kong gets its own Postgres instance (separate from Supabase). Existing declarative routes are seeded on migration.

### State Variable Scope: Configurable (Service or Project)
State variables are service-scoped by default. They can optionally be promoted to project-scope for cross-service sharing. Two existing DB tables (`workflow_state_variables`, `project_state_variables`) support this model.

### Gateway Workflows: Temporal Durable Edge Buffering
Published interfaces get a long-running Temporal gateway workflow that:
- Accepts incoming HTTP data as Temporal signals (durable, ordered)
- Forwards to internal service workflows via activities with retry policies
- Survives internal service failures, deploy gaps, and restarts
- Uses continue-as-new to manage event history growth

This prevents data loss when a project is partially deployed (some services up, others failed).

### Temporal Integration Model
Radium uses Rust codegen to convert service definitions into Temporal TypeScript SDK code. The Temporal Rust SDK lacks full feature support, so TypeScript is the execution target. Users never see Temporal, Kong, or TypeScript — they interact with Radium concepts (components, services, projects, interfaces) via CLI/API/UI.

---

## P4.1 — Bundled Deploy Pipeline

### Goal
Extract reusable deploy logic from `deploy.rs`, implement `deploy_project` with fail-fast partial deploy and detailed reporting.

### Design

**Extract core pipeline:**
Move the validate → codegen → store → update-status steps from the `deploy_workflow` handler into a reusable `deploy_single_service()` function in a new `deploy_pipeline` module. The existing single-service deploy handler calls into this extracted function.

**Project deploy flow:**
1. Authenticate and verify project ownership
2. Fetch all services in the project
3. For each service (sequentially):
   - Call `deploy_single_service()`
   - On success: add to `deployed` list
   - On failure: add to `failed`, add remaining to `skipped`, stop
4. Update project status based on results
5. Return deploy report

**Deploy report:**
```
{
  project_id: String,
  deployed: [{ service_id, compiled_at }],
  failed: Option<{ service_id, error }>,
  skipped: [{ service_id, reason }]
}
```

**Codegen extension:**
When a service has published interfaces, the codegen context includes interface metadata from `service_interfaces`. The existing templates already generate signal/query/update handlers — this wires the interface definitions into that generation.

### Files
- New: `crates/radium-workflow/src/deploy_pipeline.rs`
- Modify: `crates/radium-workflow/src/api/v1/deploy.rs` (call extracted functions)
- Modify: `crates/radium-workflow/src/api/v1/projects.rs` (implement real deploy_project)
- Modify: `crates/radium-workflow/src/codegen/typescript.rs` (interface-aware generation)

---

## P4.2 — Kong Integration

### Goal
Switch Kong to DB-backed mode, build a Rust Kong Admin API client, wire interface publish/unpublish to create/remove real Kong routes.

### Design

**Kong Admin Client (`KongClient`):**
- HTTP client wrapping Kong Admin API (port 8001)
- Methods: `create_service()`, `create_route()`, `delete_route()`, `delete_service()`, `list_routes()`
- Added to `AppState` for dependency injection

**Docker Compose changes:**
- Add `kong-database` service (PostgreSQL for Kong)
- Switch Kong from `KONG_DATABASE: "off"` to `KONG_DATABASE: postgres`
- Add Kong migration init container
- Seed existing routes (radium-workflow, radium-discovery) on first boot

**Publish flow (interfaces.rs):**
1. Create Kong service pointing to gateway HTTP handler (P4.4)
2. Create Kong route with path `/api/{kebab_service}/{kebab_interface}`
3. Apply default plugins (rate-limiting, cors, correlation-id)
4. Store Kong route ID and service ID in `public_interfaces` table

**Unpublish flow:**
1. Delete Kong route by stored ID
2. Delete Kong service by stored ID
3. Clear Kong IDs from `public_interfaces` record

**Kong Admin API calls:**
- `POST /services` — create upstream service
- `POST /services/{id}/routes` — create route
- `DELETE /routes/{id}` — remove route
- `DELETE /services/{id}` — remove service
- `POST /services/{id}/plugins` — add plugins

### Files
- New: `crates/radium-workflow/src/kong_client.rs`
- Modify: `crates/radium-workflow/src/api/v1/interfaces.rs` (wire Kong calls)
- Modify: `docker-compose.yml` (Kong DB mode)
- New: Kong migration/seed scripts

---

## P4.3 — State Variables API

### Goal
CRUD endpoints for state variables with configurable scope (service-scoped by default, project-scoped for sharing).

### Design

**Service-scoped endpoints:**
- `POST /v1/services/{id}/variables` — create variable
- `GET /v1/services/{id}/variables` — list variables
- `GET /v1/services/{id}/variables/{var_id}` — get variable
- `PUT /v1/services/{id}/variables/{var_id}` — update variable
- `DELETE /v1/services/{id}/variables/{var_id}` — delete variable

**Project-scoped endpoints (shared variables):**
- `POST /v1/projects/{id}/variables` — create shared variable
- `GET /v1/projects/{id}/variables` — list shared variables
- Same CRUD pattern as service-scoped

**Data model:**
Uses existing DB tables:
- `workflow_state_variables` — service-scoped (id, workflow_id, name, type, storage_type, schema, storage_config)
- `project_state_variables` — project-scoped (id, project_id, name, type, storage_type, schema, storage_config)
- `state_variable_metrics` — observability (access_count, size_bytes, last_accessed)

**Validation:**
- Name uniqueness within scope (no two variables with same name in same service/project)
- Valid type values
- Valid JSON schema if provided
- Ownership verification (reuse existing patterns)

**Codegen integration:**
During deploy, state variables are fetched from DB and injected into the codegen context. The existing `state_generator.rs` module generates TypeScript state management code from variable definitions.

**CLI commands:**
- `radium service variable list/create/show/update/delete`
- `radium project variable list/create/show/update/delete`

### Files
- New: `crates/radium-workflow/src/api/v1/state_variables.rs`
- Modify: `crates/radium-workflow/src/api/v1/mod.rs` (add routes)
- Modify: `crates/radium-cli/src/commands/services.rs` (variable subcommands)

---

## P4.4 — Gateway Workflows (Durable Edge Buffering)

### Goal
When an interface is published, start a long-running Temporal gateway workflow that accepts incoming data, buffers it durably, and forwards to the internal service — surviving deploy failures, service downtime, and restarts.

### Design

**Gateway Workflow (TypeScript, generated by codegen):**
- One gateway workflow per published interface
- Lifecycle: starts on interface publish, terminates on unpublish
- Receives incoming HTTP data as Temporal **signals** (durable, ordered)
- Processes signals sequentially via an **activity** that calls the internal service workflow
- Activity retry policy: exponential backoff, configurable max attempts, non-retryable error types
- **Continue-as-new** every N processed signals (default 1000) to manage event history size
- Query handler exposes buffer depth (pending unprocessed signals)

**Request flow:**
```
Client → Kong → Gateway HTTP handler (Rust) → Signal to gateway workflow (Temporal)
                                                       ↓
                                                 Process signal
                                                       ↓
                                                 Activity → internal service workflow
                                                       ↓ (retry on failure with backoff)
                                                 Delivery confirmed
```

**Gateway HTTP Handler (Rust):**
- New handler at the path Kong routes to for published interfaces
- Receives HTTP request, validates payload format
- Sends signal to the corresponding gateway workflow via Temporal gRPC client
- Returns 202 Accepted immediately (async processing)
- If gateway workflow not found: returns 503 Service Unavailable

**Temporal Client (Rust):**
- New module wrapping Temporal's gRPC API
- Methods: `start_workflow()`, `signal_workflow()`, `terminate_workflow()`, `query_workflow()`
- Used by: publish_interface (start gateway), unpublish_interface (terminate gateway), gateway handler (signal)
- Connection configured via environment variables (TEMPORAL_ADDRESS, TEMPORAL_NAMESPACE)

**Codegen additions:**
- New template: `gateway.ts.hbs` — gateway workflow code
- New template: `gateway_worker.ts.hbs` — worker that runs gateway workflows
- `GeneratedCode` struct extended with optional `gateway` and `gateway_worker` fields
- Generated only when service has published interfaces

**Continue-as-new strategy:**
- Gateway tracks count of processed signals
- At threshold, drains remaining buffered signals from the signal channel
- Calls `continueAsNew()` with pending signals as initial state
- New execution picks up immediately — zero message loss, zero downtime

### Files
- New: `crates/radium-workflow/src/temporal_client.rs`
- New: `crates/radium-workflow/src/api/v1/gateway.rs`
- New: `crates/radium-workflow/src/codegen/templates/gateway.ts.hbs`
- New: `crates/radium-workflow/src/codegen/templates/gateway_worker.ts.hbs`
- Modify: `crates/radium-workflow/src/codegen/mod.rs` + `typescript.rs`
- Modify: `crates/radium-workflow/src/api/v1/interfaces.rs` (start/stop gateway)
- Modify: `crates/radium-workflow/src/api/v1/mod.rs` (gateway routes)

---

## Execution Order

Sequential pipeline — each item builds on the previous:

1. **P4.1 (Bundled Deploy)** — Foundation. Extracts reusable deploy logic needed by everything else.
2. **P4.2 (Kong Integration)** — Routes. Needs deploy pipeline for context; gateway needs Kong routes.
3. **P4.3 (State Variables)** — Data. Independent API but benefits from deploy pipeline for codegen integration.
4. **P4.4 (Gateway Workflows)** — Capstone. Needs Kong routes (P4.2) and codegen extensions (P4.1).

## Deferred to P5

- **Gateway workflow resilience tuning** — advanced retry policies, dead letter queues, overflow strategies
- **Cross-project connectors** (SaaS feature)
- **Worker auto-management** (SaaS feature)
- **Service marketplace** (SaaS feature)

## Testing Strategy

- Unit tests for all new modules (deploy_pipeline, kong_client, state_variables, temporal_client, gateway)
- Integration tests against real Supabase and Kong instances (per project testing policy: no mocking what we own)
- Codegen tests verifying generated TypeScript compiles and contains expected patterns
- Gateway workflow tests using Temporal test server
