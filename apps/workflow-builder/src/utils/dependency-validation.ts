/**
 * Dependency Validation Utilities
 * 
 * Detects circular dependencies in blockUntil configuration
 * Validates work queue dependencies and signal/query connections
 */

import type { CircularDependencyCheckResult } from '../types/advanced-patterns';

/**
 * Node dependency representation
 */
interface DependencyNode {
  id: string;
  type: string;
  blocksOn?: string[];  // Queue names this node waits for
  addsTo?: string[];    // Queue names this node adds to
}

/**
 * Dependency graph for workflow
 */
interface DependencyGraph {
  nodes: Map<string, DependencyNode>;
  queueDependencies: Map<string, Set<string>>;  // queue -> nodes that add to it
  queueBlockers: Map<string, Set<string>>;      // queue -> nodes that block on it
}

/**
 * Check for circular dependencies in blockUntil configuration
 */
export function checkCircularDependencies(
  nodes: DependencyNode[]
): CircularDependencyCheckResult {
  // Build dependency graph
  const graph = buildDependencyGraph(nodes);

  // Detect cycles using DFS
  const visited = new Set<string>();
  const recursionStack = new Set<string>();
  const path: string[] = [];

  for (const node of nodes) {
    if (!visited.has(node.id)) {
      const cycle = detectCycleDFS(node.id, graph, visited, recursionStack, path);
      if (cycle) {
        return {
          hasCircularDependency: true,
          cycle,
          message: `Circular dependency detected: ${cycle.join(' → ')}`,
        };
      }
    }
  }

  return {
    hasCircularDependency: false,
  };
}

/**
 * Build dependency graph from nodes
 */
function buildDependencyGraph(nodes: DependencyNode[]): DependencyGraph {
  const graph: DependencyGraph = {
    nodes: new Map(),
    queueDependencies: new Map(),
    queueBlockers: new Map(),
  };

  // Initialize maps
  for (const node of nodes) {
    graph.nodes.set(node.id, node);

    // Track which nodes add to which queues
    if (node.addsTo) {
      for (const queueName of node.addsTo) {
        if (!graph.queueDependencies.has(queueName)) {
          graph.queueDependencies.set(queueName, new Set());
        }
        graph.queueDependencies.get(queueName)!.add(node.id);
      }
    }

    // Track which nodes block on which queues
    if (node.blocksOn) {
      for (const queueName of node.blocksOn) {
        if (!graph.queueBlockers.has(queueName)) {
          graph.queueBlockers.set(queueName, new Set());
        }
        graph.queueBlockers.get(queueName)!.add(node.id);
      }
    }
  }

  return graph;
}

/**
 * Detect cycle using Depth-First Search
 */
function detectCycleDFS(
  nodeId: string,
  graph: DependencyGraph,
  visited: Set<string>,
  recursionStack: Set<string>,
  path: string[]
): string[] | null {
  visited.add(nodeId);
  recursionStack.add(nodeId);
  path.push(nodeId);

  const node = graph.nodes.get(nodeId);
  if (!node) {
    path.pop();
    recursionStack.delete(nodeId);
    return null;
  }

  // Check dependencies through queues
  if (node.blocksOn) {
    for (const queueName of node.blocksOn) {
      // Get all nodes that add to this queue
      const providingNodes = graph.queueDependencies.get(queueName);
      
      if (providingNodes) {
        for (const providingNodeId of providingNodes) {
          if (recursionStack.has(providingNodeId)) {
            // Found a cycle!
            const cycleStart = path.indexOf(providingNodeId);
            return [...path.slice(cycleStart), providingNodeId];
          }

          if (!visited.has(providingNodeId)) {
            const cycle = detectCycleDFS(
              providingNodeId,
              graph,
              visited,
              recursionStack,
              path
            );
            if (cycle) {
              return cycle;
            }
          }
        }
      }
    }
  }

  path.pop();
  recursionStack.delete(nodeId);
  return null;
}

/**
 * Validate that a new blockUntil dependency doesn't create a cycle
 */
export function validateNewBlockDependency(
  nodeId: string,
  blockQueue: string,
  existingNodes: DependencyNode[]
): CircularDependencyCheckResult {
  // Find the node being modified
  const nodeIndex = existingNodes.findIndex(n => n.id === nodeId);
  if (nodeIndex === -1) {
    return {
      hasCircularDependency: false,
      message: 'Node not found',
    };
  }

  // Create a copy with the new dependency
  const updatedNodes = [...existingNodes];
  const updatedNode = { ...updatedNodes[nodeIndex] };
  updatedNode.blocksOn = [...(updatedNode.blocksOn || []), blockQueue];
  updatedNodes[nodeIndex] = updatedNode;

  // Check for cycles
  return checkCircularDependencies(updatedNodes);
}

/**
 * Get dependency chain for a node
 */
export function getDependencyChain(
  nodeId: string,
  nodes: DependencyNode[]
): string[] {
  const chain: string[] = [nodeId];
  const visited = new Set<string>([nodeId]);

  const node = nodes.find(n => n.id === nodeId);
  if (!node || !node.blocksOn) {
    return chain;
  }

  // Build graph
  const graph = buildDependencyGraph(nodes);

  // Follow dependencies
  for (const queueName of node.blocksOn) {
    const providingNodes = graph.queueDependencies.get(queueName);
    if (providingNodes) {
      for (const providingNodeId of providingNodes) {
        if (!visited.has(providingNodeId)) {
          chain.push(providingNodeId);
          visited.add(providingNodeId);
          
          // Recursively add dependencies
          const subChain = getDependencyChain(providingNodeId, nodes);
          for (const subNodeId of subChain) {
            if (!visited.has(subNodeId)) {
              chain.push(subNodeId);
              visited.add(subNodeId);
            }
          }
        }
      }
    }
  }

  return chain;
}

