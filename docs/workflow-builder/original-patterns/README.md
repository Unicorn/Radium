# Original TypeScript Patterns

This directory contains snapshots of the original TypeScript workflow compiler patterns that were implemented before the Rust migration. These snapshots serve as the reference implementation for Phase 6 component migration.

## Purpose

- Provide reference for how components were generated before Rust
- Enable comparison between original TypeScript output and new Rust-generated output
- Document the expected behavior and patterns for each component type
- Support Phase 9 Component Builder Agent training

## Component Types

### Core Flow Components
| Component | File | Description |
|-----------|------|-------------|
| Start/Trigger | `trigger.ts.snapshot` | Workflow entry point |
| Stop/End | `end.ts.snapshot` | Workflow termination point |
| Condition | `condition.ts.snapshot` | Conditional branching |
| Phase | `phase.ts.snapshot` | Sequential/parallel execution phases |

### Activity Components
| Component | File | Description |
|-----------|------|-------------|
| Activity | `activity.ts.snapshot` | Generic activity invocation |
| Agent | `agent.ts.snapshot` | AI agent activity |
| Retry | `retry.ts.snapshot` | Retry wrapper for activities |

### Advanced Workflow Components
| Component | File | Description |
|-----------|------|-------------|
| Child Workflow | `child-workflow.ts.snapshot` | Child workflow invocation |
| Signal | `signal.ts.snapshot` | Signal handlers |
| State Variable | `state-variable.ts.snapshot` | State management |

### Kong API Gateway Components
| Component | File | Description |
|-----------|------|-------------|
| Kong Logging | `kong-logging.ts.snapshot` | Request/response logging |
| Kong Cache | `kong-cache.ts.snapshot` | Response caching |
| Kong CORS | `kong-cors.ts.snapshot` | CORS policy |

### Integration Components
| Component | File | Description |
|-----------|------|-------------|
| GraphQL Gateway | `graphql-gateway.ts.snapshot` | GraphQL endpoint |
| MCP Server | `mcp-server.ts.snapshot` | Model Context Protocol server |
| API Endpoint | `api-endpoint.ts.snapshot` | REST API endpoint registration |

## How to Use These Snapshots

1. **During Phase 6**: Compare generated Rust output against these snapshots to ensure behavioral parity
2. **For Phase 9**: Use as training data for the Component Builder Agent
3. **For debugging**: Reference when TypeScript output doesn't match expected patterns

## Note on Snapshot Format

Each snapshot file contains:
1. The TypeScript code that would be generated
2. Comments explaining the component's behavior
3. Example inputs and expected outputs where applicable
