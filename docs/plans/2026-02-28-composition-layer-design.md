# Composition Layer Design (P3)

**Goal:** Build the REST API and CLI for composing components into services, grouping services into projects, and publishing services to a reusable catalog.

**Architecture:** Rename `/v1/workflows` to `/v1/services` (no backward compat needed — no users yet). Add project CRUD with auto-provisioned task queues. Add service interfaces for exposing callable endpoints. Add a service catalog for cross-project reuse.

**Tech Stack:** Rust (Axum), Supabase (PostgREST), existing DB schema (`workflows`, `projects`, `service_interfaces`, `public_interfaces`, `task_queues`)

---

## Hierarchy

```
Components (atomic units of work)
  └── Services (composed workflows, with interfaces)
        └── Projects (multi-service systems, shared queue, bundled deploy)
              └── Service Catalog (cross-project reuse)
```

---

## API Routes

### Services (replaces /v1/workflows)

```
POST   /v1/services                          — create service (requires project_id)
GET    /v1/services                          — list user's services
GET    /v1/services/{id}                     — get service
PUT    /v1/services/{id}                     — update service
DELETE /v1/services/{id}                     — delete service
POST   /v1/services/{id}/validate            — validate definition
POST   /v1/services/{id}/deploy              — deploy service
POST   /v1/services/{id}/undeploy            — undeploy service
GET    /v1/services/{id}/status              — deployment status
```

### Service Catalog

```
GET    /v1/services/catalog                  — browse published services (public visibility)
POST   /v1/services/{id}/publish             — publish to catalog
POST   /v1/services/{id}/unpublish           — remove from catalog
POST   /v1/services/catalog/{id}/import      — fork a catalog service into your project
```

### Service Interfaces

```
POST   /v1/services/{id}/interfaces          — create interface
GET    /v1/services/{id}/interfaces          — list interfaces
GET    /v1/services/{id}/interfaces/{iid}    — get interface
PUT    /v1/services/{id}/interfaces/{iid}    — update interface
DELETE /v1/services/{id}/interfaces/{iid}    — delete interface
POST   /v1/services/{id}/interfaces/{iid}/publish    — register Kong route (stub)
POST   /v1/services/{id}/interfaces/{iid}/unpublish  — remove Kong route (stub)
```

### Projects

```
POST   /v1/projects                          — create project (auto-provisions task queue)
GET    /v1/projects                          — list user's projects
GET    /v1/projects/{id}                     — get project
PUT    /v1/projects/{id}                     — update project
DELETE /v1/projects/{id}                     — delete project (cascades services)
POST   /v1/projects/{id}/deploy              — bundled deploy all services
GET    /v1/projects/{id}/status              — aggregated project status
GET    /v1/projects/{id}/services            — list services in project
```

### Components (unchanged)

```
GET    /v1/components                        — list built-in types (no auth)
POST   /v1/components                        — create custom component (auth)
GET    /v1/components/custom                 — list custom components (auth)
DELETE /v1/components/custom/{name}          — delete custom component (auth)
GET    /v1/components/{type}                 — get built-in type (no auth)
```

---

## Data Models

### CreateServiceRequest

```rust
CreateServiceRequest {
    name: String,                  // required, kebab-case
    description: Option<String>,
    project_id: String,            // required UUID
    definition: Value,             // workflow definition (YAML or JSON body)
}
```

On create:
- Auto-assigns `task_queue_id` from parent project's queue
- Sets `status_id` to draft, `visibility_id` to private
- Runs YAML→transform→validate pipeline
- Stores in `workflows` table (DB name unchanged)

### CreateInterfaceRequest

```rust
CreateInterfaceRequest {
    name: String,                  // unique within service
    display_name: Option<String>,
    description: Option<String>,
    interface_type: String,        // "signal" | "query" | "update" | "mcp" | "graphql"
    callable_name: Option<String>, // temporal function name
    input_schema: Option<Value>,
    output_schema: Option<Value>,
    is_public: bool,               // false by default
}
```

