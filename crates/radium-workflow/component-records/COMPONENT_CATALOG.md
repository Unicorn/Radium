# Radium Workflow Component Catalog

## Overview

This catalog documents all 37 workflow components in the Radium schema system. Each component has:
- Rust schema with full type safety and validation
- Shared behavior system (retry, rate limiting, circuit breaker, idempotency)
- 3-tier behavior classification (Pure, Stateful, I/O)
- TypeScript code generation support
- Migration record in YAML format

## Component Categories

### Control Flow Components (5)

| Component | File | Purpose | Temporal Type | Behavior Tier |
|-----------|------|---------|---------------|---------------|
| **trigger** | `trigger.rs` | Workflow entry point | Workflow start | N/A |
| **start** | `start.rs` | Explicit workflow start marker | Workflow node | N/A |
| **stop** | `stop.rs` | Workflow termination | Workflow end | N/A |
| **conditional** | `conditional.rs` | Branching logic (if/else) | Workflow logic | N/A |
| **loop** | `loop_component.rs` | Iteration (forEach, while, batch) | Workflow logic | N/A |

### Core Activity Components (4)

| Component | File | Purpose | Temporal Type | Behavior Tier |
|-----------|------|---------|---------------|---------------|
| **action** | `action.rs` | Generic activity invocation | Activity | I/O |
| **log** | `log.rs` | Logging with Kong integration | Activity | Pure |
| **http_request** | `http_request.rs` | External HTTP API calls | Activity | I/O |
| **database_query** | `database_query.rs` | Supabase/PostgreSQL queries | Activity | I/O |

### Agent Components (1)

| Component | File | Purpose | Temporal Type | Behavior Tier |
|-----------|------|---------|---------------|---------------|
| **agent** | `agent.rs` | LLM/AI model invocation | Activity | I/O |

### Advanced Components (4)

| Component | File | Purpose | Temporal Type | Behavior Tier |
|-----------|------|---------|---------------|---------------|
| **child_service** | `child_service.rs` | Nested workflow execution | Child Workflow | I/O |
| **message** | `message.rs` | Inter-workflow communication | Signal | I/O |
| **timer** | `timer.rs` | Temporal delays | Timer | Stateful |
| **parallel** | `parallel.rs` | Concurrent branch execution | Workflow logic | N/A |

### Execution Components (3)

| Component | File | Purpose | Temporal Type | Behavior Tier |
|-----------|------|---------|---------------|---------------|
| **shell_execute** | `shell_execute.rs` | Shell command execution | Activity | I/O |
| **npm_function** | `npm_function.rs` | NPM package function calls | Activity | I/O |
| **code_execute** | `code_execute.rs` | Sandboxed code execution | Activity | Stateful |

### Data Components (3)

| Component | File | Purpose | Temporal Type | Behavior Tier |
|-----------|------|---------|---------------|---------------|
| **data_transform** | `data_transform.rs` | Expression-based data transformation | Activity | Pure |
| **schema_validate** | `schema_validate.rs` | JSON Schema validation | Activity | Pure |
| **encode_decode** | `encode_decode.rs` | Format conversion (base64, URL, hex, JSON, CSV) | Activity | Pure |

### Security Components (3)

| Component | File | Purpose | Temporal Type | Behavior Tier |
|-----------|------|---------|---------------|---------------|
| **secret_read** | `secret_read.rs` | Secure secret retrieval | Activity | I/O |
| **oauth_token** | `oauth_token.rs` | OAuth2 token acquisition | Activity | I/O |
| **jwt_create** | `jwt_create.rs` | JWT creation and signing | Activity | Pure |

### Storage Components (3)

| Component | File | Purpose | Temporal Type | Behavior Tier |
|-----------|------|---------|---------------|---------------|
| **cache** | `cache_component.rs` | Cache get/set/delete | Activity | I/O |
| **file_write** | `file_write.rs` | File writing | Activity | I/O |
| **file_read** | `file_read.rs` | File reading | Activity | I/O |
| **object_storage** | `object_storage.rs` | S3/R2/GCS object storage | Activity | I/O |

### Network Components (4)

| Component | File | Purpose | Temporal Type | Behavior Tier |
|-----------|------|---------|---------------|---------------|
| **graphql_request** | `graphql_request.rs` | GraphQL queries/mutations | Activity | I/O |
| **grpc_call** | `grpc_call.rs` | Unary gRPC calls | Activity | I/O |
| **websocket** | `websocket.rs` | WebSocket send/receive | Activity | I/O |
| **smtp_send** | `smtp_send.rs` | SMTP email sending | Activity | I/O |

