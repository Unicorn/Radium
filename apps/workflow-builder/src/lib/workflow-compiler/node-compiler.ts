/**
 * Node-based Workflow Compiler
 * Generates Temporal TypeScript code from React Flow node/edge definitions
 */

import type { WorkflowDefinition, WorkflowNode, WorkflowEdge } from '@/types/workflow';

type NodeConfig = Record<string, unknown>;

export interface CompileOptions {
  includeComments?: boolean;
  workflowName?: string;
}

/**
 * Compile a workflow definition (nodes + edges) into Temporal TypeScript code
 */
export function compileWorkflowFromNodes(
  definition: WorkflowDefinition,
  options: CompileOptions = {}
): string {
  const { includeComments = true, workflowName = 'Workflow' } = options;

  // Build node graph
  const nodeMap = new Map<string, WorkflowNode>();
  const edges = definition.edges || [];
  
  (definition.nodes || []).forEach(node => {
    nodeMap.set(node.id, node);
  });

  // Find start node (trigger node with no incoming edges)
  const startNode = findStartNode(definition.nodes || [], edges);
  if (!startNode) {
    throw new Error('No start node found in workflow');
  }

  // Generate code
  const imports = generateImports(definition);
  const stateVariables = generateStateVariableDeclarations(definition);
  const workflowCode = generateWorkflowCode(startNode, nodeMap, edges, includeComments);

  // Use workflowName directly - Temporal expects function name to match workflow type name
  // The workflowName should be the exact name from workflow.name (e.g., "TestSimpleWorkflow")
  const functionName = workflowName;

  return `${imports}

${includeComments ? `/**
 * ${workflowName}
 * Generated Temporal workflow from node-based definition
 */` : ''}
export async function ${functionName}(input: Record<string, unknown> | undefined): Promise<unknown> {
${stateVariables}

${workflowCode}
}`;
}

/**
 * Generate imports based on node types used
 */
function generateImports(definition: WorkflowDefinition): string {
  const nodes = definition.nodes || [];
  const imports = new Set<string>();

  const workflowImports = new Set([
    'proxyActivities',
    'startChild',
    'executeChild',
    'sleep',
    'condition',
  ]);

  // Check for activities/agents
  const hasActivities = nodes.some(n => n.type === 'activity' || n.type === 'agent');
  if (hasActivities) {
    imports.add("import type * as activities from './activities';");
  }
  
  // Check for child workflows
  const hasChildWorkflows = nodes.some(n => n.type === 'child-workflow');
  if (hasChildWorkflows) {
    workflowImports.add('uuid4');
  }
  
  // Check for signals
  const hasSignals = nodes.some(n => n.type === 'signal');
  if (hasSignals) {
    imports.add("import { defineSignal, setHandler } from '@temporalio/workflow';");
  }

  imports.add(`import { ${Array.from(workflowImports).join(', ')} } from '@temporalio/workflow';`);

  return Array.from(imports).join('\n');
}

/**
 * Generate state variable declarations
 */
function generateStateVariableDeclarations(definition: WorkflowDefinition): string {
  const stateNodes = (definition.nodes || []).filter(n => n.type === 'state-variable');
  
  if (stateNodes.length === 0) {
    return '';
  }

  const declarations: string[] = [];
  
  stateNodes.forEach(node => {
    const config = node.data.config || {};
    const varName = config.name || toCamelCase(node.id);
    const initialValue = config.initialValue !== undefined 
      ? JSON.stringify(config.initialValue)
      : getDefaultInitialValue(config.operation);
    
    declarations.push(`  let ${varName} = ${initialValue};`);
  });

  return declarations.length > 0 ? '\n  // State variables\n' + declarations.join('\n') + '\n' : '';
}

function getDefaultInitialValue(operation?: string): string {
  switch (operation) {
    case 'increment':
    case 'decrement':
      return '0';
    case 'append':
      return '[]';
    default:
      return 'null';
  }
}

/**
 * Generate workflow execution code by traversing the graph
 */
function generateWorkflowCode(
  startNode: WorkflowNode,
  nodeMap: Map<string, WorkflowNode>,
  edges: WorkflowEdge[],
  includeComments: boolean
): string {
  const code: string[] = [];
  const visited = new Set<string>();
  const resultVars = new Map<string, string>(); // nodeId -> result variable name

  // Traverse graph starting from start node
  traverseNode(startNode, nodeMap, edges, code, visited, resultVars, includeComments, 1);

  return code.join('\n');
}

