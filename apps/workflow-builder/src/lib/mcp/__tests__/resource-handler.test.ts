/**
 * MCP Resource Handler Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import {
  handleMCPResource,
  listMCPResources,
} from '../resource-handler';
import { getTemporalClient } from '@/lib/temporal/connection';

vi.mock('@/lib/temporal/connection', () => ({
  getTemporalClient: vi.fn(),
}));

describe('MCP Resource Handler', () => {
  let mockSupabase: SupabaseClient<Database>;
  let mockTemporalClient: any;

  beforeEach(() => {
    mockSupabase = {
      from: vi.fn(),
    } as any;

    mockTemporalClient = {
      getHandle: vi.fn().mockReturnValue({
        query: vi.fn().mockResolvedValue({ content: 'Resource content' }),
      }),
    };

    vi.mocked(getTemporalClient).mockResolvedValue(mockTemporalClient as any);
  });

  describe('listMCPResources', () => {
    it('should return list of resources when config exists', async () => {
      const mockServiceInterface = {
        id: 'si-123',
        mcp_config: {
          resources: [
            {
              uri: 'resource://test/resource1',
              name: 'Resource 1',
              description: 'Test resource 1',
              mimeType: 'text/plain',
            },
            {
              uri: 'resource://test/resource2',
              name: 'Resource 2',
              description: 'Test resource 2',
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

      const result = await listMCPResources('si-123', mockSupabase);

      expect(result).toHaveLength(2);
      expect(result[0]).toEqual({
        uri: 'resource://test/resource1',
        name: 'Resource 1',
        description: 'Test resource 1',
        mimeType: 'text/plain',
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

      const result = await listMCPResources('si-123', mockSupabase);

      expect(result).toEqual([]);
    });
  });

  describe('handleMCPResource', () => {
    it('should return resource content from workflow', async () => {
      const mockServiceInterface = {
        id: 'si-123',
        mcp_config: {
          resources: [
            {
              uri: 'resource://test/resource1',
              name: 'Resource 1',
              mimeType: 'text/plain',
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

      const result = await handleMCPResource(
        { uri: 'resource://test/resource1' },
        'si-123',
        mockSupabase
      );

      expect(result).toEqual({
        contents: [
          {
            uri: 'resource://test/resource1',
            mimeType: 'text/plain',
            text: JSON.stringify({ content: 'Resource content' }),
          },
        ],
      });
    });

    it('should throw error when resource does not exist', async () => {
      const mockServiceInterface = {
        id: 'si-123',
        mcp_config: {
          resources: [
            {
              uri: 'resource://test/resource1',
              name: 'Resource 1',
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
        handleMCPResource({ uri: 'resource://test/nonexistent' }, 'si-123', mockSupabase)
      ).rejects.toThrow('Resource not found');
    });

    it('should return empty resource when no workflow is configured', async () => {
      const mockServiceInterface = {
        id: 'si-123',
        mcp_config: {
          resources: [
            {
              uri: 'resource://test/resource1',
              name: 'Resource 1',
              mimeType: 'text/plain',
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

      const result = await handleMCPResource(
        { uri: 'resource://test/resource1' },
        'si-123',
        mockSupabase
      );

      expect(result).toEqual({
        contents: [
          {
            uri: 'resource://test/resource1',
            mimeType: 'text/plain',
            text: '',
          },
        ],
      });
    });
  });
});

