/**
 * GraphQL Gateway Pattern
 * Generates GraphQL handler code and schema configuration
 */

import type { Pattern, WorkflowNode, GeneratorContext, CodeBlock } from '../types';
import { indent } from '../utils/ast-helpers';

/**
 * GraphQL Gateway Pattern
 * Detects graphql-gateway nodes and generates GraphQL handler code
 */
export const GraphQLGatewayPattern: Pattern = {
  name: 'graphql-gateway',
  priority: 40, // Medium priority - generates handler code

  detect: (node: WorkflowNode, context: GeneratorContext): boolean => {
    return node.type === 'graphql-gateway';
  },

  generate: (node: WorkflowNode, context: GeneratorContext): CodeBlock => {
    const indentLevel = context.currentIndent;
    const indentStr = indent(indentLevel);
    const config = node.data.config || {};
    const code: string[] = [];
    const imports: string[] = ['import { GraphQLSchema, GraphQLObjectType, GraphQLString } from \'graphql\';'];

    const componentName = node.data.label || node.data.componentName || 'GraphQL Gateway';
    const endpointPath = config.endpointPath || '/graphql';
    const schema = config.schema || config.graphqlSchema || '';
    const queries = Array.isArray(config.queries) ? config.queries : [];
    const mutations = Array.isArray(config.mutations) ? config.mutations : [];

    // GraphQL Gateway configuration
    code.push(`${indentStr}// GraphQL Gateway: ${componentName}`);
    code.push(`${indentStr}// Endpoint Path: ${endpointPath}`);
    
    if (schema) {
      code.push(`${indentStr}// GraphQL Schema (SDL):`);
      // Split schema into lines and indent each
      const schemaLines = schema.split('\n');
      schemaLines.forEach(line => {
        code.push(`${indentStr}// ${line}`);
      });
    } else {
      code.push(`${indentStr}// GraphQL Schema: Not configured`);
    }

    if (queries.length > 0) {
      code.push(`${indentStr}// Queries (${queries.length}):`);
      queries.forEach((query: any, index: number) => {
        const queryName = query.name || `query${index + 1}`;
        const workflowId = query.workflowId || query.workflow || 'Not mapped';
        code.push(`${indentStr}//   - ${queryName} -> Workflow: ${workflowId}`);
      });
    }

    if (mutations.length > 0) {
      code.push(`${indentStr}// Mutations (${mutations.length}):`);
      mutations.forEach((mutation: any, index: number) => {
        const mutationName = mutation.name || `mutation${index + 1}`;
        const workflowId = mutation.workflowId || mutation.workflow || 'Not mapped';
        code.push(`${indentStr}//   - ${mutationName} -> Workflow: ${workflowId}`);
      });
    }

    code.push(`${indentStr}// GraphQL endpoint will be available at: ${endpointPath}`);
    code.push(`${indentStr}// GraphQL handler will be generated during deployment`);

    // Generate result variable
    const resultVar = `result_${node.id.replace(/[^a-zA-Z0-9]/g, '_')}`;
    context.resultVars.set(node.id, resultVar);
    code.push(`${indentStr}const ${resultVar} = { type: 'graphql-gateway', endpoint: '${endpointPath}', queries: ${queries.length}, mutations: ${mutations.length} };`);

    return {
      code: code.join('\n'),
      imports,
      resultVar,
    };
  },
};

export default GraphQLGatewayPattern;

