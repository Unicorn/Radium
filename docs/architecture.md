# Radium Architecture

Radium is a workflow orchestration platform. Users design workflows from composable components (activities, triggers, conditionals, loops, agents, etc.), compile them to TypeScript, and deploy them as Temporal workers. The platform provides a CLI, a web UI, and REST APIs for the full lifecycle: author, validate, compile, deploy, execute, and observe.

## High-Level Architecture

```
                    +------------------+
                    |   Web UI         |
                    |  (Next.js :3010) |
                    +--------+---------+
                             |
              +--------------+--------------+
              |                             |
     +--------v---------+         +--------v---------+
     |  radium-workflow  |         |   Workflow CLI   |
     |   CLI (Rust)      |         |   (radium-cli)   |
     +--------+----------+         +--------+---------+
              |                             |
              +-------------+---------------+
                            |
                   +--------v---------+
                   |   Kong Gateway   |
                   |   (:8000/:8001)  |
                   +--------+---------+
                            |
              +-------------+---------------+
              |                             |
     +--------v---------+         +--------v---------+
     | radium-workflow   |         | radium-discovery |
     | API (Rust :3020)  |         | API (Rust :3030) |
     +--+-----+------+--+         +--------+---------+
        |     |      |                     |
   +----+  +--+--+ +-+------+        +----+----+
   |       |     | |         |        |         |
+--v--+ +--v--+ +-v------+ +-v----+ +-v------+
|Supa-| |Temp-| |Kong    | |Go-   | |Neo4j   |
|base | |oral | |Admin   | |True  | |(:7474) |
|Post-| |(:   | |API     | |Auth  | |        |
|gREST| |7233)| |(:8001) | |(:9999| +--------+
+-----+ +-----+ +--------+ +------+
```

**Data flow summary**: Users interact through the Web UI or CLI. Requests pass through Kong (API gateway) to the Rust backend services. `radium-workflow` handles workflow CRUD, compilation, deployment, and gateway traffic. `radium-discovery` provides semantic search over components and services using Neo4j. Authentication is handled by GoTrue (Supabase Auth). Data is stored in PostgreSQL via PostgREST (Supabase).

## Crate Map

| Crate | Description | Key Dependencies |
|---|---|---|
| `radium-workflow` | Workflow compiler and HTTP API server. Validates workflow definitions, generates TypeScript code, manages deploy pipeline, and handles gateway traffic. | axum, handlebars, tonic, reqwest |
| `radium-discovery` | Discovery service for component/service search with vector embeddings. Backed by Neo4j. | axum, neo4rs |
| `radium-cli` (apps/cli) | Main Radium CLI. Manages workspaces, orchestrates agents, trains models, and provides a TUI. | radium-core, radium-orchestrator, clap, ratatui (via radium-tui) |
| `radium-workflow-cli` (crates/radium-cli) | CLI client for the Workflow API. Manages services, projects, components, and migrations. | clap, reqwest |
| `radium-core` | Core library. gRPC server, sandbox execution (Docker/Seatbelt), hooks, context files, privacy, file watching, and MCP protocol support. | tonic, rusqlite, tree-sitter, ed25519-dalek |
| `radium-abstraction` | Trait definitions and shared abstractions for LLM providers, tool calling, and streaming. | async-trait, tokio |
| `radium-models` | LLM provider implementations (Anthropic, OpenAI, etc.) with streaming support. | reqwest, radium-abstraction |
| `radium-orchestrator` | Multi-agent orchestration. Skill routing, A/B testing, code analysis, and search. | radium-ml, tree-sitter, regex |
| `radium-ml` | ML utilities. Python bridge (subprocess or HTTP) for inference. Optional ONNX support planned. | radium-abstraction, tokio |
| `radium-training` | Training data management. Dataset creation, versioning, and deduplication. | walkdir, sha2 |
| `radium-tui` (apps/tui) | Terminal user interface with syntax highlighting, chat, and visual effects. | ratatui, crossterm, tachyonfx, syntect |

