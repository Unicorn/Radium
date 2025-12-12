# Temporal Integration

Temporal is the workflow orchestration engine that executes compiled workflows.

## Overview

The workflow builder:
1. Accepts visual workflow definitions (nodes + edges)
2. Compiles them to TypeScript code
3. Generated code uses Temporal TypeScript SDK
4. Temporal workers execute the workflows

## Temporal Concepts

### Workflow
A durable function that orchestrates activities and handles state.

```typescript
export async function MyWorkflow(input: Input): Promise<Output> {
  // Workflow logic
}
```

### Activity
A unit of work that can fail and be retried.

```typescript
export async function processOrder(order: Order): Promise<Result> {
  // Activity logic (can make external calls)
}
```

### Worker
A process that executes workflows and activities.

```typescript
const worker = await Worker.create({
  workflowsPath: require.resolve('./workflows'),
  activities,
  taskQueue: 'my-task-queue',
});
```

## Generated Code Patterns

### Imports
```typescript
import {
  proxyActivities,
  startChild,
  executeChild,
  sleep,
  condition,
  defineSignal,
  setHandler,
  uuid4
} from '@temporalio/workflow';
```

### Activity Invocation
```typescript
const activities = proxyActivities<typeof import('./activities')>({
  startToCloseTimeout: '30s',
});

const result = await activities.processOrder(input);
```

### Retry Policy
```typescript
const result = await activities.riskyOperation(input, {
  retry: {
    maximumAttempts: 3,
    initialInterval: '1s',
    maximumInterval: '1m',
    backoffCoefficient: 2.0,
  },
});
```

### Child Workflows
```typescript
// Fire and forget
const handle = await startChild(ChildWorkflow, {
  workflowId: 'child-' + uuid4(),
  args: [input],
});

// Wait for result
const result = await executeChild(ChildWorkflow, {
  workflowId: 'child-' + uuid4(),
  args: [input],
});
```

### Signals
```typescript
const updateSignal = defineSignal<[string]>('update');

setHandler(updateSignal, (value) => {
  state = value;
});
```

## Determinism Requirements

Temporal workflows must be deterministic for replay. Our code generation ensures:

1. **No random values** - Use `uuid4()` from Temporal
2. **No current time** - Use `workflow.now()` for time
3. **No external calls** - All I/O through activities
4. **Idempotent logic** - Same input = same output

## Task Queues

Workflows and activities are assigned to task queues:

```typescript
// Worker listens on queue
const worker = await Worker.create({
  taskQueue: 'default',
  // ...
});

// Workflow runs on queue
await client.workflow.start(MyWorkflow, {
  taskQueue: 'default',
  // ...
});
```

## Error Handling

### Activity Errors
Activities can throw errors. Temporal automatically retries based on retry policy.

### Workflow Errors
Workflows can catch activity errors:

```typescript
try {
  await activities.riskyOperation(input);
} catch (error) {
  // Handle error or throw to fail workflow
}
```

## Temporal UI

The Temporal UI shows:
- Running workflows
- Workflow history
- Activity execution
- Errors and retries

Access at: `http://localhost:8233` (default)

## References

- [Temporal TypeScript SDK](https://typescript.temporal.io/)
- [Temporal Concepts](https://docs.temporal.io/concepts)
- [Workflow Determinism](https://docs.temporal.io/workflows#deterministic-constraints)
