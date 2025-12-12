/**
 * Workflow Validation Utilities
 * Validates workflow definitions before compilation
 */

import type {
  WorkflowDefinition,
  WorkflowNode,
  WorkflowEdge,
  ValidationResult,
  CompilerError,
  CompilerWarning,
} from '../types';

/**
 * Validate a workflow definition
 */
export function validateWorkflow(workflow: WorkflowDefinition): ValidationResult {
  const errors: CompilerError[] = [];
  const warnings: CompilerWarning[] = [];

  // Validate required fields
  if (!workflow.id) {
    errors.push({
      message: 'Workflow ID is required',
      type: 'validation',
      severity: 'fatal',
    });
  }

  if (!workflow.name) {
    errors.push({
      message: 'Workflow name is required',
      type: 'validation',
      severity: 'fatal',
    });
  }

  if (!workflow.nodes || workflow.nodes.length === 0) {
    errors.push({
      message: 'Workflow must have at least one node',
      type: 'validation',
      severity: 'fatal',
    });
    return { valid: false, errors, warnings };
  }

  // Validate nodes
  const nodeErrors = validateNodes(workflow.nodes);
  errors.push(...nodeErrors);

  // Validate edges
  const edgeErrors = validateEdges(workflow.edges || [], workflow.nodes);
  errors.push(...edgeErrors);

  // Check for start node
  const hasStartNode = findStartNode(workflow.nodes, workflow.edges || []);
  if (!hasStartNode) {
    errors.push({
      message: 'Workflow must have a trigger/start node',
      type: 'validation',
      severity: 'error',
    });
  }

  // Check for orphaned nodes
  const orphanedNodes = findOrphanedNodes(workflow.nodes, workflow.edges || []);
  if (orphanedNodes.length > 0) {
    warnings.push({
      message: `Found ${orphanedNodes.length} orphaned node(s): ${orphanedNodes.map(n => n.id).join(', ')}`,
      type: 'best-practice',
    });
  }

  // Check for circular dependencies
  const circularDeps = detectCircularDependencies(workflow.nodes, workflow.edges || []);
  if (circularDeps.length > 0) {
    warnings.push({
      message: `Detected potential circular dependencies: ${circularDeps.join(' -> ')}`,
      type: 'best-practice',
    });
  }

  return {
    valid: errors.length === 0,
    errors,
    warnings,
  };
}

/**
 * Validate individual nodes
 */
function validateNodes(nodes: WorkflowNode[]): CompilerError[] {
  const errors: CompilerError[] = [];
  const nodeIds = new Set<string>();

  for (const node of nodes) {
    // Check for duplicate IDs
    if (nodeIds.has(node.id)) {
      errors.push({
        message: `Duplicate node ID: ${node.id}`,
        nodeId: node.id,
        type: 'validation',
        severity: 'error',
      });
    }
    nodeIds.add(node.id);

    // Validate required fields
    if (!node.type) {
      errors.push({
        message: `Node ${node.id} missing type`,
        nodeId: node.id,
        type: 'validation',
        severity: 'error',
      });
    }

    if (!node.data) {
      errors.push({
        message: `Node ${node.id} missing data`,
        nodeId: node.id,
        type: 'validation',
        severity: 'error',
      });
    } else {
      // Validate node-specific requirements
      const nodeValidationErrors = validateNodeType(node);
      errors.push(...nodeValidationErrors);
    }
  }

  return errors;
}

/**
 * Validate node type-specific requirements
 */
function validateNodeType(node: WorkflowNode): CompilerError[] {
  const errors: CompilerError[] = [];

  switch (node.type) {
    case 'activity':
    case 'agent':
      if (!node.data.componentName && !node.data.componentId) {
        errors.push({
          message: `${node.type} node ${node.id} must have componentName or componentId`,
          nodeId: node.id,
          type: 'validation',
          severity: 'error',
        });
      }
      break;

    case 'signal':
      if (!node.data.signalName && !node.data.config?.signalName) {
        errors.push({
          message: `Signal node ${node.id} must have signalName`,
          nodeId: node.id,
          type: 'validation',
          severity: 'error',
        });
      }
      break;

    case 'child-workflow':
      if (!node.data.componentName && !node.data.config?.workflowType) {
        errors.push({
          message: `Child workflow node ${node.id} must have workflowType`,
          nodeId: node.id,
          type: 'validation',
          severity: 'error',
        });
      }
      break;

    case 'state-variable':
      if (!node.data.config?.name) {
        errors.push({
          message: `State variable node ${node.id} must have a name in config`,
          nodeId: node.id,
          type: 'validation',
          severity: 'error',
        });
      }
      break;

    case 'data-in':
      // Data-in nodes should have endpoint path
      if (!node.data.config?.endpointPath && !node.data.endpointPath) {
        errors.push({
          message: `Data-in node ${node.id} should have an endpoint path`,
          nodeId: node.id,
          type: 'validation',
          severity: 'warning',
        });
      }
      break;

    case 'data-out':
      // Data-out nodes should have endpoint path
      if (!node.data.config?.endpointPath && !node.data.endpointPath) {
        errors.push({
          message: `Data-out node ${node.id} should have an endpoint path`,
          nodeId: node.id,
          type: 'validation',
          severity: 'warning',
        });
      }
      break;
  }

  return errors;
}