## Service Topology

All services are defined in `docker-compose.yml` and communicate over the `radium-network` Docker network.

| Service | Image / Build | Port(s) | Role |
|---|---|---|---|
| `db` | `postgres:15-alpine` | 54332:5432 | Primary PostgreSQL database. Stores all application data. Shared by Supabase services and Temporal. |
| `auth` | `supabase/gotrue:v2.143.0` | (internal 9999) | GoTrue authentication server. Issues JWTs, manages user signup/login. |
| `rest` | `postgrest/postgrest:v12.0.1` | (internal 3000) | PostgREST. Exposes PostgreSQL tables as a REST API for the Supabase client. |
| `kong` | `kong:3.8` | 8000 (proxy), 8001 (admin) | API gateway. Routes external requests to internal services. Manages published interface routes dynamically. |
| `kong-database` | `postgres:15` | (internal) | Dedicated PostgreSQL instance for Kong configuration storage. |
| `kong-migration` | `kong:3.8` | (none) | One-shot container that bootstraps the Kong database schema. |
| `temporal` | `temporalio/auto-setup:latest` | 7233, 7239 | Temporal server. Executes compiled workflow workers with durable state, retries, and scheduling. |
| `radium-workflow` | Built from `crates/radium-workflow/Dockerfile` | 3020:3000 | Workflow API server (Rust/axum). Core backend for all workflow operations. Profile: `rust`. |
| `radium-discovery` | Built from `crates/radium-discovery/Dockerfile` | 3030:3030 | Discovery API server (Rust/axum). Semantic search over components and services. Profile: `rust`. |
| `neo4j` | `neo4j:5-community` | 7474, 7687 | Graph database for the discovery service. Stores component/service relationships and vector embeddings. |
| `inbucket` | `inbucket/inbucket:3.0.3` | 9000, 2500 | Local email server for development. Captures emails sent by GoTrue. |

## Web UI (Workflow Builder)

The web UI is a Next.js application at `apps/workflow-builder/` running on port 3010. Key characteristics:

- **Framework**: Next.js 14 with React 18, Tailwind CSS, and Tamagui for cross-platform components
- **State management**: TanStack React Query with tRPC for type-safe API calls
- **Workflow editor**: React Flow for visual drag-and-drop workflow building
- **Auth**: Supabase Auth (GoTrue) via `@supabase/ssr`
- **Temporal integration**: `@temporalio/client` and `@temporalio/worker` SDKs for workflow execution
- **Testing**: Vitest (unit/integration), Playwright (E2E), k6 (performance)

## Data Flow

### Creating a Workflow Service

```
1. User creates a service via CLI or Web UI
2. Request: POST /v1/services  (with Bearer API key)
3. radium-workflow validates the request
4. radium-workflow inserts a row into the `workflows` table via Supabase REST API
5. If discovery is enabled, the service is indexed in Neo4j
6. Response: 201 Created with the new service record
```

### Deploying a Service

```
1. User triggers deploy via CLI or Web UI
2. Request: POST /v1/services/{id}/deploy
3. Deploy pipeline (deploy_pipeline.rs) executes:
   a. Fetch workflow definition from Supabase
   b. Parse and validate the workflow graph (validation/)
   c. Generate TypeScript code via Handlebars templates (codegen/)
      - workflow.ts (main workflow logic)
      - activities.ts (activity implementations)
      - worker.ts (Temporal worker bootstrap)
      - package.json, tsconfig.json
   d. Store compiled code in the `workflow_compiled_code` table
   e. Update workflow status to "deployed"
4. Response: 200 OK with compilation result
```

### Publishing an Interface (Gateway Setup)

