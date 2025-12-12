/**
 * Enhanced Workflow Validator
 * Provides comprehensive validation with specific, actionable error messages
 */

import type {
  WorkflowDefinition,
  WorkflowNode,
  WorkflowEdge,
} from '../compiler/types';
import {
  ValidationError,
  CompilationError,
} from '../errors/workflow-errors';
import { validateInterfaceComponents } from './interface-validation';

/**
 * Enhanced validation result
 */
export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
  metadata?: {
    nodeCount: number;
    edgeCount: number;
    hasStartNode: boolean;
    hasEndNode: boolean;
    orphanedNodeCount: number;
  };
}

export interface ValidationWarning {
  message: string;
  nodeId?: string;
  severity: 'low' | 'medium' | 'high';
  suggestion: string;
}

/**
 * Comprehensive workflow validation
 */
export function validateWorkflow(workflow: WorkflowDefinition): ValidationResult {
  const errors: ValidationError[] = [];
  const warnings: ValidationWarning[] = [];

  // Basic structure validation
  validateBasicStructure(workflow, errors);

  if (!workflow.nodes || workflow.nodes.length === 0) {
    // Early return if no nodes
    return {
      valid: false,
      errors,
      warnings,
      metadata: {
        nodeCount: 0,
        edgeCount: 0,
        hasStartNode: false,
        hasEndNode: false,
        orphanedNodeCount: 0,
      },
    };
  }

  // Node validation (only if nodes is an array)
  if (Array.isArray(workflow.nodes)) {
    validateNodes(workflow.nodes, errors, warnings);

    // Edge validation
    validateEdges(workflow.edges || [], workflow.nodes, errors, warnings);

    // Flow validation
    const flowResults = validateWorkflowFlow(workflow.nodes, workflow.edges || []);
  } else {
    // Return early if nodes is not an array
    return {
      valid: false,
      errors,
      warnings,
      metadata: {
        nodeCount: 0,
        edgeCount: 0,
        hasStartNode: false,
        hasEndNode: false,
        orphanedNodeCount: 0,
      },
    };
  }

  // Continue with other validations only if nodes is valid
  const flowResults = validateWorkflowFlow(workflow.nodes, workflow.edges || []);
  errors.push(...flowResults.errors);
  warnings.push(...flowResults.warnings);

  // Variable validation
  if (workflow.variables && workflow.variables.length > 0) {
    validateVariables(workflow.variables, workflow.nodes, errors, warnings);
  }

  // Interface component validation
  // Convert workflow nodes/edges to React Flow format for validation
  const reactFlowNodes = workflow.nodes.map((n: WorkflowNode) => ({
    id: n.id,
    type: n.type,
    data: n.data || {},
  }));
  const reactFlowEdges = (workflow.edges || []).map((e: WorkflowEdge) => ({
    id: e.id,
    source: e.source,
    target: e.target,
  }));
  const interfaceErrors = validateInterfaceComponents(reactFlowNodes, reactFlowEdges);
  // Convert interface validation errors to ValidationError format
  for (const interfaceError of interfaceErrors) {
    errors.push(
      new ValidationError({
        message: interfaceError.message,
        field: 'nodes',
        invalidValue: interfaceError.nodeId,
        recoverySuggestions: ['Connect the interface component to a valid target/source component'],
        details: { nodeId: interfaceError.nodeId },
      })
    );
  }

  // Settings validation
  if (workflow.settings) {
    validateSettings(workflow.settings, errors, warnings);
  }

  // Calculate metadata
  const metadata = {
    nodeCount: workflow.nodes.length,
    edgeCount: (workflow.edges || []).length,
    hasStartNode: flowResults.hasStartNode,
    hasEndNode: flowResults.hasEndNode,
    orphanedNodeCount: flowResults.orphanedNodes.length,
  };

  return {
    valid: errors.length === 0,
    errors,
    warnings,
    metadata,
  };
}

/**
 * Validate basic workflow structure
 */