/**
 * Traverse a node and generate code for it
 */
function traverseNode(
  node: WorkflowNode,
  nodeMap: Map<string, WorkflowNode>,
  edges: WorkflowEdge[],
  code: string[],
  visited: Set<string>,
  resultVars: Map<string, string>,
  includeComments: boolean,
  indent: number
): void {
  if (visited.has(node.id)) {
    return; // Already processed
  }
  visited.add(node.id);

  const indentStr = '  '.repeat(indent);
  const resultVar = `result_${node.id.replace(/-/g, '_')}`;

  // Generate code for this node
  const nodeCode = generateNodeCode(node, resultVar, resultVars, includeComments, indent, edges);
  if (nodeCode) {
    code.push(nodeCode);
  }

  // Find outgoing edges
  const outgoingEdges = edges.filter(e => e.source === node.id);
  
  // Handle different node types
  if (node.type === 'condition') {
    // Condition node has two paths: true and false
    const trueEdge = outgoingEdges.find(e => e.sourceHandle === 'true');
    const falseEdge = outgoingEdges.find(e => e.sourceHandle === 'false');

    if (trueEdge) {
      const trueNode = nodeMap.get(trueEdge.target);
      if (trueNode) {
        code.push(`${indentStr}  // True branch`);
        traverseNode(trueNode, nodeMap, edges, code, visited, resultVars, includeComments, indent + 1);
      }
    }

    if (falseEdge) {
      const falseNode = nodeMap.get(falseEdge.target);
      if (falseNode) {
        code.push(`${indentStr}  // False branch`);
        traverseNode(falseNode, nodeMap, edges, code, visited, resultVars, includeComments, indent + 1);
      }
    }
  } else {
    // Regular sequential flow
    outgoingEdges.forEach(edge => {
      const nextNode = nodeMap.get(edge.target);
      if (nextNode) {
        traverseNode(nextNode, nodeMap, edges, code, visited, resultVars, includeComments, indent);
      }
    });
  }
}

/**
 * Generate code for a specific node
 */
