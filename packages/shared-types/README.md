# @radium/shared-types

TypeScript type definitions for Radium, matching the Rust proto definitions.

## Overview

This package provides type-safe interfaces for all Radium entities and RPC messages, ensuring consistency between the frontend and backend.

## Exports

### Agent Types
- `Agent`
- `CreateAgentRequest`, `CreateAgentResponse`
- `GetAgentRequest`, `GetAgentResponse`
- `ListAgentsRequest`, `ListAgentsResponse`
- `UpdateAgentRequest`, `UpdateAgentResponse`
- `DeleteAgentRequest`, `DeleteAgentResponse`

### Workflow Types
- `Workflow`, `WorkflowStep`
- `CreateWorkflowRequest`, `CreateWorkflowResponse`
- `GetWorkflowRequest`, `GetWorkflowResponse`
- `ListWorkflowsRequest`, `ListWorkflowsResponse`
- `UpdateWorkflowRequest`, `UpdateWorkflowResponse`
- `DeleteWorkflowRequest`, `DeleteWorkflowResponse`
- `ExecuteWorkflowRequest`, `ExecuteWorkflowResponse`
- `WorkflowExecution` and related types

### Task Types
- `Task`
- `CreateTaskRequest`, `CreateTaskResponse`
- `GetTaskRequest`, `GetTaskResponse`
- `ListTasksRequest`, `ListTasksResponse`
- `UpdateTaskRequest`, `UpdateTaskResponse`
- `DeleteTaskRequest`, `DeleteTaskResponse`

### Orchestrator Types
- `RegisteredAgent`
- `ExecuteAgentRequest`, `ExecuteAgentResponse`
- `StartAgentRequest`, `StartAgentResponse`
- `StopAgentRequest`, `StopAgentResponse`
- `GetRegisteredAgentsRequest`, `GetRegisteredAgentsResponse`
- `RegisterAgentRequest`, `RegisterAgentResponse`

## Usage

```typescript
import type { Agent, Workflow, Task } from '@radium/shared-types';

const agent: Agent = {
  id: 'agent-1',
  name: 'My Agent',
  description: 'A test agent',
  configJson: '{}',
  state: 'idle',
  createdAt: new Date().toISOString(),
  updatedAt: new Date().toISOString(),
};
```

