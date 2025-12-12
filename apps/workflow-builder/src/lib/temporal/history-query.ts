/**
 * Temporal History Querying Service
 * 
 * Fetches and parses Temporal workflow execution history to extract
 * component-level execution details for UI display.
 */

import type { Client } from '@temporalio/client';
import type { WorkflowDefinition, WorkflowNode } from '@/types/workflow';

export interface ComponentExecution {
  nodeId: string;
  componentId?: string;
  componentName?: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'retrying';
  startedAt?: Date;
  completedAt?: Date;
  durationMs?: number;
  inputData?: any;
  outputData?: any;
  errorMessage?: string;
  errorType?: string;
  retryCount: number;
  isExpectedRetry: boolean;
  temporalActivityId?: string;
  temporalAttempt: number;
}

export interface ParsedExecutionHistory {
  componentExecutions: ComponentExecution[];
  workflowStatus: 'running' | 'completed' | 'failed' | 'cancelled';
  workflowStartedAt: Date;
  workflowCompletedAt?: Date;
  workflowDurationMs?: number;
  workflowError?: string;
}

/**
 * Get workflow execution history from Temporal
 */
export async function getWorkflowExecutionHistory(
  client: Client,
  workflowId: string,
  runId: string
): Promise<any> {
  const handle = client.workflow.getHandle(workflowId, runId);
  const history = await handle.fetchHistory();
  return history;
}

/**
 * Store full history JSON in database
 */
export async function storeFullHistory(
  executionId: string,
  history: any,
  supabase: any,
  systemUserId: string
): Promise<void> {
  // Store using system user context to bypass RLS
  const { error } = await supabase
    .from('workflow_executions')
    .update({
      temporal_history_json: history,
      history_synced_at: new Date().toISOString(),
      history_sync_status: 'synced',
    })
    .eq('id', executionId);

  if (error) {
    throw new Error(`Failed to store history: ${error.message}`);
  }
}

/**
 * Parse component executions from Temporal history
 */
export function parseComponentExecutions(
  history: any,
  workflowDefinition: WorkflowDefinition
): ComponentExecution[] {
  const componentExecutions: ComponentExecution[] = [];
  const nodeMap = new Map<string, WorkflowNode>();
  
  // Build node map for quick lookup
  (workflowDefinition.nodes || []).forEach(node => {
    nodeMap.set(node.id, node);
  });

  // Parse history events
  const events = history?.events || [];
  const activityExecutions = new Map<string, any>();

  for (const event of events) {
    // Activity execution started
    if (event.activityTaskScheduledEventAttributes) {
      const attrs = event.activityTaskScheduledEventAttributes;
      const activityId = attrs.activityId;
      const activityType = attrs.activityType?.name;
      
      activityExecutions.set(activityId, {
        activityId,
        activityType,
        scheduledTime: new Date(parseInt(attrs.scheduledTimeOfEvent?.seconds || '0') * 1000),
        input: attrs.input?.payloads?.[0] ? JSON.parse(Buffer.from(attrs.input.payloads[0].data, 'base64').toString()) : undefined,
        retryPolicy: attrs.retryPolicy,
      });
    }

    // Activity execution started
    if (event.activityTaskStartedEventAttributes) {
      const attrs = event.activityTaskStartedEventAttributes;
      const activityId = attrs.scheduledEventId;
      const execution = activityExecutions.get(activityId.toString());
      
      if (execution) {
        execution.startedTime = new Date(parseInt(attrs.startedTime?.seconds || '0') * 1000);
        execution.attempt = attrs.attempt || 1;
      }
    }

    // Activity execution completed
    if (event.activityTaskCompletedEventAttributes) {
      const attrs = event.activityTaskCompletedEventAttributes;
      const activityId = attrs.scheduledEventId;
      const execution = activityExecutions.get(activityId.toString());
      
      if (execution) {
        execution.completedTime = new Date(parseInt(attrs.completedTime?.seconds || '0') * 1000);
        execution.result = attrs.result?.payloads?.[0] ? JSON.parse(Buffer.from(attrs.result.payloads[0].data, 'base64').toString()) : undefined;
        execution.status = 'completed';
        
        // Map to workflow node
        const node = findNodeByActivityType(execution.activityType, nodeMap);
        if (node) {
          componentExecutions.push(createComponentExecution(node, execution, 'completed'));
        }
      }
    }

    // Activity execution failed
    if (event.activityTaskFailedEventAttributes) {
      const attrs = event.activityTaskFailedEventAttributes;
      const activityId = attrs.scheduledEventId;
      const execution = activityExecutions.get(activityId.toString());
      
      if (execution) {
        execution.failedTime = new Date(parseInt(attrs.failedTime?.seconds || '0') * 1000);
        execution.failure = attrs.failure;
        execution.status = 'failed';
        
        // Map to workflow node
        const node = findNodeByActivityType(execution.activityType, nodeMap);
        if (node) {
          componentExecutions.push(createComponentExecution(node, execution, 'failed'));
        }
      }
    }

    // Activity execution timed out
    if (event.activityTaskTimedOutEventAttributes) {
      const attrs = event.activityTaskTimedOutEventAttributes;
      const activityId = attrs.scheduledEventId;
      const execution = activityExecutions.get(activityId.toString());
      
      if (execution) {
        execution.timedOutTime = new Date(parseInt(attrs.timeoutTime?.seconds || '0') * 1000);
        execution.status = 'failed';
        execution.errorType = 'timeout';
        
        // Map to workflow node
        const node = findNodeByActivityType(execution.activityType, nodeMap);
        if (node) {
          componentExecutions.push(createComponentExecution(node, execution, 'failed'));
        }
      }
    }
  }

  return componentExecutions;
}

