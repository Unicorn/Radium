/**
 * MCP Server Builder Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import {
  getMCPServerConfig,
  buildMCPServerInfo,
} from '../server-builder';

describe('MCP Server Builder', () => {
  let mockSupabase: SupabaseClient<Database>;

  beforeEach(() => {
    mockSupabase = {
      from: vi.fn(),
    } as any;
  });

  describe('getMCPServerConfig', () => {
    it('should return MCP server config when it exists', async () => {
      const mockServiceInterface = {
        id: 'si-123',
        mcp_config: {
          serverName: 'Test MCP Server',
          version: '1.0.0',
          endpointPath: '/mcp',
          resources: [
            {
              uri: 'resource://test/resource1',
              name: 'Resource 1',
              description: 'Test resource',
              mimeType: 'text/plain',
              workflowId: 'workflow-123',
            },
          ],
          tools: [
            {
              name: 'testTool',
              description: 'Test tool',
              inputSchema: {
                type: 'object',
                properties: {
                  param: { type: 'string' },
                },
              },
              workflowId: 'workflow-456',
            },
          ],
        },
      };

      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: mockServiceInterface,
        error: null,
      });

      (mockSupabase.from as any).mockReturnValue({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      const result = await getMCPServerConfig('si-123', mockSupabase);

      expect(result).toEqual({
        serverName: 'Test MCP Server',
        version: '1.0.0',
        endpointPath: '/mcp',
        resources: [
          {
            uri: 'resource://test/resource1',
            name: 'Resource 1',
            description: 'Test resource',
            mimeType: 'text/plain',
            workflowId: 'workflow-123',
          },
        ],
        tools: [
          {
            name: 'testTool',
            description: 'Test tool',
            inputSchema: {
              type: 'object',
              properties: {
                param: { type: 'string' },
              },
            },
            workflowId: 'workflow-456',
          },
        ],
      });
    });

    it('should handle snake_case config format', async () => {
      const mockServiceInterface = {
        id: 'si-123',
        mcp_config: {
          server_name: 'Test Server',
          version: '1.0.0',
          endpoint_path: '/mcp',
        },
      };

      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: mockServiceInterface,
        error: null,
      });

      (mockSupabase.from as any).mockReturnValue({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      const result = await getMCPServerConfig('si-123', mockSupabase);

      expect(result).toEqual({
        serverName: 'Test Server',
        version: '1.0.0',
        endpointPath: '/mcp',
        resources: [],
        tools: [],
      });
    });

    it('should return null when config does not exist', async () => {
      const mockServiceInterface = {
        id: 'si-123',
        mcp_config: null,
      };

      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: mockServiceInterface,
        error: null,
      });

      (mockSupabase.from as any).mockReturnValue({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      const result = await getMCPServerConfig('si-123', mockSupabase);

      expect(result).toBeNull();
    });
  });

  describe('buildMCPServerInfo', () => {
    it('should build server info with resources and tools', () => {
      const config = {
        serverName: 'Test Server',
        version: '1.0.0',
        endpointPath: '/mcp',
        resources: [
          {
            uri: 'resource://test/resource1',
            name: 'Resource 1',
          },
        ],
        tools: [
          {
            name: 'testTool',
            description: 'Test tool',
          },
        ],
      };

      const result = buildMCPServerInfo(config);

      expect(result).toEqual({
        name: 'Test Server',
        version: '1.0.0',
        protocolVersion: '2024-11-05',
        capabilities: {
          resources: {},
          tools: {},
        },
      });
    });

    it('should build server info without resources and tools', () => {
      const config = {
        serverName: 'Test Server',
        version: '1.0.0',
        endpointPath: '/mcp',
        resources: [],
        tools: [],
      };

      const result = buildMCPServerInfo(config);

      expect(result).toEqual({
        name: 'Test Server',
        version: '1.0.0',
        protocolVersion: '2024-11-05',
        capabilities: {},
      });
    });
  });
});