function validateBasicStructure(
  workflow: WorkflowDefinition,
  errors: ValidationError[]
): void {
  if (!workflow.id) {
    errors.push(
      new ValidationError({
        message: 'Workflow ID is required',
        field: 'id',
        recoverySuggestions: ['Provide a unique workflow ID'],
      })
    );
  }

  if (!workflow.name) {
    errors.push(
      new ValidationError({
        message: 'Workflow name is required',
        field: 'name',
        recoverySuggestions: ['Provide a descriptive workflow name'],
      })
    );
  }

  if (!workflow.nodes || !Array.isArray(workflow.nodes)) {
    errors.push(
      new ValidationError({
        message: 'Workflow nodes must be an array',
        field: 'nodes',
        invalidValue: typeof workflow.nodes,
        recoverySuggestions: [
          'Add at least one node to your workflow',
          'Check that nodes is an array',
        ],
      })
    );
  }

  if (workflow.edges && !Array.isArray(workflow.edges)) {
    errors.push(
      new ValidationError({
        message: 'Workflow edges must be an array',
        field: 'edges',
        invalidValue: typeof workflow.edges,
        recoverySuggestions: ['Check that edges is an array'],
      })
    );
  }
}

/**
 * Validate individual nodes with specific error messages
 */
function validateNodes(
  nodes: WorkflowNode[],
  errors: ValidationError[],
  warnings: ValidationWarning[]
): void {
  const nodeIds = new Set<string>();
  const nodeLabels = new Map<string, number>();

  for (const node of nodes) {
    // Check for duplicate IDs
    if (nodeIds.has(node.id)) {
      errors.push(
        new ValidationError({
          message: `Duplicate node ID: ${node.id}`,
          field: 'nodes',
          invalidValue: node.id,
          recoverySuggestions: [
            'Each node must have a unique ID',
            'Remove or rename the duplicate node',
          ],
          details: { nodeId: node.id },
        })
      );
    }
    nodeIds.add(node.id);

    // Check for missing type
    if (!node.type) {
      errors.push(
        new ValidationError({
          message: `Node ${node.id} is missing a type`,
          field: 'nodes',
          recoverySuggestions: [
            'Assign a valid node type (trigger, activity, agent, etc.)',
          ],
          details: { nodeId: node.id },
        })
      );
      continue;
    }

    // Check for missing data
    if (!node.data) {
      errors.push(
        new ValidationError({
          message: `Node ${node.id} is missing data configuration`,
          field: 'nodes',
          recoverySuggestions: ['Configure the node with required data'],
          details: { nodeId: node.id, type: node.type },
        })
      );
      continue;
    }

    // Check for missing label
    if (!node.data.label) {
      warnings.push({
        message: `Node ${node.id} has no label`,
        nodeId: node.id,
        severity: 'low',
        suggestion: 'Add a descriptive label to improve workflow readability',
      });
    } else {
      // Track duplicate labels (warning only)
      const labelCount = nodeLabels.get(node.data.label) || 0;
      nodeLabels.set(node.data.label, labelCount + 1);
    }

    // Type-specific validation
    validateNodeType(node, errors, warnings);
  }

  // Warn about duplicate labels
  for (const [label, count] of nodeLabels.entries()) {
    if (count > 1) {
      warnings.push({
        message: `${count} nodes have the same label: "${label}"`,
        severity: 'medium',
        suggestion: 'Use unique labels to make nodes easier to identify',
      });
    }
  }
}

/**
 * Validate node type-specific requirements
 */
