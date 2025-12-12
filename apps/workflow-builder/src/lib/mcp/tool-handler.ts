/**
 * MCP Tool Handler
 * Handles MCP tool requests and routes them to Temporal workflows
 */

import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import { getMCPServerConfig, type MCPTool } from './server-builder';
import { getTemporalClient } from '@/lib/temporal/connection';

export interface MCPToolRequest {
  name: string;
  arguments?: Record<string, any>;
}

export interface MCPToolResponse {
  content: Array<{
    type: 'text' | 'resource';
    text?: string;
    resource?: {
      uri: string;
      mimeType?: string;
    };
  }>;
  isError?: boolean;
}

/**
 * Handle MCP tool request
 */
export async function handleMCPTool(
  request: MCPToolRequest,
  serviceInterfaceId: string,
  supabase: SupabaseClient<Database>
): Promise<MCPToolResponse> {
  const config = await getMCPServerConfig(serviceInterfaceId, supabase);
  
  if (!config) {
    throw new Error('MCP server configuration not found');
  }

  // Find tool by name
  const tool = config.tools?.find(t => t.name === request.name);
  
  if (!tool) {
    throw new Error(`Tool not found: ${request.name}`);
  }

  // Validate input against schema if provided
  if (tool.inputSchema && request.arguments) {
    // Basic validation - in production, use a proper schema validator like Zod
    const schema = tool.inputSchema;
    for (const [key, value] of Object.entries(request.arguments)) {
      if (schema.properties?.[key]) {
        const propSchema = schema.properties[key];
        // Basic type checking
        if (propSchema.type === 'string' && typeof value !== 'string') {
          throw new Error(`Invalid type for ${key}: expected string, got ${typeof value}`);
        }
        if (propSchema.type === 'number' && typeof value !== 'number') {
          throw new Error(`Invalid type for ${key}: expected number, got ${typeof value}`);
        }
        if (propSchema.type === 'boolean' && typeof value !== 'boolean') {
          throw new Error(`Invalid type for ${key}: expected boolean, got ${typeof value}`);
        }
      }
    }
  }

  // If tool has a workflow, execute it
  if (tool.workflowId) {
    try {
      const client = await getTemporalClient();
      const handle = client.getHandle(tool.workflowId);
      
      // Execute workflow with tool arguments
      const result = await handle.query('executeTool', request.arguments || {});
      
      return {
        content: [{
          type: 'text',
          text: typeof result === 'string' ? result : JSON.stringify(result),
        }],
        isError: false,
      };
    } catch (error) {
      return {
        content: [{
          type: 'text',
          text: `Error executing tool: ${error instanceof Error ? error.message : 'Unknown error'}`,
        }],
        isError: true,
      };
    }
  }

  // Default: return empty result
  return {
    content: [{
      type: 'text',
      text: '',
    }],
    isError: false,
  };
}

/**
 * List all available tools
 */
export async function listMCPTools(
  serviceInterfaceId: string,
  supabase: SupabaseClient<Database>
): Promise<MCPTool[]> {
  const config = await getMCPServerConfig(serviceInterfaceId, supabase);
  
  if (!config) {
    return [];
  }

  return config.tools || [];
}

