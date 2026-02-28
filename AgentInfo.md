# Agent Information - Radium Project

This document contains critical information for all AI agents working on the Radium project.

## Stack Information

**Languages:** Rust + TypeScript (hybrid monorepo)
**Build System:** Nx (JS/TS) + Cargo (Rust), managed with Bun
**Rust Stack:** Tokio, Tonic/gRPC, neo4rs (Neo4j), Cargo workspace
**TypeScript Apps:** Tauri desktop, Next.js web, React Native mobile
**UI Framework:** Tamagui (cross-platform)
**Test Runners:** Vitest (JS/TS via `nx run-many -t test`), `cargo test --all` (Rust)
**Type Check:** `nx run-many -t type-check` (TS), `cargo check` / `cargo clippy` (Rust, pedantic)
**Lint:** `nx run-many -t lint` (TS), `cargo clippy` (Rust)

## Source of Truth

All development instructions are maintained in: `docs/development/agent-instructions.md`

## Testing Philosophy

### CRITICAL: No Mocking Internal Code

**If we own it or write it, we test it directly - we do NOT mock it in tests.**

Do NOT mock the database or the internal API. Use a real database instance and real HTTP calls.
Mocks are only allowed for external third-party services.

#### What We Do NOT Mock (Use Real Implementations):
- Database/Supabase - Use real database instances (test database, Docker containers)
- Internal APIs - Use real HTTP calls to our own endpoints
- tRPC routers - Test with real context and real database connections
- Internal services - Execution service, compiler, deployment service, etc.
- Temporal connections - Use real Temporal client (local or test cluster)
- Internal handlers - MCP handlers, GraphQL handlers, resource handlers
- Our own modules - Any code that lives in this repository

#### What We CAN Mock (External Third-Party Only):
- External AI providers - Anthropic, OpenAI, Gemini APIs
- External SaaS services - Stripe, SendGrid, Twilio, etc.
- Third-party APIs - Services we don't own or control
- External webhooks - Outbound calls to customer systems

#### Why This Matters:
1. **Mocks can lie** - Mocked implementations drift from real behavior over time
2. **False confidence** - Tests pass but production fails because mocks don't reflect reality
3. **Integration gaps** - Mocking hides integration bugs that only surface in production
4. **Maintenance burden** - Mock updates lag behind real implementation changes

**Remember:** Tables, code, configuration, and definitions that we own—including those affecting code inside third-party systems—should be tested with real implementations because we own them.

## Quick Start

When a user references REQ-XXX:
1. Check Braingrid for the requirement and tasks
2. Ensure tasks are well-defined (break down if needed)
3. Set REQ status to IN_PROGRESS
4. Update task status in real-time as you work
5. Mark REQ as REVIEW when complete and tests pass

See `docs/development/agent-instructions.md` for complete instructions.
See `docs/TESTING.md` for comprehensive testing guidelines.
