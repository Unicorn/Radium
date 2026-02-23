# Component Schema Summary Statistics

Updated: 2026-02-23

---

## Overview

The Radium workflow component system includes 37 components across 11 categories, with a shared behavior system providing retry, rate limiting, circuit breaker, idempotency, and observability.

---

## Component Statistics

### Total Components: 37

| Category | Count | Components |
|----------|-------|------------|
| Control Flow | 5 | trigger, start, stop, conditional, loop |
| Core Activities | 4 | action, log, http_request, database_query |
| Agent | 1 | agent |
| Advanced | 4 | child_service, message, timer, parallel |
| Execution | 3 | shell_execute, npm_function, code_execute |
| Data | 3 | data_transform, schema_validate, encode_decode |
| Security | 3 | secret_read, oauth_token, jwt_create |
| Storage | 4 | cache, file_write, file_read, object_storage |
| Network | 4 | graphql_request, grpc_call, websocket, smtp_send |
| Messaging | 4 | webhook_send, queue_publish, queue_consume, event_emit |
| Flow Control | 2 | delay, batch |

### Behavior Tiers

| Tier | Count | Description |
|------|-------|-------------|
| Pure | 5 | No I/O, no side effects (data_transform, schema_validate, encode_decode, jwt_create, log) |
| Stateful | 3 | Timeout only (code_execute, timer, delay) |
| I/O | 24 | Full behaviors (retry, rate limit, circuit breaker, idempotency) |
| N/A | 5 | Control flow components (no activity execution) |

---

## Test Statistics

### Current Test Counts

| Test Suite | Tests Passed |
|------------|--------------|
| Library unit tests (incl. 23 new components) | 721 |
| Component verification | 85 |
| Integration tests | 31 |
| TypeScript verification | 20 |
| Doc tests | 2 |

**Total Tests**: 721+ passing

---

## Migration Records

### Component Records: 41 YAML files

| Record Type | Count |
|-------------|-------|
| Original 14 components | 14 |
| New components (Waves 1-4) | 23 |
| Renamed component records | 3 |
| Shared behaviors record | 1 |

---

## Shared Behavior System

### ComponentBehaviors struct provides:

| Behavior | Description | Default |
|----------|-------------|---------|
| Retry | Exponential backoff with jitter | 3 attempts, 1s initial |
| Rate Limit | Token bucket or sliding window | Varies by component |
| Circuit Breaker | Failure threshold protection | 5 failures, 30s reset |
| Idempotency | Duplicate execution prevention | Content-hash based |
| Observability | Structured logging and metrics | Info level |
| Payload Limits | Input/output size constraints | 1MB in, 5MB out |

### ComponentOutput envelope provides:

| Field | Type | Description |
|-------|------|-------------|
| success | bool | Whether the operation succeeded |
| data | Option<Value> | The operation result |
| error | Option<ComponentError> | Error details if failed |
| metadata | OutputMetadata | Timing, retries, request ID |

---

## Key Metrics

| Metric | Value |
|--------|-------|
| Total components | 37 |
| Total tests passing | 721+ |
| Behavior tiers | 3 (Pure, Stateful, I/O) |
| Backward-compatible renames | 3 |
| Component records | 41 YAML files |
| Shared behaviors module | 1 |

---

## Component Waves

| Wave | Components | Status |
|------|-----------|--------|
| Foundation | behaviors, type registry, 3 renames | Complete |
| Wave 1 | shell_execute, npm_function, code_execute, data_transform, schema_validate, secret_read, cache, file_write | Complete |
| Wave 2 | graphql_request, smtp_send, webhook_send, queue_publish, queue_consume, object_storage, file_read, encode_decode | Complete |
| Wave 3 | oauth_token, jwt_create, grpc_call, websocket, event_emit | Complete |
| Wave 4 | delay, batch | Complete |
