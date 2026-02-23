# Cloud Deployment Plan

**Status:** Living document — updated as services are added.

**Goal:** Track what needs to be deployed for the hosted/cloud version of Radium and how the services relate to each other behind a single domain.

---

## Services and Routing

All services sit behind a single domain. Routing is handled by Kong (already in the stack) or an equivalent API gateway.

| Route Prefix | Service | Port (local) | Description | Status |
|---|---|---|---|---|
| `/v1/workflows/*` | radium-workflow | 3020 | Workflow CRUD, compile, validate, deploy | Running |
| `/v1/components/*` | radium-workflow | 3020 | Component schema registry | Running |
| `/v1/discover/*` | radium-discovery | 3030 | Discovery, search, graph traversal | Running (not routed via Kong yet) |
| `/` (web UI) | workflow-builder (Next.js) | 3010 | Web application | On Hold (CLI/API focus) |
| `/auth/*` | Supabase Auth | 54321 | Authentication | Running |
| `/rest/v1/*` | Supabase PostgREST | 54321 | Database API | Running |

---

## Infrastructure Dependencies

| Service | Local Dev | Cloud/Hosted | Status |
|---|---|---|---|
| PostgreSQL (Supabase) | Docker (supabase CLI) | Supabase Cloud | Running locally |
| Neo4j | Docker container (`docker-compose.neo4j.yml`) | Neo4j AuraDB Professional | Running locally |
| Temporal | Docker (temporalite) | Temporal Cloud | Running locally |
| Kong API Gateway | Docker | Kong Cloud / self-hosted | Running locally (missing discovery routes) |
| Redis (optional, caching) | Docker | Managed Redis | Not yet needed |

---

## Environment Configuration

Same codebase, config-driven. Key env vars per service:

**radium-workflow:**
- `SUPABASE_URL`, `SUPABASE_SERVICE_ROLE_KEY`
- `WORKER_SERVICE_URL` (Temporal worker manager)
- `DISCOVERY_SERVICE_URL` (for indexing on publish/deploy)

**radium-discovery:**
- `NEO4J_URI`, `NEO4J_USER`, `NEO4J_PASSWORD` (hard-fails at boot if missing)
- `ANTHROPIC_API_KEY` or `OPENAI_API_KEY` (auto-discovered; hard-fails if neither set)
- `SUPABASE_URL`, `SUPABASE_SERVICE_ROLE_KEY` (for reindex/reconciliation)

**radium-cli:**
- `~/.radium/config.toml` — points at the domain (local or cloud)

---

## Deployment Checklist

- [x] Docker Compose for local dev (all services unified) — `docker-compose.yml` at repo root orchestrates all services
- [x] Neo4j Docker setup script + docs — `docker-compose.neo4j.yml` with neo4j:5-community, APOC plugin, persistent volume
- [x] Neo4j schema initialization (indexes, constraints) — `graph/schema.rs` runs idempotent constraints + fulltext + vector indexes at startup
- [x] Kong route configuration for discovery service — `/v1/discover/*` and `/v1/workflows/*` routes in `kong.yml`
- [x] CI/CD pipeline for radium-discovery — `.github/workflows/radium-discovery.yml` (check, fmt, clippy, test, build, security)
- [ ] AuraDB provisioning for hosted environment
- [x] Embedding provider configuration per environment — hard-fail if no API key; Anthropic + OpenAI providers implemented
- [x] Monitoring/health checks for discovery service — `/health` endpoint returns service name + version
- [ ] Reindex job for Neo4j recovery
- [ ] Rate limiting configuration per environment

---

## Discovery Service Details (radium-discovery)

Implemented in `crates/radium-discovery/`. Provides workflow/component discoverability via Neo4j graph database with semantic embedding search.

**Endpoints:**
| Endpoint | Method | Description |
|---|---|---|
| `/health` | GET | Health check |
| `/v1/discover/index` | POST | Index a workflow/component |
| `/v1/discover/index/{id}` | PUT, DELETE | Update/remove from index |
| `/v1/discover/index/{id}/telemetry` | POST | Record usage telemetry |
| `/v1/discover/compare` | GET | Compare two items |
| `/v1/discover/search` | POST | Semantic + keyword search |
| `/v1/discover/{id}/related` | GET | Find related items |
| `/v1/discover/{id}/dependencies` | GET | Dependency graph |
| `/v1/discover/{id}/dependents` | GET | Reverse dependency graph |

**Architecture:**
- Neo4j graph store with uniqueness constraints, fulltext indexes, and vector indexes
- Embedding providers: Anthropic and OpenAI (auto-discovered from env vars)
- Relationship inference pipeline: definition parsing + schema similarity
- Integrated with radium-workflow: indexes on publish, records telemetry on deploy

**Key Files:**
- `src/api/` — Axum route handlers (index, search, compare, related, telemetry)
- `src/graph/` — Neo4j client, queries, schema initialization
- `src/embeddings/` — Provider trait + Anthropic/OpenAI implementations
- `src/inference/` — Definition parser, schema similarity engine
- `tests/api_integration.rs` — API integration test suite

---

## Open Questions

- CDN / edge caching for discovery search results?
- Multi-region for Neo4j AuraDB?
- Secrets management approach for cloud (Vault, AWS Secrets Manager, etc.)?
- Blue/green or rolling deploys for Rust services?

---

*Last updated: 2026-02-23*