/**
 * Identify expected vs unexpected retries based on node retry policy
 */
export function identifyExpectedRetries(
  componentExecutions: ComponentExecution[],
  workflowDefinition: WorkflowDefinition
): ComponentExecution[] {
  const nodeMap = new Map<string, WorkflowNode>();
  (workflowDefinition.nodes || []).forEach(node => {
    nodeMap.set(node.id, node);
  });

  return componentExecutions.map(exec => {
    const node = nodeMap.get(exec.nodeId);
    if (!node) return exec;

    const retryPolicy = node.data.retryPolicy;
    if (!retryPolicy || retryPolicy.strategy === 'none') {
      // No retries expected
      exec.isExpectedRetry = false;
      return exec;
    }

    // Check if retry count is within expected range
    if (retryPolicy.strategy === 'keep-trying') {
      // All retries are expected
      exec.isExpectedRetry = exec.retryCount > 0;
    } else if (retryPolicy.strategy === 'fail-after-x') {
      // Retries up to maxAttempts are expected
      const maxAttempts = retryPolicy.maxAttempts || 3;
      exec.isExpectedRetry = exec.retryCount > 0 && exec.retryCount < maxAttempts;
    } else if (retryPolicy.strategy === 'exponential-backoff') {
      // Retries up to maxAttempts are expected
      const maxAttempts = retryPolicy.maxAttempts || 5;
      exec.isExpectedRetry = exec.retryCount > 0 && exec.retryCount < maxAttempts;
    }

    return exec;
  });
}

/**
 * Find node by activity type name
 */
function findNodeByActivityType(
  activityType: string,
  nodeMap: Map<string, WorkflowNode>
): WorkflowNode | undefined {
  // Try to match by component name
  for (const [nodeId, node] of nodeMap.entries()) {
    if (node.data.componentName === activityType) {
      return node;
    }
    // Also try camelCase conversion
    if (toCamelCase(nodeId) === activityType) {
      return node;
    }
  }
  return undefined;
}

/**
 * Create component execution from activity execution data
 */
function createComponentExecution(
  node: WorkflowNode,
  execution: any,
  status: 'completed' | 'failed'
): ComponentExecution {
  const startedAt = execution.startedTime || execution.scheduledTime;
  const completedAt = execution.completedTime || execution.failedTime || execution.timedOutTime;
  const durationMs = startedAt && completedAt 
    ? completedAt.getTime() - startedAt.getTime()
    : undefined;

  return {
    nodeId: node.id,
    componentId: node.data.componentId,
    componentName: node.data.componentName || node.data.label,
    status,
    startedAt,
    completedAt,
    durationMs,
    inputData: execution.input,
    outputData: execution.result,
    errorMessage: execution.failure?.message,
    errorType: execution.errorType || (execution.failure ? 'activity_failure' : undefined),
    retryCount: (execution.attempt || 1) - 1,
    isExpectedRetry: false, // Will be set by identifyExpectedRetries
    temporalActivityId: execution.activityId,
    temporalAttempt: execution.attempt || 1,
  };
}

/**
 * Helper: Convert string to camelCase
 */
function toCamelCase(str: string): string {
  return str
    .replace(/[^a-zA-Z0-9]+(.)/g, (_, chr) => chr.toUpperCase())
    .replace(/^[A-Z]/, chr => chr.toLowerCase());
}

