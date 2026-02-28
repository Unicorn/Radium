# Composition Layer — Deferred Items

Items explicitly deferred during P3 implementation (2026-02-28). These are captured in the design doc (`docs/plans/2026-02-28-composition-layer-design.md`) and noted here with implementation context.

---

## 1. Full Deploy Pipeline for Bundled Project Deploy

**What:** `POST /v1/projects/{id}/deploy` currently only updates `status_id` to DEPLOYED for each service. It does NOT run the full deploy pipeline (validation → codegen → compiled code storage) that individual `POST /v1/services/{id}/deploy` runs via `deploy.rs`.

**Where:** `crates/radium-workflow/src/api/v1/projects.rs:506` — has a TODO comment.

**Why deferred:** The deploy pipeline in `deploy.rs` is tightly coupled to a single workflow. Refactoring it to be callable from project-level deploy requires extracting shared logic. Medium effort.

**To implement:** Extract the core deploy logic from `deploy.rs` into a reusable function (validate → codegen → store compiled code → update status), then call it in a loop from `deploy_project`.

---

## 2. Kong API Integration for Published Interfaces

**What:** `POST /v1/services/{id}/interfaces/{iid}/publish` and `unpublish` currently only write/delete rows in the `public_interfaces` table. They do NOT call the Kong Admin API to create/delete actual HTTP routes.

**Where:** `crates/radium-workflow/src/api/v1/interfaces.rs` — publish_interface and unpublish_interface handlers.

**DB columns ready:** `public_interfaces` table has `kong_route_id` and `kong_service_id` columns (currently NULL).

**To implement:** Add a Kong client (HTTP calls to Kong Admin API), create Kong service + route on publish, delete on unpublish, store IDs back to `public_interfaces`.

---

## 3. Temporal Direct Interfaces (Service-to-Service)

**What:** Services should be able to call each other directly via Temporal primitives (signal/query/update) without going through HTTP/Kong. This enables code-level service composition.

**Why deferred:** Requires Temporal SDK integration and a service discovery mechanism for internal routing. Large effort.

**Design note:** The `interface_type` enum already includes "signal", "query", "update" which map to Temporal primitives. The infrastructure is ready for this.

---

## 4. Cross-Project Connectors

**What:** The `project_connectors` table exists in the DB schema (links source_project → target_project via a specific service and nexus endpoint). No API was built for it.

**Why deferred:** Depends on Temporal direct interfaces (#3) to be useful. The connector is the "wiring" between projects.

---

## 5. Worker Management

**What:** Start/stop/health-check Temporal workers at the project level. Currently workers are assumed to be running externally.

**Why deferred:** Requires infrastructure for worker lifecycle (likely Kubernetes or Docker-based). Out of scope for API-first phase.

---

## 6. Service Marketplace Enhancements

**What:** Ratings, reviews, usage statistics, and download counts for catalog services. Currently the catalog is browse-and-import only.

**Why deferred:** Nice-to-have, depends on having actual users.

---

## 7. Project State Variables API

**What:** Shared state/variables across services within a project. The design mentions projects having shared state but no API was built for managing it.

**Why deferred:** Needs design work on what "shared state" means in practice (environment variables? key-value store? typed config?).
