/**
 * State Management Pattern
 * Generates state variable declarations and operations
 * Supports different storage types: workflow (in-memory), database, redis
 */

import type { Pattern, WorkflowNode, GeneratorContext, CodeBlock } from '../types';
import { indent, toCamelCase, formatValue } from '../utils/ast-helpers';

/**
 * State Management Pattern
 * Handles state-variable nodes with support for different storage types
 */
export const StateManagementPattern: Pattern = {
  name: 'state-management',
  priority: 90,

  detect: (node: WorkflowNode, context: GeneratorContext): boolean => {
    return node.type === 'state-variable';
  },

  generate: (node: WorkflowNode, context: GeneratorContext): CodeBlock => {
    const config = node.data.config || {};
    const varName = config.name || toCamelCase(node.id);
    const operation = config.operation || 'set';
    const scope = config.scope || 'workflow'; // 'workflow' or 'project'
    const storageType = config.storageType || 'workflow'; // 'workflow', 'database', 'redis'
    const variableId = config.variableId || node.id; // ID of the state variable record

    const indentLevel = context.currentIndent;
    const indentStr = indent(indentLevel);

    const code: string[] = [];
    const declarations: string[] = [];
    const imports: string[] = [];

    // For workflow storage (in-memory), use simple variable declarations
    if (storageType === 'workflow') {
      // Generate declaration for first occurrence
      const stateVarNodes = context.nodes.filter(
        n => n.type === 'state-variable' && 
             (n.data.config?.name === varName || toCamelCase(n.id) === varName) &&
             (n.data.config?.storageType || 'workflow') === 'workflow'
      );
      const isFirstOccurrence = stateVarNodes[0]?.id === node.id;

      if (isFirstOccurrence) {
        const initialValue = config.initialValue !== undefined
          ? formatValue(config.initialValue)
          : getDefaultInitialValue(operation);

        declarations.push(`let ${varName} = ${initialValue};`);
      }

      // Generate operation
      code.push(`${indentStr}// State variable: ${varName} (workflow scope, in-memory)`);

      switch (operation) {
        case 'set':
          const value = config.value !== undefined ? formatValue(config.value) : 'null';
          code.push(`${indentStr}${varName} = ${value};`);
          break;

        case 'append':
          const appendValue = config.value !== undefined ? formatValue(config.value) : 'null';
          code.push(`${indentStr}${varName}.push(${appendValue});`);
          break;

        case 'increment':
          code.push(`${indentStr}${varName}++;`);
          break;

        case 'decrement':
          code.push(`${indentStr}${varName}--;`);
          break;

        case 'get':
          code.push(`${indentStr}// Current value: ${varName}`);
          break;

        default:
          code.push(`${indentStr}// Unknown operation: ${operation}`);
      }
    } else {
      // For database/redis storage, use activity calls
      imports.push(`import { proxyActivities } from '@temporalio/workflow';`);
      
      // Declare state activities proxy if not already declared
      const stateProxyVar = 'stateActivities';
      const hasStateProxy = declarations.some(d => d.includes(stateProxyVar));
      
      if (!hasStateProxy) {
        declarations.push(
          `const ${stateProxyVar} = proxyActivities<typeof import('./activities').stateActivities>({`,
          `  startToCloseTimeout: '30s',`,
          `});`
        );
      }

      code.push(`${indentStr}// State variable: ${varName} (${scope} scope, ${storageType} storage)`);

      const storageConfig = config.storageConfig || {};
      const activityInput = JSON.stringify({
        variableId,
        variableName: varName,
        scope,
        storageType,
        storageConfig,
      });

      switch (operation) {
        case 'set':
          const value = config.value !== undefined ? formatValue(config.value) : 'null';
          code.push(
            `${indentStr}await ${stateProxyVar}.setStateVariable({`,
            `${indentStr}  ...${activityInput.replace(/"/g, '')},`,
            `${indentStr}  value: ${value},`,
            `${indentStr}});`
          );
          break;

        case 'append':
          const appendValue = config.value !== undefined ? formatValue(config.value) : 'null';
          code.push(
            `${indentStr}await ${stateProxyVar}.appendStateVariable({`,
            `${indentStr}  ...${activityInput.replace(/"/g, '')},`,
            `${indentStr}  value: ${appendValue},`,
            `${indentStr}});`
          );
          break;

        case 'increment':
          const amount = config.amount !== undefined ? config.amount : 1;
          code.push(
            `${indentStr}const ${varName} = await ${stateProxyVar}.incrementStateVariable({`,
            `${indentStr}  ...${activityInput.replace(/"/g, '')},`,
            `${indentStr}  amount: ${amount},`,
            `${indentStr}});`
          );
          break;

        case 'decrement':
          const decAmount = config.amount !== undefined ? config.amount : 1;
          code.push(
            `${indentStr}const ${varName} = await ${stateProxyVar}.decrementStateVariable({`,
            `${indentStr}  ...${activityInput.replace(/"/g, '')},`,
            `${indentStr}  amount: ${decAmount},`,
            `${indentStr}});`
          );
          break;

        case 'get':
          code.push(
            `${indentStr}const ${varName} = await ${stateProxyVar}.getStateVariable(${activityInput});`
          );
          break;

        default:
          code.push(`${indentStr}// Unknown operation: ${operation}`);
      }
    }

    return {
      code: code.join('\n'),
      declarations,
      imports,
    };
  },
};

/**
 * Get default initial value based on operation type
 */
function getDefaultInitialValue(operation?: string): string {
  switch (operation) {
    case 'increment':
    case 'decrement':
      return '0';
    case 'append':
      return '[]';
    case 'set':
    default:
      return 'null';
  }
}

export default StateManagementPattern;
