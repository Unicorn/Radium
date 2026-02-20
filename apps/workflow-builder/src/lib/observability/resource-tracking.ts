/**
 * Resource Tracking Utilities
 *
 * Helper functions for tracking resource usage (interfaces, variables, agents, connectors)
 * during workflow execution. These are designed to be used by activities and service
 * code to capture resource events without blocking the main execution flow.
 */

import { recordResourceEvent } from './metrics-collector';
import type {
  ResourceType,
  ResourceSubtype,
  ResourceOperation,
  ResourceEventStatus,
} from './types';

/**
 * Context for resource tracking within a workflow execution
 */
export interface ResourceTrackingContext {
  projectId: string;
  workflowId?: string;
  executionId?: string;
  componentMetricId?: string;
}

/**
 * Track an interface invocation (service call between workflows)
 */
export async function trackInterfaceCall(
  ctx: ResourceTrackingContext,
  options: {
    interfaceId?: string;
    interfaceName: string;
    interfaceType: 'service_interface' | 'public_interface';
    direction?: 'inbound' | 'outbound';
    method?: string;
    path?: string;
  },
  operation: () => Promise<{ result: unknown; requestSize?: number; responseSize?: number }>
): Promise<unknown> {
  const startTime = new Date();

  try {
    const { result, requestSize, responseSize } = await operation();
    const endTime = new Date();
    const durationMs = endTime.getTime() - startTime.getTime();

    await recordResourceEvent({
      projectId: ctx.projectId,
      workflowId: ctx.workflowId,
      executionId: ctx.executionId,
      componentMetricId: ctx.componentMetricId,
      resourceType: 'interface',
      resourceSubtype: options.interfaceType,
      resourceId: options.interfaceId,
      resourceName: options.interfaceName,
      operation: 'invoke',
      direction: options.direction,
      durationMs,
      requestSizeBytes: requestSize,
      responseSizeBytes: responseSize,
      status: 'success',
      startedAt: startTime,
      completedAt: endTime,
      metadata: {
        method: options.method,
        path: options.path,
      },
    }).catch((err) => {
      console.warn('[Observability] Failed to record interface event:', err);
    });

    return result;
  } catch (error) {
    const endTime = new Date();
    const durationMs = endTime.getTime() - startTime.getTime();

    await recordResourceEvent({
      projectId: ctx.projectId,
      workflowId: ctx.workflowId,
      executionId: ctx.executionId,
      componentMetricId: ctx.componentMetricId,
      resourceType: 'interface',
      resourceSubtype: options.interfaceType,
      resourceId: options.interfaceId,
      resourceName: options.interfaceName,
      operation: 'invoke',
      direction: options.direction,
      durationMs,
      status: 'failure',
      errorType: error instanceof Error ? error.name : 'Error',
      startedAt: startTime,
      completedAt: endTime,
      metadata: {
        method: options.method,
        path: options.path,
      },
    }).catch((err) => {
      console.warn('[Observability] Failed to record failed interface event:', err);
    });

    throw error;
  }
}

/**
 * Track a variable read/write operation
 */
export async function trackVariableOperation(
  ctx: ResourceTrackingContext,
  options: {
    variableId?: string;
    variableName: string;
    variableType: 'state_variable' | 'workflow_variable';
    operation: 'read' | 'write';
  },
  operation: () => Promise<unknown>
): Promise<unknown> {
  const startTime = new Date();

  try {
    const result = await operation();
    const endTime = new Date();
    const durationMs = endTime.getTime() - startTime.getTime();

    await recordResourceEvent({
      projectId: ctx.projectId,
      workflowId: ctx.workflowId,
      executionId: ctx.executionId,
      componentMetricId: ctx.componentMetricId,
      resourceType: 'variable',
      resourceSubtype: options.variableType,
      resourceId: options.variableId,
      resourceName: options.variableName,
      operation: options.operation,
      durationMs,
      status: 'success',
      startedAt: startTime,
      completedAt: endTime,
    }).catch((err) => {
      console.warn('[Observability] Failed to record variable event:', err);
    });

    return result;
  } catch (error) {
    const endTime = new Date();
    const durationMs = endTime.getTime() - startTime.getTime();

    await recordResourceEvent({
      projectId: ctx.projectId,
      workflowId: ctx.workflowId,
      executionId: ctx.executionId,
      componentMetricId: ctx.componentMetricId,
      resourceType: 'variable',
      resourceSubtype: options.variableType,
      resourceId: options.variableId,
      resourceName: options.variableName,
      operation: options.operation,
      durationMs,
      status: 'failure',
      errorType: error instanceof Error ? error.name : 'Error',
      startedAt: startTime,
      completedAt: endTime,
    }).catch((err) => {
      console.warn('[Observability] Failed to record failed variable event:', err);
    });

    throw error;
  }
}

/**
 * Track an agent/LLM call
 */