function validateNodeType(
  node: WorkflowNode,
  errors: ValidationError[],
  warnings: ValidationWarning[]
): void {
  switch (node.type) {
    case 'activity':
    case 'agent':
      if (!node.data.componentName && !node.data.componentId) {
        errors.push(
          new ValidationError({
            message: `${node.type} node "${node.data.label || node.id}" must reference a component`,
            field: 'nodes',
            recoverySuggestions: [
              'Select a component from the component library',
              'Ensure the component exists and is published',
            ],
            details: { nodeId: node.id, type: node.type },
          })
        );
      }

      // Warn if no config
      if (!node.data.config || Object.keys(node.data.config).length === 0) {
        warnings.push({
          message: `${node.type} node "${node.data.label || node.id}" has no configuration`,
          nodeId: node.id,
          severity: 'medium',
          suggestion: 'Configure the node with required parameters',
        });
      }

      // Warn if no timeout
      if (!node.data.timeout) {
        warnings.push({
          message: `${node.type} node "${node.data.label || node.id}" has no timeout`,
          nodeId: node.id,
          severity: 'low',
          suggestion: 'Set a timeout to prevent indefinite execution',
        });
      }
      break;

    case 'signal':
      if (!node.data.signalName && !node.data.config?.signalName) {
        errors.push(
          new ValidationError({
            message: `Signal node "${node.data.label || node.id}" must have a signal name`,
            field: 'nodes',
            recoverySuggestions: [
              'Specify a signal name in the node configuration',
              'Signal names should be descriptive and unique',
            ],
            details: { nodeId: node.id },
          })
        );
      }
      break;

    case 'kong-logging':
      // Kong logging component must have a connector selected
      if (!node.data.config?.connectorId) {
        errors.push(
          new ValidationError({
            message: `Kong Logging component "${node.data.label || node.id}" must have a logging connector selected`,
            field: 'nodes',
            recoverySuggestions: [
              'Select a logging connector in the component properties',
              'Logging connectors include HTTP log, Syslog, File log, TCP log, and UDP log',
            ],
            details: { nodeId: node.id, type: node.type },
          })
        );
      }
      break;

    case 'kong-cache':
      // Kong cache component must have a Redis connector selected
      if (!node.data.config?.connectorId) {
        errors.push(
          new ValidationError({
            message: `Kong Cache component "${node.data.label || node.id}" must have a Redis connector selected`,
            field: 'nodes',
            recoverySuggestions: [
              'Select a Redis connector in the component properties',
              'Redis connectors include Upstash, Redis Cloud, and other Redis-compatible services',
            ],
            details: { nodeId: node.id, type: node.type },
          })
        );
      }
      // Kong cache component must have a cache key
      if (!node.data.config?.cacheKey) {
        errors.push(
          new ValidationError({
            message: `Kong Cache component "${node.data.label || node.id}" must have a cache key`,
            field: 'nodes',
            recoverySuggestions: [
              'A cache key will be auto-generated when you add the component',
              'You can edit the cache key until you save the component',
            ],
            details: { nodeId: node.id, type: node.type },
          })
        );
      }
      break;

    case 'kong-cors':
      // Kong CORS component should have at least one allowed origin
      if (!node.data.config?.allowedOrigins || (Array.isArray(node.data.config.allowedOrigins) && node.data.config.allowedOrigins.length === 0)) {
        errors.push(
          new ValidationError({
            message: `Kong CORS component "${node.data.label || node.id}" must have at least one allowed origin configured`,
            field: 'nodes',
            recoverySuggestions: [
              'Configure at least one allowed origin in the component properties',
              'Examples: https://example.com or * for all origins (not recommended for production)',
            ],
            details: { nodeId: node.id, type: node.type },
          })
        );
      }
      // Kong CORS component should have at least one allowed method
      if (!node.data.config?.allowedMethods || (Array.isArray(node.data.config.allowedMethods) && node.data.config.allowedMethods.length === 0)) {
        errors.push(
          new ValidationError({
            message: `Kong CORS component "${node.data.label || node.id}" must have at least one allowed method configured`,
            field: 'nodes',
            recoverySuggestions: [
              'Configure at least one allowed HTTP method in the component properties',
              'Common methods: GET, POST, PUT, DELETE, OPTIONS',
            ],
            details: { nodeId: node.id, type: node.type },
          })
        );
      }
      break;

    case 'graphql-gateway':
      // GraphQL gateway component must have a schema defined
      if (!node.data.config?.schema) {
        errors.push(
          new ValidationError({
            message: `GraphQL Gateway component "${node.data.label || node.id}" must have a GraphQL schema defined`,
            field: 'nodes',
            recoverySuggestions: [
              'Define a GraphQL schema in the component properties',
              'The schema should include type definitions, queries, and mutations',
            ],
            details: { nodeId: node.id, type: node.type },
          })
        );
      }
      // GraphQL gateway should have at least one query or mutation
      const queries = node.data.config?.queries || [];
      const mutations = node.data.config?.mutations || [];
      if (queries.length === 0 && mutations.length === 0) {
        errors.push(
          new ValidationError({
            message: `GraphQL Gateway component "${node.data.label || node.id}" must have at least one query or mutation defined`,
            field: 'nodes',
            recoverySuggestions: [
              'Add at least one query or mutation to the GraphQL schema',
              'Queries map to GET operations, mutations map to POST/PUT/DELETE operations',
            ],
            details: { nodeId: node.id, type: node.type },
          })
        );
      }
      break;

    case 'mcp-server':
      // MCP server component must have a server name
      if (!node.data.config?.serverName) {
        errors.push(
          new ValidationError({
            message: `MCP Server component "${node.data.label || node.id}" must have a server name configured`,
            field: 'nodes',
            recoverySuggestions: [
              'Configure a server name in the component properties',
              'The server name identifies this MCP server instance',
            ],
            details: { nodeId: node.id, type: node.type },
          })
        );
      }
      // MCP server should have at least one resource or tool
      const resources = node.data.config?.resources || [];
      const tools = node.data.config?.tools || [];
      if (resources.length === 0 && tools.length === 0) {
        errors.push(
          new ValidationError({
            message: `MCP Server component "${node.data.label || node.id}" must have at least one resource or tool defined`,
            field: 'nodes',
            recoverySuggestions: [
              'Add at least one resource or tool to the MCP server',
              'Resources expose data, tools expose executable functions',
            ],
            details: { nodeId: node.id, type: node.type },
          })
        );
      }
      break;

    case 'child-workflow':
      if (!node.data.componentName && !node.data.config?.workflowType) {
        errors.push(
          new ValidationError({
            message: `Child workflow node "${node.data.label || node.id}" must reference a workflow`,
            field: 'nodes',
            recoverySuggestions: [
              'Select a workflow to execute as a child',
              'Ensure the target workflow is published',
            ],
            details: { nodeId: node.id },
          })
        );
      }
      break;

    case 'state-variable':
      if (!node.data.config?.name) {
        errors.push(
          new ValidationError({
            message: `State variable node "${node.data.label || node.id}" must have a variable name`,
            field: 'nodes',
            recoverySuggestions: [
              'Specify a variable name',
              'Variable names should be camelCase',
            ],
            details: { nodeId: node.id },
          })
        );
      }

      if (!node.data.config?.operation) {
        errors.push(
          new ValidationError({
            message: `State variable node "${node.data.label || node.id}" must have an operation (set/update)`,
            field: 'nodes',
            recoverySuggestions: ['Specify whether to set or update the variable'],
            details: { nodeId: node.id },
          })
        );
      }
      break;

    case 'api-endpoint':
      if (!node.data.config?.endpointPath) {
        errors.push(
          new ValidationError({
            message: `API endpoint node "${node.data.label || node.id}" must have an endpoint path`,
            field: 'nodes',
            recoverySuggestions: [
              'Specify the API endpoint path (e.g., /api/webhooks/start)',
            ],
            details: { nodeId: node.id },
          })
        );
      }

      if (!node.data.config?.method) {
        warnings.push({
          message: `API endpoint node "${node.data.label || node.id}" has no HTTP method`,
          nodeId: node.id,
          severity: 'medium',
          suggestion: 'Specify the HTTP method (GET, POST, etc.)',
        });
      }
      break;

    case 'condition':
      if (!node.data.config?.condition && !node.data.config?.expression) {
        errors.push(
          new ValidationError({
            message: `Condition node "${node.data.label || node.id}" must have a condition expression`,
            field: 'nodes',
            recoverySuggestions: [
              'Add a condition expression',
              'Example: input.status === "approved"',
            ],
            details: { nodeId: node.id },
          })
        );
      }
      break;

    case 'loop':
      if (!node.data.config?.items && !node.data.config?.iterator) {
        errors.push(
          new ValidationError({
            message: `Loop node "${node.data.label || node.id}" must specify items to iterate`,
            field: 'nodes',
            recoverySuggestions: [
              'Specify the array to iterate over',
              'Example: input.items or state.records',
            ],
            details: { nodeId: node.id },
          })
        );
      }
      break;
  }

  // Validate retry policy if present
  if (node.data.retryPolicy) {
    validateRetryPolicy(node.data.retryPolicy, node, errors, warnings);
  }
}

