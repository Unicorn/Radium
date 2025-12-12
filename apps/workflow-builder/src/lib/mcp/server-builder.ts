/**
 * MCP Server Builder
 * Builds MCP server instances from component configuration
 */

import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';

export interface MCPResource {
  uri: string;
  name: string;
  description?: string;
  mimeType?: string;
  workflowId?: string;
}

export interface MCPTool {
  name: string;
  description?: string;
  inputSchema?: Record<string, any>;
  workflowId?: string;
}

export interface MCPServerConfig {
  serverName: string;
  version: string;
  endpointPath: string;
  resources?: MCPResource[];
  tools?: MCPTool[];
}

/**
 * Get MCP server configuration from service interface
 */
export async function getMCPServerConfig(
  serviceInterfaceId: string,
  supabase: SupabaseClient<Database>
): Promise<MCPServerConfig | null> {
  const { data: serviceInterface, error } = await supabase
    .from('service_interfaces')
    .select('mcp_config')
    .eq('id', serviceInterfaceId)
    .single();

  if (error || !serviceInterface || !serviceInterface.mcp_config) {
    return null;
  }

  const config = serviceInterface.mcp_config as any;

  return {
    serverName: config.serverName || config.server_name || 'MCP Server',
    version: config.version || '1.0.0',
    endpointPath: config.endpointPath || config.endpoint_path || '/mcp',
    resources: config.resources || [],
    tools: config.tools || [],
  };
}

/**
 * Build MCP server info response
 */
export function buildMCPServerInfo(config: MCPServerConfig) {
  return {
    name: config.serverName,
    version: config.version,
    protocolVersion: '2024-11-05', // MCP protocol version
    capabilities: {
      resources: config.resources && config.resources.length > 0 ? {} : undefined,
      tools: config.tools && config.tools.length > 0 ? {} : undefined,
    },
  };
}

