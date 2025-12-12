/**
 * Service Classifier
 * 
 * Automatically classifies workflows as 'service' (long-running) or 'task' (short-running)
 * based on structural patterns. This classification is used to determine if continue-as-new
 * should be automatically enabled.
 * 
 * Principle: All classification is automatic - users never see or configure this.
 */

import type { WorkflowDefinition } from '../compiler/types';

export type WorkflowType = 'service' | 'task';

/**
 * Classify a workflow as 'service' or 'task' based on structural patterns
 * 
 * Service Indicators (any of these = service):
 * - Has signal handler nodes
 * - Has query handler nodes
 * - Has loop nodes with no max iterations (infinite loops)
 * - Has no end nodes (workflow never terminates)
 * - Has scheduled triggers
 * 
 * Task Indicators (all of these = task):
 * - Has explicit end node
 * - All loops have max iterations
 * - No signal handlers
 * - No query handlers
 * - Has clear termination path
 */
export function classifyWorkflow(definition: WorkflowDefinition): WorkflowType {
  const { nodes } = definition;
  
  // Check for service indicators
  const hasSignalHandlers = nodes.some(node => node.type === 'signal');
  const hasQueryHandlers = nodes.some(node => 
    node.type === 'query' || 
    (node.type === 'data-out' && node.data.config?.interfaceType === 'query')
  );
  const hasInfiniteLoops = nodes.some(node => {
    if (node.type !== 'loop') return false;
    const maxIterations = node.data.config?.maxIterations;
    return maxIterations === undefined || maxIterations === null || maxIterations <= 0;
  });
  const hasNoEndNode = !nodes.some(node => node.type === 'end');
  
  // If any service indicator is present, classify as service
  if (hasSignalHandlers || hasQueryHandlers || hasInfiniteLoops || hasNoEndNode) {
    return 'service';
  }
  
  // Check for task indicators
  const hasEndNode = nodes.some(node => node.type === 'end');
  const allLoopsHaveMaxIterations = nodes
    .filter(node => node.type === 'loop')
    .every(node => {
      const maxIterations = node.data.config?.maxIterations;
      return maxIterations !== undefined && maxIterations !== null && maxIterations > 0;
    });
  const hasNoSignalHandlers = !hasSignalHandlers;
  const hasNoQueryHandlers = !hasQueryHandlers;
  
  // If all task indicators are present, classify as task
  if (hasEndNode && allLoopsHaveMaxIterations && hasNoSignalHandlers && hasNoQueryHandlers) {
    return 'task';
  }
  
  // Default to service if ambiguous (safer for long-running workflows)
  return 'service';
}

/**
 * Check if a workflow has scheduled triggers
 * This is checked separately as it requires database context
 */
export function hasScheduledTriggers(isScheduled: boolean): boolean {
  return isScheduled;
}