function generateNodeCode(
  node: WorkflowNode,
  resultVar: string,
  resultVars: Map<string, string>,
  includeComments: boolean,
  indent: number,
  edges: WorkflowEdge[]
): string {
  const indentStr = '  '.repeat(indent);
  const config: NodeConfig = node.data.config || {};
  const comment = includeComments ? `// ${node.data.label || node.id}` : '';

  switch (node.type) {
    case 'trigger':
      if (config.cronSchedule) {
        return `${indentStr}${comment}
${indentStr}// Cron-scheduled workflow: ${config.cronSchedule}`;
      }
      return `${indentStr}${comment}
${indentStr}// Workflow started`;
    
    case 'activity':
    case 'agent': {
      const activityName = node.data.componentName || toCamelCase(node.id);
      resultVars.set(node.id, resultVar);
      
      // Build activity options
      const activityOptions: string[] = [];
      
      // Add timeout if specified
      const nodeTimeout = node.data.timeout;
      if (nodeTimeout) {
        activityOptions.push(`startToCloseTimeout: '${nodeTimeout}'`);
      }
      
      // Generate retry policy if configured
      const retryPolicy = node.data.retryPolicy;
      if (retryPolicy && retryPolicy.strategy !== 'none') {
        const retryPolicyCode = generateRetryPolicyCode(retryPolicy, indentStr);
        if (retryPolicyCode) {
          activityOptions.push(`retry: ${retryPolicyCode.replace(/^ {2}/gm, '    ')}`);
        }
      }
      
      // Build input parameter using upstream result when available
      const incoming = edges.filter(edge => edge.target === node.id);
      const mappedInputs = incoming
        .map(edge => resultVars.get(edge.source))
        .filter((value): value is string => Boolean(value));

      const inputParam =
        mappedInputs.length === 1
          ? mappedInputs[0]
          : mappedInputs.length > 1
            ? `{ ${mappedInputs.map((value, index) => `arg${index + 1}: ${value}`).join(', ')} }`
            : 'input';
      
      if (activityOptions.length > 0) {
        return `${indentStr}${comment}
${indentStr}const ${resultVar} = await ${activityName}(${inputParam}, {
${indentStr}  ${activityOptions.join(`,\n${indentStr}  `)},
${indentStr}});`;
      }
      
      return `${indentStr}${comment}
${indentStr}const ${resultVar} = await ${activityName}(${inputParam});`;
    }
    
    case 'child-workflow': {
      const workflowType = (config.workflowType as string) || node.data.componentName || 'ChildWorkflow';
      const childInput = config.inputMapping 
        ? generateInputMapping(config.inputMapping as Record<string, string>, resultVars)
        : 'input';
      const executionType = (config.executionType as string) || 'startChild';
      const taskQueue =
        typeof config.taskQueue === 'string' ? `taskQueue: '${config.taskQueue}',\n${indentStr}  ` : '';
      resultVars.set(node.id, resultVar);
      
      if (executionType === 'executeChild') {
        return `${indentStr}${comment}
${indentStr}const ${resultVar} = await executeChild(${workflowType}Workflow, {
${indentStr}  ${taskQueue}workflowId: '${node.id}-' + uuid4(),
${indentStr}  args: [${childInput}],
${indentStr}});`;
      } else {
        return `${indentStr}${comment}
${indentStr}const ${resultVar} = await startChild(${workflowType}Workflow, {
${indentStr}  ${taskQueue}workflowId: '${node.id}-' + uuid4(),
${indentStr}  args: [${childInput}],
${indentStr}});`;
      }
    }
    
    case 'condition': {
      const expression = (config.expression as string) || 'true';
      resultVars.set(node.id, resultVar);
      return `${indentStr}${comment}
${indentStr}const ${resultVar} = ${expression};`;
    }
    
    case 'phase':
      return generatePhaseCode(node, config, resultVars, includeComments, indent);
    
    case 'retry':
      return generateRetryCode(node, config, resultVars, includeComments, indent);
    
    case 'state-variable':
      return generateStateVariableCode(node, config, includeComments, indent);
    
    case 'signal': {
      const signalName = node.data.signalName || config.signalName || 'signal';
      return `${indentStr}${comment}
${indentStr}// Signal handler: ${signalName}
${indentStr}// Signal logic would be implemented here`;
    }
    
    case 'api-endpoint':
      return `${indentStr}${comment}
${indentStr}// API endpoint registered externally`;
    
    case 'kong-logging': {
      const componentName = node.data.label || node.data.componentName || 'Kong Logging';
      const connectorName = config.connectorName || config.connectorId || 'Not configured';
      const enabledEndpoints = Array.isArray(config.enabledEndpoints) 
        ? config.enabledEndpoints.length 
        : 0;
      resultVars.set(node.id, resultVar);
      return `${indentStr}${comment}
${indentStr}// Kong Logging Configuration: ${componentName}
${indentStr}// Project-level logging component
${indentStr}// Connector: ${connectorName}
${indentStr}// Enabled endpoints: ${enabledEndpoints}
${indentStr}// Logging plugin will be configured on Kong routes during deployment
${indentStr}const ${resultVar} = { type: 'kong-logging-config', connector: '${connectorName}' };`;
    }
    
    case 'kong-cache': {
      const componentName = node.data.label || node.data.componentName || 'Kong Cache';
      const cacheKey = config.cacheKey || 'not-configured';
      const connectorName = config.connectorName || config.connectorId || 'Not configured';
      const ttl = config.ttlSeconds || config.ttl || 3600;
      const isSaved = config.isSaved === true;
      resultVars.set(node.id, resultVar);
      let cacheCode = `${indentStr}${comment}
${indentStr}// Kong Cache Configuration: ${componentName}
${indentStr}// Cache Key: ${cacheKey}${isSaved ? ' (immutable)' : ' (editable until save)'}
${indentStr}// Redis Connector: ${connectorName}
${indentStr}// TTL: ${ttl} seconds
${indentStr}// Cache plugin will be configured on Kong routes during deployment`;
      if (config.markedForDeletion === true) {
        cacheCode += `\n${indentStr}// Cache key marked for deletion - will be removed from Redis on next deploy`;
      }
      cacheCode += `\n${indentStr}const ${resultVar} = { type: 'kong-cache-config', cacheKey: '${cacheKey}', ttl: ${ttl} };`;
      return cacheCode;
    }
    
    case 'kong-cors': {
      const componentName = node.data.label || node.data.componentName || 'Kong CORS';
      const allowedOrigins = Array.isArray(config.allowedOrigins) 
        ? config.allowedOrigins 
        : config.allowedOrigins 
          ? [config.allowedOrigins] 
          : ['*'];
      const allowedMethods = Array.isArray(config.allowedMethods) 
        ? config.allowedMethods 
        : config.allowedMethods 
          ? [config.allowedMethods] 
          : ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'OPTIONS'];
      const allowCredentials = config.allowCredentials === true;
      const maxAge = config.maxAge || 3600;
      resultVars.set(node.id, resultVar);
      return `${indentStr}${comment}
${indentStr}// Kong CORS Configuration: ${componentName}
${indentStr}// Allowed Origins: ${allowedOrigins.join(', ')}
${indentStr}// Allowed Methods: ${allowedMethods.join(', ')}
${indentStr}// Allow Credentials: ${allowCredentials}
${indentStr}// Max Age: ${maxAge} seconds
${indentStr}// CORS plugin will be configured on Kong routes during deployment
${indentStr}const ${resultVar} = { type: 'kong-cors-config', origins: ${JSON.stringify(allowedOrigins)} };`;
    }
    
    case 'graphql-gateway': {
      const componentName = node.data.label || node.data.componentName || 'GraphQL Gateway';
      const endpointPath = config.endpointPath || '/graphql';
      const queries = Array.isArray(config.queries) ? config.queries : [];
      const mutations = Array.isArray(config.mutations) ? config.mutations : [];
      resultVars.set(node.id, resultVar);
      return `${indentStr}${comment}
${indentStr}// GraphQL Gateway: ${componentName}
${indentStr}// Endpoint Path: ${endpointPath}
${indentStr}// Queries: ${queries.length}, Mutations: ${mutations.length}
${indentStr}// GraphQL endpoint will be available at: ${endpointPath}
${indentStr}// GraphQL handler will be generated during deployment
${indentStr}const ${resultVar} = { type: 'graphql-gateway', endpoint: '${endpointPath}', queries: ${queries.length}, mutations: ${mutations.length} };`;
    }
    
    case 'mcp-server': {
      const componentName = node.data.label || node.data.componentName || 'MCP Server';
      const serverName = config.serverName || config.name || 'mcp-server';
      const version = config.version || '1.0.0';
      const endpointPath = config.endpointPath || '/mcp';
      const resources = Array.isArray(config.resources) ? config.resources : [];
      const tools = Array.isArray(config.tools) ? config.tools : [];
      resultVars.set(node.id, resultVar);
      return `${indentStr}${comment}
${indentStr}// MCP Server: ${componentName}
${indentStr}// Server Name: ${serverName}
${indentStr}// Version: ${version}
${indentStr}// Endpoint Path: ${endpointPath}
${indentStr}// Resources: ${resources.length}, Tools: ${tools.length}
${indentStr}// MCP endpoint will be available at: ${endpointPath}
${indentStr}// MCP server handler will be generated during deployment
${indentStr}const ${resultVar} = { type: 'mcp-server', serverName: '${serverName}', version: '${version}', resources: ${resources.length}, tools: ${tools.length} };`;
    }
    
    default:
      return `${indentStr}${comment}
${indentStr}// Node type: ${node.type}`;
  }
}

