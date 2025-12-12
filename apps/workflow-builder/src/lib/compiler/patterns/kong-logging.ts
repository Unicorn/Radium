/**
 * Kong Logging Pattern
 * Generates Kong logging plugin configuration for project-level logging
 */

import type { Pattern, WorkflowNode, GeneratorContext, CodeBlock } from '../types';
import { indent } from '../utils/ast-helpers';

/**
 * Kong Logging Pattern
 * Detects kong-logging nodes and generates logging configuration metadata
 */
export const KongLoggingPattern: Pattern = {
  name: 'kong-logging',
  priority: 30, // Lower priority - configuration, not execution code

  detect: (node: WorkflowNode, context: GeneratorContext): boolean => {
    return node.type === 'kong-logging';
  },

  generate: (node: WorkflowNode, context: GeneratorContext): CodeBlock => {
    const indentLevel = context.currentIndent;
    const indentStr = indent(indentLevel);
    const config = node.data.config || {};
    const code: string[] = [];
    const imports: string[] = [];

    const componentName = node.data.label || node.data.componentName || 'Kong Logging';
    const connectorName = config.connectorName || config.connectorId || 'Not configured';
    const enabledEndpoints = Array.isArray(config.enabledEndpoints) 
      ? config.enabledEndpoints.length 
      : 0;

    // Kong logging is project-level configuration
    // It doesn't generate workflow code, but documents the logging setup
    code.push(`${indentStr}// Kong Logging Configuration: ${componentName}`);
    code.push(`${indentStr}// Project-level logging component`);
    code.push(`${indentStr}// Connector: ${connectorName}`);
    code.push(`${indentStr}// Enabled endpoints: ${enabledEndpoints}`);
    code.push(`${indentStr}// Logging plugin will be configured on Kong routes during deployment`);
    code.push(`${indentStr}// This component configures logging for all data-in/data-out endpoints in the project`);

    // Generate result variable (even though it's not used in workflow execution)
    const resultVar = `result_${node.id.replace(/[^a-zA-Z0-9]/g, '_')}`;
    context.resultVars.set(node.id, resultVar);
    code.push(`${indentStr}const ${resultVar} = { type: 'kong-logging-config', connector: '${connectorName}' };`);

    return {
      code: code.join('\n'),
      imports,
      resultVar,
    };
  },
};

export default KongLoggingPattern;