/**
 * Validate edges with specific error messages
 */
function validateEdges(
  edges: WorkflowEdge[],
  nodes: WorkflowNode[],
  errors: ValidationError[],
  warnings: ValidationWarning[]
): void {
  const nodeIds = new Set(nodes.map((n) => n.id));
  const edgeIds = new Set<string>();
  const connectionMap = new Map<string, number>();

  for (const edge of edges) {
    // Check for duplicate edge IDs
    if (edgeIds.has(edge.id)) {
      errors.push(
        new ValidationError({
          message: `Duplicate edge ID: ${edge.id}`,
          field: 'edges',
          invalidValue: edge.id,
          recoverySuggestions: ['Each edge must have a unique ID'],
        })
      );
    }
    edgeIds.add(edge.id);

    // Validate source node exists
    if (!nodeIds.has(edge.source)) {
      errors.push(
        new ValidationError({
          message: `Edge ${edge.id} references non-existent source node: ${edge.source}`,
          field: 'edges',
          recoverySuggestions: [
            'Remove the invalid edge',
            'Ensure all referenced nodes exist',
          ],
          details: { edgeId: edge.id, sourceNode: edge.source },
        })
      );
    }

    // Validate target node exists
    if (!nodeIds.has(edge.target)) {
      errors.push(
        new ValidationError({
          message: `Edge ${edge.id} references non-existent target node: ${edge.target}`,
          field: 'edges',
          recoverySuggestions: [
            'Remove the invalid edge',
            'Ensure all referenced nodes exist',
          ],
          details: { edgeId: edge.id, targetNode: edge.target },
        })
      );
    }

    // Check for self-loops
    if (edge.source === edge.target) {
      errors.push(
        new ValidationError({
          message: `Edge ${edge.id} creates a self-loop on node ${edge.source}`,
          field: 'edges',
          recoverySuggestions: [
            'Remove the self-loop',
            'Use a different target node',
          ],
          details: { edgeId: edge.id, nodeId: edge.source },
        })
      );
    }

    // Track connections per node
    const sourceKey = `${edge.source}-out`;
    const targetKey = `${edge.target}-in`;
    connectionMap.set(sourceKey, (connectionMap.get(sourceKey) || 0) + 1);
    connectionMap.set(targetKey, (connectionMap.get(targetKey) || 0) + 1);
  }

  // Warn about nodes with many connections
  for (const [key, count] of connectionMap.entries()) {
    if (count > 5) {
      const [nodeId, direction] = key.split('-');
      warnings.push({
        message: `Node ${nodeId} has ${count} ${direction === 'in' ? 'incoming' : 'outgoing'} connections`,
        nodeId,
        severity: 'low',
        suggestion: 'Consider simplifying the workflow structure',
      });
    }
  }
}

