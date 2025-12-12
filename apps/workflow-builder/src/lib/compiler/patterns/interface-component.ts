/**
 * Interface Component Pattern
 * Handles data-in and data-out interface components
 * These are pass-through nodes that connect to other components
 */

import type { Pattern, WorkflowNode, GeneratorContext, CodeBlock } from '../types';
import { indent } from '../utils/ast-helpers';

/**
 * Interface Component Pattern
 * Detects data-in and data-out nodes and generates pass-through code
 */
export const InterfaceComponentPattern: Pattern = {
  name: 'interface-component',
  priority: 50, // Medium priority

  detect: (node: WorkflowNode, context: GeneratorContext): boolean => {
    return node.type === 'data-in' || node.type === 'data-out';
  },

  generate: (node: WorkflowNode, context: GeneratorContext): CodeBlock => {
    const indentLevel = context.currentIndent;
    const indentStr = indent(indentLevel);
    const code: string[] = [];
    const imports: string[] = [];

    // Interface components are pass-through nodes
    // They don't generate code themselves, but connect to other components
    // The actual signal/query handling is done by the service interface system
    
    if (node.type === 'data-in') {
      // Data-in: receives data via signal, passes to connected component
      code.push(`${indentStr}// Data In Interface: ${node.data.label || node.id}`);
      code.push(`${indentStr}// Receives data via signal and passes to connected component`);
      
      // Get the connected target node
      const outgoingEdges = context.edges.filter(e => e.source === node.id);
      if (outgoingEdges.length > 0) {
        const targetNodeId = outgoingEdges[0].target;
        const targetResult = context.resultVars.get(targetNodeId);
        
        if (targetResult) {
          // Pass through the result from the target
          const resultVar = `result_${node.id.replace(/[^a-zA-Z0-9]/g, '_')}`;
          context.resultVars.set(node.id, resultVar);
          code.push(`${indentStr}const ${resultVar} = ${targetResult};`);
        } else {
          // No target result yet, use input
          const resultVar = `result_${node.id.replace(/[^a-zA-Z0-9]/g, '_')}`;
          context.resultVars.set(node.id, resultVar);
          code.push(`${indentStr}const ${resultVar} = input; // Data received via signal`);
        }
      } else {
        // No connection, just pass input through
        const resultVar = `result_${node.id.replace(/[^a-zA-Z0-9]/g, '_')}`;
        context.resultVars.set(node.id, resultVar);
        code.push(`${indentStr}const ${resultVar} = input; // Data received via signal`);
      }
    } else if (node.type === 'data-out') {
      // Data-out: provides data via query, gets from connected source
      code.push(`${indentStr}// Data Out Interface: ${node.data.label || node.id}`);
      code.push(`${indentStr}// Provides data via query from connected source`);
      
      // Get the connected source node
      const incomingEdges = context.edges.filter(e => e.target === node.id);
      if (incomingEdges.length > 0) {
        const sourceNodeId = incomingEdges[0].source;
        const sourceResult = context.resultVars.get(sourceNodeId);
        
        if (sourceResult) {
          // Pass through the result from the source
          const resultVar = `result_${node.id.replace(/[^a-zA-Z0-9]/g, '_')}`;
          context.resultVars.set(node.id, resultVar);
          code.push(`${indentStr}const ${resultVar} = ${sourceResult};`);
        } else {
          // No source result yet, use state or default
          const resultVar = `result_${node.id.replace(/[^a-zA-Z0-9]/g, '_')}`;
          context.resultVars.set(node.id, resultVar);
          code.push(`${indentStr}const ${resultVar} = {}; // Data to provide via query`);
        }
      } else {
        // No connection, return empty
        const resultVar = `result_${node.id.replace(/[^a-zA-Z0-9]/g, '_')}`;
        context.resultVars.set(node.id, resultVar);
        code.push(`${indentStr}const ${resultVar} = {}; // No data source connected`);
      }
    }

    const resultVar = `result_${node.id.replace(/[^a-zA-Z0-9]/g, '_')}`;
    if (!context.resultVars.has(node.id)) {
      context.resultVars.set(node.id, resultVar);
    }

    return {
      code: code.join('\n'),
      imports,
      resultVar: context.resultVars.get(node.id) || resultVar,
    };
  },
};

export default InterfaceComponentPattern;