/**
 * Generate code for phase node
 */
function generatePhaseCode(
  node: WorkflowNode,
  config: NodeConfig,
  resultVars: Map<string, string>,
  includeComments: boolean,
  indent: number
): string {
  const indentStr = '  '.repeat(indent);
  const phaseName = (config.name as string) || node.data.label || 'Phase';
  const isSequential = config.sequential !== false;
  const maxConcurrent = typeof config.maxConcurrency === 'number' ? config.maxConcurrency : 4;

  const code: string[] = [];
  code.push(`${indentStr}// Phase: ${phaseName}`);
  
  if (isSequential) {
    code.push(`${indentStr}// Sequential execution`);
  } else {
    code.push(`${indentStr}// Concurrent execution (max ${maxConcurrent})`);
    code.push(`${indentStr}const activeTasks = new Map();`);
    code.push(`${indentStr}const maxConcurrent = ${maxConcurrent};`);
  }

  return code.join('\n');
}

/**
 * Generate code for retry node
 */
function generateRetryCode(
  node: WorkflowNode,
  config: NodeConfig,
  resultVars: Map<string, string>,
  includeComments: boolean,
  indent: number
): string {
  const indentStr = '  '.repeat(indent);
  const maxAttempts = typeof config.maxAttempts === 'number' ? config.maxAttempts : 3;
  const retryOn = (config.retryOn as string) || 'failure';
  const backoff = (config.backoff as NodeConfig) || {};
  const backoffType = (backoff.type as string) || 'exponential';
  const initialInterval = (backoff.initialInterval as string) || '1s';
  const multiplier = typeof backoff.multiplier === 'number' ? backoff.multiplier : 2;

  const code: string[] = [];
  code.push(`${indentStr}// Retry loop: ${node.data.label || 'Retry'}`);
  code.push(`${indentStr}let attempts = 0;`);
  code.push(`${indentStr}const maxAttempts = ${maxAttempts};`);
  code.push(`${indentStr}let lastError: Error | null = null;`);
  code.push(``);
  code.push(`${indentStr}while (attempts < maxAttempts) {`);
  
  if (retryOn === 'condition') {
    const condition = config.condition || 'result.success === false';
    code.push(`${indentStr}  const result = await /* execute activity/block */;`);
    code.push(`${indentStr}  if (!(${condition})) {`);
    code.push(`${indentStr}    break; // Success`);
    code.push(`${indentStr}  }`);
  } else {
    code.push(`${indentStr}  try {`);
    code.push(`${indentStr}    const result = await /* execute activity/block */;`);
    code.push(`${indentStr}    if (result.success !== false) {`);
    code.push(`${indentStr}      break; // Success`);
    code.push(`${indentStr}    }`);
    code.push(`${indentStr}  } catch (error) {`);
    code.push(`${indentStr}    lastError = error as Error;`);
    code.push(`${indentStr}  }`);
  }

  code.push(``);
  code.push(`${indentStr}  attempts++;`);
  code.push(`${indentStr}  if (attempts >= maxAttempts) {`);
  code.push(`${indentStr}    throw lastError || new Error('Max retry attempts reached');`);
  code.push(`${indentStr}  }`);
  
  if (backoffType !== 'none') {
    if (backoffType === 'exponential') {
      code.push(`${indentStr}  // Exponential backoff: ${initialInterval} * ${multiplier}^(attempts-1)`);
      code.push(`${indentStr}  const backoffMs = Math.pow(${multiplier}, attempts - 1) * 1000;`);
    } else if (backoffType === 'linear') {
      code.push(`${indentStr}  // Linear backoff: ${initialInterval} * attempts`);
      code.push(`${indentStr}  const backoffMs = attempts * 1000;`);
    } else {
      code.push(`${indentStr}  // Fixed backoff: ${initialInterval}`);
      code.push(`${indentStr}  const backoffMs = 1000;`);
    }
    code.push(`${indentStr}  await sleep(backoffMs);`);
  }
  
  code.push(`${indentStr}}`);

  return code.join('\n');
}