/**
 * Validate workflow flow (start node, orphans, cycles)
 */
function validateWorkflowFlow(
  nodes: WorkflowNode[],
  edges: WorkflowEdge[]
): {
  errors: ValidationError[];
  warnings: ValidationWarning[];
  hasStartNode: boolean;
  hasEndNode: boolean;
  orphanedNodes: string[];
} {
  const errors: ValidationError[] = [];
  const warnings: ValidationWarning[] = [];

  // Find start nodes
  const nodesWithIncoming = new Set(edges.map((e) => e.target));
  const startNodes = nodes.filter(
    (node) =>
      (node.type === 'trigger' || node.type === 'api-endpoint') &&
      !nodesWithIncoming.has(node.id)
  );

  if (startNodes.length === 0) {
    errors.push(
      new ValidationError({
        message: 'Workflow must have a trigger or API endpoint node with no incoming connections',
        field: 'nodes',
        recoverySuggestions: [
          'Add a Trigger node or API Endpoint node',
          'Ensure the start node has no incoming connections',
        ],
      })
    );
  } else if (startNodes.length > 1) {
    warnings.push({
      message: `Workflow has ${startNodes.length} potential start nodes: ${startNodes.map((n) => n.data.label || n.id).join(', ')}`,
      severity: 'medium',
      suggestion: 'Consider consolidating to a single entry point',
    });
  }

  // Find end nodes
  const nodesWithOutgoing = new Set(edges.map((e) => e.source));
  const endNodes = nodes.filter(
    (node) => node.type === 'end' || !nodesWithOutgoing.has(node.id)
  );

  if (endNodes.length === 0 && nodes.length > 0) {
    warnings.push({
      message: 'Workflow has no end nodes',
      severity: 'low',
      suggestion: 'Add an End node to mark workflow completion',
    });
  }

  // Find orphaned nodes (no connections)
  const connectedNodes = new Set<string>();
  for (const edge of edges) {
    connectedNodes.add(edge.source);
    connectedNodes.add(edge.target);
  }

  const orphanedNodes = nodes
    .filter(
      (node) =>
        node.type !== 'trigger' &&
        node.type !== 'api-endpoint' &&
        node.type !== 'end' &&
        !connectedNodes.has(node.id)
    )
    .map((n) => n.id);

  if (orphanedNodes.length > 0) {
    const nodeLabels = nodes
      .filter((n) => orphanedNodes.includes(n.id))
      .map((n) => (n.data?.label) || n.id)
      .join(', ');

    warnings.push({
      message: `Found ${orphanedNodes.length} disconnected node(s): ${nodeLabels}`,
      severity: 'high',
      suggestion: 'Connect these nodes or remove them from the workflow',
    });
  }

  // Check for circular dependencies
  const cycles = detectCycles(nodes, edges);
  if (cycles.length > 0) {
    const cycleStr = cycles.map((n) => n.data.label || n.id).join(' → ');
    warnings.push({
      message: `Detected circular dependency: ${cycleStr}`,
      severity: 'high',
      suggestion: 'Break the cycle or use loop nodes for intentional iteration',
    });
  }

  return {
    errors,
    warnings,
    hasStartNode: startNodes.length > 0,
    hasEndNode: endNodes.length > 0,
    orphanedNodes,
  };
}

