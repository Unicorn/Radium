/**
 * Interface Validation
 * Validates connections for data-in and data-out interface components
 */

import type { Edge, Node } from 'react-flow-renderer';

export interface ValidationError {
  nodeId: string;
  message: string;
}

/**
 * Validate data-in component connection
 * Data-in components must have an outgoing edge to a component that accepts input
 */
export function validateDataInConnection(
  nodeId: string,
  edges: Edge[],
  nodes: Node[]
): ValidationError | null {
  // Find the data-in node
  const dataInNode = nodes.find(n => n.id === nodeId);
  if (!dataInNode || dataInNode.type !== 'data-in') {
    return null; // Not a data-in node, skip validation
  }

  // Check if there's an outgoing edge
  const outgoingEdge = edges.find(e => e.source === nodeId);
  if (!outgoingEdge) {
    return {
      nodeId,
      message: `Your data-in component "${dataInNode.data?.label || dataInNode.data?.displayName || nodeId}" isn't connected to another component. Please connect it to a component that accepts data input.`,
    };
  }

  // Check if target node exists and can accept input
  const targetNode = nodes.find(n => n.id === outgoingEdge.target);
  if (!targetNode) {
    return {
      nodeId,
      message: `Your data-in component "${dataInNode.data?.label || dataInNode.data?.displayName || nodeId}" is connected to a non-existent node.`,
    };
  }

  // Validate that target can accept input (not an end node, not another data-in)
  const invalidTargetTypes = ['end', 'data-in', 'trigger'];
  if (invalidTargetTypes.includes(targetNode.type || '')) {
    return {
      nodeId,
      message: `Your data-in component "${dataInNode.data?.label || dataInNode.data?.displayName || nodeId}" is connected to an invalid target type "${targetNode.type}".`,
    };
  }

  return null; // Valid connection
}

/**
 * Validate data-out component connection
 * Data-out components must have an incoming edge from a data source (state variable or activity that returns data)
 */
export function validateDataOutConnection(
  nodeId: string,
  edges: Edge[],
  nodes: Node[]
): ValidationError | null {
  // Find the data-out node
  const dataOutNode = nodes.find(n => n.id === nodeId);
  if (!dataOutNode || dataOutNode.type !== 'data-out') {
    return null; // Not a data-out node, skip validation
  }

  // Check if there's an incoming edge
  const incomingEdge = edges.find(e => e.target === nodeId);
  if (!incomingEdge) {
    return {
      nodeId,
      message: `Your data-out component "${dataOutNode.data?.label || dataOutNode.data?.displayName || nodeId}" isn't connected to a data source. Please connect it to a state variable or an activity that returns data.`,
    };
  }

  // Check if source node exists
  const sourceNode = nodes.find(n => n.id === incomingEdge.source);
  if (!sourceNode) {
    return {
      nodeId,
      message: `Your data-out component "${dataOutNode.data?.label || dataOutNode.data?.displayName || nodeId}" is connected to a non-existent source node.`,
    };
  }

  // Validate that source can provide data
  // Valid sources: state-variable, activity, agent, data-out (chaining), query
  const validSourceTypes = ['state-variable', 'activity', 'agent', 'data-out', 'query'];
  if (!validSourceTypes.includes(sourceNode.type || '')) {
    return {
      nodeId,
      message: `Your data-out component "${dataOutNode.data?.label || dataOutNode.data?.displayName || nodeId}" is connected to an invalid source type "${sourceNode.type}". It must be connected to a state variable or an activity/agent that returns data.`,
    };
  }

  return null; // Valid connection
}

/**
 * Validate all interface components in a workflow
 */
export function validateInterfaceComponents(
  nodes: Node[],
  edges: Edge[]
): ValidationError[] {
  const errors: ValidationError[] = [];

  for (const node of nodes) {
    if (node.type === 'data-in') {
      const error = validateDataInConnection(node.id, edges, nodes);
      if (error) {
        errors.push(error);
      }
    } else if (node.type === 'data-out') {
      const error = validateDataOutConnection(node.id, edges, nodes);
      if (error) {
        errors.push(error);
      }
    }
  }

  return errors;
}