/**
 * Generate code for state variable node
 */
function generateStateVariableCode(
  node: WorkflowNode,
  config: NodeConfig,
  includeComments: boolean,
  indent: number
): string {
  const indentStr = '  '.repeat(indent);
  const varName = (config.name as string) || toCamelCase(node.id);
  const operation = (config.operation as string) || 'set';
  const value = config.value;

  const code: string[] = [];
  code.push(`${indentStr}// State variable: ${varName}`);

  switch (operation) {
    case 'set':
      code.push(`${indentStr}${varName} = ${value !== undefined ? JSON.stringify(value) : 'null'};`);
      break;
    case 'append':
      code.push(`${indentStr}${varName}.push(${value !== undefined ? JSON.stringify(value) : 'null'});`);
      break;
    case 'increment':
      code.push(`${indentStr}${varName}++;`);
      break;
    case 'decrement':
      code.push(`${indentStr}${varName}--;`);
      break;
    case 'get':
      code.push(`${indentStr}// Get ${varName}: ${varName}`);
      break;
  }

  return code.join('\n');
}

/**
 * Generate input mapping for child workflow
 */
function generateInputMapping(mapping: Record<string, string>, resultVars: Map<string, string>): string {
  const entries = Object.entries(mapping).map(([key, value]) => {
    // Check if value references a previous result
    const resultVar = Array.from(resultVars.entries()).find(([nodeId]) => 
      value.includes(nodeId) || value === nodeId
    );
    
    if (resultVar) {
      return `${key}: ${resultVar[1]}`;
    }
    
    return `${key}: ${JSON.stringify(value)}`;
  });

  return `{ ${entries.join(', ')} }`;
}

