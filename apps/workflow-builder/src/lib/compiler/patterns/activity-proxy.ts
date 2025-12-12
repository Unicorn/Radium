/**
 * Activity Proxy Pattern
 * Generates proxyActivities() code for activity and agent nodes
 */

import type { Pattern, WorkflowNode, GeneratorContext, CodeBlock } from '../types';
import { indent, toCamelCase } from '../utils/ast-helpers';

/**
 * Activity Proxy Pattern
 * Detects activity/agent nodes and generates proxyActivities code
 */
export const ActivityProxyPattern: Pattern = {
  name: 'activity-proxy',
  priority: 100, // High priority - needs to run early

  detect: (node: WorkflowNode, context: GeneratorContext): boolean => {
    return node.type === 'activity' || node.type === 'agent';
  },

  generate: (node: WorkflowNode, context: GeneratorContext): CodeBlock => {
    const activityName = node.data.activityName || node.data.componentName || toCamelCase(node.id);
    const nodeTimeout = node.data.timeout || '5 minutes';
    const retryPolicy = node.data.retryPolicy;
    const config = node.data.config || {};

    // Generate result variable name
    const resultVar = `result_${node.id.replace(/[^a-zA-Z0-9]/g, '_')}`;
    context.resultVars.set(node.id, resultVar);

    // Build activity call
    const indentLevel = context.currentIndent;
    const indentStr = indent(indentLevel);

    // Check if this is the first activity - if so, generate all proxy setups
    const activityNodes = context.nodes.filter(n => n.type === 'activity' || n.type === 'agent');
    const isFirstActivity = activityNodes[0]?.id === node.id;

    const code: string[] = [];
    const imports: string[] = [];
    const declarations: string[] = [];

    // Generate all proxy setups for first activity
    // Temporal requires separate proxyActivities calls for different timeout/retry configurations
    if (isFirstActivity) {
      imports.push(`import { proxyActivities } from '@temporalio/workflow';`);
      imports.push(`import type * as activities from './activities';`);

      // Group activities by timeout and retry policy
      const activityGroups = groupActivitiesByConfig(activityNodes);
      
      // Generate a proxy for each group
      for (const [groupKey, group] of activityGroups.entries()) {
        const groupActivities = group.map(n =>
          n.data.activityName || n.data.componentName || toCamelCase(n.id)
        );
        
        const timeout = group[0].data.timeout || '5 minutes';
        const retry = group[0].data.retryPolicy;
        
        // Generate proxy variable name based on timeout
        const proxyVarName = generateProxyVarName(timeout, retry);
        
        // Store proxy mapping in context for later use
        if (!context.proxyMap) {
          context.proxyMap = new Map();
        }
        group.forEach(n => {
          const key = `${n.data.timeout || 'default'}_${JSON.stringify(retry || {})}`;
          context.proxyMap!.set(n.id, proxyVarName);
        });

        // Build retry config
        let retryConfig = '';
        if (retry && retry.strategy !== 'none') {
          const retryOptions = buildRetryOptions(retry, '    ');
          retryConfig = `,\n  retry: ${retryOptions}`;
        } else {
          // Default retry policy - but for timeout tests, we might want no retries
          // However, Temporal best practice is to have some retries, so keep default
          retryConfig = `,\n  retry: {\n    initialInterval: '1s',\n    backoffCoefficient: 2,\n    maximumAttempts: 3,\n  }`;
        }

        // Don't destructure - keep the proxy object so we can use it with dot notation
        declarations.push(
          `// Activity proxies (timeout: ${timeout})\nconst ${proxyVarName} = proxyActivities<typeof activities>({\n` +
          `  startToCloseTimeout: '${timeout}'${retryConfig}\n` +
          `});`
        );
      }
    }

    // Generate activity invocation
    code.push(`${indentStr}// Execute ${node.type}: ${node.data.label || node.id}`);

    // Build input parameter
    const inputParam = buildInputParameter(node, context);

    // Get the proxy variable name for this activity
    const proxyVarName = context.proxyMap?.get(node.id);
    
    // Use the proxy object directly (not destructured) to access the activity
    // Format: proxyVarName.activityName(input)
    if (proxyVarName) {
      code.push(`${indentStr}const ${resultVar} = await ${proxyVarName}.${activityName}(${inputParam});`);
    } else {
      // Fallback to default (shouldn't happen if grouping worked)
      code.push(`${indentStr}const ${resultVar} = await ${activityName}(${inputParam});`);
    }

    if (node.type === 'agent') {
      code.push(`${indentStr}// Agent response stored in ${resultVar}`);
    }

    return {
      code: code.join('\n'),
      imports,
      declarations,
      resultVar,
    };
  },

  dependencies: ['@temporalio/workflow', '@temporalio/activity'],
};

/**
 * Build input parameter for activity call
 * Exported for unit testing of input-mapping behavior.
 */
export function buildInputParameter(node: WorkflowNode, context: GeneratorContext): string {
  const config = node.data.config || {};

  // Check if there's explicit input mapping
  if (config.inputMapping && typeof config.inputMapping === 'object') {
    const mappings = Object.entries(config.inputMapping).map(([key, value]) => {
      // Check if value references a previous result
      const referencedVar = context.resultVars.get(value as string);
      if (referencedVar) {
        return `${key}: ${referencedVar}`;
      }
      return `${key}: ${JSON.stringify(value)}`;
    });

    return `{ ${mappings.join(', ')} }`;
  }

  // Check if there's a previous result to use
  const incomingEdges = context.edges.filter(e => e.target === node.id);
  if (incomingEdges.length > 0) {
    const sourceNodeId = incomingEdges[0].source;
    const sourceResult = context.resultVars.get(sourceNodeId);

    if (sourceResult) {
      return sourceResult;
    }
  }

  // Default to 'input'
  return 'input';
}

