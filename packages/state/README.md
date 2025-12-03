# @radium/state

Zustand stores for managing Radium application state.

## Overview

This package provides reactive state management stores for agents, workflows, tasks, and orchestrator operations using Zustand.

## Stores

### Agent Store

```typescript
import { useAgentStore } from '@radium/state';
import { createRadiumClient } from '@radium/api-client';

const client = createRadiumClient({ baseUrl: 'http://localhost:50051' });
const { agents, loading, error, fetchAgents } = useAgentStore();

// Fetch all agents
await fetchAgents(client.agents);

// Create agent
await createAgent(client.agents, newAgent);

// Update agent
await updateAgent(client.agents, updatedAgent);

// Delete agent
await deleteAgent(client.agents, 'agent-1');
```

### Workflow Store

```typescript
import { useWorkflowStore } from '@radium/state';

const { workflows, fetchWorkflows, executeWorkflow } = useWorkflowStore();

await fetchWorkflows(client.workflows);
await executeWorkflow(client.workflows, 'workflow-1', false);
```

### Task Store

```typescript
import { useTaskStore } from '@radium/state';

const { tasks, fetchTasks } = useTaskStore();
await fetchTasks(client.tasks);
```

### Orchestrator Store

```typescript
import { useOrchestratorStore } from '@radium/state';

const { registeredAgents, executeAgent } = useOrchestratorStore();

await executeAgent(client.orchestrator, {
  agentId: 'agent-1',
  input: 'Hello',
  modelType: 'gemini',
});
```

## Store State

All stores provide:
- `loading: boolean` - Loading state
- `error: string | null` - Error message if any
- Data arrays (e.g., `agents`, `workflows`, `tasks`)
- Selected item (e.g., `selectedAgent`, `selectedWorkflow`)

## React Usage

```typescript
import { useAgentStore } from '@radium/state';

function AgentList() {
  const { agents, loading, fetchAgents } = useAgentStore();
  
  useEffect(() => {
    fetchAgents(client.agents);
  }, []);
  
  if (loading) return <div>Loading...</div>;
  
  return (
    <ul>
      {agents.map(agent => (
        <li key={agent.id}>{agent.name}</li>
      ))}
    </ul>
  );
}
```

