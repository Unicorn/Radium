/**
 * Temporal Interceptors
 *
 * Interceptors for workflow and activity telemetry collection.
 * Part of the Workflow Observability Platform.
 *
 * Two interceptors work together for full traceability:
 * 1. workflowContextInterceptors - Runs in workflow code, propagates database IDs
 * 2. ActivityTelemetryInterceptor - Runs in worker, captures metrics with context
 */

// Activity interceptor (worker-side)
export {
  ActivityTelemetryInterceptor,
  createActivityTelemetryInterceptor,
  type TelemetryInterceptorOptions,
} from './activity-telemetry-interceptor';

// Workflow interceptor (workflow-side) - propagates context to activities
export {
  workflowContextInterceptors,
  createWorkflowInput,
  hasWorkflowContext,
  CONTEXT_HEADERS,
  type WorkflowExecutionContext,
} from './workflow-context-interceptor';