/**
 * Build retry options object for activity call
 * Exported for unit testing of retry configuration generation.
 */
export function buildRetryOptions(
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
  lines.push('{');

  switch (retryPolicy.strategy) {
    case 'keep-trying':
      lines.push(`${indentStr}maximumAttempts: Infinity,`);
      if (retryPolicy.initialInterval) {
        lines.push(`${indentStr}initialInterval: '${retryPolicy.initialInterval}',`);
      } else {
        lines.push(`${indentStr}initialInterval: '1s',`);
      }
      if (retryPolicy.maxInterval) {
        lines.push(`${indentStr}maximumInterval: '${retryPolicy.maxInterval}',`);
      } else {
        lines.push(`${indentStr}maximumInterval: '1h',`);
      }
      if (retryPolicy.backoffCoefficient) {
        lines.push(`${indentStr}backoffCoefficient: ${retryPolicy.backoffCoefficient},`);
      } else {
        lines.push(`${indentStr}backoffCoefficient: 2.0,`);
      }
      break;

    case 'fail-after-x':
      if (retryPolicy.maxAttempts) {
        lines.push(`${indentStr}maximumAttempts: ${retryPolicy.maxAttempts},`);
      } else {
        lines.push(`${indentStr}maximumAttempts: 3,`);
      }
      if (retryPolicy.initialInterval) {
        lines.push(`${indentStr}initialInterval: '${retryPolicy.initialInterval}',`);
      } else {
        lines.push(`${indentStr}initialInterval: '1s',`);
      }
      if (retryPolicy.maxInterval) {
        lines.push(`${indentStr}maximumInterval: '${retryPolicy.maxInterval}',`);
      }
      if (retryPolicy.backoffCoefficient) {
        lines.push(`${indentStr}backoffCoefficient: ${retryPolicy.backoffCoefficient},`);
      }
      break;

    case 'exponential-backoff':
      if (retryPolicy.maxAttempts) {
        lines.push(`${indentStr}maximumAttempts: ${retryPolicy.maxAttempts},`);
      } else {
        lines.push(`${indentStr}maximumAttempts: 5,`);
      }
      if (retryPolicy.initialInterval) {
        lines.push(`${indentStr}initialInterval: '${retryPolicy.initialInterval}',`);
      } else {
        lines.push(`${indentStr}initialInterval: '1s',`);
      }
      if (retryPolicy.maxInterval) {
        lines.push(`${indentStr}maximumInterval: '${retryPolicy.maxInterval}',`);
      } else {
        lines.push(`${indentStr}maximumInterval: '1h',`);
      }
      if (retryPolicy.backoffCoefficient) {
        lines.push(`${indentStr}backoffCoefficient: ${retryPolicy.backoffCoefficient},`);
      } else {
        lines.push(`${indentStr}backoffCoefficient: 2.0,`);
      }
      break;

    case 'none':
    default:
      lines.push(`${indentStr}maximumAttempts: 1,`);
      break;
  }

  lines.push(`${indentStr}}`);
  return lines.join('\n');
}

/**
 * Group activities by their timeout and retry configuration
 * Activities with the same config will share a proxy.
 * Exported for unit testing of grouping behavior.
 */
export function groupActivitiesByConfig(activityNodes: WorkflowNode[]): Map<string, WorkflowNode[]> {
  const groups = new Map<string, WorkflowNode[]>();

  for (const node of activityNodes) {
    const timeout = node.data.timeout || '5 minutes';
    const retry = node.data.retryPolicy || {};
    
    // Create a stable key for grouping - normalize retry policy
    const retryKey = retry.strategy ? JSON.stringify({
      strategy: retry.strategy,
      maxAttempts: retry.maxAttempts,
      initialInterval: retry.initialInterval,
      maxInterval: retry.maxInterval,
      backoffCoefficient: retry.backoffCoefficient,
    }) : '{}';
    
    const groupKey = `${timeout}_${retryKey}`;

    if (!groups.has(groupKey)) {
      groups.set(groupKey, []);
    }
    groups.get(groupKey)!.push(node);
  }

  return groups;
}

/**
 * Generate a proxy variable name based on timeout and retry config
 * Must be a valid JavaScript identifier.
 * Exported for unit testing of naming behavior.
 */
export function generateProxyVarName(timeout: string, retry?: any): string {
  // Sanitize timeout: "2s" -> "2s", "5 minutes" -> "5minutes", "30s" -> "30s"
  const timeoutPart = timeout.replace(/[^a-zA-Z0-9]/g, '');
  
  // Sanitize retry strategy: "exponential-backoff" -> "ExponentialBackoff"
  let retryPart = '';
  if (retry && retry.strategy && retry.strategy !== 'none') {
    const strategy = retry.strategy
      .split(/[-_]/)
      .map(word => word.charAt(0).toUpperCase() + word.slice(1))
      .join('');
    retryPart = `Retry${strategy}`;
  }
  
  // Generate valid identifier: activities2s, activities5minutesRetryExponentialBackoff, etc.
  const baseName = `activities${timeoutPart}${retryPart}`;
  
  // Ensure it starts with a letter (not a number)
  if (/^[0-9]/.test(baseName)) {
    return `activitiesTimeout${timeoutPart}${retryPart}`;
  }
  
  return baseName;
}

export default ActivityProxyPattern;
