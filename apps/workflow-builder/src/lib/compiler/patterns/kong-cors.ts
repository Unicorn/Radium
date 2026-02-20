/**
 * Kong CORS Pattern
 * Generates Kong CORS plugin configuration
 */

import type { Pattern, WorkflowNode, GeneratorContext, CodeBlock } from '../types';
import { indent } from '../utils/ast-helpers';

/**
 * Kong CORS Pattern
 * Detects kong-cors nodes and generates CORS configuration metadata
 */
export const KongCorsPattern: Pattern = {
  name: 'kong-cors',
  priority: 30, // Lower priority - configuration, not execution code

  detect: (node: WorkflowNode, context: GeneratorContext): boolean => {
    return node.type === 'kong-cors';
  },

  generate: (node: WorkflowNode, context: GeneratorContext): CodeBlock => {
    const indentLevel = context.currentIndent;
    const indentStr = indent(indentLevel);
    const config = node.data.config || {};
    const code: string[] = [];
    const imports: string[] = [];

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
    const allowedHeaders = Array.isArray(config.allowedHeaders) 
      ? config.allowedHeaders 
      : config.allowedHeaders 
        ? [config.allowedHeaders] 
        : ['*'];
    const exposedHeaders = Array.isArray(config.exposedHeaders) 
      ? config.exposedHeaders 
      : [];
    const allowCredentials = config.allowCredentials === true;
    const maxAge = config.maxAge || 3600;
    const preflightContinue = config.preflightContinue === true;

    // Kong CORS configuration
    code.push(`${indentStr}// Kong CORS Configuration: ${componentName}`);
    code.push(`${indentStr}// Allowed Origins: ${allowedOrigins.join(', ')}`);
    code.push(`${indentStr}// Allowed Methods: ${allowedMethods.join(', ')}`);
    code.push(`${indentStr}// Allowed Headers: ${allowedHeaders.join(', ')}`);
    if (exposedHeaders.length > 0) {
      code.push(`${indentStr}// Exposed Headers: ${exposedHeaders.join(', ')}`);
    }
    code.push(`${indentStr}// Allow Credentials: ${allowCredentials}`);
    code.push(`${indentStr}// Max Age: ${maxAge} seconds`);
    code.push(`${indentStr}// Preflight Continue: ${preflightContinue}`);
    code.push(`${indentStr}// CORS plugin will be configured on Kong routes during deployment`);

    // Generate result variable
    const resultVar = `result_${node.id.replace(/[^a-zA-Z0-9]/g, '_')}`;
    context.resultVars.set(node.id, resultVar);
    code.push(`${indentStr}const ${resultVar} = { type: 'kong-cors-config', origins: ${JSON.stringify(allowedOrigins)} };`);

    return {
      code: code.join('\n'),
      imports,
      resultVar,
    };
  },
};

export default KongCorsPattern;

