# ExecutionService

The ExecutionService provides a high-level API for managing workflow execution tracking in the workflow-builder application. It handles all CRUD operations for workflow executions and provides statistics and analytics.

## Features

- Create and track workflow executions
- Update execution status (running, completed, failed, cancelled, timed_out)
- Retrieve execution details and history
- Calculate execution statistics
- List executions by workflow or user
- Soft delete executions

## Architecture

The ExecutionService uses **Supabase** (PostgreSQL) for data persistence. The `workflow_executions` table stores:

- Execution metadata (ID, workflow ID, Temporal workflow/run IDs)
- Status tracking (running, completed, failed, etc.)
- Timing information (started_at, completed_at, duration_ms)
- Input/output data (JSON fields)
- Error messages
- User tracking (created_by)
- Activity count (activities_executed)

## Usage

### Basic Usage

```typescript
import { createExecutionService } from '@/lib/execution';
import { createClient } from '@/lib/supabase/server';

// Create service instance
const supabase = createClient();
const executionService = createExecutionService(supabase);

// Create a new execution
const execution = await executionService.createExecution({
  workflowId: 'workflow-123',
  temporalWorkflowId: 'wf-abc-123',
  temporalRunId: 'run-xyz-456',
  userId: 'user-1',
  input: { param1: 'value1' },
});

// Update execution status
await executionService.markCompleted(
  'wf-abc-123',
  { result: 'success' },
  5000 // duration in ms
);

// Get execution details
const details = await executionService.getExecution('wf-abc-123');

// List executions for a workflow
const executions = await executionService.listExecutions('workflow-123', {
  limit: 20,
  status: 'completed',
});

// Get statistics
const stats = await executionService.getStats('workflow-123');
console.log(`Success rate: ${stats.successRate}%`);
console.log(`Average duration: ${stats.avgDuration}ms`);
```

### In tRPC Procedures

```typescript
import { createExecutionService } from '@/lib/execution';
import { protectedProcedure } from '../trpc';

export const workflowRouter = createTRPCRouter({
  execute: protectedProcedure
    .input(z.object({ workflowId: z.string() }))
    .mutation(async ({ ctx, input }) => {
      const executionService = createExecutionService(ctx.supabase);

      // Create execution record
      const execution = await executionService.createExecution({
        workflowId: input.workflowId,
        temporalWorkflowId: `wf-${Date.now()}`,
        temporalRunId: `run-${Date.now()}`,
        userId: ctx.user.id,
      });

      // Start Temporal workflow...
      // Monitor execution...

      return execution;
    }),
});
```

## API Reference

### createExecution(data: CreateExecutionInput)

Creates a new workflow execution record.

**Parameters:**
- `workflowId`: The workflow being executed
- `temporalWorkflowId`: Unique Temporal workflow ID
- `temporalRunId`: Temporal run ID
- `userId`: User who initiated the execution
- `input?`: Optional input data (JSON)

**Returns:** `Promise<WorkflowExecution>`

### updateExecution(temporalWorkflowId, data: UpdateExecutionInput)

Updates an execution by Temporal workflow ID.

**Parameters:**
- `status?`: New status (running, completed, failed, etc.)
- `output?`: Output data (JSON)
- `error?`: Error message
- `completedAt?`: Completion timestamp
- `durationMs?`: Duration in milliseconds
- `activitiesExecuted?`: Number of activities executed

**Returns:** `Promise<WorkflowExecution>`

### getExecution(temporalWorkflowId)

Retrieves an execution with related workflow information.

**Returns:** `Promise<ExecutionWithWorkflow | null>`

### listExecutions(workflowId, options?)

Lists executions for a workflow with optional filtering.

**Options:**
- `limit?`: Maximum results (default: 50)
- `offset?`: Pagination offset (default: 0)
- `status?`: Filter by status
- `orderBy?`: Sort field (default: 'started_at')
- `orderDirection?`: Sort direction (default: 'desc')

**Returns:** `Promise<WorkflowExecution[]>`

### getStats(workflowId)

Calculates execution statistics for a workflow.

