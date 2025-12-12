/**
 * MCP Tool Handler Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import {
  handleMCPTool,
  listMCPTools,
} from '../tool-handler';
import { getTemporalClient } from '@/lib/temporal/connection';

vi.mock('@/lib/temporal/connection', () => ({
  getTemporalClient: vi.fn(),
}));

describe('MCP Tool Handler', () => {
  let mockSupabase: SupabaseClient<Database>;
  let mockTemporalClient: any;

  beforeEach(() => {
    mockSupabase = {
      from: vi.fn(),
    } as any;

    mockTemporalClient = {
      getHandle: vi.fn().mockReturnValue({
        query: vi.fn().mockResolvedValue({ result: 'Tool execution result' }),
      }),
    };

    vi.mocked(getTemporalClient).mockResolvedValue(mockTemporalClient as any);
  });

  describe('listMCPTools', () => {
    it('should return list of tools when config exists', async () => {
      const mockServiceInterface = {
        id: 'si-123',
        mcp_config: {
          tools: [
            {
              name: 'testTool1',
              description: 'Test tool 1',
              inputSchema: {
                type: 'object',
                properties: {
                  param1: { type: 'string' },
                },
              },
            },
            {
              name: 'testTool2',
              description: 'Test tool 2',
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

      const result = await listMCPTools('si-123', mockSupabase);

      expect(result).toHaveLength(2);
      expect(result[0]).toEqual({
        name: 'testTool1',
        description: 'Test tool 1',
        inputSchema: {
          type: 'object',
          properties: {
            param1: { type: 'string' },
          },
        },
      });
    });

    it('should return empty array when config does not exist', async () => {
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

      const result = await listMCPTools('si-123', mockSupabase);

      expect(result).toEqual([]);
    });
  });

  describe('handleMCPTool', () => {
    it('should execute tool with workflow and return result', async () => {
      const mockServiceInterface = {
        id: 'si-123',
        mcp_config: {
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
              workflowId: 'workflow-123',
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

      const result = await handleMCPTool(
        {
          name: 'testTool',
          arguments: { param: 'value' },
        },
        'si-123',
        mockSupabase
      );

      expect(result).toEqual({
        content: [
          {
            type: 'text',
            text: JSON.stringify({ result: 'Tool execution result' }),
          },
        ],
        isError: false,
      });
    });

    it('should validate input against schema', async () => {
      const mockServiceInterface = {
        id: 'si-123',
        mcp_config: {
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
              workflowId: 'workflow-123',
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

      // Should throw error for invalid type
      await expect(
        handleMCPTool(
          {
            name: 'testTool',
            arguments: { param: 123 }, // Should be string
          },
          'si-123',
          mockSupabase
        )
      ).rejects.toThrow('Invalid type for param');
    });

    it('should throw error when tool does not exist', async () => {
      const mockServiceInterface = {
        id: 'si-123',
        mcp_config: {
          tools: [
            {
              name: 'testTool',
              description: 'Test tool',
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

      await expect(
        handleMCPTool({ name: 'nonexistentTool' }, 'si-123', mockSupabase)
      ).rejects.toThrow('Tool not found');
    });

    it('should return error response when workflow execution fails', async () => {
      const mockServiceInterface = {
        id: 'si-123',
        mcp_config: {
          tools: [
            {
              name: 'testTool',
              description: 'Test tool',
              workflowId: 'workflow-123',
            },
          ],
        },
      };

      mockTemporalClient.getHandle.mockReturnValue({
        query: vi.fn().mockRejectedValue(new Error('Workflow error')),
      });

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

      const result = await handleMCPTool(
        { name: 'testTool', arguments: {} },
        'si-123',
        mockSupabase
      );

      expect(result.isError).toBe(true);
      expect(result.content[0].text).toContain('Error executing tool');
    });
  });
});

