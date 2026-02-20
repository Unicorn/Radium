/**
 * Workflow Context Interceptor
 *
 * Propagates workflow execution context (database IDs) to all child activities.
 * This enables full traceability from component metrics back to the specific
 * workflow execution for billing and analytics.
 *
 * Context Propagation Flow:
 * 1. Workflow starts with context in _context field of input
 * 2. This interceptor captures the context at workflow start
 * 3. Every activity call gets headers injected with the context
 * 4. Activity telemetry interceptor reads headers and records metrics
 *
 * Headers propagated:
 * - x-project-id: Database UUID of the project
 * - x-workflow-id: Database UUID of the workflow definition
 * - x-execution-id: Database UUID of the workflow_executions record
 * - x-node-id: Node ID in the workflow graph (when available)
 */

import type {
  WorkflowInterceptorsFactory,
  WorkflowInboundCallsInterceptor,
  WorkflowOutboundCallsInterceptor,
  WorkflowExecuteInput,
  ActivityInput,
  LocalActivityInput,
  StartChildWorkflowExecutionInput,
  ContinueAsNewInput,
} from '@temporalio/workflow';
import type { Headers } from '@temporalio/common';

/**
 * Context keys for workflow-to-activity propagation
 */
export const CONTEXT_HEADERS = {
  PROJECT_ID: 'x-project-id',
  WORKFLOW_ID: 'x-workflow-id',
  EXECUTION_ID: 'x-execution-id',
  NODE_ID: 'x-node-id',
} as const;

/**
 * Workflow execution context for billing/analytics
 */
export interface WorkflowExecutionContext {
  projectId: string;
  workflowId: string;
  executionId?: string;
}

/**
 * Extract context from workflow input
 * Workflows should pass context as the first argument or as a _context field
 */
function extractContextFromInput(args: unknown[]): WorkflowExecutionContext | null {
  if (!args || args.length === 0) return null;

  const firstArg = args[0];
  if (!firstArg || typeof firstArg !== 'object') return null;

  const input = firstArg as Record<string, unknown>;

  // Check for explicit _context field
  if (input._context && typeof input._context === 'object') {
    const ctx = input._context as Record<string, unknown>;
    if (ctx.projectId && ctx.workflowId) {
      return {
        projectId: String(ctx.projectId),
        workflowId: String(ctx.workflowId),
        executionId: ctx.executionId ? String(ctx.executionId) : undefined,
      };
    }
  }

  // Check for top-level context fields
  if (input.projectId && input.workflowId) {
    return {
      projectId: String(input.projectId),
      workflowId: String(input.workflowId),
      executionId: input.executionId ? String(input.executionId) : undefined,
    };
  }

  return null;
}

/**
 * Extract nodeId from activity arguments
 * Activities may receive nodeId as part of their input
 */
function extractNodeIdFromArgs(args: unknown[]): string | null {
  if (!args || args.length === 0) return null;

  const firstArg = args[0];
  if (!firstArg || typeof firstArg !== 'object') return null;

  const input = firstArg as Record<string, unknown>;

  if (typeof input.nodeId === 'string') {
    return input.nodeId;
  }

  if (typeof input.node_id === 'string') {
    return input.node_id;
  }

  return null;
}

/**
 * Create a string-encoded payload for headers
 * Temporal headers use Payload type, we store as JSON-encoded strings
 */
function encodeHeaderValue(value: string): { data: Uint8Array } {
  return { data: new TextEncoder().encode(JSON.stringify(value)) };
}

/**
 * Merge context into existing headers
 * Returns new headers object with context values added
 */
function mergeContextHeaders(
  existingHeaders: Headers,
  context: WorkflowExecutionContext,
  args?: unknown[]
): Headers {
  const result: Headers = { ...existingHeaders };

  result[CONTEXT_HEADERS.PROJECT_ID] = encodeHeaderValue(context.projectId);
  result[CONTEXT_HEADERS.WORKFLOW_ID] = encodeHeaderValue(context.workflowId);

  if (context.executionId) {
    result[CONTEXT_HEADERS.EXECUTION_ID] = encodeHeaderValue(context.executionId);
  }

  if (args) {
    const nodeId = extractNodeIdFromArgs(args);
    if (nodeId) {
      result[CONTEXT_HEADERS.NODE_ID] = encodeHeaderValue(nodeId);
    }
  }

  return result;
}

/**
 * Factory function to create workflow context interceptors
 *
 * Usage in workflow registration:
 * ```typescript
 * import { workflowContextInterceptors } from './interceptors';
 *
 * // In bundled workflow code
 * export const interceptors = workflowContextInterceptors;
 * ```
 */
export const workflowContextInterceptors: WorkflowInterceptorsFactory = () => {
  // Shared context reference between inbound and outbound interceptors
  let capturedContext: WorkflowExecutionContext | null = null;

  const inbound: WorkflowInboundCallsInterceptor = {
    async execute(
      input: WorkflowExecuteInput,
      next: (input: WorkflowExecuteInput) => Promise<unknown>
    ): Promise<unknown> {
      // Extract context from workflow arguments
      capturedContext = extractContextFromInput(input.args);

      if (capturedContext) {
        console.log(
          `[WorkflowContext] Captured context: project=${capturedContext.projectId}, ` +
            `workflow=${capturedContext.workflowId}, execution=${capturedContext.executionId || 'N/A'}`
        );
      }

      return next(input);
    },
  };

  const outbound: WorkflowOutboundCallsInterceptor = {
    scheduleActivity(input: ActivityInput, next) {
      if (capturedContext) {
        const newHeaders = mergeContextHeaders(input.headers, capturedContext, input.args);
        return next({ ...input, headers: newHeaders });
      }
      return next(input);
    },

    scheduleLocalActivity(input: LocalActivityInput, next) {
      if (capturedContext) {
        const newHeaders = mergeContextHeaders(input.headers, capturedContext, input.args);
        return next({ ...input, headers: newHeaders });
      }
      return next(input);
    },

    startChildWorkflowExecution(input: StartChildWorkflowExecutionInput, next) {
      if (capturedContext) {
        // Propagate parent context to child workflows for lineage tracing
        const newHeaders = mergeContextHeaders(input.headers, capturedContext);
        return next({ ...input, headers: newHeaders });
      }
      return next(input);
    },

    continueAsNew(input: ContinueAsNewInput, next) {
      if (capturedContext) {
        const newHeaders = mergeContextHeaders(input.headers, capturedContext);
        return next({ ...input, headers: newHeaders });
      }
      return next(input);
    },
  };

  return {
    inbound: [inbound],
    outbound: [outbound],
  };
};

/**
 * Helper to create workflow input with context
 * Use this when starting workflows to ensure context is propagated
 */
export function createWorkflowInput<T extends Record<string, unknown>>(
  context: WorkflowExecutionContext,
  input: T
): T & { _context: WorkflowExecutionContext } {
  return {
    ...input,
    _context: context,
  };
}

/**
 * Type guard to check if input has context
 */
export function hasWorkflowContext(
  input: unknown
): input is { _context: WorkflowExecutionContext } {
  if (!input || typeof input !== 'object') return false;
  const obj = input as Record<string, unknown>;
  return (
    obj._context !== undefined &&
    typeof obj._context === 'object' &&
    obj._context !== null &&
    typeof (obj._context as Record<string, unknown>).projectId === 'string' &&
    typeof (obj._context as Record<string, unknown>).workflowId === 'string'
  );
}
