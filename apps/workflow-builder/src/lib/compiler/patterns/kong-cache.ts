/**
 * Kong Cache Pattern
 * Generates Kong cache plugin configuration and handles cache key lifecycle
 */

import type { Pattern, WorkflowNode, GeneratorContext, CodeBlock } from '../types';
import { indent } from '../utils/ast-helpers';

/**
 * Kong Cache Pattern
 * Detects kong-cache nodes and generates cache configuration metadata
 */
export const KongCachePattern: Pattern = {
  name: 'kong-cache',
  priority: 30, // Lower priority - configuration, not execution code

  detect: (node: WorkflowNode, context: GeneratorContext): boolean => {
    return node.type === 'kong-cache';
  },

  generate: (node: WorkflowNode, context: GeneratorContext): CodeBlock => {
    const indentLevel = context.currentIndent;
    const indentStr = indent(indentLevel);
    const config = node.data.config || {};
    const code: string[] = [];
    const imports: string[] = [];

    const componentName = node.data.label || node.data.componentName || 'Kong Cache';
    const cacheKey = config.cacheKey || 'not-configured';
    const connectorName = config.connectorName || config.connectorId || 'Not configured';
    const ttl = config.ttlSeconds || config.ttl || 3600;
    const cacheKeyStrategy = config.cacheKeyStrategy || 'default';
    const contentTypes = Array.isArray(config.contentTypes) 
      ? config.contentTypes.join(', ') 
      : 'application/json';
    const responseCodes = Array.isArray(config.responseCodes) 
      ? config.responseCodes.join(', ') 
      : '200, 201';
    const isSaved = config.isSaved === true;

    // Kong cache configuration
    code.push(`${indentStr}// Kong Cache Configuration: ${componentName}`);
    code.push(`${indentStr}// Cache Key: ${cacheKey}${isSaved ? ' (immutable)' : ' (editable until save)'}`);
    code.push(`${indentStr}// Redis Connector: ${connectorName}`);
    code.push(`${indentStr}// TTL: ${ttl} seconds`);
    code.push(`${indentStr}// Cache Key Strategy: ${cacheKeyStrategy}`);
    code.push(`${indentStr}// Content Types: ${contentTypes}`);
    code.push(`${indentStr}// Response Codes: ${responseCodes}`);
    code.push(`${indentStr}// Cache plugin will be configured on Kong routes during deployment`);

    // Handle cache key deletion if component is marked for deletion
    if (config.markedForDeletion === true) {
      code.push(`${indentStr}// Cache key marked for deletion - will be removed from Redis on next deploy`);
    }

    // Generate result variable
    const resultVar = `result_${node.id.replace(/[^a-zA-Z0-9]/g, '_')}`;
    context.resultVars.set(node.id, resultVar);
    code.push(`${indentStr}const ${resultVar} = { type: 'kong-cache-config', cacheKey: '${cacheKey}', ttl: ${ttl} };`);

    return {
      code: code.join('\n'),
      imports,
      resultVar,
    };
  },
};

export default KongCachePattern;