/**
 * Validate workflow variables
 */
function validateVariables(
  variables: any[],
  nodes: WorkflowNode[],
  errors: ValidationError[],
  warnings: ValidationWarning[]
): void {
  const variableNames = new Set<string>();

  for (const variable of variables) {
    if (!variable.name) {
      errors.push(
        new ValidationError({
          message: 'Variable must have a name',
          field: 'variables',
          recoverySuggestions: ['Add a unique variable name'],
        })
      );
      continue;
    }

    if (variableNames.has(variable.name)) {
      errors.push(
        new ValidationError({
          message: `Duplicate variable name: ${variable.name}`,
          field: 'variables',
          invalidValue: variable.name,
          recoverySuggestions: ['Use unique variable names'],
        })
      );
    }
    variableNames.add(variable.name);

    if (!variable.type) {
      errors.push(
        new ValidationError({
          message: `Variable ${variable.name} must have a type`,
          field: 'variables',
          recoverySuggestions: [
            'Specify a type: string, number, boolean, array, or object',
          ],
          details: { variableName: variable.name },
        })
      );
    }
  }

  // Check if variables are used
  const stateNodes = nodes.filter((n) => n.type === 'state-variable');
  const usedVariables = new Set(
    stateNodes.map((n) => n.data.config?.name).filter(Boolean)
  );

  for (const variable of variables) {
    if (!usedVariables.has(variable.name)) {
      warnings.push({
        message: `Variable "${variable.name}" is declared but never used`,
        severity: 'low',
        suggestion: 'Remove unused variables or add state variable nodes',
      });
    }
  }
}

/**
 * Validate workflow settings
 */