```
1. User publishes a service interface
2. Request: POST /v1/services/{id}/interfaces/{iid}/publish
3. radium-workflow creates a Kong route via the Kong Admin API (:8001)
   - Route maps a public path to /v1/gateway/{interface_id}
4. A record is inserted into `public_interfaces` with the Kong route/service IDs
5. External traffic flow:
   a. External request hits Kong (:8000) at the published path
   b. Kong forwards to radium-workflow at /v1/gateway/{interface_id}
   c. Gateway handler signals the corresponding Temporal workflow
   d. Returns 202 Accepted immediately (async processing)
```

## Authentication Model

Radium uses three authentication mechanisms depending on the access path:

| Context | Mechanism | Details |
|---|---|---|
| **Web UI** | GoTrue JWT | Users sign up/login via Supabase Auth. The Next.js app uses `@supabase/ssr` to manage sessions. JWTs are validated by PostgREST for direct database access. |
| **Management API** | API keys (Bearer token) | CLI and programmatic access use API keys stored as SHA-256 hashes in the `api_keys` table. Keys are sent as `Authorization: Bearer sk_live_...`. The Rust server hashes the key and validates against Supabase. |
| **Gateway (published interfaces)** | None (public) / Interface API keys (planned) | Published interface endpoints are currently public. Interface-level API key authentication is planned as a future feature. |

API key validation flow:
1. Extract `Bearer` token from the `Authorization` header
2. SHA-256 hash the raw key (lowercase hex, 64 chars)
3. Query `api_keys` table via Supabase REST for matching active record
4. Check expiration timestamp if set
5. Return `AuthenticatedUser` with `user_id`, `project_id`, and `key_id`

## Key Modules in radium-workflow

| Module | Purpose |
|---|---|
| `api/` | HTTP router (axum). Top-level routes (`/compile`, `/validate`, `/health`) and the `/v1` sub-router. Includes auth middleware, error handling, and request state. |
| `api/v1/` | Versioned API handlers: `services`, `projects`, `components`, `interfaces`, `gateway`, `deploy`, `state_variables`. |
| `codegen/` | TypeScript code generation using Handlebars templates. Produces workflow, activity, and worker files from validated definitions. |
| `validation/` | Workflow graph validation. Checks for cycles, unreachable nodes, missing connections, and schema compliance. |
| `schema/` | Workflow definition schema types. Defines the structure of nodes, edges, and component configurations. |
| `deploy_pipeline.rs` | End-to-end deploy pipeline: fetch, validate, codegen, store, update status. Used by both single-service and project-level deploy. |
| `security/` | Rate limiting (sliding window and token bucket), input sanitization, and audit logging. |
| `monitoring/` | Health checks, metrics registry, and trace collection. |
| `kong_client.rs` | Kong Admin API client. Creates/deletes routes and services for published interfaces. |
| `temporal_client.rs` | Temporal gRPC client. Signals gateway workflows and manages workflow lifecycle. |
| `supabase/` | Supabase REST API client. Wraps PostgREST calls for all database operations. |
| `discovery/` | Client for the radium-discovery service. Indexes components and services for search. |
| `versioning.rs` | Semantic versioning utilities. Parses, compares, and bumps component versions. |
| `expressions/` | Expression parser and evaluator. Supports variable references and TypeScript code generation for workflow expressions. |
| `migration/` | Component migration framework. Handles schema changes across component versions. |
| `verification/` | Post-compilation verification pipeline (tsc type-checking, ESLint). |
| `change_detection.rs` | Detects changes in workflow definitions to enable incremental compilation. |
| `performance/` | Compilation cache and profiler. Tracks compilation stages and enables caching for faster rebuilds. |
| `yaml_format/` | YAML workflow definition parser. Transforms YAML workflow files into the internal schema representation. |

## Database Schema

The database is PostgreSQL, accessed via PostgREST (Supabase). Schema files are in `apps/workflow-builder/supabase/volumes/db/init/`.

### Core Tables

