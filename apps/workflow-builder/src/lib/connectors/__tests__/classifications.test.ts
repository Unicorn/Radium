/**
 * Tests for connector classification utilities
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import {
  getConnectorsByClassification,
  addClassification,
  removeClassification,
  getConnectorClassifications,
} from '../classifications';

describe('Connector Classifications', () => {
  let mockSupabase: SupabaseClient<Database>;

  beforeEach(() => {
    mockSupabase = {
      from: vi.fn(),
    } as any;
  });

  describe('getConnectorsByClassification', () => {
    it('should return connectors with the specified classification', async () => {
      const mockConnectors = [
        {
          id: 'conn-1',
          name: 'upstash-redis',
          display_name: 'Upstash Redis',
          description: 'Redis connector',
          connector_type: 'database',
          is_active: true,
          classifications: ['redis'],
        },
        {
          id: 'conn-2',
          name: 'redis-cloud',
          display_name: 'Redis Cloud',
          description: 'Another Redis connector',
          connector_type: 'database',
          is_active: true,
          classifications: ['redis'],
        },
      ];

      const mockQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        contains: vi.fn().mockReturnThis(),
        order: vi.fn().mockResolvedValue({ data: mockConnectors, error: null }),
      };

      (mockSupabase.from as any).mockReturnValue(mockQuery);

      const result = await getConnectorsByClassification(
        'project-1',
        'redis',
        mockSupabase
      );

      expect(mockSupabase.from).toHaveBeenCalledWith('connectors');
      expect(mockQuery.eq).toHaveBeenCalledWith('project_id', 'project-1');
      expect(mockQuery.eq).toHaveBeenCalledWith('is_active', true);
      expect(mockQuery.contains).toHaveBeenCalledWith('classifications', ['redis']);
      expect(result).toHaveLength(2);
      expect(result[0].id).toBe('conn-1');
      expect(result[0].classifications).toEqual(['redis']);
    });

    it('should throw error if query fails', async () => {
      const mockQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        contains: vi.fn().mockReturnThis(),
        order: vi.fn().mockResolvedValue({
          data: null,
          error: { message: 'Database error' },
        }),
      };

      (mockSupabase.from as any).mockReturnValue(mockQuery);

      await expect(
        getConnectorsByClassification('project-1', 'redis', mockSupabase)
      ).rejects.toThrow('Failed to get connectors by classification');
    });

    it('should return empty array if no connectors found', async () => {
      const mockQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
        contains: vi.fn().mockReturnThis(),
        order: vi.fn().mockResolvedValue({ data: [], error: null }),
      };

      (mockSupabase.from as any).mockReturnValue(mockQuery);

      const result = await getConnectorsByClassification(
        'project-1',
        'http-log',
        mockSupabase
      );

      expect(result).toEqual([]);
    });
  });

  describe('addClassification', () => {
    it('should add classification to connector', async () => {
      const mockQuery = {
        insert: vi.fn().mockResolvedValue({ error: null }),
      };

      (mockSupabase.from as any).mockReturnValue(mockQuery);

      await addClassification('conn-1', 'redis', mockSupabase);

      expect(mockSupabase.from).toHaveBeenCalledWith('connector_classifications');
      expect(mockQuery.insert).toHaveBeenCalledWith({
        connector_id: 'conn-1',
        classification: 'redis',
      });
    });

    it('should not throw if classification already exists (unique constraint)', async () => {
      const mockQuery = {
        insert: vi.fn().mockResolvedValue({
          error: { code: '23505', message: 'Unique constraint violation' },
        }),
      };

      (mockSupabase.from as any).mockReturnValue(mockQuery);

      // Should not throw for unique constraint violation
      await expect(
        addClassification('conn-1', 'redis', mockSupabase)
      ).resolves.not.toThrow();
    });

    it('should throw error for other database errors', async () => {
      const mockQuery = {
        insert: vi.fn().mockResolvedValue({
          error: { code: '23503', message: 'Foreign key violation' },
        }),
      };

      (mockSupabase.from as any).mockReturnValue(mockQuery);

      await expect(
        addClassification('conn-1', 'redis', mockSupabase)
      ).rejects.toThrow('Failed to add classification');
    });
  });

  describe('removeClassification', () => {
    it('should remove classification from connector', async () => {
      const mockQuery = {
        delete: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
      };

      (mockQuery.eq as any).mockImplementation((field: string, value: any) => {
        if (field === 'connector_id') {
          return {
            eq: (field2: string, value2: any) => ({
              eq: vi.fn().mockResolvedValue({ error: null }),
            }),
          };
        }
        return mockQuery;
      });

      (mockSupabase.from as any).mockReturnValue(mockQuery);

      await removeClassification('conn-1', 'redis', mockSupabase);

      expect(mockSupabase.from).toHaveBeenCalledWith('connector_classifications');
      expect(mockQuery.delete).toHaveBeenCalled();
    });

    it('should throw error if deletion fails', async () => {
      const mockQuery = {
        delete: vi.fn().mockReturnThis(),
        eq: vi.fn().mockReturnThis(),
      };

      (mockQuery.eq as any).mockImplementation((field: string, value: any) => {
        if (field === 'connector_id') {
          return {
            eq: (field2: string, value2: any) => ({
              eq: vi.fn().mockResolvedValue({
                error: { message: 'Deletion failed' },
              }),
            }),
          };
        }
        return mockQuery;
      });

      (mockSupabase.from as any).mockReturnValue(mockQuery);

      await expect(
        removeClassification('conn-1', 'redis', mockSupabase)
      ).rejects.toThrow('Failed to remove classification');
    });
  });

  describe('getConnectorClassifications', () => {
    it('should return all classifications for a connector', async () => {
      const mockClassifications = [
        { classification: 'redis' },
        { classification: 'cache' },
      ];

      const mockQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockResolvedValue({
          data: mockClassifications,
          error: null,
        }),
      };

      (mockSupabase.from as any).mockReturnValue(mockQuery);

      const result = await getConnectorClassifications('conn-1', mockSupabase);

      expect(mockSupabase.from).toHaveBeenCalledWith('connector_classifications');
      expect(mockQuery.eq).toHaveBeenCalledWith('connector_id', 'conn-1');
      expect(result).toEqual(['redis', 'cache']);
    });

    it('should return empty array if no classifications found', async () => {
      const mockQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockResolvedValue({ data: [], error: null }),
      };

      (mockSupabase.from as any).mockReturnValue(mockQuery);

      const result = await getConnectorClassifications('conn-1', mockSupabase);

      expect(result).toEqual([]);
    });

    it('should throw error if query fails', async () => {
      const mockQuery = {
        select: vi.fn().mockReturnThis(),
        eq: vi.fn().mockResolvedValue({
          data: null,
          error: { message: 'Database error' },
        }),
      };

      (mockSupabase.from as any).mockReturnValue(mockQuery);

      await expect(
        getConnectorClassifications('conn-1', mockSupabase)
      ).rejects.toThrow('Failed to get connector classifications');
    });
  });
});