Stored in `service_interfaces` table.

### CreateProjectRequest

```rust
CreateProjectRequest {
    name: String,                  // required
    description: Option<String>,
}
```

On create:
- Auto-provisions task queue: `{user_id_prefix}-{project-kebab-name}-queue`
- Creates entry in `task_queues` table atomically
- Sets `is_active = true`

### Service Catalog Import

```rust
ImportServiceRequest {
    project_id: String,            // target project UUID
}
```

On import:
- Copies definition from source service
- Creates new service in target project with independent lifecycle
- Records `parent_workflow_id` pointing to source for lineage

---

## CLI Commands

### Service Commands

```
radium service list [--project <id>]
radium service create <file> --project <id>
radium service show <id>
radium service update <id> <file>
radium service delete <id>
radium service validate <file>
radium service deploy <id>
radium service undeploy <id>
radium service status <id>
radium service publish <id>
radium service unpublish <id>
radium service catalog [--search <query>]
radium service import <catalog_id> --project <id>
radium service interface list <service_id>
radium service interface create <service_id> <file>
radium service interface publish <service_id> <interface_id>
radium service interface unpublish <service_id> <interface_id>
radium service interface delete <service_id> <interface_id>
```

### Project Commands

```
radium project list
radium project create --name <name> [--description <desc>]
radium project show <id>
radium project update <id> --name <name>
radium project delete <id>
radium project deploy <id>
radium project status <id>
radium project services <id>
```

Old top-level commands (`create`, `list`, `show`, `deploy`, etc.) are removed.

---

## Key Design Decisions

1. **DB table stays `workflows`** — the API says "services", the DB says "workflows". No migration needed.
2. **Project = deployment unit** — shared task queue, workers at project level, bundled deploy.
3. **Task queue auto-provisioned** — format `{user_id_prefix}-{project-kebab-name}-queue`.
4. **Service catalog uses visibility** — `public` visibility = listed in catalog.
5. **Import = deep copy** — independent lifecycle, `parent_workflow_id` for lineage.
6. **Kong integration stubbed** — interface publish/unpublish record to DB but don't call Kong API yet.
7. **Workers deferred** — start/stop workers is future work.
8. **Temporal direct interfaces deferred** — service-to-service without HTTP is future.
9. **Cross-project connectors deferred** — `project_connectors` table exists but API deferred.

---

## File Changes

| File | Action |
|------|--------|
| `crates/radium-workflow/src/api/v1/services.rs` | New (renamed from workflows.rs) |
| `crates/radium-workflow/src/api/v1/projects.rs` | New |
| `crates/radium-workflow/src/api/v1/interfaces.rs` | New |
| `crates/radium-workflow/src/api/v1/mod.rs` | Updated routes |
| `crates/radium-workflow/src/api/v1/workflows.rs` | Removed |
| `crates/radium-workflow/src/api/v1/deploy.rs` | Moved under services or integrated |
| `crates/radium-workflow/src/main.rs` | Updated if needed |
| `crates/radium-cli/src/main.rs` | Restructured commands |
| `crates/radium-cli/src/commands/services.rs` | New (replaces workflows.rs) |
| `crates/radium-cli/src/commands/projects.rs` | New |
| `crates/radium-cli/src/commands/workflows.rs` | Removed |

---

## Testing Strategy

- Unit tests for all validation (name format, required fields, project ownership)
- Unit tests for request/response serialization
- Integration stubs (`#[ignore]`) for Supabase-dependent operations
- CLI parse tests for all new commands
- Verify existing test count doesn't regress (currently 809)

---

## Deferred Items

- Kong API integration (publish/unpublish are DB-only stubs)
- Worker management (start/stop/health)
- Temporal direct interfaces (service-to-service code-level calls)
- Project state variables API
