# Radium Shared Packages

This directory contains shared packages used across Radium applications (web and desktop).

## Package Structure

### `shared-types`
TypeScript type definitions matching the Rust proto definitions. Provides type-safe interfaces for all Radium entities.

**Usage:**
```typescript
import { Agent, Workflow, Task } from '@radium/shared-types';
```

### `api-client`
gRPC-Web client for communicating with the Radium backend. Provides service wrappers for all gRPC endpoints.

**Usage:**
```typescript
import { createRadiumClient } from '@radium/api-client';

const client = createRadiumClient({ baseUrl: 'http://localhost:50051' });
const agents = await client.agents.listAgents();
```

### `state`
Zustand stores for managing application state. Provides reactive state management for agents, workflows, tasks, and orchestrator operations.

**Usage:**
```typescript
import { useAgentStore } from '@radium/state';

const { agents, fetchAgents } = useAgentStore();
await fetchAgents(client.agents);
```

### `ui`
React component library with reusable UI components for building Radium applications.

**Usage:**
```typescript
import { AgentTable, Dashboard, WorkflowEditor } from '@radium/ui';
```

## Dependency Graph

```
shared-types
    ↓
api-client
    ↓
state ──→ ui
    ↓
apps (desktop, web)
```

## Building Packages

```bash
# Build all packages
nx run-many -t build --projects=tag:scope:shared

# Type check all packages
nx run-many -t type-check --projects=tag:type:lib
```

## Adding a New Package

1. Create a new directory under `packages/`
2. Add `package.json` with proper dependencies
3. Add `tsconfig.json` extending `../../tsconfig.base.json`
4. Add `project.json` for Nx configuration
5. Update `tsconfig.base.json` paths if needed
6. Add package to dependency graph in consuming packages

