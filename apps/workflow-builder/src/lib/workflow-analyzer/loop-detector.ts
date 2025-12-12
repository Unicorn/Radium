/**
 * Loop Detector
 * 
 * Detects loops that may need continue-as-new handling.
 * Identifies long-running loops that could benefit from history reset.
 */

import type { WorkflowDefinition, WorkflowNode } from '../compiler/types';

/**
 * Detect loops that may need continue-as-new
 * 
 * Loop Indicators:
 * - Node with type === 'loop'
 * - Loop with no maxIterations (infinite loop)
 * - Loop with maxIterations > 100 (potentially long-running)
 * - Loop that contains signal handlers (indicates service pattern)
 */
export function detectLongRunningLoops(definition: WorkflowDefinition): string[] {
  const { nodes, edges } = definition;
  const longRunningLoopIds: string[] = [];
  
  // Find all loop nodes
  const loopNodes = nodes.filter(node => node.type === 'loop');
  
  for (const loopNode of loopNodes) {
    const config = loopNode.data.config || {};
    const maxIterations = config.maxIterations;
    
    // Check if loop is infinite or long-running
    const isInfinite = maxIterations === undefined || maxIterations === null || maxIterations <= 0;
    const isLongRunning = maxIterations !== undefined && maxIterations > 100;
    
    // Check if loop contains signal handlers (find nodes inside loop)
    const nodesInLoop = getNodesInLoop(loopNode.id, nodes, edges);
    const hasSignalHandlers = nodesInLoop.some(node => node.type === 'signal');
    
    // Mark as long-running if any indicator is present
    if (isInfinite || isLongRunning || hasSignalHandlers) {
      longRunningLoopIds.push(loopNode.id);
    }
  }
  
  return longRunningLoopIds;
}

/**
 * Get all nodes that are inside a loop
 * This is a simplified version - in a full implementation, we'd need to
 * properly traverse the graph to find nodes within the loop's scope
 */
function getNodesInLoop(
  loopNodeId: string,
  nodes: WorkflowNode[],
  edges: Array<{ source: string; target: string }>
): WorkflowNode[] {
  // Find edges that start from the loop node
  const loopEdges = edges.filter(edge => edge.source === loopNodeId);
  
  // Get target node IDs
  const targetIds = new Set(loopEdges.map(edge => edge.target));
  
  // Find all nodes that are targets of the loop
  const nodesInLoop: WorkflowNode[] = [];
  const visited = new Set<string>();
  
  // Recursively find all nodes reachable from loop targets
  function traverse(nodeId: string) {
    if (visited.has(nodeId)) return;
    visited.add(nodeId);
    
    const node = nodes.find(n => n.id === nodeId);
    if (node) {
      nodesInLoop.push(node);
      
      // Find edges from this node
      const outgoingEdges = edges.filter(edge => edge.source === nodeId);
      for (const edge of outgoingEdges) {
        traverse(edge.target);
      }
    }
  }
  
  // Start traversal from loop targets
  for (const targetId of targetIds) {
    traverse(targetId);
  }
  
  return nodesInLoop;
}

