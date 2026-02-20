/**
 * MCP API Route
 * Handles MCP (Model Context Protocol) requests and routes them to Temporal workflows
 */

import { NextRequest, NextResponse } from 'next/server';
import { createClient } from '@/lib/supabase/server';
import { getMCPServerConfig, buildMCPServerInfo } from '@/lib/mcp/server-builder';
import { handleMCPResource, listMCPResources } from '@/lib/mcp/resource-handler';
import { handleMCPTool, listMCPTools } from '@/lib/mcp/tool-handler';

export async function POST(request: NextRequest) {
  try {
    // Get service interface ID from headers or query params
    const serviceInterfaceId = request.headers.get('X-Service-Interface-Id') || 
                                new URL(request.url).searchParams.get('serviceInterfaceId');

    if (!serviceInterfaceId) {
      return NextResponse.json(
        { error: { code: -32600, message: 'Service interface ID is required' } },
        { status: 400 }
      );
    }

    // Get request body
    const body = await request.json();
    const { method, params } = body;

    if (!method) {
      return NextResponse.json(
        { error: { code: -32600, message: 'Method is required' } },
        { status: 400 }
      );
    }

    // Get Supabase client
    const supabase = await createClient();

    // Handle different MCP methods
    switch (method) {
      case 'initialize': {
        // Return server info
        const config = await getMCPServerConfig(serviceInterfaceId, supabase as any);
        if (!config) {
          return NextResponse.json(
            { error: { code: -32601, message: 'MCP server configuration not found' } },
            { status: 404 }
          );
        }
        return NextResponse.json({
          result: buildMCPServerInfo(config),
        });
      }

      case 'resources/list': {
        const resources = await listMCPResources(serviceInterfaceId, supabase as any);
        return NextResponse.json({
          result: {
            resources: resources.map(r => ({
              uri: r.uri,
              name: r.name,
              description: r.description,
              mimeType: r.mimeType,
            })),
          },
        });
      }

      case 'resources/read': {
        if (!params || !params.uri) {
          return NextResponse.json(
            { error: { code: -32602, message: 'Resource URI is required' } },
            { status: 400 }
          );
        }
        const resourceResponse = await handleMCPResource(
          { uri: params.uri },
          serviceInterfaceId,
          supabase as any
        );
        return NextResponse.json({ result: resourceResponse });
      }

      case 'tools/list': {
        const tools = await listMCPTools(serviceInterfaceId, supabase as any);
        return NextResponse.json({
          result: {
            tools: tools.map(t => ({
              name: t.name,
              description: t.description,
              inputSchema: t.inputSchema,
            })),
          },
        });
      }

      case 'tools/call': {
        if (!params || !params.name) {
          return NextResponse.json(
            { error: { code: -32602, message: 'Tool name is required' } },
            { status: 400 }
          );
        }
        const toolResponse = await handleMCPTool(
          {
            name: params.name,
            arguments: params.arguments,
          },
          serviceInterfaceId,
          supabase as any
        );
        return NextResponse.json({ result: toolResponse });
      }

      default:
        return NextResponse.json(
          { error: { code: -32601, message: `Unknown method: ${method}` } },
          { status: 400 }
        );
    }
  } catch (error) {
    // eslint-disable-next-line no-console
    console.error('MCP request error:', error);
    return NextResponse.json(
      {
        error: {
          code: -32603,
          message: error instanceof Error ? error.message : 'Internal server error',
        },
      },
      { status: 500 }
    );
  }
}

export async function GET(_request: NextRequest) {
  return NextResponse.json(
    { message: 'MCP endpoint - use POST for MCP protocol requests' },
    { status: 200 }
  );
}

