/**
 * MCP Server Pattern
 * Generates MCP server implementation code
 */

import type { Pattern, WorkflowNode, GeneratorContext, CodeBlock } from '../types';
import { indent } from '../utils/ast-helpers';

/**
 * MCP Server Pattern
 * Detects mcp-server nodes and generates MCP server handler code
 */
export const MCPServerPattern: Pattern = {
  name: 'mcp-server',
  priority: 40, // Medium priority - generates handler code

  detect: (node: WorkflowNode, context: GeneratorContext): boolean => {
    return node.type === 'mcp-server';
  },

  generate: (node: WorkflowNode, context: GeneratorContext): CodeBlock => {
    const indentLevel = context.currentIndent;
    const indentStr = indent(indentLevel);
    const config = node.data.config || {};
    const code: string[] = [];
    const imports: string[] = [];

    const componentName = node.data.label || node.data.componentName || 'MCP Server';
    const serverName = config.serverName || config.name || 'mcp-server';
    const version = config.version || '1.0.0';
    const endpointPath = config.endpointPath || '/mcp';
    const resources = Array.isArray(config.resources) ? config.resources : [];
    const tools = Array.isArray(config.tools) ? config.tools : [];

    // MCP Server configuration
    code.push(`${indentStr}// MCP Server: ${componentName}`);
    code.push(`${indentStr}// Server Name: ${serverName}`);
    code.push(`${indentStr}// Version: ${version}`);
    code.push(`${indentStr}// Endpoint Path: ${endpointPath}`);

    if (resources.length > 0) {
      code.push(`${indentStr}// Resources (${resources.length}):`);
      resources.forEach((resource: any, index: number) => {
        const resourceName = resource.name || resource.uri || `resource${index + 1}`;
        const workflowId = resource.workflowId || resource.workflow || 'Not mapped';
        code.push(`${indentStr}//   - ${resourceName} -> Workflow: ${workflowId}`);
      });
    }

    if (tools.length > 0) {
      code.push(`${indentStr}// Tools (${tools.length}):`);
      tools.forEach((tool: any, index: number) => {
        const toolName = tool.name || `tool${index + 1}`;
        const workflowId = tool.workflowId || tool.workflow || 'Not mapped';
        const inputSchema = tool.inputSchema || tool.schema || 'Not configured';
        code.push(`${indentStr}//   - ${toolName} -> Workflow: ${workflowId}`);
        if (inputSchema && typeof inputSchema === 'object') {
          code.push(`${indentStr}//     Input Schema: ${JSON.stringify(inputSchema)}`);
        }
      });
    }

    code.push(`${indentStr}// MCP endpoint will be available at: ${endpointPath}`);
    code.push(`${indentStr}// MCP server handler will be generated during deployment`);

    // Generate result variable
    const resultVar = `result_${node.id.replace(/[^a-zA-Z0-9]/g, '_')}`;
    context.resultVars.set(node.id, resultVar);
    code.push(`${indentStr}const ${resultVar} = { type: 'mcp-server', serverName: '${serverName}', version: '${version}', resources: ${resources.length}, tools: ${tools.length} };`);

    return {
      code: code.join('\n'),
      imports,
      resultVar,
    };
  },
};

export default MCPServerPattern;

