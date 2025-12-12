/**
 * MCP Resource Handler
 * Handles MCP resource requests and routes them to Temporal workflows
 */

import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import { getMCPServerConfig, type MCPResource } from './server-builder';
import { getTemporalClient } from '@/lib/temporal/connection';

export interface MCPResourceRequest {
  uri: string;
}

export interface MCPResourceResponse {
  contents: Array<{
    uri: string;
    mimeType?: string;
    text?: string;
    blob?: string;
  }>;
}

/**
 * Handle MCP resource request
 */
export async function handleMCPResource(
  request: MCPResourceRequest,
  serviceInterfaceId: string,
  supabase: SupabaseClient<Database>
): Promise<MCPResourceResponse> {
  const config = await getMCPServerConfig(serviceInterfaceId, supabase);
  
  if (!config) {
    throw new Error('MCP server configuration not found');
  }

  // Find resource by URI
  const resource = config.resources?.find(r => r.uri === request.uri);
  
  if (!resource) {
    throw new Error(`Resource not found: ${request.uri}`);
  }

  // If resource has a workflow, execute it
  if (resource.workflowId) {
    try {
      const client = await getTemporalClient();
      const handle = client.getHandle(resource.workflowId);
      
      // Query workflow for resource content
      const content = await handle.query('getResource', { uri: request.uri });
      
      return {
        contents: [{
          uri: request.uri,
          mimeType: resource.mimeType || 'text/plain',
          text: typeof content === 'string' ? content : JSON.stringify(content),
        }],
      };
    } catch (error) {
      throw new Error(`Failed to get resource from workflow: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  // Default: return empty resource
  return {
    contents: [{
      uri: request.uri,
      mimeType: resource.mimeType || 'text/plain',
      text: '',
    }],
  };
}

/**
 * List all available resources
 */
export async function listMCPResources(
  serviceInterfaceId: string,
  supabase: SupabaseClient<Database>
): Promise<MCPResource[]> {
  const config = await getMCPServerConfig(serviceInterfaceId, supabase);
  
  if (!config) {
    return [];
  }

  return config.resources || [];
}

