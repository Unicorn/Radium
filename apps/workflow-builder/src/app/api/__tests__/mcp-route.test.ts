/**
 * MCP API Route Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { POST, GET } from '../mcp/route';
import { NextRequest } from 'next/server';
import { createClient } from '@/lib/supabase/server';
import { getMCPServerConfig, buildMCPServerInfo } from '@/lib/mcp/server-builder';
import { handleMCPResource, listMCPResources } from '@/lib/mcp/resource-handler';
import { handleMCPTool, listMCPTools } from '@/lib/mcp/tool-handler';

vi.mock('@/lib/supabase/server', () => ({
  createClient: vi.fn(),
}));

vi.mock('@/lib/mcp/server-builder', () => ({
  getMCPServerConfig: vi.fn(),
  buildMCPServerInfo: vi.fn(),
}));

vi.mock('@/lib/mcp/resource-handler', () => ({
  handleMCPResource: vi.fn(),
  listMCPResources: vi.fn(),
}));

vi.mock('@/lib/mcp/tool-handler', () => ({
  handleMCPTool: vi.fn(),
  listMCPTools: vi.fn(),
}));

describe('MCP API Route', () => {
  let mockSupabase: any;

  beforeEach(() => {
    vi.clearAllMocks();

    mockSupabase = {};
    vi.mocked(createClient).mockResolvedValue(mockSupabase as any);
  });

  describe('POST', () => {
    it('should handle initialize method', async () => {
      const serviceInterfaceId = 'si-123';
      const mockConfig = {
        serverName: 'Test Server',
        version: '1.0.0',
        endpointPath: '/mcp',
        resources: [],
        tools: [],
      };
      const mockServerInfo = {
        name: 'Test Server',
        version: '1.0.0',
        protocolVersion: '2024-11-05',
        capabilities: {},
      };

      vi.mocked(getMCPServerConfig).mockResolvedValue(mockConfig);
      vi.mocked(buildMCPServerInfo).mockReturnValue(mockServerInfo);

      const request = new NextRequest('http://localhost/api/mcp?serviceInterfaceId=si-123', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          method: 'initialize',
          params: {},
        }),
      });

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(200);
      expect(data.result).toEqual(mockServerInfo);
    });

    it('should handle resources/list method', async () => {
      const serviceInterfaceId = 'si-123';
      const mockResources = [
        {
          uri: 'resource://test/resource1',
          name: 'Resource 1',
          description: 'Test resource',
        },
      ];

      vi.mocked(listMCPResources).mockResolvedValue(mockResources as any);

      const request = new NextRequest('http://localhost/api/mcp?serviceInterfaceId=si-123', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          method: 'resources/list',
          params: {},
        }),
      });

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(200);
      expect(data.result.resources).toEqual(mockResources);
    });

    it('should handle resources/read method', async () => {
      const serviceInterfaceId = 'si-123';
      const mockResourceResponse = {
        contents: [
          {
            uri: 'resource://test/resource1',
            mimeType: 'text/plain',
            text: 'Resource content',
          },
        ],
      };

      vi.mocked(handleMCPResource).mockResolvedValue(mockResourceResponse);

      const request = new NextRequest('http://localhost/api/mcp?serviceInterfaceId=si-123', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          method: 'resources/read',
          params: {
            uri: 'resource://test/resource1',
          },
        }),
      });

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(200);
      expect(data.result).toEqual(mockResourceResponse);
    });

    it('should handle tools/list method', async () => {
      const serviceInterfaceId = 'si-123';
      const mockTools = [
        {
          name: 'testTool',
          description: 'Test tool',
          inputSchema: {
            type: 'object',
            properties: {
              param: { type: 'string' },
            },
          },
        },
      ];

      vi.mocked(listMCPTools).mockResolvedValue(mockTools as any);

      const request = new NextRequest('http://localhost/api/mcp?serviceInterfaceId=si-123', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          method: 'tools/list',
          params: {},
        }),
      });

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(200);
      expect(data.result.tools).toEqual(mockTools);
    });

    it('should handle tools/call method', async () => {
      const serviceInterfaceId = 'si-123';
      const mockToolResponse = {
        content: [
          {
            type: 'text',
            text: 'Tool result',
          },
        ],
        isError: false,
      };

      vi.mocked(handleMCPTool).mockResolvedValue(mockToolResponse);

      const request = new NextRequest('http://localhost/api/mcp?serviceInterfaceId=si-123', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          method: 'tools/call',
          params: {
            name: 'testTool',
            arguments: { param: 'value' },
          },
        }),
      });

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(200);
      expect(data.result).toEqual(mockToolResponse);
    });

    it('should return 400 when service interface ID is missing', async () => {
      const request = new NextRequest('http://localhost/api/mcp', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          method: 'initialize',
        }),
      });

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(400);
      expect(data.error.code).toBe(-32600);
      expect(data.error.message).toContain('Service interface ID is required');
    });

    it('should return 400 when method is missing', async () => {
      const request = new NextRequest('http://localhost/api/mcp?serviceInterfaceId=si-123', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          params: {},
        }),
      });

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(400);
      expect(data.error.code).toBe(-32600);
      expect(data.error.message).toContain('Method is required');
    });

    it('should return 400 for unknown method', async () => {
      const request = new NextRequest('http://localhost/api/mcp?serviceInterfaceId=si-123', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          method: 'unknown/method',
          params: {},
        }),
      });

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(400);
      expect(data.error.code).toBe(-32601);
      expect(data.error.message).toContain('Unknown method');
    });

    it('should handle errors gracefully', async () => {
      vi.mocked(getMCPServerConfig).mockRejectedValue(new Error('Database error'));

      const request = new NextRequest('http://localhost/api/mcp?serviceInterfaceId=si-123', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          method: 'initialize',
          params: {},
        }),
      });

      const response = await POST(request);
      const data = await response.json();

      expect(response.status).toBe(500);
      expect(data.error.code).toBe(-32603);
    });
  });

  describe('GET', () => {
    it('should return info message', async () => {
      const request = new NextRequest('http://localhost/api/mcp', {
        method: 'GET',
      });

      const response = await GET(request);
      const data = await response.json();

      expect(response.status).toBe(200);
      expect(data.message).toContain('MCP endpoint');
    });
  });
});