**Returns:** `Promise<ExecutionStats>` with:
- `total`: Total number of executions
- `running/completed/failed/cancelled/timedOut`: Count by status
- `avgDuration`: Average execution duration (ms)
- `minDuration/maxDuration`: Min/max durations
- `successRate`: Success percentage

### Convenience Methods

- `markCompleted(temporalWorkflowId, output?, durationMs?)` - Mark as completed
- `markFailed(temporalWorkflowId, error, durationMs?)` - Mark as failed
- `markCancelled(temporalWorkflowId, durationMs?)` - Mark as cancelled
- `markTimedOut(temporalWorkflowId, durationMs?)` - Mark as timed out
- `deleteExecution(executionId)` - Delete an execution
- `listExecutionsByUser(userId, options?)` - List user's executions

## Database Schema

The `workflow_executions` table:

```sql
CREATE TABLE workflow_executions (
  id TEXT PRIMARY KEY,
  workflow_id TEXT NOT NULL REFERENCES workflows(id),
  temporal_workflow_id TEXT UNIQUE NOT NULL,
  temporal_run_id TEXT NOT NULL,
  status TEXT NOT NULL, -- running, completed, failed, cancelled, timed_out
  started_at TIMESTAMP WITH TIME ZONE NOT NULL,
  completed_at TIMESTAMP WITH TIME ZONE,
  duration_ms INTEGER,
  input JSONB,
  output JSONB,
  error_message TEXT,
  activities_executed INTEGER,
  created_by TEXT REFERENCES users(id), -- Added by migration
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_workflow_executions_created_by ON workflow_executions(created_by);
CREATE INDEX idx_workflow_executions_user_status ON workflow_executions(created_by, status);
```

## Migration

To add the `created_by` field to existing deployments, run:

```bash
# Apply the migration
supabase db push --file supabase/migrations/20250119_add_created_by_to_workflow_executions.sql
```

Or in production:

```sql
-- Run the migration SQL manually
ALTER TABLE workflow_executions ADD COLUMN created_by TEXT;
ALTER TABLE workflow_executions ADD CONSTRAINT workflow_executions_created_by_fkey
  FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE;
CREATE INDEX idx_workflow_executions_created_by ON workflow_executions(created_by);
CREATE INDEX idx_workflow_executions_user_status ON workflow_executions(created_by, status);
```

## Testing

The ExecutionService has comprehensive test coverage (>80%):

```bash
npm test -- src/lib/execution/__tests__/execution-service.test.ts
```

Tests cover:
- CRUD operations
- Status updates
- Error handling
- Statistics calculation
- Filtering and pagination
- User-based queries

## Performance Considerations

- **Indexes**: The table has indexes on `created_by`, `(created_by, status)`, and `temporal_workflow_id`
- **Pagination**: Use `limit` and `offset` for large result sets
- **Caching**: Consider caching frequently accessed executions
- **Cleanup**: Implement periodic cleanup of old executions to prevent table bloat

## Error Handling

All methods throw descriptive errors:

```typescript
try {
  await executionService.updateExecution('wf-123', { status: 'completed' });
} catch (error) {
  console.error('Failed to update execution:', error.message);
  // Error message includes context: "Failed to update execution: [reason]"
}
```

## Integration with Temporal

The ExecutionService is designed to work seamlessly with Temporal:

1. Create execution record when starting a Temporal workflow
2. Update status as workflow progresses
3. Store final result or error when workflow completes
4. Track execution metrics for monitoring

```typescript
// Start workflow
const handle = await client.workflow.start(workflowName, { ... });

// Create execution record
const execution = await executionService.createExecution({
  workflowId,
  temporalWorkflowId: handle.workflowId,
  temporalRunId: handle.firstExecutionRunId,
  userId,
});

// Monitor in background
handle.result()
  .then(result => executionService.markCompleted(handle.workflowId, result))
  .catch(error => executionService.markFailed(handle.workflowId, error.message));
```

## Future Enhancements

Potential improvements:

- [ ] Real-time execution updates via WebSockets
- [ ] Execution event streaming
- [ ] Advanced analytics (percentile calculations, trend analysis)
- [ ] Execution replay functionality
- [ ] Cost tracking per execution
- [ ] SLA monitoring and alerting