/**
 * Find the start node (trigger with no incoming edges)
 */
function findStartNode(nodes: WorkflowNode[], edges: WorkflowEdge[]): WorkflowNode | null {
  // Find trigger nodes
  const triggerNodes = nodes.filter(n => n.type === 'trigger');
  
  // Find one with no incoming edges
  for (const node of triggerNodes) {
    const hasIncoming = edges.some(e => e.target === node.id);
    if (!hasIncoming) {
      return node;
    }
  }
  
  // Fallback: return first trigger node
  return triggerNodes[0] || nodes[0] || null;
}

/**
 * Generate retry policy code for Temporal activities
 */
function generateRetryPolicyCode(
  retryPolicy: {
    strategy: 'keep-trying' | 'fail-after-x' | 'exponential-backoff' | 'none';
    maxAttempts?: number;
    initialInterval?: string;
    maxInterval?: string;
    backoffCoefficient?: number;
  },
  indentStr: string
): string {
  const lines: string[] = [];
  
  lines.push(`${indentStr}  retry: {`);
  
  switch (retryPolicy.strategy) {
    case 'keep-trying':
      // Unlimited retries with exponential backoff (leave maximumAttempts undefined)
      if (retryPolicy.initialInterval) {
        lines.push(`${indentStr}    initialInterval: '${retryPolicy.initialInterval}',`);
      } else {
        lines.push(`${indentStr}    initialInterval: '1s',`);
      }
      if (retryPolicy.maxInterval) {
        lines.push(`${indentStr}    maximumInterval: '${retryPolicy.maxInterval}',`);
      } else {
        lines.push(`${indentStr}    maximumInterval: '1h',`);
      }
      if (retryPolicy.backoffCoefficient) {
        lines.push(`${indentStr}    backoffCoefficient: ${retryPolicy.backoffCoefficient},`);
      } else {
        lines.push(`${indentStr}    backoffCoefficient: 2.0,`);
      }
      break;
      
    case 'fail-after-x':
      // Retry up to maxAttempts times
      if (retryPolicy.maxAttempts) {
        lines.push(`${indentStr}    maximumAttempts: ${retryPolicy.maxAttempts},`);
      } else {
        lines.push(`${indentStr}    maximumAttempts: 3,`);
      }
      if (retryPolicy.initialInterval) {
        lines.push(`${indentStr}    initialInterval: '${retryPolicy.initialInterval}',`);
      } else {
        lines.push(`${indentStr}    initialInterval: '1s',`);
      }
      if (retryPolicy.maxInterval) {
        lines.push(`${indentStr}    maximumInterval: '${retryPolicy.maxInterval}',`);
      }
      if (retryPolicy.backoffCoefficient) {
        lines.push(`${indentStr}    backoffCoefficient: ${retryPolicy.backoffCoefficient},`);
      }
      break;
      
    case 'exponential-backoff':
      // Exponential backoff with max attempts
      if (retryPolicy.maxAttempts) {
        lines.push(`${indentStr}    maximumAttempts: ${retryPolicy.maxAttempts},`);
      } else {
        lines.push(`${indentStr}    maximumAttempts: 5,`);
      }
      if (retryPolicy.initialInterval) {
        lines.push(`${indentStr}    initialInterval: '${retryPolicy.initialInterval}',`);
      } else {
        lines.push(`${indentStr}    initialInterval: '1s',`);
      }
      if (retryPolicy.maxInterval) {
        lines.push(`${indentStr}    maximumInterval: '${retryPolicy.maxInterval}',`);
      } else {
        lines.push(`${indentStr}    maximumInterval: '1h',`);
      }
      if (retryPolicy.backoffCoefficient) {
        lines.push(`${indentStr}    backoffCoefficient: ${retryPolicy.backoffCoefficient},`);
      } else {
        lines.push(`${indentStr}    backoffCoefficient: 2.0,`);
      }
      break;
      
    case 'none':
    default:
      // No retries
      lines.push(`${indentStr}    maximumAttempts: 1,`);
      break;
  }
  
  lines.push(`${indentStr}  }`);
  
  return lines.join('\n');
}

/**
 * Helper: Convert string to camelCase
 */
function toCamelCase(str: string): string {
  return str
    .replace(/[^a-zA-Z0-9]+(.)/g, (_, chr) => chr.toUpperCase())
    .replace(/^[A-Z]/, chr => chr.toLowerCase());
}