export async function trackAgentCall(
  ctx: ResourceTrackingContext,
  options: {
    agentId?: string;
    agentName: string;
    modelName: string;
  },
  operation: () => Promise<{
    result: unknown;
    promptTokens?: number;
    completionTokens?: number;
    totalTokens?: number;
  }>
): Promise<unknown> {
  const startTime = new Date();

  try {
    const { result, promptTokens, completionTokens, totalTokens } = await operation();
    const endTime = new Date();
    const durationMs = endTime.getTime() - startTime.getTime();

    await recordResourceEvent({
      projectId: ctx.projectId,
      workflowId: ctx.workflowId,
      executionId: ctx.executionId,
      componentMetricId: ctx.componentMetricId,
      resourceType: 'agent',
      resourceSubtype: 'ai_agent',
      resourceId: options.agentId,
      resourceName: options.agentName,
      operation: 'call',
      durationMs,
      status: 'success',
      modelName: options.modelName,
      promptTokens,
      completionTokens,
      totalTokens,
      startedAt: startTime,
      completedAt: endTime,
    }).catch((err) => {
      console.warn('[Observability] Failed to record agent event:', err);
    });

    return result;
  } catch (error) {
    const endTime = new Date();
    const durationMs = endTime.getTime() - startTime.getTime();

    await recordResourceEvent({
      projectId: ctx.projectId,
      workflowId: ctx.workflowId,
      executionId: ctx.executionId,
      componentMetricId: ctx.componentMetricId,
      resourceType: 'agent',
      resourceSubtype: 'ai_agent',
      resourceId: options.agentId,
      resourceName: options.agentName,
      operation: 'call',
      durationMs,
      status: 'failure',
      modelName: options.modelName,
      errorType: error instanceof Error ? error.name : 'Error',
      startedAt: startTime,
      completedAt: endTime,
    }).catch((err) => {
      console.warn('[Observability] Failed to record failed agent event:', err);
    });

    throw error;
  }
}

/**
 * Track a connector operation (project connector, database connector)
 */
export async function trackConnectorOperation(
  ctx: ResourceTrackingContext,
  options: {
    connectorId?: string;
    connectorName: string;
    connectorType: 'project_connector' | 'database_connector';
    targetProjectId?: string;
    targetService?: string;
    operation: 'connect' | 'invoke' | 'call';
  },
  operation: () => Promise<{ result: unknown; requestSize?: number; responseSize?: number }>
): Promise<unknown> {
  const startTime = new Date();

  try {
    const { result, requestSize, responseSize } = await operation();
    const endTime = new Date();
    const durationMs = endTime.getTime() - startTime.getTime();

    await recordResourceEvent({
      projectId: ctx.projectId,
      workflowId: ctx.workflowId,
      executionId: ctx.executionId,
      componentMetricId: ctx.componentMetricId,
      resourceType: 'connector',
      resourceSubtype: options.connectorType,
      resourceId: options.connectorId,
      resourceName: options.connectorName,
      operation: options.operation,
      durationMs,
      requestSizeBytes: requestSize,
      responseSizeBytes: responseSize,
      status: 'success',
      targetProjectId: options.targetProjectId,
      targetService: options.targetService,
      startedAt: startTime,
      completedAt: endTime,
    }).catch((err) => {
      console.warn('[Observability] Failed to record connector event:', err);
    });

    return result;
  } catch (error) {
    const endTime = new Date();
    const durationMs = endTime.getTime() - startTime.getTime();

    await recordResourceEvent({
      projectId: ctx.projectId,
      workflowId: ctx.workflowId,
      executionId: ctx.executionId,
      componentMetricId: ctx.componentMetricId,
      resourceType: 'connector',
      resourceSubtype: options.connectorType,
      resourceId: options.connectorId,
      resourceName: options.connectorName,
      operation: options.operation,
      durationMs,
      status: 'failure',
      targetProjectId: options.targetProjectId,
      targetService: options.targetService,
      errorType: error instanceof Error ? error.name : 'Error',
      startedAt: startTime,
      completedAt: endTime,
    }).catch((err) => {
      console.warn('[Observability] Failed to record failed connector event:', err);
    });

    throw error;
  }
}

/**
 * Simple fire-and-forget event recording (for cases where wrapping isn't practical)
 */
export function recordResourceEventAsync(
  ctx: ResourceTrackingContext,
  event: {
    resourceType: ResourceType;
    resourceSubtype?: ResourceSubtype;
    resourceId?: string;
    resourceName: string;
    operation: ResourceOperation;
    durationMs?: number;
    status: ResourceEventStatus;
    errorType?: string;
    // Agent fields
    modelName?: string;
    promptTokens?: number;
    completionTokens?: number;
    totalTokens?: number;
    // Connector fields
    targetProjectId?: string;
    targetService?: string;
    // Size tracking
    requestSizeBytes?: number;
    responseSizeBytes?: number;
    metadata?: Record<string, unknown>;
  }
): void {
  const now = new Date();

  recordResourceEvent({
    projectId: ctx.projectId,
    workflowId: ctx.workflowId,
    executionId: ctx.executionId,
    componentMetricId: ctx.componentMetricId,
    resourceType: event.resourceType,
    resourceSubtype: event.resourceSubtype,
    resourceId: event.resourceId,
    resourceName: event.resourceName,
    operation: event.operation,
    durationMs: event.durationMs,
    status: event.status,
    errorType: event.errorType,
    modelName: event.modelName,
    promptTokens: event.promptTokens,
    completionTokens: event.completionTokens,
    totalTokens: event.totalTokens,
    targetProjectId: event.targetProjectId,
    targetService: event.targetService,
    requestSizeBytes: event.requestSizeBytes,
    responseSizeBytes: event.responseSizeBytes,
    startedAt: now,
    completedAt: now,
    metadata: event.metadata,
  }).catch((err) => {
    console.warn('[Observability] Failed to record resource event:', err);
  });
}
