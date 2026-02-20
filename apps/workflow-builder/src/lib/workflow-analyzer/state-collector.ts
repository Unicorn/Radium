/**
 * State Collector
 * 
 * Collects all workflow state that needs to be preserved across continue-as-new.
 * Automatically identifies all variables, state variables, loop counters, and signal queues.
 */

import type { WorkflowDefinition } from '../compiler/types';

/**
 * Map of state variable names to their types
 */
export interface StateMap {
  [key: string]: {
    type: string;
    source: 'variable' | 'state-variable' | 'loop-counter' | 'signal-queue' | 'internal';
  };
}

/**
 * Collect all workflow state that needs to be preserved
 * 
 * Collects:
 * - All workflow variables
 * - All state variables from state-variable nodes
 * - All loop iteration counters
 * - All signal queue state
 * - Internal tracking variables
 */
export function collectWorkflowState(definition: WorkflowDefinition): StateMap {
  const stateMap: StateMap = {};
  
  // Collect workflow variables
  for (const variable of definition.variables || []) {
    stateMap[variable.name] = {
      type: variable.type,
      source: 'variable',
    };
  }
  
  // Collect state variables from state-variable nodes
  const stateVariableNodes = definition.nodes.filter(node => node.type === 'state-variable');
  for (const node of stateVariableNodes) {
    const varName = node.data.config?.variableName || node.data.label;
    if (varName) {
      stateMap[varName] = {
        type: node.data.config?.variableType || 'object',
        source: 'state-variable',
      };
    }
  }
  
  // Collect loop counters (one per loop node)
  const loopNodes = definition.nodes.filter(node => node.type === 'loop');
  for (const loopNode of loopNodes) {
    const counterName = `_${loopNode.id}_iterationCount`;
    stateMap[counterName] = {
      type: 'number',
      source: 'loop-counter',
    };
  }
  
  // Collect signal queues (one per signal handler)
  const signalNodes = definition.nodes.filter(node => node.type === 'signal');
  if (signalNodes.length > 0) {
    stateMap['_signalQueue'] = {
      type: 'array',
      source: 'signal-queue',
    };
  }
  
  // Add internal tracking variables
  stateMap['_workflowStartTime'] = {
    type: 'number',
    source: 'internal',
  };
  stateMap['_historyResetCount'] = {
    type: 'number',
    source: 'internal',
  };
  
  return stateMap;
}

/**
 * Generate state object code for continue-as-new
 * Creates a TypeScript object literal with all state variables
 */
export function generateStateObject(stateMap: StateMap): string {
  const stateEntries: string[] = [];
  
  for (const [name, info] of Object.entries(stateMap)) {
    // Skip internal variables that are set in continue-as-new call
    if (info.source === 'internal' && (name === '_workflowStartTime' || name === '_historyResetCount')) {
      continue;
    }
    
    stateEntries.push(`    ${name}: ${name}`);
  }
  
  // Add internal variables at the end
  stateEntries.push(`    _workflowStartTime: workflowInfo().runStartTime.getTime(),`);
  stateEntries.push(`    _historyResetCount: (_historyResetCount || 0) + 1`);
  
  return stateEntries.join(',\n');
}
