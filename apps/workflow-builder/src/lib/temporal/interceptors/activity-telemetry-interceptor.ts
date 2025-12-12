/**
 * Activity Telemetry Interceptor
 *
 * Captures performance metrics for activity executions.
 * Part of the Workflow Observability Platform for monitoring
 * activity performance, error rates, and resource utilization.
 *
 * This interceptor is transparent to activity code - no changes needed
 * to existing activities. It captures:
 * - Execution duration
 * - Success/failure rates
 * - Retry tracking
 * - Activity type distribution
 *
 * Context Propagation:
 * The workflow-context-interceptor injects headers with database IDs:
 * - x-project-id: Database UUID of the project
 * - x-workflow-id: Database UUID of the workflow definition
 * - x-execution-id: Database UUID of the workflow_executions record
 * - x-node-id: Node ID in the workflow graph
 *
 * This enables full traceability from component metrics back to specific
 * workflow executions for billing and analytics.
 */

import type {
  ActivityInboundCallsInterceptor,
  ActivityExecuteInput,
  Next,
} from '@temporalio/worker';
import { Context } from '@temporalio/activity';
import { recordComponentMetric } from '../../observability/metrics-collector';
import type { MetricStatus } from '../../observability/types';
import { CONTEXT_HEADERS } from './workflow-context-interceptor';

/**
 * Configuration options for the telemetry interceptor
 */
export interface TelemetryInterceptorOptions {
  /** Project ID for metric attribution */
  projectId: string;
  /** Workflow ID (if known at worker creation time) */
  workflowId?: string;
  /** Enable detailed input/output logging (performance impact) */
  capturePayloads?: boolean;
  /** Sample rate for detailed metrics (0-1, default 1.0) */
  sampleRate?: number;
}

/**
 * Activity telemetry interceptor
 *
 * Wraps activity execution to capture performance metrics
 */
export class ActivityTelemetryInterceptor implements ActivityInboundCallsInterceptor {
  private options: Required<TelemetryInterceptorOptions>;

  constructor(options: TelemetryInterceptorOptions) {
    this.options = {
      projectId: options.projectId,
      workflowId: options.workflowId || 'unknown',
      capturePayloads: options.capturePayloads ?? false,
      sampleRate: options.sampleRate ?? 1.0,
    };
  }

  /**
   * Intercept activity execution
   */
  async execute(
    input: ActivityExecuteInput,
    next: Next<ActivityInboundCallsInterceptor, 'execute'>
  ): Promise<unknown> {
    const context = Context.current();
    const activityInfo = context.info;
    const startTime = Date.now();

    // Determine if we should sample this execution
    const shouldSample = Math.random() < this.options.sampleRate;

    // Extract workflow context from headers (set by workflow-context-interceptor)
    // Headers are the authoritative source for database UUIDs
    const headers = input.headers;
    const headerProjectId = this.getHeader(headers, CONTEXT_HEADERS.PROJECT_ID);
    const headerWorkflowId = this.getHeader(headers, CONTEXT_HEADERS.WORKFLOW_ID);
    const headerExecutionId = this.getHeader(headers, CONTEXT_HEADERS.EXECUTION_ID);
    const headerNodeId = this.getHeader(headers, CONTEXT_HEADERS.NODE_ID);

    // Use header values first, then fall back to options/activity context
    const projectId = headerProjectId || this.options.projectId;
    const workflowId = headerWorkflowId || this.options.workflowId;
    const executionId = headerExecutionId;
    const nodeId =
      headerNodeId ||
      ((input.args?.[0] as Record<string, unknown> | undefined)?.nodeId as string | undefined);

    let status: MetricStatus = 'completed';
    let errorType: string | undefined;

    try {
      // Execute the actual activity
      const result = await next(input);

      const endTime = Date.now();
      const durationMs = endTime - startTime;

      // Record success metric
      if (shouldSample) {
        await this.recordMetric({
          projectId,
          workflowId,
          executionId,
          activityType: activityInfo.activityType,
          nodeId,
          durationMs,
          status: 'completed',
          attemptNumber: activityInfo.attempt,
          startTime,
          endTime,
          inputArgs: input.args,
          result,
        });
      }

      return result;
    } catch (error: unknown) {
      const endTime = Date.now();
      const durationMs = endTime - startTime;

      // Determine error type for categorization
      const err = error as Error & { name?: string; message?: string };
      if (err.name === 'TimeoutError' || err.message?.includes('timeout')) {
        status = 'timeout';
        errorType = 'timeout';
      } else if (err.name === 'CancelledFailure') {
        status = 'cancelled';
        errorType = 'cancelled';
      } else {
        status = 'failed';
        errorType = err.name || 'unknown';
      }

      // Record failure metric
      if (shouldSample) {
        await this.recordMetric({
          projectId,
          workflowId,
          executionId,
          activityType: activityInfo.activityType,
          nodeId,
          durationMs,
          status,
          attemptNumber: activityInfo.attempt,
          startTime,
          endTime,
          inputArgs: input.args,
          errorType,
          errorMessage: err.message,
        });
      }

      // Re-throw to preserve normal error handling
      throw error;
    }
  }

