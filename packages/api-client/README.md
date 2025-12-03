# @radium/api-client

gRPC-Web client for communicating with the Radium backend.

## Overview

This package provides a type-safe client for all Radium gRPC endpoints, abstracting away the details of gRPC-Web communication.

## Usage

### Creating a Client

```typescript
import { createRadiumClient } from '@radium/api-client';

const client = createRadiumClient({
  baseUrl: 'http://localhost:50051',
  timeout: 30000, // optional
});
```

### Using Services

```typescript
// Agent operations
const agents = await client.agents.listAgents();
const agent = await client.agents.getAgent({ agentId: 'agent-1' });
await client.agents.createAgent({ agent: newAgent });
await client.agents.updateAgent({ agent: updatedAgent });
await client.agents.deleteAgent({ agentId: 'agent-1' });

// Workflow operations
const workflows = await client.workflows.listWorkflows();
await client.workflows.executeWorkflow({ workflowId: 'wf-1', useParallel: false });

// Task operations
const tasks = await client.tasks.listTasks();

// Orchestrator operations
const registered = await client.orchestrator.getRegisteredAgents();
await client.orchestrator.executeAgent({
  agentId: 'agent-1',
  input: 'Hello',
  modelType: 'gemini',
});
```

### Error Handling

All service methods return Promises that reject on error:

```typescript
try {
  const agents = await client.agents.listAgents();
} catch (error) {
  console.error('Failed to fetch agents:', error);
}
```

## Services

- `AgentService` - Agent CRUD operations
- `WorkflowService` - Workflow CRUD and execution
- `TaskService` - Task CRUD operations
- `OrchestratorService` - Agent orchestration operations