/**
 * Get all nodes that depend on a given queue
 */
export function getQueueDependents(
  queueName: string,
  nodes: DependencyNode[]
): string[] {
  return nodes
    .filter(node => node.blocksOn?.includes(queueName))
    .map(node => node.id);
}

/**
 * Get all nodes that add to a given queue
 */
export function getQueueProviders(
  queueName: string,
  nodes: DependencyNode[]
): string[] {
  return nodes
    .filter(node => node.addsTo?.includes(queueName))
    .map(node => node.id);
}

/**
 * Validate signal/query name uniqueness within workflow
 */
export function validateSignalQueryUniqueness(
  workflowId: string,
  signals: Array<{ name: string; id?: string }>,
  queries: Array<{ name: string; id?: string }>
): {
  valid: boolean;
  errors: Array<{ type: 'signal' | 'query'; name: string; message: string }>;
} {
  const errors: Array<{ type: 'signal' | 'query'; name: string; message: string }> = [];

  // Check signal name uniqueness
  const signalNames = new Set<string>();
  for (const signal of signals) {
    if (signalNames.has(signal.name)) {
      errors.push({
        type: 'signal',
        name: signal.name,
        message: `Duplicate signal name: ${signal.name}`,
      });
    }
    signalNames.add(signal.name);
  }

  // Check query name uniqueness
  const queryNames = new Set<string>();
  for (const query of queries) {
    if (queryNames.has(query.name)) {
      errors.push({
        type: 'query',
        name: query.name,
        message: `Duplicate query name: ${query.name}`,
      });
    }
    queryNames.add(query.name);
  }

  // Check for name collisions between signals and queries
  for (const signal of signals) {
    if (queryNames.has(signal.name)) {
      errors.push({
        type: 'signal',
        name: signal.name,
        message: `Signal name ${signal.name} conflicts with a query name`,
      });
    }
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}

/**
 * Validate work queue name uniqueness within workflow
 */
export function validateWorkQueueUniqueness(
  workflowId: string,
  workQueues: Array<{ name: string; id?: string }>
): {
  valid: boolean;
  errors: Array<{ name: string; message: string }>;
} {
  const errors: Array<{ name: string; message: string }> = [];
  const queueNames = new Set<string>();

  for (const queue of workQueues) {
    if (queueNames.has(queue.name)) {
      errors.push({
        name: queue.name,
        message: `Duplicate work queue name: ${queue.name}`,
      });
    }
    queueNames.add(queue.name);
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}

/**
 * Validate that referenced work queues exist
 */
export function validateWorkQueueReferences(
  nodes: Array<{
    id: string;
    workQueueTarget?: string | null;
    blockUntilQueue?: string | null;
  }>,
  workQueues: Array<{ name: string }>
): {
  valid: boolean;
  errors: Array<{ nodeId: string; queueName: string; message: string }>;
} {
  const errors: Array<{ nodeId: string; queueName: string; message: string }> = [];
  const queueNames = new Set(workQueues.map(q => q.name));

  for (const node of nodes) {
    // Check workQueueTarget
    if (node.workQueueTarget && !queueNames.has(node.workQueueTarget)) {
      errors.push({
        nodeId: node.id,
        queueName: node.workQueueTarget,
        message: `Work queue '${node.workQueueTarget}' does not exist`,
      });
    }

    // Check blockUntilQueue
    if (node.blockUntilQueue && !queueNames.has(node.blockUntilQueue)) {
      errors.push({
        nodeId: node.id,
        queueName: node.blockUntilQueue,
        message: `Work queue '${node.blockUntilQueue}' does not exist`,
      });
    }
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}

/**
 * Get topological sort of nodes based on dependencies
 * Useful for determining execution order
 */
export function getTopologicalSort(nodes: DependencyNode[]): string[] | null {
  const graph = buildDependencyGraph(nodes);
  const inDegree = new Map<string, number>();
  const result: string[] = [];
  const queue: string[] = [];

  // Calculate in-degree for each node
  for (const node of nodes) {
    inDegree.set(node.id, 0);
  }

  for (const node of nodes) {
    if (node.blocksOn) {
      for (const queueName of node.blocksOn) {
        const providingNodes = graph.queueDependencies.get(queueName);
        if (providingNodes) {
          for (const providingNodeId of providingNodes) {
            inDegree.set(node.id, (inDegree.get(node.id) || 0) + 1);
          }
        }
      }
    }
  }

  // Add nodes with no dependencies to queue
  for (const [nodeId, degree] of inDegree.entries()) {
    if (degree === 0) {
      queue.push(nodeId);
    }
  }

  // Process queue
  while (queue.length > 0) {
    const nodeId = queue.shift()!;
    result.push(nodeId);

    const node = graph.nodes.get(nodeId);
    if (node?.addsTo) {
      for (const queueName of node.addsTo) {
        const dependentNodes = graph.queueBlockers.get(queueName);
        if (dependentNodes) {
          for (const dependentNodeId of dependentNodes) {
            const newDegree = (inDegree.get(dependentNodeId) || 0) - 1;
            inDegree.set(dependentNodeId, newDegree);
            
            if (newDegree === 0) {
              queue.push(dependentNodeId);
            }
          }
        }
      }
    }
  }

  // If we didn't process all nodes, there's a cycle
  if (result.length !== nodes.length) {
    return null;  // Cycle detected
  }

  return result;
}