function validateSettings(
  settings: any,
  errors: ValidationError[],
  warnings: ValidationWarning[]
): void {
  if (settings.timeout) {
    const timeout = parseTimeout(settings.timeout);
    if (timeout === null) {
      errors.push(
        new ValidationError({
          message: `Invalid timeout format: ${settings.timeout}`,
          field: 'settings.timeout',
          invalidValue: settings.timeout,
          recoverySuggestions: [
            'Use format: "30s", "5m", "1h"',
            'Timeout must be a positive duration',
          ],
        })
      );
    } else if (timeout > 24 * 60 * 60 * 1000) {
      warnings.push({
        message: `Workflow timeout is very long: ${settings.timeout}`,
        severity: 'medium',
        suggestion: 'Consider reducing timeout or splitting into smaller workflows',
      });
    }
  }

  if (settings.retryPolicy) {
    validateRetryPolicy(settings.retryPolicy, null, errors, warnings);
  }
}

/**
 * Validate retry policy configuration
 */
function validateRetryPolicy(
  policy: any,
  node: WorkflowNode | null,
  errors: ValidationError[],
  warnings: ValidationWarning[]
): void {
  const context = node ? `node "${node.data.label || node.id}"` : 'workflow';

  if (!policy.strategy) {
    errors.push(
      new ValidationError({
        message: `Retry policy for ${context} must have a strategy`,
        field: 'retryPolicy',
        recoverySuggestions: [
          'Choose a strategy: keep-trying, fail-after-x, exponential-backoff, or none',
        ],
        details: { nodeId: node?.id },
      })
    );
    return;
  }

  const validStrategies = ['keep-trying', 'fail-after-x', 'exponential-backoff', 'none'];
  if (!validStrategies.includes(policy.strategy)) {
    errors.push(
      new ValidationError({
        message: `Invalid retry strategy for ${context}: ${policy.strategy}`,
        field: 'retryPolicy.strategy',
        invalidValue: policy.strategy,
        recoverySuggestions: [
          `Use one of: ${validStrategies.join(', ')}`,
        ],
        details: { nodeId: node?.id },
      })
    );
  }

  if (policy.strategy === 'fail-after-x' && !policy.maxAttempts) {
    errors.push(
      new ValidationError({
        message: `Retry policy for ${context} with "fail-after-x" must specify maxAttempts`,
        field: 'retryPolicy.maxAttempts',
        recoverySuggestions: ['Set maxAttempts to a positive number'],
        details: { nodeId: node?.id },
      })
    );
  }

  if (policy.maxAttempts !== undefined && policy.maxAttempts < 1) {
    errors.push(
      new ValidationError({
        message: `maxAttempts for ${context} must be at least 1`,
        field: 'retryPolicy.maxAttempts',
        invalidValue: policy.maxAttempts,
        recoverySuggestions: ['Set maxAttempts to 1 or higher'],
        details: { nodeId: node?.id },
      })
    );
  }

  if (policy.maxAttempts && policy.maxAttempts > 100) {
    warnings.push({
      message: `Retry policy for ${context} has very high maxAttempts: ${policy.maxAttempts}`,
      nodeId: node?.id,
      severity: 'medium',
      suggestion: 'Consider reducing maxAttempts or using keep-trying strategy',
    });
  }

  if (policy.backoffCoefficient !== undefined && policy.backoffCoefficient < 1) {
    errors.push(
      new ValidationError({
        message: `backoffCoefficient for ${context} must be at least 1`,
        field: 'retryPolicy.backoffCoefficient',
        invalidValue: policy.backoffCoefficient,
        recoverySuggestions: ['Set backoffCoefficient to 1.0 or higher'],
        details: { nodeId: node?.id },
      })
    );
  }
}

/**
 * Detect circular dependencies using DFS
 */
function detectCycles(nodes: WorkflowNode[], edges: WorkflowEdge[]): WorkflowNode[] {
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
        return path.map((id) => nodes.find((n) => n.id === id)!).filter(Boolean);
      }
    }
  }

  return [];
}

/**
 * Parse timeout string to milliseconds
 */
function parseTimeout(timeout: string): number | null {
  const match = timeout.match(/^(\d+)(s|m|h)$/);
  if (!match) return null;

  const value = parseInt(match[1]);
  const unit = match[2];

  const multipliers: Record<string, number> = {
    s: 1000,
    m: 60 * 1000,
    h: 60 * 60 * 1000,
  };

  return value * multipliers[unit];
}