### Messaging Components (4)

| Component | File | Purpose | Temporal Type | Behavior Tier |
|-----------|------|---------|---------------|---------------|
| **webhook_send** | `webhook_send.rs` | HTTP webhook callbacks with HMAC signing | Activity | I/O |
| **queue_publish** | `queue_publish.rs` | Message queue publishing | Activity | I/O |
| **queue_consume** | `queue_consume.rs` | Message queue consumption | Activity | I/O |
| **event_emit** | `event_emit.rs` | Event bus emission | Activity | I/O |

### Flow Control Components (3)

| Component | File | Purpose | Temporal Type | Behavior Tier |
|-----------|------|---------|---------------|---------------|
| **delay** | `delay.rs` | Workflow pause/sleep (Temporal timer) | Activity | Stateful |
| **batch** | `batch.rs` | Batch processing with concurrency | Activity | I/O |

### Shared Behaviors

| Module | File | Purpose |
|--------|------|---------|
| **behaviors** | `behaviors.rs` | Retry, rate limiting, circuit breaker, idempotency, observability, payload limits |

---

## Behavior Tiers

### Pure (no I/O, no side effects)
- `data_transform`, `schema_validate`, `encode_decode`, `jwt_create`, `log`
- No retry, rate limiting, or circuit breaker needed

### Stateful (timeout only)
- `code_execute`, `timer`, `delay`
- Has timeout but no external I/O requiring retry

### I/O (full behaviors)
- All network, storage, messaging, and execution components
- Full retry, rate limiting, circuit breaker, idempotency, and observability

---

## Backward Compatibility

Three components were renamed with backward-compatible aliases:

| Old Name | New Name | Alias Support |
|----------|----------|---------------|
| `activity` | `action` | `#[serde(alias = "activity")]` |
| `child_workflow` | `child_service` | `#[serde(alias = "child_workflow")]` |
| `signal` | `message` | `#[serde(alias = "signal")]` |

---

## Serialization Standards

All components follow these serialization rules:

1. **JSON Field Names**: camelCase (e.g., `triggerType`, `activityName`)
2. **Enum Values**: lowercase or kebab-case depending on context
3. **Optional Fields**: Omitted when `None` (skip_serializing_if)
4. **Default Values**: Provided via `#[serde(default)]`

---

## File Structure

```
crates/radium-workflow/
├── src/schema/components/
│   ├── mod.rs              # Module exports
│   ├── behaviors.rs        # Shared behavior system
│   ├── trigger.rs          # Control flow
│   ├── start.rs
│   ├── stop.rs
│   ├── conditional.rs
│   ├── loop_component.rs
│   ├── action.rs           # Core activities (renamed from activity)
│   ├── activity.rs         # Legacy re-export
│   ├── log.rs
│   ├── http_request.rs
│   ├── database_query.rs
│   ├── agent.rs            # AI/LLM
│   ├── child_service.rs    # Advanced (renamed from child_workflow)
│   ├── child_workflow.rs   # Legacy re-export
│   ├── message.rs          # Advanced (renamed from signal)
│   ├── signal.rs           # Legacy re-export
│   ├── timer.rs
│   ├── parallel.rs
│   ├── shell_execute.rs    # Execution
│   ├── npm_function.rs
│   ├── code_execute.rs
│   ├── data_transform.rs   # Data
│   ├── schema_validate.rs
│   ├── encode_decode.rs
│   ├── secret_read.rs      # Security
│   ├── oauth_token.rs
│   ├── jwt_create.rs
│   ├── cache_component.rs  # Storage
│   ├── file_write.rs
│   ├── file_read.rs
│   ├── object_storage.rs
│   ├── graphql_request.rs  # Network
│   ├── grpc_call.rs
│   ├── websocket.rs
│   ├── smtp_send.rs
│   ├── webhook_send.rs     # Messaging
│   ├── queue_publish.rs
│   ├── queue_consume.rs
│   ├── event_emit.rs
│   ├── delay.rs            # Flow control
│   └── batch.rs
├── component-records/
│   └── *.yaml              # 37+ YAML records
└── tests/
    ├── component_verification.rs
    └── typescript_verification.rs
```