  /**
   * Extract header value from headers
   * Headers are Record<string, Payload> where Payload has { data: Uint8Array }
   * Values are JSON-encoded strings set by workflow-context-interceptor
   */
  private getHeader(
    headers: Record<string, unknown> | undefined,
    key: string
  ): string | undefined {
    if (!headers) return undefined;

    const payload = headers[key] as { data?: Uint8Array } | undefined;
    if (!payload || !payload.data) return undefined;

    try {
      // Decode Uint8Array to string, then parse JSON
      const decoded = new TextDecoder().decode(payload.data);
      // JSON.parse to remove the quotes from JSON-encoded string
      return JSON.parse(decoded) as string;
    } catch {
      // If decoding fails, try direct string extraction
      if (typeof payload === 'string') return payload;
      return undefined;
    }
  }

  /**
   * Record the metric to the collector
   */
  private async recordMetric(params: {
    projectId: string;
    workflowId: string;
    executionId?: string;
    activityType: string;
    nodeId?: string;
    durationMs: number;
    status: MetricStatus;
    attemptNumber: number;
    startTime: number;
    endTime: number;
    inputArgs?: unknown[];
    result?: unknown;
    errorType?: string;
    errorMessage?: string;
  }): Promise<void> {
    try {
      const componentType = this.determineComponentType(params.activityType);

      const metadata: Record<string, unknown> = {};

      // Capture payload sizes if enabled
      if (this.options.capturePayloads) {
        try {
          metadata.inputSizeBytes = JSON.stringify(params.inputArgs || []).length;
          if (params.result !== undefined) {
            metadata.outputSizeBytes = JSON.stringify(params.result).length;
          }
        } catch {
          // Ignore serialization errors
        }
      }

      // Add error message to metadata if present
      if (params.errorMessage) {
        metadata.errorMessage = params.errorMessage.substring(0, 500);
      }

      await recordComponentMetric({
        projectId: params.projectId,
        workflowId: params.workflowId,
        executionId: params.executionId,
        componentType,
        componentName: params.activityType,
        nodeId: params.nodeId,
        durationMs: params.durationMs,
        status: params.status,
        isRetry: params.attemptNumber > 1,
        attemptNumber: params.attemptNumber,
        startedAt: new Date(params.startTime),
        completedAt: new Date(params.endTime),
        errorType: params.errorType,
        metadata,
      });
    } catch (err) {
      // Log but don't throw - metrics are non-critical
      console.error('[Telemetry] Failed to record metric:', err);
    }
  }

  /**
   * Determine component type from activity name
   * Maps activity names to logical component types for better analytics
   */
  private determineComponentType(activityType: string): string {
    const lower = activityType.toLowerCase();

    // Agent/AI activities
    if (
      lower.includes('agent') ||
      lower.includes('llm') ||
      lower.includes('ai') ||
      lower.includes('claude') ||
      lower.includes('openai') ||
      lower.includes('gpt')
    ) {
      return 'agent';
    }

    // State activities
    if (
      lower.startsWith('get') ||
      lower.startsWith('set') ||
      lower.includes('state') ||
      lower.includes('variable')
    ) {
      return 'state';
    }

    // Notification activities
    if (
      lower.includes('notify') ||
      lower.includes('slack') ||
      lower.includes('email') ||
      lower.includes('sms') ||
      lower.includes('webhook')
    ) {
      return 'notification';
    }

    // HTTP/API activities
    if (
      lower.includes('http') ||
      lower.includes('request') ||
      lower.includes('api') ||
      lower.includes('fetch')
    ) {
      return 'http';
    }

    // Database activities
    if (
      lower.includes('query') ||
      lower.includes('database') ||
      lower.includes('sql') ||
      lower.includes('db')
    ) {
      return 'database';
    }

    // Transform activities
    if (
      lower.includes('transform') ||
      lower.includes('map') ||
      lower.includes('filter') ||
      lower.includes('reduce') ||
      lower.includes('parse')
    ) {
      return 'transform';
    }

    // Default
    return 'activity';
  }
}

/**
 * Factory function to create activity telemetry interceptors
 *
 * Usage in worker-manager.ts:
 * ```typescript
 * interceptors: {
 *   activity: [() => createActivityTelemetryInterceptor({
 *     projectId: project.id,
 *     sampleRate: 1.0,
 *   })],
 * }
 * ```
 */
export function createActivityTelemetryInterceptor(
  options: TelemetryInterceptorOptions
): ActivityInboundCallsInterceptor {
  return new ActivityTelemetryInterceptor(options);
}
