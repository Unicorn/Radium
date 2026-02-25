# Radium Roadmap

**Updated:** 2026-02-24
**Focus:** CLI/API-first (Web UI on hold)

---

## Completed

- [x] Phases 1-9: Workflow Builder Rust migration (Kong, compiler, 14 components, variables, security, monitoring, performance, component builder agent)
- [x] CLI-First Implementation: Full workflow CRUD, deploy/undeploy/status via CLI and REST API
- [x] Discovery Service: Neo4j graph, semantic search, relationship inference, embedding providers
- [x] Core Components Expansion: 37 components across 11 categories with shared behavior system
- [x] Type Registry: Three-layer type system with Schema.org shadow mapping
- [x] Component Renames: activity→action, child_workflow→child_service, signal→message (backward-compatible)
- [x] Migrate Command: `radium-workflow migrate` for updating old YAML files
- [x] Deploy Endpoint Tests: 23 tests covering full deploy lifecycle
- [x] P1: Infrastructure — Kong routes, CI/CD for discovery, unified Docker Compose
- [x] P2: Component Lifecycle — versioning module (SemVer), change detection (schema diff), component creation API (POST /v1/components)

---

## Active Roadmap

### P3: Composition Layer (core product value)

| ID | Task | Effort | Status | Plan File |
|----|------|--------|--------|-----------|
| P3.1 | Service composition API (compose components → services) | Large | Not Started | — |
| P3.2 | Project management API (group services → projects) | Medium | Not Started | — |
| P3.3 | CLI commands for services and projects | Medium | Not Started | — |

**Dependencies:** P3.1 benefits from P2.3. P3.2 depends on P3.1. P3.3 depends on P3.1 + P3.2.

---

## Deferred (on hold or future)

| ID | Item | Reason | Plan File |
|----|------|--------|-----------|
| D1 | Web UI | On hold — CLI/API focus | `workflow-builder/open-source/` |
| D2 | Phase 10 (npm packages, marketplace, service versioning) | Hosted-product scope | `workflow-builder/hosted-version/` |
| D3 | Playwright E2E tests (Phase 11) | Depends on Web UI | `workflow-builder/open-source/phase-11-playwright-test-infrastructure.md` |
| D4 | Radium Integration R1-R7 | Systems diverged by design; most R-tasks obsolete | `workflow-builder/open-source/radium-integration.md` |
| D5 | Service Builder Agent (OSS + Hosted) | Depends on Web UI | `workflow-builder/open-source/service-builder-agent-expansion-oss.md` |
| D6 | Component Update UI | Depends on Web UI (CLI portion is P2.2) | `workflow-builder/open-source/component-builder-update-capability.md` |
| D7 | Version Bump Detection UI | Depends on Web UI (core logic is P2.2) | `workflow-builder/open-source/version-bump-detection-ui.md` |
| D8 | Hosted Component Scalability | Future hosted-product work | `workflow-builder/hosted-version/hosted-component-scalability.md` |
| D9 | Hosted Service Builder | Future hosted-product work | `workflow-builder/hosted-version/service-builder-agent-expansion-hosted.md` |
| D10 | Embedding as Radium Service | After discovery stabilizes | `future/embedding-as-radium-service.md` |

---

## Execution Order

```
P1.1 Kong routes ──────┐
P1.2 CI/CD discovery ──┼── ✅ DONE
P1.3 Docker Compose ───┘
         │
         ▼
P2.1 Component versioning ──► P2.2 Change detection ── ✅ DONE
P2.3 Component creation API ──────────────────────────
         │
         ▼
P3.1 Service composition ──► P3.2 Project management ──► P3.3 CLI commands  ← NEXT
```

---

## Cloud Deployment Checklist

See `cloud-deployment.md` for the full infrastructure checklist. Key gaps:
- [x] Kong discovery routes (P1.1)
- [x] CI/CD for radium-discovery (P1.2)
- [x] Unified Docker Compose (P1.3)
- [ ] AuraDB provisioning (future, when going to hosted)
- [ ] Reindex job for Neo4j recovery (future)
- [ ] Rate limiting per environment (future)
