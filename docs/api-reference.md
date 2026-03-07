# Radium Workflow API Reference

## Table of Contents

- [Overview](#overview)
- [Authentication](#authentication)
- [Error Format](#error-format)
- [Rate Limiting](#rate-limiting)
- [Core Endpoints](#core-endpoints)
  - [Health](#health)
  - [Compile](#compile)
  - [Validate](#validate)
- [Components](#components)
  - [List Components](#list-components)
  - [Get Component](#get-component)
  - [Create Custom Component](#create-custom-component)
  - [List Custom Components](#list-custom-components)
  - [Delete Custom Component](#delete-custom-component)
- [Projects](#projects)
  - [Create Project](#create-project)
  - [List Projects](#list-projects)
  - [Get Project](#get-project)
  - [Update Project](#update-project)
  - [Delete Project](#delete-project)
  - [Deploy Project](#deploy-project)
  - [Project Status](#project-status)
  - [List Project Services](#list-project-services)
- [Services](#services)
  - [Create Service](#create-service)
  - [List Services](#list-services)
  - [Get Service](#get-service)
  - [Update Service](#update-service)
  - [Delete Service](#delete-service)
  - [Validate Service](#validate-service)
  - [Publish Service](#publish-service)
  - [Unpublish Service](#unpublish-service)
- [Service Catalog](#service-catalog)
  - [List Catalog](#list-catalog)
  - [Import Service](#import-service)
- [Deploy Pipeline](#deploy-pipeline)
  - [Deploy Service](#deploy-service)
  - [Undeploy Service](#undeploy-service)
  - [Service Deployment Status](#service-deployment-status)
- [Interfaces](#interfaces)
  - [Create Interface](#create-interface)
  - [List Interfaces](#list-interfaces)
  - [Get Interface](#get-interface)
  - [Update Interface](#update-interface)
  - [Delete Interface](#delete-interface)
  - [Publish Interface](#publish-interface)
  - [Unpublish Interface](#unpublish-interface)
- [State Variables (Service-Scoped)](#state-variables-service-scoped)
  - [Create Service Variable](#create-service-variable)
  - [List Service Variables](#list-service-variables)
  - [Get Service Variable](#get-service-variable)
  - [Update Service Variable](#update-service-variable)
  - [Delete Service Variable](#delete-service-variable)
- [State Variables (Project-Scoped)](#state-variables-project-scoped)
  - [Create Project Variable](#create-project-variable)
  - [List Project Variables](#list-project-variables)
  - [Get Project Variable](#get-project-variable)
  - [Update Project Variable](#update-project-variable)
  - [Delete Project Variable](#delete-project-variable)
- [Gateway](#gateway)
  - [Handle Gateway Request](#handle-gateway-request)

---

## Overview

The Radium Workflow API is served over HTTP. Core endpoints (`/health`, `/compile`, `/validate`) are always available. Versioned endpoints under `/v1` require a Supabase backend and Bearer token authentication.

Base URL: `http://localhost:3020` (default)

All request and response bodies use `application/json` unless otherwise noted. Service creation and update endpoints also accept `application/x-yaml` or `text/yaml` for YAML workflow definitions.

---

## Authentication

Most `/v1` endpoints require a Bearer token in the `Authorization` header. Tokens are API keys stored in the `api_keys` table and validated against Supabase on every request.

```
Authorization: Bearer rad_k1_abc123def456...
```

The gateway endpoint (`POST /v1/gateway/{interface_id}`) does **not** require Bearer token authentication -- it is the public-facing entry point for published interfaces.

---

## Error Format

All errors return a JSON envelope with a consistent structure:

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Workflow 'abc-123' not found",
    "details": []
  }
}
```

The gateway endpoint uses a simplified error envelope without `details`:

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Interface 'abc-123' is not published or not active"
  }
}
```

### Common Error Codes

| Status | Code | Description |
|--------|------|-------------|
| 400 | `BAD_REQUEST` | Invalid input or missing required fields |
| 401 | `UNAUTHORIZED` | Missing or invalid Bearer token |
| 404 | `NOT_FOUND` | Requested resource does not exist |
| 422 | `VALIDATION_FAILED` | Workflow definition failed validation |
| 429 | `RATE_LIMITED` | Too many requests; retry after the indicated interval |
| 500 | `INTERNAL_ERROR` | Unexpected server error |
| 503 | `SERVICE_UNAVAILABLE` | Required backend service (e.g., Temporal) is unavailable |

---

## Rate Limiting

Authenticated endpoints enforce a per-user rate limit. When the limit is exceeded, the API returns `429 Too Many Requests` with a message indicating when to retry:

```json
{
  "error": {
    "code": "RATE_LIMITED",
    "message": "Rate limit exceeded. Try again in 60 seconds.",
    "details": []
  }
}
```

---

## Core Endpoints

### Health

Check server health.

`GET /health`

**Auth:** None

**Response:** `200 OK`

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 3621
}
```

---

### Compile

Compile a workflow definition into executable TypeScript code.

`POST /compile`

**Auth:** None

**Request body:**

```json
{
  "workflow": {
    "id": "wf_order_pipeline",
    "name": "Order Pipeline",
    "nodes": [
      {
        "id": "trigger",
        "node_type": "trigger",
        "data": { "label": "Start" },
        "position": { "x": 0, "y": 0 }
      },
      {
        "id": "process",
        "node_type": "activity",
        "data": { "label": "Process Order", "activity_name": "processOrder" },
        "position": { "x": 200, "y": 0 }
      },
      {
        "id": "end",
        "node_type": "end",
        "data": { "label": "End" },
        "position": { "x": 400, "y": 0 }
      }
    ],
    "edges": [
      { "id": "e1", "source": "trigger", "target": "process" },
      { "id": "e2", "source": "process", "target": "end" }
    ],
    "variables": [],
    "settings": {}
  },
  "options": {
    "strictMode": false,
    "includeComments": true,
    "skipVerification": false
  }
}
```

**Response:** `200 OK`

```json
{
  "success": true,
  "code": {
    "workflow": "// generated TypeScript workflow code...",
    "activities": "// generated activity stubs..."
  },
  "metadata": {
    "nodeCount": 3,
    "edgeCount": 2,
    "compilationTimeMs": 12,
    "version": "0.1.0"
  }
}
```

On validation failure the response still returns `200` but with `success: false`:

```json
{
  "success": false,
  "errors": [
    {
      "code": "NO_START_NODE",
      "message": "Workflow must have exactly one start/trigger node",
      "severity": "error"
    }
  ],
  "warnings": [],
  "metadata": {
    "nodeCount": 2,
    "edgeCount": 1,
    "compilationTimeMs": 1,
    "version": "0.1.0"
  }
}
```

---

### Validate

Validate a workflow definition without generating code.

`POST /validate`

**Auth:** None

**Request body:** A `WorkflowDefinition` JSON object (same structure as the `workflow` field in the compile request).

**Response:** `200 OK`

```json
{
  "valid": true,
  "errors": [],
  "warnings": [],
  "suggestions": [
    "Consider adding retry policy to 2 activities"
  ]
}
```

---

## Components

### List Components

List all available workflow component types (built-in and custom).

`GET /v1/components`

**Auth:** None (built-in list is public; custom components require Bearer token)

**Response:** `200 OK`

```json
[
  {
    "name": "http_request",
    "category": "networking",
    "description": "Makes an HTTP request to an external service",
    "config_fields": [
      {
        "name": "url",
        "field_type": "string",
        "required": true,
        "description": "The URL to send the request to"
      },
      {
        "name": "method",
        "field_type": "string",
        "required": true,
        "description": "HTTP method (GET, POST, PUT, DELETE)"
      }
    ],
    "version": "1.0.0",
    "behavior_tier": "io",
    "deprecated": false,
    "canonical_name": null
  }
]
```

---

### Get Component

Get a single component type by name.

`GET /v1/components/{component_type}`

**Auth:** None

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `component_type` | string | Component name (e.g., `http_request`, `conditional`) |

**Response:** `200 OK`

Returns a single `ComponentType` object (same shape as items in the list response).

| Status | Description |
|--------|-------------|
| 200 | Component found |
| 404 | Component type not found |

---

### Create Custom Component

Register a user-defined custom component type.

`POST /v1/components`

**Auth:** Bearer token

**Request body:**

```json
{
  "name": "slack_notifier",
  "display_name": "Slack Notifier",
  "description": "Sends a message to a Slack channel",
  "category": "messaging",
  "version": "1.0.0",
  "behavior_tier": "io",
  "input_schema": {
    "type": "object",
    "properties": {
      "channel": { "type": "string" },
      "message": { "type": "string" }
    },
    "required": ["channel", "message"]
  },
  "output_schema": {
    "type": "object",
    "properties": {
      "ts": { "type": "string" }
    }
  },
  "config_fields": [
    {
      "name": "webhook_url",
      "field_type": "string",
      "required": true,
      "description": "Slack incoming webhook URL"
    }
  ]
}
```

Valid categories: `control_flow`, `activities`, `agent`, `orchestration`, `execution`, `data`, `security`, `storage`, `networking`, `messaging`, `flow_control`

Valid behavior tiers: `pure`, `stateful`, `io`, `n/a`

**Response:** `201 Created`

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "slack_notifier",
  "display_name": "Slack Notifier",
  "description": "Sends a message to a Slack channel",
  "component_type_id": "00000000-0000-0000-0000-000000000001",
  "version": "1.0.0",
  "created_by": "user-abc123",
  "visibility_id": "00000000-0000-0000-0000-000000000001",
  "input_schema": { "type": "object" },
  "output_schema": { "type": "object" },
  "config_schema": { "type": "object" },
  "is_active": true,
  "deprecated": false,
  "created_at": "2026-03-07T12:00:00Z",
  "updated_at": "2026-03-07T12:00:00Z"
}
```

| Status | Description |
|--------|-------------|
| 201 | Component created |
| 400 | Invalid category, behavior tier, version, or empty name |
| 401 | Missing or invalid Bearer token |
| 429 | Rate limit exceeded |
| 500 | Database operation failed |

---

### List Custom Components

List custom components created by the authenticated user.

`GET /v1/components/custom`

**Auth:** Bearer token

**Response:** `200 OK`

```json
{
  "components": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "slack_notifier",
      "display_name": "Slack Notifier",
      "description": "Sends a message to a Slack channel",
      "component_type_id": "00000000-0000-0000-0000-000000000001",
      "version": "1.0.0",
      "created_by": "user-abc123",
      "visibility_id": "00000000-0000-0000-0000-000000000001",
      "created_at": "2026-03-07T12:00:00Z",
      "updated_at": "2026-03-07T12:00:00Z"
    }
  ],
  "total": 1
}
```

---

### Delete Custom Component

Delete a custom component by name.

`DELETE /v1/components/custom/{name}`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `name` | string | Custom component name |

**Response:** `204 No Content`

| Status | Description |
|--------|-------------|
| 204 | Component deleted |
| 401 | Missing or invalid Bearer token |
| 500 | Database operation failed |

---

## Projects

### Create Project

Create a new project with an auto-provisioned Temporal task queue.

`POST /v1/projects`

**Auth:** Bearer token

**Request body:**

```json
{
  "name": "E-Commerce Platform",
  "description": "Order processing and inventory management workflows"
}
```

**Response:** `201 Created`

```json
{
  "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "name": "E-Commerce Platform",
  "description": "Order processing and inventory management workflows",
  "task_queue_name": "a1b2c3d4-e-commerce-platform-queue",
  "is_active": true,
  "created_at": "2026-03-07T12:00:00Z",
  "updated_at": "2026-03-07T12:00:00Z"
}
```

| Status | Description |
|--------|-------------|
| 201 | Project created |
| 400 | Project name is empty |
| 401 | Missing or invalid Bearer token |
| 429 | Rate limit exceeded |
| 500 | Database operation failed |

---

### List Projects

List all projects for the authenticated user.

`GET /v1/projects`

**Auth:** Bearer token

**Response:** `200 OK`

```json
{
  "projects": [
    {
      "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "name": "E-Commerce Platform",
      "description": "Order processing and inventory management workflows",
      "is_active": true,
      "created_at": "2026-03-07T12:00:00Z"
    }
  ],
  "total": 1
}
```

---

### Get Project

Get a single project by ID.

`GET /v1/projects/{id}`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Project ID |

**Response:** `200 OK`

```json
{
  "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "name": "E-Commerce Platform",
  "description": "Order processing and inventory management workflows",
  "task_queue_name": "a1b2c3d4-e-commerce-platform-queue",
  "is_active": true,
  "created_at": "2026-03-07T12:00:00Z",
  "updated_at": "2026-03-07T12:00:00Z"
}
```

| Status | Description |
|--------|-------------|
| 200 | Project found |
| 401 | Missing or invalid Bearer token |
| 404 | Project not found |

---

### Update Project

Update a project's name and description.

`PUT /v1/projects/{id}`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Project ID |

**Request body:**

```json
{
  "name": "E-Commerce Platform v2",
  "description": "Updated order processing workflows"
}
```

**Response:** `200 OK`

Returns the full `ProjectResponse` object.

| Status | Description |
|--------|-------------|
| 200 | Project updated |
| 401 | Missing or invalid Bearer token |
| 404 | Project not found |

---

### Delete Project

Delete a project.

`DELETE /v1/projects/{id}`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Project ID |

**Response:** `204 No Content`

| Status | Description |
|--------|-------------|
| 204 | Project deleted |
| 401 | Missing or invalid Bearer token |
| 404 | Project not found |

---

### Deploy Project

Deploy all services in a project. Runs the full deploy pipeline (validate, codegen, store, update status) for each service sequentially. Fail-fast: on first failure, remaining services are marked as skipped.

`POST /v1/projects/{id}/deploy`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Project ID |

**Response:** `200 OK`

```json
{
  "project_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "deployed": [
    {
      "service_id": "svc-001",
      "compiled_at": "2026-03-07T12:01:00Z"
    },
    {
      "service_id": "svc-002",
      "compiled_at": "2026-03-07T12:01:01Z"
    }
  ],
  "failed": null,
  "skipped": []
}
```

On partial failure:

```json
{
  "project_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "deployed": [
    {
      "service_id": "svc-001",
      "compiled_at": "2026-03-07T12:01:00Z"
    }
  ],
  "failed": {
    "service_id": "svc-002",
    "error": "Validation failed: no start node"
  },
  "skipped": [
    {
      "service_id": "svc-003",
      "reason": "Skipped due to earlier failure"
    }
  ]
}
```

| Status | Description |
|--------|-------------|
| 200 | Deploy completed (check `failed` field for partial failures) |
| 400 | Project has no services to deploy |
| 401 | Missing or invalid Bearer token |
| 404 | Project not found |

---

### Project Status

Get aggregated deployment status for a project.

`GET /v1/projects/{id}/status`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Project ID |

**Response:** `200 OK`

```json
{
  "project_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "project_name": "E-Commerce Platform",
  "total_services": 3,
  "deployed": 2,
  "draft": 1,
  "services": [
    { "id": "svc-001", "name": "Order Processor", "status_id": "00000000-0000-0000-0000-000000000003" },
    { "id": "svc-002", "name": "Inventory Check", "status_id": "00000000-0000-0000-0000-000000000003" },
    { "id": "svc-003", "name": "Notification Sender", "status_id": "00000000-0000-0000-0000-000000000001" }
  ]
}
```

---

### List Project Services

List all services belonging to a project.

`GET /v1/projects/{id}/services`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Project ID |

**Response:** `200 OK`

```json
{
  "services": [
    { "id": "svc-001", "name": "Order Processor", "status_id": "00000000-0000-0000-0000-000000000003" },
    { "id": "svc-002", "name": "Inventory Check", "status_id": "00000000-0000-0000-0000-000000000001" }
  ],
  "total": 2
}
```

---

## Services

### Create Service

Create a new workflow service from a JSON or YAML definition.

`POST /v1/services`

**Auth:** Bearer token

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | string (UUID) | No | Assign the service to a project |

**Content-Type:** `application/json` or `application/x-yaml` / `text/yaml`

**Request body (JSON):**

```json
{
  "name": "Order Processor",
  "description": "Processes incoming orders",
  "trigger": {
    "type": "signal",
    "config": { "signal_name": "new_order" }
  },
  "steps": [
    {
      "id": "validate",
      "type": "activity",
      "config": { "activity_name": "validateOrder" }
    },
    {
      "id": "fulfill",
      "type": "activity",
      "config": { "activity_name": "fulfillOrder" }
    }
  ]
}
```

**Response:** `201 Created`

```json
{
  "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "name": "Order Processor",
  "description": "Processes incoming orders",
  "status_id": "00000000-0000-0000-0000-000000000001",
  "version": "1.0.0",
  "definition": { },
  "created_at": "2026-03-07T12:00:00Z",
  "updated_at": "2026-03-07T12:00:00Z"
}
```

| Status | Description |
|--------|-------------|
| 201 | Service created |
| 400 | Invalid JSON or YAML body |
| 401 | Missing or invalid Bearer token |
| 422 | Workflow transformation/validation failed |
| 429 | Rate limit exceeded |

---

### List Services

List all workflow services for the authenticated user.

`GET /v1/services`

**Auth:** Bearer token

**Response:** `200 OK`

```json
{
  "workflows": [
    {
      "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
      "name": "Order Processor",
      "description": "Processes incoming orders",
      "status_id": "00000000-0000-0000-0000-000000000001",
      "version": "1.0.0",
      "created_at": "2026-03-07T12:00:00Z"
    }
  ],
  "total": 1
}
```

---

### Get Service

Get a single workflow service by ID.

`GET /v1/services/{id}`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Service ID |

**Response:** `200 OK`

```json
{
  "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "name": "Order Processor",
  "description": "Processes incoming orders",
  "status_id": "00000000-0000-0000-0000-000000000001",
  "version": "1.0.0",
  "definition": { },
  "created_at": "2026-03-07T12:00:00Z",
  "updated_at": "2026-03-07T12:00:00Z"
}
```

| Status | Description |
|--------|-------------|
| 200 | Service found |
| 401 | Missing or invalid Bearer token |
| 404 | Service not found |

---

### Update Service

Update a workflow service definition.

`PUT /v1/services/{id}`

**Auth:** Bearer token

**Content-Type:** `application/json` or `application/x-yaml` / `text/yaml`

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Service ID |

**Request body:** Same format as create (YAML or JSON workflow definition).

**Response:** `200 OK`

Returns the full `WorkflowResponse` object.

| Status | Description |
|--------|-------------|
| 200 | Service updated |
| 400 | Invalid JSON or YAML body |
| 401 | Missing or invalid Bearer token |
| 404 | Service not found |
| 422 | Workflow transformation/validation failed |

---

### Delete Service

Delete a workflow service.

`DELETE /v1/services/{id}`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Service ID |

**Response:** `204 No Content`

| Status | Description |
|--------|-------------|
| 204 | Service deleted |
| 401 | Missing or invalid Bearer token |
| 404 | Service not found |

---

### Validate Service

Validate a stored workflow service definition.

`POST /v1/services/{id}/validate`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Service ID |

**Response:** `200 OK`

```json
{
  "valid": true,
  "errors": [],
  "warnings": [],
  "suggestions": [
    "Consider adding retry policy to 2 activities"
  ]
}
```

On validation failure:

```json
{
  "valid": false,
  "errors": [
    {
      "code": "NO_START_NODE",
      "message": "Workflow must have exactly one start/trigger node",
      "severity": "error"
    },
    {
      "code": "ORPHAN_NODE",
      "message": "Node 'step3' is not connected to the graph",
      "node_id": "step3",
      "severity": "error"
    }
  ],
  "warnings": [],
  "suggestions": []
}
```

| Status | Description |
|--------|-------------|
| 200 | Validation completed (check `valid` field) |
| 401 | Missing or invalid Bearer token |
| 404 | Service not found |

---

### Publish Service

Set a service's visibility to public so it appears in the catalog.

`POST /v1/services/{id}/publish`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Service ID |

**Response:** `200 OK`

```json
{
  "status": "published",
  "message": "Service is now publicly visible in the catalog"
}
```

| Status | Description |
|--------|-------------|
| 200 | Service published |
| 401 | Missing or invalid Bearer token |
| 404 | Service not found |

---

### Unpublish Service

Set a service's visibility back to private.

`POST /v1/services/{id}/unpublish`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Service ID |

**Response:** `200 OK`

```json
{
  "status": "unpublished",
  "message": "Service is now private"
}
```

| Status | Description |
|--------|-------------|
| 200 | Service unpublished |
| 401 | Missing or invalid Bearer token |
| 404 | Service not found |

---

## Service Catalog

### List Catalog

List publicly available services from all users.

`GET /v1/services/catalog`

**Auth:** Bearer token

**Response:** `200 OK`

```json
{
  "services": [
    {
      "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
      "name": "Order Processor",
      "description": "Processes incoming orders",
      "status_id": "00000000-0000-0000-0000-000000000003",
      "version": "1.0.0",
      "created_at": "2026-03-07T12:00:00Z"
    }
  ],
  "total": 1
}
```

---

### Import Service

Import a service from the catalog into your project.

`POST /v1/services/catalog/{source_id}/import`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `source_id` | string (UUID) | ID of the catalog service to import |

**Request body:**

```json
{
  "project_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
}
```

**Response:** `201 Created`

Returns the newly created `WorkflowResponse` for the imported service.

| Status | Description |
|--------|-------------|
| 201 | Service imported |
| 400 | Missing project_id or source service is not public |
| 401 | Missing or invalid Bearer token |
| 404 | Source service not found |

---

## Deploy Pipeline

### Deploy Service

Validate, compile, and deploy a single workflow service.

`POST /v1/services/{id}/deploy`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Service ID |

**Response:** `200 OK`

```json
{
  "workflow_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "status": "deployed",
  "compiled_at": "2026-03-07T12:05:00Z",
  "message": "Workflow compiled and deployed successfully"
}
```

| Status | Description |
|--------|-------------|
| 200 | Deployment succeeded |
| 401 | Missing or invalid Bearer token |
| 404 | Service not found |
| 422 | Workflow validation failed (details in `details` array) |
| 429 | Rate limit exceeded |
| 500 | Codegen or database error |

---

### Undeploy Service

Revert a deployed service back to draft status.

`POST /v1/services/{id}/undeploy`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Service ID |

**Response:** `200 OK`

```json
{
  "workflow_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "status": "draft",
  "message": "Workflow undeployed and reverted to draft"
}
```

| Status | Description |
|--------|-------------|
| 200 | Service undeployed |
| 401 | Missing or invalid Bearer token |
| 404 | Service not found |

---

### Service Deployment Status

Check the deployment status of a workflow service.

`GET /v1/services/{id}/status`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Service ID |

**Response:** `200 OK`

```json
{
  "workflow_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "deployment_status": "deployed",
  "last_deployed_at": "2026-03-07T12:05:00Z"
}
```

Possible `deployment_status` values: `draft`, `compiled`, `deployed`

| Status | Description |
|--------|-------------|
| 200 | Status retrieved |
| 401 | Missing or invalid Bearer token |
| 404 | Service not found |

---

## Interfaces

Service interfaces define how external systems interact with a deployed workflow. Each interface has a type (`signal`, `query`, `update`, `mcp`, `graphql`) and can be published to create a public gateway route.

### Create Interface

Create a new interface on a service.

`POST /v1/services/{id}/interfaces`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Service ID |

**Request body:**

```json
{
  "name": "submit_order",
  "display_name": "Submit Order",
  "description": "Accepts a new order for processing",
  "interface_type": "signal",
  "callable_name": "submitOrder",
  "input_schema": {
    "type": "object",
    "properties": {
      "order_id": { "type": "string" },
      "items": { "type": "array" }
    },
    "required": ["order_id", "items"]
  },
  "output_schema": {
    "type": "object",
    "properties": {
      "accepted": { "type": "boolean" }
    }
  },
  "is_public": false
}
```

Valid `interface_type` values: `signal`, `query`, `update`, `mcp`, `graphql`

**Response:** `201 Created`

```json
{
  "id": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
  "workflow_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "name": "submit_order",
  "display_name": "Submit Order",
  "description": "Accepts a new order for processing",
  "interface_type": "signal",
  "callable_name": "submitOrder",
  "input_schema": { "type": "object" },
  "output_schema": { "type": "object" },
  "is_public": false,
  "created_at": "2026-03-07T12:00:00Z",
  "updated_at": "2026-03-07T12:00:00Z"
}
```

| Status | Description |
|--------|-------------|
| 201 | Interface created |
| 400 | Invalid interface_type or empty name |
| 401 | Missing or invalid Bearer token |
| 404 | Service not found |

---

### List Interfaces

List all interfaces for a service.

`GET /v1/services/{id}/interfaces`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Service ID |

**Response:** `200 OK`

```json
{
  "interfaces": [
    {
      "id": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
      "workflow_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
      "name": "submit_order",
      "interface_type": "signal",
      "is_public": false,
      "created_at": "2026-03-07T12:00:00Z",
      "updated_at": "2026-03-07T12:00:00Z"
    }
  ],
  "total": 1
}
```

---

### Get Interface

Get a single interface by ID.

`GET /v1/services/{id}/interfaces/{iid}`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Service ID |
| `iid` | string (UUID) | Interface ID |

**Response:** `200 OK`

Returns the full `InterfaceResponse` object.

| Status | Description |
|--------|-------------|
| 200 | Interface found |
| 401 | Missing or invalid Bearer token |
| 404 | Service or interface not found |

---

### Update Interface

Update an interface's properties.

`PUT /v1/services/{id}/interfaces/{iid}`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Service ID |
| `iid` | string (UUID) | Interface ID |

**Request body** (all fields optional):

```json
{
  "name": "submit_order_v2",
  "display_name": "Submit Order v2",
  "description": "Updated order submission interface",
  "interface_type": "signal",
  "input_schema": { "type": "object" },
  "is_public": true
}
```

**Response:** `200 OK`

Returns the full `InterfaceResponse` object.

| Status | Description |
|--------|-------------|
| 200 | Interface updated |
| 400 | Invalid interface_type or empty name |
| 401 | Missing or invalid Bearer token |
| 404 | Service or interface not found |

---

### Delete Interface

Delete an interface from a service.

`DELETE /v1/services/{id}/interfaces/{iid}`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Service ID |
| `iid` | string (UUID) | Interface ID |

**Response:** `204 No Content`

| Status | Description |
|--------|-------------|
| 204 | Interface deleted |
| 401 | Missing or invalid Bearer token |
| 404 | Service or interface not found |

---

### Publish Interface

Publish an interface to make it accessible via the public gateway. Generates a route path, creates Kong gateway resources (when available), starts a Temporal gateway workflow, and records the public interface.

`POST /v1/services/{id}/interfaces/{iid}/publish`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Service ID |
| `iid` | string (UUID) | Interface ID |

**Response:** `201 Created`

```json
{
  "id": "c3d4e5f6-a7b8-9012-cdef-123456789012",
  "service_interface_id": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
  "route_path": "/api/order-processor/submit-order",
  "http_method": "POST",
  "kong_route_id": "kong-route-abc123",
  "kong_service_id": "kong-svc-def456",
  "gateway_workflow_id": "gateway-b2c3d4e5",
  "is_active": true,
  "created_at": "2026-03-07T12:10:00Z",
  "updated_at": "2026-03-07T12:10:00Z"
}
```

The `kong_route_id`, `kong_service_id`, and `gateway_workflow_id` fields are omitted when Kong or Temporal are not configured.

| Status | Description |
|--------|-------------|
| 201 | Interface published |
| 401 | Missing or invalid Bearer token |
| 404 | Service or interface not found |
| 500 | Kong or Temporal operation failed |

---

### Unpublish Interface

Remove a published interface from the gateway. Deletes Kong resources (if present), terminates the gateway workflow, and removes the public interface record.

`POST /v1/services/{id}/interfaces/{iid}/unpublish`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Service ID |
| `iid` | string (UUID) | Interface ID |

**Response:** `204 No Content`

| Status | Description |
|--------|-------------|
| 204 | Interface unpublished |
| 401 | Missing or invalid Bearer token |
| 404 | Service not found |

---

## State Variables (Service-Scoped)

State variables provide persistent storage for workflow execution state. Service-scoped variables are tied to a specific workflow service.

### Create Service Variable

`POST /v1/services/{id}/variables`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Service ID |

**Request body:**

```json
{
  "name": "order_count",
  "type": "number",
  "storage_type": "database",
  "schema": { "minimum": 0 },
  "storage_config": { "ttl": 3600 }
}
```

Valid `type` values: `string`, `number`, `boolean`, `object`, `array`

Valid `storage_type` values: `database`, `cache`

**Response:** `201 Created`

```json
{
  "id": "d4e5f6a7-b8c9-0123-def0-123456789abc",
  "workflow_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "name": "order_count",
  "type": "number",
  "storage_type": "database",
  "schema": { "minimum": 0 },
  "storage_config": { "ttl": 3600 },
  "created_at": "2026-03-07T12:00:00Z",
  "updated_at": "2026-03-07T12:00:00Z"
}
```

| Status | Description |
|--------|-------------|
| 201 | Variable created |
| 400 | Invalid type, storage_type, or empty name |
| 401 | Missing or invalid Bearer token |
| 404 | Service not found |

---

### List Service Variables

`GET /v1/services/{id}/variables`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Service ID |

**Response:** `200 OK`

```json
{
  "variables": [
    {
      "id": "d4e5f6a7-b8c9-0123-def0-123456789abc",
      "workflow_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
      "name": "order_count",
      "type": "number",
      "storage_type": "database",
      "created_at": "2026-03-07T12:00:00Z",
      "updated_at": "2026-03-07T12:00:00Z"
    }
  ],
  "total": 1
}
```

---

### Get Service Variable

`GET /v1/services/{id}/variables/{var_id}`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Service ID |
| `var_id` | string (UUID) | Variable ID |

**Response:** `200 OK`

Returns the full `ServiceVariableResponse` object.

| Status | Description |
|--------|-------------|
| 200 | Variable found |
| 401 | Missing or invalid Bearer token |
| 404 | Service or variable not found |

---

### Update Service Variable

`PUT /v1/services/{id}/variables/{var_id}`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Service ID |
| `var_id` | string (UUID) | Variable ID |

**Request body** (all fields optional):

```json
{
  "name": "total_orders",
  "type": "number",
  "storage_type": "database",
  "schema": { "minimum": 0, "maximum": 1000000 }
}
```

**Response:** `200 OK`

Returns the full `ServiceVariableResponse` object.

| Status | Description |
|--------|-------------|
| 200 | Variable updated |
| 400 | Invalid type, storage_type, or empty name |
| 401 | Missing or invalid Bearer token |
| 404 | Service or variable not found |

---

### Delete Service Variable

`DELETE /v1/services/{id}/variables/{var_id}`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Service ID |
| `var_id` | string (UUID) | Variable ID |

**Response:** `204 No Content`

| Status | Description |
|--------|-------------|
| 204 | Variable deleted |
| 401 | Missing or invalid Bearer token |
| 404 | Service or variable not found |

---

## State Variables (Project-Scoped)

Project-scoped state variables are shared across all services within a project.

### Create Project Variable

`POST /v1/projects/{id}/variables`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Project ID |

**Request body:**

```json
{
  "name": "global_config",
  "type": "object",
  "storage_type": "database",
  "schema": {
    "type": "object",
    "properties": {
      "max_retries": { "type": "number" },
      "timeout_ms": { "type": "number" }
    }
  }
}
```

**Response:** `201 Created`

```json
{
  "id": "e5f6a7b8-c9d0-1234-ef01-23456789abcd",
  "project_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "name": "global_config",
  "type": "object",
  "storage_type": "database",
  "schema": { "type": "object" },
  "created_at": "2026-03-07T12:00:00Z",
  "updated_at": "2026-03-07T12:00:00Z"
}
```

| Status | Description |
|--------|-------------|
| 201 | Variable created |
| 400 | Invalid type, storage_type, or empty name |
| 401 | Missing or invalid Bearer token |
| 404 | Project not found |

---

### List Project Variables

`GET /v1/projects/{id}/variables`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Project ID |

**Response:** `200 OK`

```json
{
  "variables": [
    {
      "id": "e5f6a7b8-c9d0-1234-ef01-23456789abcd",
      "project_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "name": "global_config",
      "type": "object",
      "storage_type": "database",
      "created_at": "2026-03-07T12:00:00Z",
      "updated_at": "2026-03-07T12:00:00Z"
    }
  ],
  "total": 1
}
```

---

### Get Project Variable

`GET /v1/projects/{id}/variables/{var_id}`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Project ID |
| `var_id` | string (UUID) | Variable ID |

**Response:** `200 OK`

Returns the full `ProjectVariableResponse` object.

| Status | Description |
|--------|-------------|
| 200 | Variable found |
| 401 | Missing or invalid Bearer token |
| 404 | Project or variable not found |

---

### Update Project Variable

`PUT /v1/projects/{id}/variables/{var_id}`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Project ID |
| `var_id` | string (UUID) | Variable ID |

**Request body** (all fields optional):

```json
{
  "name": "global_settings",
  "type": "object",
  "storage_type": "database"
}
```

**Response:** `200 OK`

Returns the full `ProjectVariableResponse` object.

| Status | Description |
|--------|-------------|
| 200 | Variable updated |
| 400 | Invalid type, storage_type, or empty name |
| 401 | Missing or invalid Bearer token |
| 404 | Project or variable not found |

---

### Delete Project Variable

`DELETE /v1/projects/{id}/variables/{var_id}`

**Auth:** Bearer token

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string (UUID) | Project ID |
| `var_id` | string (UUID) | Variable ID |

**Response:** `204 No Content`

| Status | Description |
|--------|-------------|
| 204 | Variable deleted |
| 401 | Missing or invalid Bearer token |
| 404 | Project or variable not found |

---

## Gateway

### Handle Gateway Request

Public-facing endpoint for published interfaces. Kong routes external traffic to this endpoint. The handler verifies the interface is published and active, signals the corresponding Temporal gateway workflow, and returns `202 Accepted` immediately.

`POST /v1/gateway/{interface_id}`

**Auth:** None (public endpoint)

**Path parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `interface_id` | string (UUID) | Published interface ID |

**Headers:**

| Header | Required | Description |
|--------|----------|-------------|
| `X-Request-ID` | No | Request tracking ID. Auto-generated (UUID v4) if not provided. |

**Request body:**

Any valid JSON payload. An empty body defaults to `{}`.

```json
{
  "order_id": "ORD-2026-001",
  "items": [
    { "sku": "WIDGET-A", "quantity": 3 },
    { "sku": "GADGET-B", "quantity": 1 }
  ]
}
```

**Response:** `202 Accepted`

```json
{
  "status": "accepted",
  "request_id": "a7b8c9d0-e1f2-3456-7890-abcdef012345",
  "message": "Request queued for processing"
}
```

| Status | Description |
|--------|-------------|
| 202 | Request accepted and queued |
| 404 | Interface not published or not active |
| 500 | Invalid JSON body or failed to signal gateway workflow |
| 503 | Gateway workflow engine (Temporal) is not available |