/**
 * Validate edges
 */
function validateEdges(edges: WorkflowEdge[], nodes: WorkflowNode[]): CompilerError[] {
  const errors: CompilerError[] = [];
  const nodeIds = new Set(nodes.map(n => n.id));
  const edgeIds = new Set<string>();

  for (const edge of edges) {
    // Check for duplicate edge IDs
    if (edgeIds.has(edge.id)) {
      errors.push({
        message: `Duplicate edge ID: ${edge.id}`,
        type: 'validation',
        severity: 'error',
      });
    }
    edgeIds.add(edge.id);

    // Validate source node exists
    if (!nodeIds.has(edge.source)) {
      errors.push({
        message: `Edge ${edge.id} references non-existent source node: ${edge.source}`,
        type: 'validation',
        severity: 'error',
      });
    }

    // Validate target node exists
    if (!nodeIds.has(edge.target)) {
      errors.push({
        message: `Edge ${edge.id} references non-existent target node: ${edge.target}`,
        type: 'validation',
        severity: 'error',
      });
    }

    // Check for self-loops
    if (edge.source === edge.target) {
      errors.push({
        message: `Edge ${edge.id} creates a self-loop on node ${edge.source}`,
        type: 'validation',
        severity: 'error',
      });
    }
  }

  return errors;
}

/**
 * Find the start node (trigger with no incoming edges)
 */
function findStartNode(nodes: WorkflowNode[], edges: WorkflowEdge[]): WorkflowNode | null {
  const nodesWithIncoming = new Set(edges.map(e => e.target));

  for (const node of nodes) {
    if (node.type === 'trigger' && !nodesWithIncoming.has(node.id)) {
      return node;
    }
  }

  return null;
}

/**
 * Find orphaned nodes (nodes with no incoming or outgoing edges)
 */
function findOrphanedNodes(nodes: WorkflowNode[], edges: WorkflowEdge[]): WorkflowNode[] {
  const nodesWithConnections = new Set<string>();

  for (const edge of edges) {
    nodesWithConnections.add(edge.source);
    nodesWithConnections.add(edge.target);
  }

  return nodes.filter(node =>
    node.type !== 'trigger' &&
    node.type !== 'end' &&
    !nodesWithConnections.has(node.id)
  );
}

/**
 * Detect circular dependencies using DFS
 */
function detectCircularDependencies(nodes: WorkflowNode[], edges: WorkflowEdge[]): string[] {
  const adjList = new Map<string, string[]>();

  // Build adjacency list
  for (const node of nodes) {
    adjList.set(node.id, []);
  }

  for (const edge of edges) {
    const neighbors = adjList.get(edge.source) || [];
    neighbors.push(edge.target);
    adjList.set(edge.source, neighbors);
  }

  // DFS to detect cycles
  const visited = new Set<string>();
  const recursionStack = new Set<string>();
  const path: string[] = [];

  function dfs(nodeId: string): boolean {
    visited.add(nodeId);
    recursionStack.add(nodeId);
    path.push(nodeId);

    const neighbors = adjList.get(nodeId) || [];
    for (const neighbor of neighbors) {
      if (!visited.has(neighbor)) {
        if (dfs(neighbor)) {
          return true;
        }
      } else if (recursionStack.has(neighbor)) {
        // Found a cycle
        return true;
      }
    }

    recursionStack.delete(nodeId);
    path.pop();
    return false;
  }

  // Check all nodes
  for (const node of nodes) {
    if (!visited.has(node.id)) {
      if (dfs(node.id)) {
        return path;
      }
    }
  }

  return [];
}

/**
 * Validate retry policy configuration
 */
export function validateRetryPolicy(policy: any): CompilerError[] {
  const errors: CompilerError[] = [];

  if (!policy) {
    return errors;
  }

  if (!policy.strategy) {
    errors.push({
      message: 'Retry policy must have a strategy',
      type: 'validation',
      severity: 'error',
    });
  }

  if (policy.strategy === 'fail-after-x' && !policy.maxAttempts) {
    errors.push({
      message: 'Retry policy with "fail-after-x" strategy must have maxAttempts',
      type: 'validation',
      severity: 'error',
    });
  }

  if (policy.maxAttempts && policy.maxAttempts < 1) {
    errors.push({
      message: 'Retry policy maxAttempts must be at least 1',
      type: 'validation',
      severity: 'error',
    });
  }

  if (policy.backoffCoefficient && policy.backoffCoefficient < 1) {
    errors.push({
      message: 'Retry policy backoffCoefficient must be at least 1',
      type: 'validation',
      severity: 'error',
    });
  }

  return errors;
}
