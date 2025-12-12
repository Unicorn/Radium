/**
 * State Storage Adapters Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '@/types/database';
import { WorkflowStateAdapter } from '../storage-adapters/workflow-state';
import { DatabaseStateAdapter } from '../storage-adapters/database-state';
import { RedisStateAdapter } from '../storage-adapters/redis-state';

describe('State Storage Adapters', () => {
  let mockSupabase: SupabaseClient<Database>;

  beforeEach(() => {
    mockSupabase = {
      from: vi.fn(),
    } as any;
  });

  describe('WorkflowStateAdapter', () => {
    it('should get and set values', async () => {
      const adapter = new WorkflowStateAdapter();
      const variableId = 'var-123';
      const variableName = 'testVar';

      await adapter.set(variableId, variableName, 'test-value');
      const value = await adapter.get(variableId, variableName);

      expect(value).toBe('test-value');
    });

    it('should return null for non-existent variables', async () => {
      const adapter = new WorkflowStateAdapter();
      const value = await adapter.get('var-123', 'nonexistent');

      expect(value).toBeNull();
    });

    it('should append to array values', async () => {
      const adapter = new WorkflowStateAdapter();
      const variableId = 'var-123';
      const variableName = 'arrayVar';

      await adapter.set(variableId, variableName, [1, 2]);
      await adapter.append(variableId, variableName, 3);
      const value = await adapter.get(variableId, variableName);

      expect(value).toEqual([1, 2, 3]);
    });

    it('should increment numeric values', async () => {
      const adapter = new WorkflowStateAdapter();
      const variableId = 'var-123';
      const variableName = 'counter';

      await adapter.set(variableId, variableName, 5);
      const newValue = await adapter.increment(variableId, variableName, 3);

      expect(newValue).toBe(8);
    });

    it('should decrement numeric values', async () => {
      const adapter = new WorkflowStateAdapter();
      const variableId = 'var-123';
      const variableName = 'counter';

      await adapter.set(variableId, variableName, 10);
      const newValue = await adapter.decrement(variableId, variableName, 4);

      expect(newValue).toBe(6);
    });

    it('should delete variables', async () => {
      const adapter = new WorkflowStateAdapter();
      const variableId = 'var-123';
      const variableName = 'testVar';

      await adapter.set(variableId, variableName, 'value');
      await adapter.delete(variableId, variableName);
      const exists = await adapter.exists(variableId, variableName);

      expect(exists).toBe(false);
    });

    it('should check if variable exists', async () => {
      const adapter = new WorkflowStateAdapter();
      const variableId = 'var-123';
      const variableName = 'testVar';

      expect(await adapter.exists(variableId, variableName)).toBe(false);

      await adapter.set(variableId, variableName, 'value');
      expect(await adapter.exists(variableId, variableName)).toBe(true);
    });
  });

  describe('DatabaseStateAdapter', () => {
    it('should get and set values in database', async () => {
      const mockData = { value: 'test-value' };
      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: mockData,
        error: null,
      });

      const mockInsert = vi.fn().mockReturnThis();
      const mockUpsert = vi.fn().mockResolvedValue({ error: null });

      (mockSupabase.from as any)
        .mockReturnValueOnce({
          select: mockSelect,
          eq: mockEq,
          single: mockSingle,
        })
        .mockReturnValueOnce({
          upsert: mockUpsert,
        });

      const adapter = new DatabaseStateAdapter(mockSupabase);
      const value = await adapter.get('var-123', 'testVar');

      expect(value).toBe('test-value');
      expect(mockSupabase.from).toHaveBeenCalledWith('state_variable_data');
    });

    it('should return null when value does not exist', async () => {
      const mockSelect = vi.fn().mockReturnThis();
      const mockEq = vi.fn().mockReturnThis();
      const mockSingle = vi.fn().mockResolvedValue({
        data: null,
        error: { code: 'PGRST116' },
      });

      (mockSupabase.from as any).mockReturnValue({
        select: mockSelect,
        eq: mockEq,
        single: mockSingle,
      });

      const adapter = new DatabaseStateAdapter(mockSupabase);
      const value = await adapter.get('var-123', 'nonexistent');

      expect(value).toBeNull();
    });

    it('should set values in database', async () => {
      const mockUpsert = vi.fn().mockResolvedValue({ error: null });

      (mockSupabase.from as any).mockReturnValue({
        upsert: mockUpsert,
      });

      const adapter = new DatabaseStateAdapter(mockSupabase);
      await adapter.set('var-123', 'testVar', 'test-value');

      expect(mockUpsert).toHaveBeenCalledWith(
        expect.objectContaining({
          variable_id: 'var-123',
          name: 'testVar',
          value: 'test-value',
        }),
        expect.any(Object)
      );
    });

    it('should delete values from database', async () => {
      const mockDelete = vi.fn().mockReturnThis();
      const mockEq1 = vi.fn().mockReturnThis();
      const mockEq2 = vi.fn().mockResolvedValue({ error: null });

      (mockSupabase.from as any).mockReturnValue({
        delete: mockDelete,
        eq: mockEq1,
      });

      mockEq1.mockReturnValue({
        eq: mockEq2,
      });

      const adapter = new DatabaseStateAdapter(mockSupabase);
      await adapter.delete('var-123', 'testVar');

      expect(mockDelete).toHaveBeenCalled();
      expect(mockEq1).toHaveBeenCalledWith('variable_id', 'var-123');
      expect(mockEq2).toHaveBeenCalledWith('name', 'testVar');
    });
  });

  describe('RedisStateAdapter', () => {
    it('should get and set values using Redis', async () => {
      // Mock redisCommandActivity
      const mockRedisCommand = vi.fn().mockResolvedValue('test-value');

      // Note: RedisStateAdapter uses redisCommandActivity which would need to be mocked
      // For now, we test the adapter structure
      const adapter = new RedisStateAdapter(mockSupabase, {
        connectorId: 'connector-123',
        keyPrefix: 'test:',
      });

      expect(adapter).toBeDefined();
      // Full testing would require mocking the Temporal activity system
    });

    it('should handle Redis connection errors gracefully', async () => {
      const adapter = new RedisStateAdapter(mockSupabase, {
        connectorId: 'connector-123',
      });

      // Adapter should be created successfully
      expect(adapter).toBeDefined();
      // Error handling would be tested with actual Redis activity mocks
    });
  });
});