| Table | Purpose |
|---|---|
| `users` | Application users. Linked to GoTrue auth via `auth_user_id`. Has a role reference. |
| `projects` | Top-level organizational unit. Groups workflows, connectors, and workers under a task queue. |
| `workflows` | Workflow service definitions. Stores the JSON definition, compiled TypeScript, Temporal IDs, scheduling config, and deployment status. |
| `workflow_nodes` | Individual nodes in a workflow graph. References a component and stores position/config. |
| `workflow_edges` | Connections between workflow nodes. Defines the execution graph. |
| `components` | Reusable workflow components (activities, agents, transforms, connectors, triggers). Has input/output/config schemas. |
| `activities` | Registered Temporal activities with function metadata and schemas. |

### Deployment and Execution

| Table | Purpose |
|---|---|
| `workflow_compiled_code` | Stores generated TypeScript artifacts (workflow, activities, worker, package.json, tsconfig). |
| `workflow_executions` | Execution history. Tracks Temporal workflow/run IDs, status, duration, input/output. |
| `workflow_workers` | Running worker instances. Tracks process ID, heartbeat, resource usage. |
| `task_queues` | Named Temporal task queues. Each project has a default queue. |

### Interfaces and Gateway

| Table | Purpose |
|---|---|
| `service_interfaces` | Defines callable interfaces on a workflow (signal, query, update, MCP, GraphQL). |
| `public_interfaces` | Published interfaces exposed via Kong. Stores Kong route/service IDs. |
| `service_interface_endpoints` | HTTP endpoint definitions for service interfaces. |
| `api_keys` | Hashed API keys for authentication. Scoped to user and optionally to project/interface. RLS-enabled. |

### Signals and State

| Table | Purpose |
|---|---|
| `workflow_signals` | Signal definitions for a workflow. Can be linked to a work queue. |
| `workflow_queries` | Query definitions for a workflow. Can be linked to a work queue. |
| `workflow_work_queues` | Internal work queues within a workflow (signal/query pairs for buffered processing). |
| `workflow_state_variables` | Per-workflow state variable definitions with type and storage config. |
| `project_state_variables` | Per-project state variable definitions shared across workflows. |

### Observability

| Table | Purpose |
|---|---|
| `component_metrics` | Per-invocation component execution metrics (duration, memory, CPU, retry info). |
| `component_usage_daily` | Daily aggregated component usage statistics. |
| `workflow_execution_metrics` | Per-execution workflow metrics (trigger type, activity count, resource usage). |
| `workflow_usage_daily` | Daily aggregated workflow execution statistics. |
| `resource_events` | External resource access events (API calls, DB queries, LLM usage with token counts). |
| `resource_usage_daily` | Daily aggregated resource usage statistics. |
| `activity_statistics` | Per-activity execution statistics within a project. |
| `state_variable_metrics` | Access patterns and size tracking for state variables. |

### Lookup Tables

| Table | Purpose |
|---|---|
| `user_roles` | Role definitions (admin, developer, viewer) with permission sets. |
| `workflow_statuses` | Status enum (draft, active, archived, deploying, error). |
| `component_types` | Component type classification (activity, agent, transform, connector, trigger). |
| `component_visibility` | Visibility levels (private, team, public). |
| `activity_categories` | Activity categorization (communication, data, integration, utility, AI). |
| `component_categories` | Hierarchical category tree for component organization and UI display. |

### Other Tables

| Table | Purpose |
|---|---|
| `connectors` | External service connectors with encrypted credentials and OAuth config. |
| `project_connectors` | Cross-project service connections (Nexus endpoints). |
| `agent_prompts` | Stored AI agent prompt templates with versioning and deprecation support. |
| `agent_builder_sessions` | Interactive agent prompt building session history. |
| `agent_test_sessions` | Agent testing sessions linked to Temporal executions. |
| `component_category_mapping` | Many-to-many mapping between components and categories. |
| `component_keywords` | Search keywords for components with relevance scores. |
| `component_use_cases` | Documented use cases for components. |
