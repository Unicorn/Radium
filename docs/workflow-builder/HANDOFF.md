# Workflow Builder - Agent Handoff Document

This document provides the context needed for the next AI agent to continue development
in the Radium-only context. This is a critical document - read it completely before
starting Phase 5 work.

## Executive Summary

The workflow builder has been migrated from `production-agent-coordinators` to `Radium`.

**What was completed (Phases 1-4):**
- Rust schema validation system (radium-workflow crate)
- TypeScript code generation via Handlebars templates
- Basic workflow compilation (start, activity, end nodes)
- Kong component support (logging, cache, CORS)
- Phase 4 migration to Radium repository

**What remains (Phases 5-9):**
- Variables & State (Phase 5)
- Component Migration (Phase 6)
- Advanced Features (Phase 7)
- Production Hardening (Phase 8)
- Component Builder Agent (Phase 9)

## Current State

### Codebase Locations

```
Radium/
  crates/
    radium-workflow/           <- Rust compiler (65 tests passing)
      src/
        schema/                <- Workflow and component schemas
        codegen/               <- TypeScript generation
        validation/            <- Schema validation
        api/                   <- HTTP API server
        templates/             <- Handlebars templates
      tests/                   <- Unit and integration tests

  apps/
    workflow-builder/          <- React/Next.js frontend
      src/
        components/            <- UI components
        lib/                   <- Utilities and helpers
        app/                   <- Next.js app router pages
      public/                  <- Static assets

  docs/
    workflow-builder/          <- Documentation (you're here)
      original-patterns/       <- TypeScript snapshots for reference
      component-specs/         <- YAML behavior specifications
      architecture-decisions/  <- ADRs explaining key decisions
      migration-records/       <- Template and example records
      plans/                   <- Phase 5-9 plan files
```

### Build Commands

```bash
# Rust compiler
cd Radium
cargo build -p radium-workflow          # Build
cargo test -p radium-workflow           # Run tests (65 passing)
cargo run -p radium-workflow            # Run API server

# React frontend (not yet fully integrated)
cd Radium/apps/workflow-builder
npm install                              # Install deps
npm run build                            # Build
npm run dev                              # Dev server
```

### API Endpoints

The Rust compiler exposes these endpoints (default port 3020):

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/api/compile` | POST | Compile workflow to TypeScript |
| `/api/validate` | POST | Validate workflow schema |

### Supported Component Types

Currently implemented in Rust:
- `trigger` - Workflow start
- `activity` - Activity invocation
- `agent` - AI agent activity
- `condition` - Conditional branching
- `child-workflow` - Child workflow invocation
- `phase` - Execution grouping
- `retry` - Retry wrapper
- `signal` - Signal handlers
- `state-variable` - State management
- `kong-logging` - Kong logging plugin
- `kong-cache` - Kong caching plugin
- `kong-cors` - Kong CORS plugin
- `graphql-gateway` - GraphQL endpoint
- `mcp-server` - MCP protocol server
- `api-endpoint` - REST endpoint registration
- `end` - Workflow end

## Known Issues and Gotchas

### 1. HTML Escaping in Templates

**Issue:** Handlebars escapes HTML by default, breaking expressions like `>` in conditions.

**Fix Applied:** Templates use triple braces `{{{expression}}}` for unescaped output.

**Where:** `templates/workflow.ts.hbs` and related templates.

### 2. Frontend Not Fully Integrated

The React frontend was copied but may need path updates:
- API endpoints may still point to old URLs
- Environment variables may need updating
- Build may fail until dependencies are resolved

**Next step:** Run `npm install` and `npm run build`, fix any path issues.

### 3. Edition Mismatch

Radium workspace uses `edition = "2024"` but radium-workflow uses `edition = "2021"`.
This is intentional - Rust 2024 edition changes may break existing code.
Consider updating to 2024 edition after Phase 5 stabilization.

### 4. Some Components Are Placeholder

Kong components generate configuration objects, not executable code.
Actual Kong integration happens at deployment time (not implemented in Phase 1-4).

## Phase 5 Starting Point

Phase 5 focuses on Variables & State. See `docs/workflow-builder/plans/phase-5-variables-state.md`.

Key tasks:
1. Enhance state variable schema
2. Add storage adapter abstraction
3. Implement database state storage
4. Implement Redis state storage
5. Add variable scoping (workflow vs project)

Starting files:
- `crates/radium-workflow/src/schema/state.rs` (to create)
- `crates/radium-workflow/src/codegen/state.rs` (to create)
- `apps/workflow-builder/src/lib/state/` (existing code to migrate patterns from)

## Key Architectural Decisions

Read the ADRs in `docs/workflow-builder/architecture-decisions/`:

1. **ADR-001:** Why Rust for schema validation
2. **ADR-002:** Component abstraction for external services
3. **ADR-003:** Temporal integration patterns
4. **ADR-004:** Schema structure design
5. **ADR-005:** Code generation approach

## Reference Documentation

### Original Patterns
`docs/workflow-builder/original-patterns/` contains TypeScript snapshots showing
how each component should generate code. Use these for Phase 6 comparison.

### Component Specs
`docs/workflow-builder/component-specs/` contains YAML specs defining inputs,
outputs, and behaviors for each component type.

### Migration Records
`docs/workflow-builder/migration-records/` contains templates for documenting
component migration decisions (for Phase 6 and Phase 9 training).

## External Systems

### Temporal
- Workflow orchestration engine
- Workflows compile to Temporal TypeScript SDK code
- Activities execute in workers
- See `architecture-decisions/ADR-003-temporal-integration.md`

### Kong
- API Gateway for HTTP routing
- Components configure Kong plugins (logging, cache, CORS)
- Configuration is declarative, applied at deployment
- See component specs for Kong components

### Supabase
- Database and auth
- Workflows stored in Supabase tables
- RLS policies for access control
- Schema in `apps/workflow-builder/supabase/`

## Testing Approach

### Rust Tests
```bash
cargo test -p radium-workflow
```
- Unit tests for schema validation
- Unit tests for code generation
- Integration tests for full compilation

### Frontend Tests
```bash
cd apps/workflow-builder
npm run test
```

### Verification
Compare Rust-generated TypeScript with original patterns to ensure behavioral parity.

## Contact and History

This migration was completed by an AI agent in December 2024.
The original codebase was in `production-agent-coordinators/packages/workflow-builder/`.
That repository retains a copy of the pre-migration code for reference.

## Next Steps for Phase 5 Agent

1. Read this document completely
2. Review the Phase 5 plan: `docs/workflow-builder/plans/phase-5-variables-state.md`
3. Verify builds work: `cargo build -p radium-workflow`
4. Run tests: `cargo test -p radium-workflow`
5. Begin Phase 5 implementation

Good luck!
